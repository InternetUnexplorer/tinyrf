use crate::common::message::{ServerMessage, WorkerMessage};
use crate::common::util::{read_json, write_json};
use failure::Fail;
use log::info;
use std::io;
use std::io::{BufReader, BufWriter};
use std::net::TcpStream;

pub type WorkerResult<T> = Result<T, WorkerError>;

#[derive(Fail, Debug)]
pub enum WorkerError {
    #[fail(display = "error connecting to server: {}", 0)]
    ConnectError(#[fail(cause)] io::Error),
    #[fail(display = "I/O error: {}", 0)]
    IoError(#[fail(cause)] io::Error),
}

pub struct Worker<'a> {
    reader: BufReader<&'a TcpStream>,
    writer: BufWriter<&'a TcpStream>,
}

impl<'a> Worker<'a> {
    /// Connect to the server and handle messages
    pub fn connect(address: &str, port: u16) -> WorkerResult<()> {
        info!("connecting to {}:{}...", address, port);

        let stream = TcpStream::connect((address, port)).map_err(WorkerError::ConnectError)?;

        info!("connected to server.");

        let mut worker = Worker {
            reader: BufReader::new(&stream),
            writer: BufWriter::new(&stream),
        };

        loop {
            let message = worker.read_message()?;
            worker.handle_message(message)?;
        }
    }

    /// Read a message from the server (blocking)
    fn read_message(&mut self) -> WorkerResult<ServerMessage> {
        Ok(read_json(&mut self.reader)?)
    }

    /// Send a message to the server
    fn write_message(&mut self, message: WorkerMessage) -> WorkerResult<()> {
        Ok(write_json(&mut self.writer, message)?)
    }

    /// Handle a message from the server
    fn handle_message(&mut self, message: ServerMessage) -> WorkerResult<()> {
        info!("-> {:?}", message);
        // TODO
        Ok(())
    }
}

impl From<io::Error> for WorkerError {
    fn from(error: io::Error) -> Self {
        Self::IoError(error)
    }
}
