use serde_json::{Value, json, to_string};

use std::string::String;
use tokio::codec::{LinesCodec, Encoder, Decoder};
use std::io;
use bytes::BytesMut;

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

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct CommandsCodec {
    line_codec: LinesCodec
}

impl CommandsCodec
{
    pub fn new() -> Self {
        CommandsCodec {
            line_codec: LinesCodec::new()
        }
    }
}

impl Decoder for CommandsCodec
{
    type Item = Command;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Command>, io::Error> {
        match self.line_codec.decode(buf) {
            Ok(Some(value)) => Ok(Some(parse_json_string(value))),
            Ok(None) => Ok(None),
            Err(value) => Err(value)
        }
    }

    fn decode_eof(&mut self, buf: &mut BytesMut) -> Result<Option<Command>, io::Error> {
        match self.line_codec.decode_eof(buf) {
            Ok(Some(value)) => Ok(Some(parse_json_string(value))),
            Ok(None) => Ok(None),
            Err(value) => Err(value)
        }
    }
}

impl Encoder for CommandsCodec {
    type Item = Event;
    type Error = io::Error;

    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), Self::Error> {
        self.line_codec.encode(item.to_string(), dst)
    }
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

impl Event {
    pub fn found(path: &str) -> Event {
        Event::ImageFound { path: String::from(path) }
    }

    pub fn not_found(path: &str) -> Event {
        Event::ImageNotFound { path: String::from(path) }
    }

    pub fn resize_complete(target: &str) -> Event {
        Event::ImageResizeComplete { target: String::from(target) }
    }

    pub fn resize_failed(target: &str, reason: &str) -> Event {
        Event::ImageResizeFailed { target: String::from(target), reason: String::from(reason) }
    }

    pub fn to_string(&self) -> String {
        let json = match self {
            Event::ImageFound { path } => json!({
                "event": "found",
                "path": path
            }),
            Event::ImageNotFound { path } => json!({
                "event": "not_found",
                "path": path
            }),
            Event::ImageResizeComplete { target } => json!({
                "event": "resize_completed",
                "target": target
            }),
            Event::ImageResizeFailed { target, reason } => json!({
                "event": "resize_failed",
                "target": target,
                "reason": reason
            }),
        };

        to_string(&json).unwrap()
    }
}

fn parse_json_string(json: String) -> Command {
    match serde_json::from_str(&json) {
        Ok(value) => create_command_from_parsed_json_value(value),
        _ => Command::WrongCommand
    }
}

fn create_command_from_parsed_json_value(json: Value) -> Command {
    match json["command"].as_str() {
        Some("find") => match &json["path"] {
            Value::String(path) => Command::FindImage {
                path: path.to_string()
            },
            _ => Command::WrongCommand
        },
        Some("resize") => Command::ResizeImage {
            source: String::from(json["source"].as_str().unwrap()),
            sizes: collect_image_sizes_from_json(json),
        },
        _ => Command::WrongCommand
    }
}

fn collect_image_sizes_from_json(json: Value) -> Vec<(String, u32, u32)> {
    match &json["sizes"] {
        Value::Array(list_sizes) => list_sizes
            .into_iter()
            .filter_map(parse_size_item)
            .collect(),
        _ => vec![]
    }
}

fn parse_size_item(size: &Value) -> Option<(String, u32, u32)> {
    match size.as_array() {
        Some(sizes) => match &sizes[..] {
            [target, width, height] if is_size_tuple(target, width, height) => Some((
                String::from(target.as_str().unwrap()),
                width.as_u64()? as u32,
                height.as_u64()? as u32)),
            _ => None
        }
        _ => None
    }
}

fn is_size_tuple(target: &Value, width: &Value, height: &Value) -> bool {
    return target.is_string() && width.is_number() && height.is_number();
}

#[cfg(test)]
mod parser {
    use super::{parse_json_string, Command};

