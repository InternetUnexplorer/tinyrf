use serde::{Deserialize, Serialize};
use std::fmt;
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

/// The result of a render task
pub(crate) type RenderTaskResult = Result<(), ()>;

#[derive(Debug, Deserialize, Serialize, Copy, Clone)]
pub(crate) enum FileExt {
    PNG,
}

impl fmt::Display for FileExt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let extension = match self {
            Self::PNG => "png",
        };
        write!(f, "{}", extension)
    }
}
