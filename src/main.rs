use serde_json::{Result, Value, json};
use std::string::String;

#[derive(Debug, PartialEq)]
pub enum Command {
    FindImage {
        path: String,
    },
    ResizeImage {
        source: String,
        sizes: Vec<(String, u32, u32)>,
    },
    WrongCommand,
}

#[derive(Debug, PartialEq)]
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
        reason: String,
    },
}

pub fn create_from_json_string(json: String) -> Command {
    let json_structure: Value = serde_json::from_str(&json).unwrap();

    let command_type = &json_structure["type"];

    if command_type == "find" {
        return match &json_structure["path"] {
            Value::String(String) => Command::FindImage {
                path: String::from(json_structure["path"].as_str().unwrap())
            },
            _ => Command::WrongCommand
        };
    } else if command_type == "resize" {
        return Command::ResizeImage {
            source: String::from(json_structure["source"].as_str().unwrap()),
            sizes: vec![]
        }
    }

    Command::WrongCommand
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
                String::from("{\"type\":\"find\",\"path\":\"/path/to/file\"}")
            )
        )
    }

    #[test]
    fn when_type_is_wrong_creates_wrong_command() {
        assert_eq!(
            Command::WrongCommand,
            create_from_json_string(
                String::from("{\"type\":\"something\"}")
            )
        )
    }

    #[test]
    fn when_type_is_not_provided_creates_wrong_command() {
        assert_eq!(Command::WrongCommand, create_from_json_string(
            String::from("{}")
        ));
    }

    #[test]
    fn when_path_is_missing_for_find_command_creates_wrong_command() {
        assert_eq!(Command::WrongCommand, create_from_json_string(
            String::from("{\"type\": \"find\"}")
        ));
    }

    #[test]
    fn when_resize_image_type_is_provided_creates_resize_image_command() {
        assert_eq!(
            Command::ResizeImage {
                source: String::from("path/to/image.jpg"),
                sizes: vec![
                    (String::from("path/to/320x400/image.jpg"), 320, 400)
                ],
            },
            create_from_json_string(
                String::from(
                    "{\
                        \"type\":\"resize\", \
                        \"source\":\"path/to/image.jpg\", \
                        \"sizes\": [[\"path/to/320x400/image.jpg\", 320, 400]]\
                    }"
                )
            )
        );
    }
}