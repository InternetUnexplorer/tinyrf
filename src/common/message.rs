use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A message sent from the server to the worker
#[derive(Debug, Deserialize, Serialize)]
pub enum ServerMessage {
    /// Start a new render
    StartRender { animation: Uuid, frame: u32 },
    /// Cancel the current render
    CancelRender,
}

/// A message sent from the worker to the server
#[derive(Debug, Deserialize, Serialize)]
pub enum WorkerMessage {
    /// Connected and ready to receive
    Init { name: Option<String> },
    /// An error occurred while downloading
    DownloadFailed,
    /// An error occurred while uploading
    UploadFailed,
    /// Rendering finished
    RenderFinished,
}
