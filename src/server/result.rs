use std::io::Error;

use failure::Fail;

pub type ServerResult = Result<(), ServerError>;

#[derive(Fail, Debug)]
pub enum ServerError {
    #[fail(display = "unable to start server: {}", 0)]
    InitError(Error),
}
