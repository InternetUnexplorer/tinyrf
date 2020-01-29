#![allow(dead_code)]

mod common;
mod server;
mod worker;

use crate::server::args::ServerArgs;
use crate::worker::args::WorkerArgs;
use crate::{server::Server, worker::Worker};
use failure::Error;
use log::{error, LevelFilter};
use std::process::exit;
use structopt::StructOpt;

#[derive(StructOpt)]
#[structopt(name = "tinyrf", about = "A simple Blender render farm")]
struct Args {
    /// Enables debugging information
    #[structopt(short = "v", long = "verbose", global = true)]
    verbose: bool,
    #[structopt(subcommand)]
    command: Command,
}

#[derive(StructOpt)]
enum Command {
    /// Joins a server as a client
    Client,
    /// Hosts a server
    Server(ServerArgs),
    /// Joins a server as a worker
    Worker(WorkerArgs),
}

fn main() {
    // Parse command-line arguments
    let args = Args::from_args();

    // Set up logging
    init_logger(args.verbose);

    // Run the client, server, or worker and check the result
    let result: Result<(), Error> = match args.command {
        Command::Client => unimplemented!(),
        Command::Server(args) => Server::run(args).map_err(|e| e.into()),
        Command::Worker(args) => Worker::run(args).map_err(|e| e.into()),
    };
    if let Err(error) = result {
        error!("{}", error);
        exit(1);
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
