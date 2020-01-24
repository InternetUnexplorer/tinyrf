use crate::common::message::{ServerMessage, WorkerMessage};
use crate::common::util::{read_json, write_json};
use failure::Fail;
use log::info;
use std::io;
use std::io::{BufReader, BufWriter};
use std::net::TcpStream;

pub struct Connection<'a> {
    reader: BufReader<&'a TcpStream>,
    writer: BufWriter<&'a TcpStream>,
}

pub type ConnectionResult<T> = Result<T, ConnectionError>;

#[derive(Fail, Debug)]
pub enum ConnectionError {
    #[fail(display = "I/O error: {}", 0)]
    IoError(#[fail(cause)] io::Error),
}

impl<'a> Connection<'a> {
    /// Handle an incoming worker connection
    pub fn handle(stream: TcpStream) {
        info!("worker connected: {}", stream.peer_addr().unwrap());

        let mut connection = Connection {
            reader: BufReader::new(&stream),
            writer: BufWriter::new(&stream),
        };

        // TODO
    }

    /// Read a message from the worker (blocking)
    fn read_message(&mut self) -> ConnectionResult<WorkerMessage> {
        Ok(read_json(&mut self.reader)?)
    }

    /// Send a message to the worker
    fn write_message(&mut self, message: ServerMessage) -> ConnectionResult<()> {
        Ok(write_json(&mut self.writer, message)?)
    }
}

impl From<io::Error> for ConnectionError {
    fn from(error: io::Error) -> Self {
        Self::IoError(error)
    }
}
