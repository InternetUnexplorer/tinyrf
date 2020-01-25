use std::fmt;

use serde::{Deserialize, Serialize};

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
