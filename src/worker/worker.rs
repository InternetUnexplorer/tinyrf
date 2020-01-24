use std::net::TcpStream;

use log::info;

use crate::worker::result::{WorkerError, WorkerResult};

pub fn run_worker(address: &str, port: u16) -> WorkerResult {
    info!("connecting to {}:{}...", address, port);

    let mut stream = TcpStream::connect((address, port)).map_err(WorkerError::ConnectError)?;

    loop {} // TODO
}
