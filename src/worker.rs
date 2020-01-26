mod render;

use crate::common::file::{get_project_file, init_working_dir};
use crate::common::message::{ServerMessage, WorkerMessage};
use crate::common::net::{read_json, write_json};
use crate::common::transfer::{recv_file, send_file};
use failure::Fail;
use log::{debug, error, info};
use std::io::{BufReader, BufWriter};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::{fs, io};
use uuid::Uuid;

pub(super) type WorkerResult<T> = Result<T, WorkerError>;

#[derive(Fail, Debug)]
pub(super) enum WorkerError {
    #[fail(display = "Error initializing working directory: {}", 0)]
    WorkingDirError(#[fail(cause)] io::Error),
    #[fail(display = "Error connecting to server: {}", 0)]
    ConnectError(#[fail(cause)] io::Error),
    #[fail(display = "I/O error: {}", 0)]
    IoError(#[fail(cause)] io::Error),
    #[fail(display = "Error transferring file: {}", 0)]
    TransferError(#[fail(cause)] io::Error),
}

pub(super) struct Worker<'a> {
    name: Option<String>,
    reader: BufReader<&'a TcpStream>,
    writer: BufWriter<&'a TcpStream>,
    working_dir: PathBuf,
}

impl<'a> Worker<'a> {
    /// Connect to the server and handle messages
    pub(super) fn connect(name: Option<String>, address: &str, port: u16) -> WorkerResult<()> {
        // If no name has been specified, try to use the hostname
        let name = name.or_else(Self::get_hostname);

        // Initialize the working directory
        let working_dir = init_working_dir("worker").map_err(WorkerError::WorkingDirError)?;

        info!("Connecting to {}:{}...", address, port);

        // Attempt to open a connection to the server
        let stream = TcpStream::connect((address, port)).map_err(WorkerError::ConnectError)?;

        info!("Connected to server!");

        let mut worker = Worker {
            name,
            reader: BufReader::new(&stream),
            writer: BufWriter::new(&stream),
            working_dir,
        };

        // Send the init message
        worker.write_message(WorkerMessage::Init {
            name: worker.name.clone(),
        })?;

        // Read and handle messages from the server
        loop {
            let message = worker.read_message()?;
            worker.handle_message(message)?;
        }
    }

    /// Handle a message from the server
    fn handle_message(&mut self, message: ServerMessage) -> WorkerResult<()> {
        match message {
            ServerMessage::StartRender(task) => {
                // Download the project file
                info!("Downloading project \"{}\"...", task.project_name);
                self.download_project(&task.project_uuid)?;
                // Render the frame
                info!("Rendering frame {}...", task.frame);
                match render::render(&task, &self.working_dir) {
                    Ok(output_file) => {
                        info!(
                            "Uploading file \"{}\"...",
                            output_file.file_name().unwrap().to_string_lossy()
                        );
                        // Send the result to the server
                        self.write_message(WorkerMessage::RenderResult(Ok(())))?;
                        // Upload the output file
                        self.upload_output(&output_file)?;
                        Ok(info!("Upload complete"))
                    }
                    Err(error) => {
                        error!("Render failed: {}", error);
                        // Send the result to the server
                        Ok(self.write_message(WorkerMessage::RenderResult(Err(())))?)
                    }
                }
            }
        }
    }

    /// Download the project file for a render task
    fn download_project(&mut self, project_uuid: &Uuid) -> WorkerResult<()> {
        let project_file = get_project_file(&self.working_dir, project_uuid);
        // Create the parent directory if it does not exist
        let project_dir = project_file.parent().unwrap();
        if !project_dir.is_dir() {
            fs::create_dir(project_dir)?;
        }
        // Download the file
        recv_file(&mut self.reader, &mut self.writer, &project_file)
            .map_err(WorkerError::TransferError)
    }

    /// Upload the output of a render
    fn upload_output(&mut self, output_file: &Path) -> WorkerResult<()> {
        // Upload the file
        send_file(&mut self.reader, &mut self.writer, output_file)
            .map_err(WorkerError::TransferError)?;
        // Remove the file after uploading
        Ok(fs::remove_file(output_file)?)
    }

    /// Read a message from the server (blocking)
    fn read_message(&mut self) -> io::Result<ServerMessage> {
        let message = read_json(&mut self.reader)?;
        debug!("Server -> {:?}", &message);
        Ok(message)
    }

    /// Send a message to the server
    fn write_message(&mut self, message: WorkerMessage) -> io::Result<()> {
        debug!("Server <- {:?}", &message);
        write_json(&mut self.writer, message)
    }

    /// Attempt to get the hostname and convert it to a string
    fn get_hostname() -> Option<String> {
        hostname::get()
            .map(|s| String::from(s.to_string_lossy()))
            .ok()
    }
}

impl From<io::Error> for WorkerError {
    fn from(error: io::Error) -> Self {
        Self::IoError(error)
    }
}
