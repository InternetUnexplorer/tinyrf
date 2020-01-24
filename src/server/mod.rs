pub mod result;

use std::net::{TcpListener, TcpStream};
use std::thread;

use log::info;

use crate::server::result::{ServerError, ServerResult};

pub fn handle_worker(stream: TcpStream) {
    info!("worker connected: {}", stream.peer_addr().unwrap());
    // TODO
}

pub fn run_server(address: &str, port: u16) -> ServerResult {
    info!("starting server on {}:{}...", address, port);

    let listener = TcpListener::bind((address, port)).map_err(ServerError::InitError)?;

    for stream in listener.incoming().filter_map(|stream| stream.ok()) {
        thread::spawn(move || handle_worker(stream));
    }

    Ok(())
}
