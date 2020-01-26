use crate::common::file::{get_output_file, get_project_file};
use crate::common::message::{ServerMessage, WorkerMessage};
use crate::common::net::{read_file, read_json, write_file, write_json};
use crate::common::render_task::{RenderTask, RenderTaskResult};
use crate::server::scheduler::{SchedulerRenderMessage, SchedulerResultMessage};
use crossbeam_channel::{Receiver, Sender};
use failure::Fail;
use log::{error, info};
use std::io::{BufReader, BufWriter};
use std::net::{IpAddr, TcpStream};
use std::path::Path;
use std::{fmt, io};

pub(super) struct Connection<'a> {
    name: Option<String>,
    addr: IpAddr,
    reader: BufReader<&'a TcpStream>,
    writer: BufWriter<&'a TcpStream>,
    render_recv: Receiver<SchedulerRenderMessage>,
    result_send: Sender<SchedulerResultMessage>,
    project_dir: &'a Path,
}

type ConnectionResult<T> = Result<T, ConnectionError>;

#[derive(Fail, Debug)]
enum ConnectionError {
    #[fail(display = "I/O error: {}", 0)]
    IoError(#[fail(cause)] io::Error),
    #[fail(display = "error sending file: {}", 0)]
    SendFileError(#[fail(cause)] io::Error),
    #[fail(display = "error receiving file: {}", 0)]
    RecvFileError(#[fail(cause)] io::Error),
    #[fail(display = "unexpected message: {:?}", 0)]
    MessageError(WorkerMessage),
}

impl Connection<'_> {
    /// Handle an incoming worker connection
    pub(super) fn handle(
        stream: TcpStream,
        render_recv: Receiver<SchedulerRenderMessage>,
        result_send: Sender<SchedulerResultMessage>,
        project_dir: &'_ Path,
    ) {
        let mut connection = Connection {
            name: None,
            addr: stream.peer_addr().unwrap().ip(),
            reader: BufReader::new(&stream),
            writer: BufWriter::new(&stream),
            render_recv,
            result_send,
            project_dir,
        };

        // Read the init message from the worker
        if let Ok(WorkerMessage::Init { name }) = connection.read_message() {
            // Set the worker name
            connection.name = name;

            info!("worker connected: {}", connection);

            // Wait for and handle render tasks until an error occurs
            let error = loop {
                // Wait for a render task from the scheduler
                let render_task = connection.render_recv.recv().unwrap().0;
                // Send the task to the worker and get the result
                let result = connection.handle_render_task(render_task.clone());
                // Handle the result
                match result {
                    Ok(result) => connection.send_result(render_task, result),
                    Err(error) => {
                        connection.send_result(render_task, Err(()));
                        break error;
                    }
                }
            };
            error!("worker disconnected: {}: {}", connection, error);
        }
    }

    /// Send a render task to the worker and get the result back
    fn handle_render_task(
        &mut self,
        render_task: RenderTask,
    ) -> ConnectionResult<RenderTaskResult> {
        // Get the project file
        let project_file = get_project_file(self.project_dir, &render_task.project_uuid);
        // Send the render information to the worker
        self.write_message(ServerMessage::StartRender(render_task.clone()))?;
        // Send the project file to the worker
        write_file(&mut self.writer, &project_file).map_err(ConnectionError::SendFileError)?;
        // Wait for a result message from the worker
        match self.read_message()? {
            WorkerMessage::RenderResult(result) => {
                // If the result was success, download the output from the worker
                if result.is_ok() {
                    let output_file = get_output_file(self.project_dir, &render_task);
                    read_file(&mut self.reader, &output_file)
                        .map_err(ConnectionError::RecvFileError)?;
                }
                Ok(result)
            }
            message => Err(ConnectionError::MessageError(message)),
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

    /// Send the result of a render task to the scheduler
    fn send_result(&mut self, render_task: RenderTask, result: RenderTaskResult) {
        self.result_send
            .send(SchedulerResultMessage(render_task, result))
            .unwrap();
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
            Some(name) => write!(f, "{} ({})", name, self.addr),
            None => write!(f, "{}", self.addr),
        }
    }
}
