use std::net::TcpStream;

use log::info;

pub fn run_worker(address: &str, port: u16) -> Result<(), &'static str> {
    info!("Connecting to server...");

    let _stream = TcpStream::connect(format!("{}:{}", address, port))
        .map_err(|_| "Unable to connect to server.")?;

    loop {} // TODO
}
