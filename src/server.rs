pub(super) mod args;
mod connection;
mod project;
mod scheduler;

use crate::common::file::{get_project_file, init_working_dir};
use crate::common::render_task::FileExt;
use crate::server::args::ServerArgs;
use crate::server::connection::Connection;
use crate::server::project::Project;
use crate::server::scheduler::{Scheduler, SchedulerManageMessage};
use crossbeam_channel::Sender;
use failure::Fail;
use log::{debug, info};
use std::net::TcpListener;
use std::path::Path;
use std::{io, thread};

pub(super) type ServerResult<T> = Result<T, ServerError>;

#[derive(Fail, Debug)]
pub(super) enum ServerError {
    #[fail(display = "Error initializing working directory: {}", 0)]
    WorkingDirError(#[fail(cause)] io::Error),
    #[fail(display = "Error starting server: {}", 0)]
    InitError(#[fail(cause)] io::Error),
}

pub(super) struct Server {}

impl Server {
    /// Run the server
    pub(super) fn run(args: ServerArgs) -> ServerResult<()> {
        info!("Starting server on {}:{}...", args.address, args.port);

        // Initialize the working directory
        let working_dir = init_working_dir("server").map_err(ServerError::WorkingDirError)?;

        // Bind to the socket
        debug!("Binding to socket...");
        let listener = TcpListener::bind((args.address.as_str(), args.port))
            .map_err(ServerError::InitError)?;

        // Start the scheduler in a new thread
        debug!("Starting scheduler...");
        let (render_recv, result_send, manage_send) = Scheduler::start();

        info!("Server started!");

        // Add some test projects
        for index in 1..=4 {
            let project = Project::new(format!("Project {}", index), FileExt::PNG, 1, index * 2);
            Self::add_dummy_project(project, &working_dir, &manage_send);
        }

        // Handle incoming connections
        for stream in listener.incoming().filter_map(|stream| stream.ok()) {
            // Clone render and result channel endpoints
            let render_recv = render_recv.clone();
            let result_send = result_send.clone();
            let working_dir = working_dir.clone();
            // Spawn a thread to handle the connection
            thread::spawn(move || {
                Connection::handle(stream, render_recv, result_send, &working_dir)
            });
        }
        unreachable!();
    }

    // Send a project to the scheduler, copying the project file from /tmp/untitled.blend
    fn add_dummy_project(
        project: Project,
        working_dir: &Path,
        manage_send: &Sender<SchedulerManageMessage>,
    ) {
        // Copy the project file from /tmp/untitled.blend
        let project_file = get_project_file(working_dir, &project.uuid);
        std::fs::create_dir(project_file.parent().unwrap())
            .expect("unable to create project directory");
        std::fs::copy("/tmp/untitled.blend", project_file)
            .expect("unable to copy dummy project file");
        // Send the project to the scheduler
        manage_send.send(SchedulerManageMessage::AddProject(project)).unwrap();
    }
}
