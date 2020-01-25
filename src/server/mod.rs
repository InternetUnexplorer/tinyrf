pub(crate) mod connection;
pub(crate) mod project;
pub(crate) mod scheduler;

use crate::server::connection::Connection;
use crate::server::scheduler::Scheduler;
use failure::Fail;
use log::info;
use std::{io, net::TcpListener, thread};

pub(super) type ServerResult = Result<(), ServerError>;

#[derive(Fail, Debug)]
pub(super) enum ServerError {
    #[fail(display = "error starting server: {}", 0)]
    InitError(#[fail(cause)] io::Error),
}

pub(super) struct Server {}

impl Server {
    /// Start the server
    pub(super) fn run(address: &str, port: u16) -> ServerResult {
        info!("starting server on {}:{}...", address, port);

        // Start the scheduler in a new thread
        let (render_recv, result_send, manage_send) = Scheduler::start();

        // Bind to the socket
        let listener = TcpListener::bind((address, port)).map_err(ServerError::InitError)?;

        info!("server started.");

        // Handle incoming connections
        for stream in listener.incoming().filter_map(|stream| stream.ok()) {
            // Clone render and result channel endpoints
            let render_recv = render_recv.clone();
            let result_send = result_send.clone();
            // Spawn a thread to handle the connection
            thread::spawn(move || Connection::handle(stream, render_recv, result_send));
        }
        unreachable!();
    }
}
