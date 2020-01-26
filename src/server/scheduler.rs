use crate::common::render_task::{RenderTask, RenderTaskResult};
use crate::server::project::Project;
use crossbeam_channel::{Receiver, Select, Sender};
use log::{debug, info};
use std::collections::HashMap;
use std::thread;
use uuid::Uuid;

/// A message sent by the scheduler with a frame to render
#[derive(Debug)]
pub(super) struct SchedulerRenderMessage(pub RenderTask);

/// A message sent to the scheduler with the result of a render
#[derive(Debug)]
pub(super) struct SchedulerResultMessage(pub RenderTask, pub RenderTaskResult);

/// A message sent to the scheduler with a project management task
#[derive(Debug)]
pub(super) enum SchedulerManageMessage {
    AddProject(Project),
}

pub(crate) struct Scheduler {
    projects: HashMap<Uuid, Project>,
    render_send: Sender<SchedulerRenderMessage>,
    result_recv: Receiver<SchedulerResultMessage>,
    manage_recv: Receiver<SchedulerManageMessage>,
}

impl Scheduler {
    /// Create a scheduler and start it in a new thread
    pub(super) fn start() -> (
        Receiver<SchedulerRenderMessage>,
        Sender<SchedulerResultMessage>,
        Sender<SchedulerManageMessage>,
    ) {
        // Initialize render message channel (blocking)
        let (render_send, render_recv) = crossbeam_channel::bounded(0);
        // Initialize result message channel
        let (result_send, result_recv) = crossbeam_channel::unbounded();
        // Initialize management message channel
        let (manage_send, manage_recv) = crossbeam_channel::unbounded();

        // Create the scheduler
        let mut scheduler =
            Scheduler { projects: HashMap::new(), render_send, result_recv, manage_recv };

        // Start the scheduler in a new thread
        thread::spawn(move || scheduler.run());

        (render_recv, result_send, manage_send)
    }

    /// Run the scheduler
    fn run(&mut self) -> ! {
        // Create a selector over the result and management channels
        let mut selector = Select::new();
        let result_recv = self.result_recv.clone();
        let manage_recv = self.manage_recv.clone();
        selector.recv(&result_recv);
        selector.recv(&manage_recv);

        loop {
            // If there is a project waiting, send a render message
            if let Some(project) = self.get_waiting_project() {
                // Get the first waiting frame
                let frame = project.waiting_frames.pop_front().unwrap();
                // Move it to the assigned frames
                debug!("Moving project {} frame {} to the ASSIGNED queue", &project.uuid, frame);
                assert!(project.assigned_frames.insert(frame));
                // Send a render message
                let message = SchedulerRenderMessage(RenderTask {
                    project_uuid: project.uuid.clone(),
                    project_name: project.name.clone(),
                    frame,
                    output_ext: project.output_ext,
                });
                self.send_render_msg(message);
            } else {
                // Block until there are messages
                let _ = selector.ready();
            }
            // Handle messages
            while let Ok(message) = self.result_recv.try_recv() {
                self.handle_result_msg(message);
            }
            // Handle management messages
            while let Ok(message) = self.manage_recv.try_recv() {
                self.handle_manage_msg(message);
            }
        }
    }

    /// Get the first project with waiting frames
    fn get_waiting_project(&mut self) -> Option<&mut Project> {
        self.projects.iter_mut().map(|p| p.1).filter(|p| p.num_waiting_frames() > 0).next()
    }

    /// Send a render message and wait for it to be received
    fn send_render_msg(&mut self, message: SchedulerRenderMessage) {
        self.render_send.send(message).unwrap();
    }

    /// Handle a result message
    fn handle_result_msg(&mut self, message: SchedulerResultMessage) {
        let SchedulerResultMessage(render_task, result) = message;
        // Get the project the frame belongs to
        let project = self.projects.get_mut(&render_task.project_uuid).unwrap();
        // Remove the frame from the assigned queue
        assert!(project.assigned_frames.remove(&render_task.frame));
        // Handle the result
        match result {
            Ok(()) => {
                // Move the project to the completed queue
                debug!(
                    "Moving project {} frame {} to the COMPLETED queue",
                    &render_task.project_uuid, render_task.frame
                );
                project.completed_frames.push_back(render_task.frame);
                // Remove the project if it is completed
                if project.complete() {
                    info!("Project \"{}\" is finished", project);
                    let uuid = project.uuid.clone();
                    assert!(self.projects.remove(&uuid).is_some());
                }
            }
            Err(()) => {
                // Move the project to the waiting queue
                debug!(
                    "Moving project {} frame {} to the WAITING queue",
                    &render_task.project_uuid, render_task.frame
                );
                project.waiting_frames.push_back(render_task.frame)
            }
        }
    }

    /// Handle a management message
    fn handle_manage_msg(&mut self, message: SchedulerManageMessage) {
        match message {
            SchedulerManageMessage::AddProject(project) => {
                // Add the project
                info!("Adding project \"{}\"", &project);
                assert!(self.projects.insert(project.uuid.clone(), project).is_none());
            }
        }
    }
}
