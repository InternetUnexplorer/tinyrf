pub(crate) mod connection;
pub(crate) mod project;
pub(crate) mod scheduler;

use crate::common::file::{get_project_file, init_working_dir};
use crate::common::render_task::FileExt;
use crate::server::connection::Connection;
use crate::server::project::Project;
use crate::server::scheduler::{Scheduler, SchedulerManageMessage};
use failure::Fail;
use log::info;
use std::{io, net::TcpListener, thread};

pub(super) type ServerResult<T> = Result<T, ServerError>;

#[derive(Fail, Debug)]
pub(super) enum ServerError {
    #[fail(display = "error initializing working directory: {}", 0)]
    WorkingDirError(#[fail(cause)] io::Error),
    #[fail(display = "error starting server: {}", 0)]
    InitError(#[fail(cause)] io::Error),
}

pub(super) struct Server {}

impl Server {
    /// Start the server
    pub(super) fn run(address: &str, port: u16) -> ServerResult<()> {
        info!("starting server on {}:{}...", address, port);

        // Initialize the working directory
        let working_dir = init_working_dir("server").map_err(ServerError::WorkingDirError)?;

        // Bind to the socket
        let listener = TcpListener::bind((address, port)).map_err(ServerError::InitError)?;

        // Start the scheduler in a new thread
        let (render_recv, result_send, manage_send) = Scheduler::start();

        info!("server started.");

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
}
