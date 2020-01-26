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

/// A message sent during file transfer
#[derive(Debug, Deserialize, Serialize)]
pub(crate) enum TransferMessage {
    /// Ready to receive
    RecvReady { offset: u64, has_compression: bool },
    /// Ready to send
    SendReady { length: u64, use_compression: bool },
}
