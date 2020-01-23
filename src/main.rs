use std::env::args;

use log::error;

use crate::server::server::run_server;
use crate::worker::worker::run_worker;

mod common;
mod server;
mod worker;

const ADDRESS: &str = "localhost";
const PORT: u16 = 4096;

fn main() {
    pretty_env_logger::init();

    if let Err(message) = match args().skip(1).next().as_ref().map(|arg| arg.as_str()) {
        Some("server") => run_server(ADDRESS, PORT),
        Some("worker") => run_worker(ADDRESS, PORT),
        _ => Ok(eprintln!("Expected server or worker")),
    } {
        error!("{}", message);
    }
}
