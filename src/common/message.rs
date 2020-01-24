use serde::{Deserialize, Serialize};

/// A message sent from the server to the worker
#[derive(Debug, Deserialize, Serialize)]
pub enum ServerMessage {}

/// A message sent from the worker to the server
#[derive(Debug, Deserialize, Serialize)]
pub enum WorkerMessage {}
