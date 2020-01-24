#![allow(dead_code)] // TODO: for prototyping

mod common;
mod server;
mod worker;

use crate::{server::Server, worker::Worker};
use log::{error, LevelFilter};
use std::process::exit;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "tinyrf", about = "A simple Blender render farm")]
struct Config {
    /// Hosts a server
    #[structopt(short = "s", long = "server")]
    server: bool,

    /// Server address
    #[structopt(name = "ADDRESS", default_value = "localhost")]
    address: String,

    /// Server port
    #[structopt(short = "p", long = "port", default_value = "4049")]
    port: u16,

    /// Worker name
    #[structopt(short = "n", long = "name")]
    name: Option<String>,
}

fn main() {
    // Parse command-line arguments
    let config = Config::from_args();

    // Initialize logging with LevelFilter::Info
    pretty_env_logger::formatted_builder()
        .filter_level(LevelFilter::Info)
        .init();

    // Run the server or the worker
    if config.server {
        run_server(config);
    } else {
        run_worker(config);
    }
}

/// Run the server, printing an error message on failure
fn run_server(config: Config) {
    if let Err(error) = Server::run(&config.address, config.port) {
        error!("{}", error);
        exit(2);
    }
}

/// Run the worker, printing an error message on failure
fn run_worker(config: Config) {
    if let Err(error) = Worker::connect(config.name, &config.address, config.port) {
        error!("{}", error);
        exit(2);
    }
}
