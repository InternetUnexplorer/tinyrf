use crate::common::render_task::{RenderTask, RenderTaskResult};
use crate::server::project::Project;
use crossbeam_channel::{Receiver, Select, Sender};
use log::{debug, error, info};
use std::collections::{HashMap, VecDeque};
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
    // Add a project to the queue
    AddProject(Project),
    // Retry a project's failed frames
    RetryFailed(Uuid),
}

pub(crate) struct Scheduler {
    projects: HashMap<Uuid, Project>,
    queue: VecDeque<Uuid>,
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
        let mut scheduler = Scheduler {
            projects: HashMap::new(),
            queue: VecDeque::new(),
            render_send,
            result_recv,
            manage_recv,
        };

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
            // Check if there is a project with waiting frames
            if let Some(project_uuid) = self.queue.pop_front() {
                // Assign the first waiting frame and send a render message
                self.assign_first_waiting_frame(&project_uuid);
                // Move the project back into the queue if it still has waiting frames
                if self.projects.get(&project_uuid).unwrap().num_waiting() > 0 {
                    self.queue.push_back(project_uuid)
                }
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

    /// Assign the first waiting frame of a project and send a render message
    fn assign_first_waiting_frame(&mut self, project_uuid: &Uuid) {
        // Get the first waiting frame of the project
        let project = self.projects.get_mut(&project_uuid).unwrap();
        let frame = project.waiting_frames.pop_front().unwrap();
        // Move the frame to the assigned queue
        debug!("Moving project {} frame {} to the ASSIGNED queue", &project.uuid, frame);
        assert!(project.assigned_frames.insert(frame));
        // Send a render message
        let message = SchedulerRenderMessage(RenderTask {
            project_uuid: project.uuid.clone(),
            project_name: project.name.clone(),
            frame,
            output_ext: project.output_ext,
        });
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
                // Move the frame to the completed queue
                debug!(
                    "Moving project {} frame {} to the COMPLETED queue",
                    &render_task.project_uuid, render_task.frame
                );
                project.completed_frames.push_back(render_task.frame);
                // Print a message if the project is complete
                if project.complete() {
                    info!("Project \"{}\" is finished", project);
                }
            }
            Err(()) => {
                // Move the frame to the failed queue
                debug!(
                    "Moving project {} frame {} to the FAILED queue",
                    &render_task.project_uuid, render_task.frame
                );
                project.failed_frames.push_back(render_task.frame);
            }
        }
        // If this was the last assigned frame, check if there are failed frames
        if project.num_waiting() == 0 && project.num_assigned() == 0 && project.num_failed() > 0 {
            error!("Some frames of \"{}\" failed to render", project);
        }
    }

    /// Handle a management message
    fn handle_manage_msg(&mut self, message: SchedulerManageMessage) {
        match message {
            // Add a project to the queue
            SchedulerManageMessage::AddProject(project) => {
                info!("Adding project \"{}\"", &project);
                self.queue.push_back(project.uuid.clone());
                assert!(self.projects.insert(project.uuid.clone(), project).is_none());
            }
            // Retry a project's failed frames
            SchedulerManageMessage::RetryFailed(project_uuid) => {
                match self.projects.get_mut(&project_uuid) {
                    Some(project) => {
                        // Move failed frames back to the waiting queue
                        project.retry_failed();
                        // Add the project to the queue if it is not already present
                        if !self.queue.contains(&project_uuid) {
                            self.queue.push_back(project_uuid);
                        }
                    }
                    None => error!("Project {} not found", project_uuid),
                }
            }
        }
    }
}
