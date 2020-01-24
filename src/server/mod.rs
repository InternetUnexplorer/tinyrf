pub mod connection;
pub mod project;

use crate::server::connection::Connection;
use crate::server::project::Project;
use failure::Fail;
use log::info;
use std::collections::HashMap;
use std::{io, net::TcpListener, thread};
use uuid::Uuid;

pub type ServerResult = Result<(), ServerError>;

#[derive(Fail, Debug)]
pub enum ServerError {
    #[fail(display = "error starting server: {}", 0)]
    InitError(#[fail(cause)] io::Error),
}

pub struct Server {
    listener: TcpListener,
    projects: HashMap<Uuid, Project>,
}

impl Server {
    /// Start the server
    pub fn run(address: &str, port: u16) -> ServerResult {
        info!("starting server on {}:{}...", address, port);

        let server = Server {
            listener: TcpListener::bind((address, port)).map_err(ServerError::InitError)?,
            projects: HashMap::new(),
        };

        info!("server started.");

        server.handle_connections();
    }

    /// Wait for and handle incoming connections
    fn handle_connections(&self) -> ! {
        for stream in self.listener.incoming().filter_map(|stream| stream.ok()) {
            thread::spawn(move || Connection::handle(stream));
        }
        unreachable!();
    }
}
