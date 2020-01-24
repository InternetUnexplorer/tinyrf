#![allow(dead_code)] // TODO: for prototyping

mod common;
mod server;
mod worker;

use crate::{server::Server, worker::Worker};
use log::error;
use std::{env::args, process::exit};

const ADDRESS: &str = "localhost";
const PORT: u16 = 4096;

fn main() {
    pretty_env_logger::init();

    match args().nth(1).as_ref().map(|arg| arg.as_str()) {
        Some("server") => {
            if let Err(error) = Server::run(ADDRESS, PORT) {
                error!("{}", error);
                exit(2);
            }
        }
        Some("worker") => {
            if let Err(error) = Worker::connect(ADDRESS, PORT) {
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
