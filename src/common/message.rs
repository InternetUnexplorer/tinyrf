use crate::common::render_task::RenderTask;
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
    /// Rendering finished, ready to upload
    RenderFinished,
    /// Rendering failed
    RenderFailed,
}
