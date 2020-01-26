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

    /// Enables debugging messages
    #[structopt(short = "v", long = "verbose")]
    verbose: bool,

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

    // Set up logging
    init_logger(config.verbose);

    // Run the server or the worker
    if config.server {
        run_server(config);
    } else {
        run_worker(config);
    }
}

/// Configure and initialize the global logger
fn init_logger(debug: bool) {
    let level = match debug {
        true => LevelFilter::Trace,
        false => LevelFilter::Info,
    };
    env_logger::builder().filter_level(level).format_module_path(false).init();
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
