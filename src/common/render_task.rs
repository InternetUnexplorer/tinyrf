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

/// The extension of an output file of a render task
#[derive(Debug, Deserialize, Serialize, Copy, Clone)]
pub(crate) enum FileExt {
    BMP, // BMP
    RGB, // Iris
    PNG, // PNG
    JPG, // JPEG
    JP2, // JPEG 2000
    TGA, // Targa/Targa Raw
}

impl fmt::Display for FileExt {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::BMP => write!(f, "bmp"),
            Self::RGB => write!(f, "rgb"),
            Self::PNG => write!(f, "png"),
            Self::JPG => write!(f, "jpg"),
            Self::JP2 => write!(f, "jp2"),
            Self::TGA => write!(f, "tga"),
        }
    }
}
