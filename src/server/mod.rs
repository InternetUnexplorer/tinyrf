pub mod connection;

use crate::server::connection::Connection;
use failure::Fail;
use log::info;
use std::{io, net::TcpListener, thread};

pub type ServerResult = Result<(), ServerError>;

#[derive(Fail, Debug)]
pub enum ServerError {
    #[fail(display = "error starting server: {}", 0)]
    InitError(#[fail(cause)] io::Error),
}

pub struct Server {
    listener: TcpListener,
}

impl Server {
    /// Start the server
    pub fn run(address: &str, port: u16) -> ServerResult {
        info!("starting server on {}:{}...", address, port);

        let server = Server {
            listener: TcpListener::bind((address, port)).map_err(ServerError::InitError)?,
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
