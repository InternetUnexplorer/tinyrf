mod render;

use crate::common::file::{get_project_file, init_working_dir};
use crate::common::message::{ServerMessage, WorkerMessage};
use crate::common::net::{read_file, read_json, write_file, write_json};
use failure::Fail;
use log::{error, info};
use std::fs;
use std::fs::remove_file;
use std::io;
use std::io::{BufReader, BufWriter};
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use uuid::Uuid;

pub(super) type WorkerResult<T> = Result<T, WorkerError>;

#[derive(Fail, Debug)]
pub(super) enum WorkerError {
    #[fail(display = "error initializing working directory: {}", 0)]
    WorkingDirError(#[fail(cause)] io::Error),
    #[fail(display = "error connecting to server: {}", 0)]
    ConnectError(#[fail(cause)] io::Error),
    #[fail(display = "I/O error: {}", 0)]
    IoError(#[fail(cause)] io::Error),
    #[fail(display = "error sending file: {}", 0)]
    SendFileError(#[fail(cause)] io::Error),
    #[fail(display = "error receiving file: {}", 0)]
    RecvFileError(#[fail(cause)] io::Error),
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

        info!("connecting to {}:{}...", address, port);

        // Attempt to open a connection to the server
        let stream = TcpStream::connect((address, port)).map_err(WorkerError::ConnectError)?;

        info!("connected to server.");

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
                info!("(1/4) downloading project \"{}\"...", task.project_name);
                self.download_project(&task.project_uuid)
                    .map_err(WorkerError::RecvFileError)?;
                // Render the frame
                info!("(2/4) rendering frame {}...", task.frame);
                match render::render(&task, &self.working_dir) {
                    Ok(output_file) => {
                        info!(
                            "(3/4) uploading \"{}\"...",
                            output_file.file_name().unwrap().to_string_lossy()
                        );
                        // Send the result to the server
                        self.write_message(WorkerMessage::RenderResult(Ok(())))?;
                        // Upload the output file
                        self.upload_output(&output_file)
                            .map_err(WorkerError::SendFileError)?;
                        info!("(4/4) upload complete.");
                    }
                    Err(error) => {
                        error!("render failed: {}", error);
                        // Send the result to the server
                        self.write_message(WorkerMessage::RenderResult(Err(())))?;
                    }
                }
            }
        }
        Ok(())
    }

    /// Download the project file for a render task
    fn download_project(&mut self, project_uuid: &Uuid) -> io::Result<()> {
        let project_file = get_project_file(&self.working_dir, project_uuid);
        // Create the parent directory if it does not exist
        let project_dir = project_file.parent().unwrap();
        if !project_dir.is_dir() {
            fs::create_dir(project_dir)?;
        }
        // TODO: allow for existing file to be reused (tell server to skip download)
        // Remove the file if it already exists
        if project_file.is_file() {
            remove_file(&project_file)?;
        }
        // Download the file
        read_file(&mut self.reader, &project_file)
    }

    /// Upload the output of a render
    fn upload_output(&mut self, output_file: &Path) -> io::Result<()> {
        // Upload the file
        write_file(&mut self.writer, output_file)?;
        // Remove the file after uploading
        Ok(fs::remove_file(output_file)?)
    }

    /// Read a message from the server (blocking)
    fn read_message(&mut self) -> io::Result<ServerMessage> {
        read_json(&mut self.reader)
    }

    /// Send a message to the server
    fn write_message(&mut self, message: WorkerMessage) -> io::Result<()> {
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
