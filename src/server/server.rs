use std::net::{TcpListener, TcpStream};
use std::thread;

use log::info;

pub fn handle_worker(stream: TcpStream) -> () {
    info!("Worker connected: {}", stream.peer_addr().unwrap().ip());
    // TODO
}

pub fn run_server(address: &str, port: u16) -> Result<(), &'static str> {
    info!("Starting server on {}:{}...", address, port);
    let listener = TcpListener::bind(format!("{}:{}", address, port))
        .map_err(|_| "Failed to start server.")?;

    info!("Server started.");

    for stream in listener.incoming().filter_map(|stream| stream.ok()) {
        thread::spawn(move || handle_worker(stream));
    }

    Ok(())
}
