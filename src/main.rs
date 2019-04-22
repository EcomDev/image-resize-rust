#[macro_use]
extern crate serde_derive;
extern crate serde_json;

use serde::Deserialize;

#[derive(Debug,PartialEq)]
pub enum Command {
    FindImage {
        path: String,
    },
    ResizeImage {
        source: String,
        sizes: Vec<(String, u32, u32)>
    }
}

#[derive(Debug,PartialEq)]
pub enum Event {
    ImageFound {
        path: String
    },
    ImageNotFound {
        path: String
    },
    ImageResizeComplete {
        target: String
    },
    ImageResizeFailed {
        target: String,
        reason: String
    }
}

pub fn create_from_json_string(json: String) -> Command {
    Command::FindImage {
        path: "no-op".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_find_image_command_from_json() {
        assert_eq!(
            Command::FindImage {
                path: "/path/to/file".to_string()
            },
            create_from_json_string(
                "{\"type\":\"find\",\"path\":\"/path/to/file\"}".to_string()
            )
        )
    }
}