use crate::common::render_task::{RenderTask, RenderTaskResult};
use serde::{Deserialize, Serialize};

/// A message sent from the server to the worker
#[derive(Debug, Deserialize, Serialize)]
pub(crate) enum ServerMessage {
    /// Start a new render
    StartRender(RenderTask),
}

/// A message sent from the worker to the server
#[derive(Debug, Deserialize, Serialize)]
pub(crate) enum WorkerMessage {
    /// Connected and ready to render
    Init { name: Option<String> },
    /// Render task finished with result
    RenderResult(RenderTaskResult),
}
