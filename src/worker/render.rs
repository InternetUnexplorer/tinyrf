use crate::common::file::{get_output_file, get_project_file};
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

pub(super) fn render(task: &RenderTask, working_dir: &Path) -> RenderResult<PathBuf> {
    // Get the project and output files for the render task
    let project_file = get_project_file(working_dir, &task.project_uuid);
    let output_file = get_output_file(working_dir, task);

    // Create and configure the render process
    let mut command = Command::new("blender");

    // See https://docs.blender.org/manual/en/latest/advanced/command_line/arguments.html
    command
        .arg("--background")
        .arg(&project_file)
        .arg("--render-output")
        .arg(&output_file.with_file_name(format!("####.{}", task.output_ext)))
        .arg("--render-frame")
        .arg(&task.frame.to_string());

    // Discard output
    command.stdout(Stdio::null()).stderr(Stdio::null());

    // Spawn the process and wait for it to exit
    let status = command.spawn()?.wait()?;

    // Check for a nonzero status code
    if !status.success() {
        Err(RenderError::ExitError(status))?;
    }

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
