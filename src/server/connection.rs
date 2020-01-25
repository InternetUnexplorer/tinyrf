use crate::common::message::{ServerMessage, WorkerMessage};
use crate::common::net::{read_json, write_json};
use crate::server::scheduler::{SchedulerRenderMessage, SchedulerResultMessage};
use crossbeam_channel::{Receiver, Sender};
use failure::Fail;
use log::{error, info};
use std::io::{BufReader, BufWriter};
use std::net::{IpAddr, TcpStream};
use std::{fmt, io};

pub(super) struct Connection<'a> {
    name: Option<String>,
    addr: IpAddr,
    reader: BufReader<&'a TcpStream>,
    writer: BufWriter<&'a TcpStream>,
    render_recv: Receiver<SchedulerRenderMessage>,
    result_send: Sender<SchedulerResultMessage>,
}

type ConnectionResult<T> = Result<T, ConnectionError>;

#[derive(Fail, Debug)]
enum ConnectionError {
    #[fail(display = "I/O error: {}", 0)]
    IoError(#[fail(cause)] io::Error),
    #[fail(display = "unexpected message: {:?}", 0)]
    MessageError(WorkerMessage),
}

impl Connection<'_> {
    /// Handle an incoming worker connection
    pub(super) fn handle(
        stream: TcpStream,
        render_recv: Receiver<SchedulerRenderMessage>,
        result_send: Sender<SchedulerResultMessage>,
    ) {
        let mut connection = Connection {
            name: None,
            addr: stream.peer_addr().unwrap().ip(),
            reader: BufReader::new(&stream),
            writer: BufWriter::new(&stream),
            render_recv,
            result_send,
        };

        // Read the init message from the worker
        if let Ok(WorkerMessage::Init { name }) = connection.read_message() {
            // Set the worker name
            connection.name = name;

            info!("worker connected: {}", connection);

            // Wait for and send render tasks until an error occurs
            if let Err(error) = connection.communicate() {
                error!("worker disconnected: {}: {}", connection, error);
            }
        }
    }

    /// Wait for render tasks and send them to the worker
    fn communicate(&mut self) -> ConnectionResult<()> {
        loop {
            // Wait for a render message from the scheduler
            let render_message = self.render_recv.recv().unwrap();
            // Send the render information to the worker
            self.write_message(ServerMessage::StartRender(render_message.0))?;
            // TODO
        }
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

impl fmt::Display for Connection<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.name {
            Some(name) => write!(f, "{} ({}) ", name, self.addr),
            None => write!(f, "{}", self.addr),
        }
    }
}
