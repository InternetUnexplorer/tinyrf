use crate::common::render_task::RenderTask;
use log::debug;
use std::env::temp_dir;
use std::fs::create_dir_all;
use std::path::{Path, PathBuf};
use std::{io, process};
use uuid::Uuid;

/// Create and return a new working directory
pub(crate) fn init_working_dir(prefix: &str) -> io::Result<PathBuf> {
    // Append the PID to the working directory name to ensure it is unique
    let working_dir = temp_dir().join("render").join(format!("{}_{}", prefix, process::id()));
    // Create the directory (raises an error if the directory already exists)
    create_dir_all(&working_dir)?;

    debug!("Working directory set to {:?}", &working_dir);
    Ok(working_dir)
}

/// Get the path to the project file for the specified project
pub(crate) fn get_project_file(working_dir: &Path, project_uuid: &Uuid) -> PathBuf {
    working_dir.join(project_uuid.to_string()).join("project.blend")
}

/// Get the path to the output file for the specified render task
pub(crate) fn get_output_file(working_dir: &Path, render_task: &RenderTask) -> PathBuf {
    working_dir
        .join(render_task.project_uuid.to_string())
        .join(format!("{:04}.{}", render_task.frame, render_task.output_ext))
}
