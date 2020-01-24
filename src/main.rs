mod common;
mod server;
mod worker;

use std::env::args;
use std::process::exit;

use log::error;

use crate::server::run_server;
use crate::worker::run_worker;

const ADDRESS: &str = "localhost";
const PORT: u16 = 4096;

fn main() {
    pretty_env_logger::init();

    match args().nth(1).as_ref().map(|arg| arg.as_str()) {
        Some("server") => {
            if let Err(error) = run_server(ADDRESS, PORT) {
                error!("{}", error);
                exit(2);
            }
        }
        Some("worker") => {
            if let Err(error) = run_worker(ADDRESS, PORT) {
                error!("{}", error);
                exit(2);
            }
        }
        _ => {
            eprintln!("expected server or worker");
            exit(1);
        }
    }
}
