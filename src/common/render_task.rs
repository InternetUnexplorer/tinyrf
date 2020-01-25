use crate::common::file_ext::FileExt;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub(crate) type Frame = u32;

/// Information about a frame that should be rendered and the project it belongs to
#[derive(Debug, Deserialize, Serialize, Clone)]
pub(crate) struct RenderTask {
    pub project_uuid: Uuid,
    pub project_name: String,
    pub frame: Frame,
    pub output_ext: FileExt,
}
