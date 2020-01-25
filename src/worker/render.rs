use crate::common::render_task::RenderTask;
use failure::Fail;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitStatus, Stdio};

pub(super) type RenderResult<T> = Result<T, RenderError>;

#[derive(Fail, Debug)]
pub(super) enum RenderError {
    #[fail(display = "error starting render process: {}", 0)]
    InitError(#[fail(cause)] io::Error),
    #[fail(display = "render process exited with error: {}", 0)]
    ExitError(ExitStatus),
    #[fail(display = "render output missing")]
    OutputError,
}

pub(super) fn render(task: &RenderTask, project_file: &Path) -> RenderResult<PathBuf> {
    // Use the frame number as the output file name (four digits, zero-padded)
    let output_file_name = format!("####.{}", task.output_ext);

    // Create and configure the render process
    let mut command = Command::new("blender");

    // See https://docs.blender.org/manual/en/latest/advanced/command_line/arguments.html
    command
        .arg("--background")
        .arg("--render-output")
        .arg(&project_file.with_file_name(output_file_name))
        .arg("--render-frame")
        .arg(&task.frame.to_string())
        .arg("--")
        .arg(&project_file);

    // Discard output
    command.stdout(Stdio::null()).stderr(Stdio::null());

    // Spawn the process and wait for it to exit
    let status = command.spawn()?.wait()?;

    // Check for a nonzero status code
    if !status.success() {
        Err(RenderError::ExitError(status))?;
    }

    // Check whether the output file exists
    let output_file_name = format!("{:04}.{}", task.frame, task.output_ext);
    let output_file = project_file.with_file_name(output_file_name);

    if output_file.is_file() {
        Ok(output_file)
    } else {
        Err(RenderError::OutputError)
    }
}

impl From<io::Error> for RenderError {
    fn from(error: io::Error) -> Self {
        Self::InitError(error)
    }
}