    #[test]
    fn creates_find_image_command_from_json() {
        assert_eq!(
            Command::FindImage {
                path: String::from("/path/to/file")
            },
            parse_json_string(
                String::from("{\"command\":\"find\",\"path\":\"/path/to/file\"}")
            )
        )
    }

    #[test]
    fn when_type_is_wrong_creates_wrong_command() {
        assert_eq!(
            Command::WrongCommand,
            parse_json_string(
                String::from("{\"command\":\"something\"}")
            )
        )
    }

    #[test]
    fn when_type_is_not_provided_creates_wrong_command() {
        assert_eq!(Command::WrongCommand, parse_json_string(
            String::from("{}")
        ));
    }

    #[test]
    fn when_path_is_missing_for_find_command_creates_wrong_command() {
        assert_eq!(Command::WrongCommand, parse_json_string(
            String::from("{\"command\": \"find\"}")
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
            parse_json_string(
                String::from(
                    "{\
                    \"command\":\"resize\", \
                    \"source\":\"path/to/image.jpg\", \
                    \"sizes\": [[\"path/to/320x400/image.jpg\", 320, 400]]\
                }"
                )
            )
        );
    }

    #[test]
    fn when_one_of_the_resize_items_is_invalid_it_ignores_it() {
        assert_eq!(
            Command::ResizeImage {
                source: String::from("path/to/image.jpg"),
                sizes: vec![
                    (String::from("path/to/320x400/image.jpg"), 320, 400),
                    (String::from("path/to/400x500/image.jpg"), 400, 500),
                ],
            },
            parse_json_string(
                String::from(
                    "{\
                    \"command\":\"resize\", \
                    \"source\":\"path/to/image.jpg\", \
                    \"sizes\": [\
                        [\"path/to/320x400/image.jpg\", 320, 400],\
                        [\"path/to/320x400/image.jpg\", 320],\
                        [\"path/to/400x500/image.jpg\", 400, 500]\
                    ]\
                }"
                )
            )
        );
    }

    #[test]
    fn when_empty_resize_list_creates_empty_resize_list_command() {
        assert_eq!(
            Command::ResizeImage {
                source: String::from("path/to/image.jpg"),
                sizes: vec![],
            },
            parse_json_string(
                String::from(
                    "{\
                    \"command\":\"resize\", \
                    \"source\":\"path/to/image.jpg\", \
                    \"sizes\": []\
                }"
                )
            )
        );
    }

    #[test]
    fn when_invalid_json_provided_creates_wrong_command_instance() {
        assert_eq!(Command::WrongCommand, parse_json_string(String::from("{invalidjson}")));
    }
}

#[cfg(test)]
mod serializer {
    use super::Event;

    #[test]
    fn converts_event_for_found_image_into_json_string() {
        assert_eq!(
            String::from("{\"event\":\"found\",\"path\":\"/file/path.jpg\"}"),
            Event::ImageFound {
                path: String::from("/file/path.jpg")
            }.to_string()
        )
    }

    #[test]
    fn converts_event_for_not_found_image_into_json_string() {
        assert_eq!(
            String::from("{\"event\":\"not_found\",\"path\":\"/file/path2.jpg\"}"),
            Event::ImageNotFound {
                path: String::from("/file/path2.jpg")
            }.to_string()
        )
    }

    #[test]
    fn converts_event_completed_resize_into_json_string() {
        assert_eq!(
            String::from("{\"event\":\"resize_completed\",\"target\":\"/file/path3.jpg\"}"),
            Event::ImageResizeComplete {
                target: String::from("/file/path3.jpg")
            }.to_string()
        )
    }

    #[test]
    fn converts_event_failed_resize_into_json_string() {
        assert_eq!(
            String::from("{\"event\":\"resize_failed\",\"reason\":\"Something is wrong\",\"target\":\"/file/path4.jpg\"}"),
            Event::resize_failed("/file/path4.jpg", "Something is wrong").to_string()
        )
    }
}