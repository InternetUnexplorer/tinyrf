use std::io::Error;

use failure::Fail;

pub type WorkerResult = Result<(), WorkerError>;

#[derive(Fail, Debug)]
pub enum WorkerError {
    #[fail(display = "unable to connect to server: {}", 0)]
    ConnectError(Error),
}
