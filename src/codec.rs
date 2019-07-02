use tokio::codec::{LinesCodec, Encoder, Decoder};
use std::io;
use bytes::BytesMut;

use crate::commands::{Command, create_command};
use crate::events::Event;

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
            Ok(Some(value)) => Ok(Some(create_command(value))),
            Ok(None) => Ok(None),
            Err(value) => Err(value)
        }
    }

    fn decode_eof(&mut self, buf: &mut BytesMut) -> Result<Option<Command>, io::Error> {
        match self.line_codec.decode_eof(buf) {
            Ok(Some(value)) => Ok(Some(create_command(value))),
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
#[cfg(test)]
mod tests
{
    use super::*;
    use bytes::{BytesMut, BufMut};

    #[test]
    fn decodes_none_commands_when_buffer_is_empty()
    {
        let mut codec = CommandsCodec::new();
        let bytes = &mut BytesMut::new();
        bytes.reserve(1);
        assert_eq!(None, codec.decode(bytes).unwrap());
    }

    #[test]
    fn decodes_invalid_command_line_as_wrong_command()
    {
        let mut codec = CommandsCodec::new();
        let bytes = &mut BytesMut::new();
        bytes.reserve(1000);
        bytes.put("{}\n");
        assert_eq!(Command::WrongCommand, codec.decode(bytes).unwrap().unwrap());
    }

    #[test]
    fn decodes_simple_find_command()
    {
        let mut codec = CommandsCodec::new();
        let bytes = &mut BytesMut::new();
        bytes.reserve(1000);
        bytes.put("{\"command\": \"find\", \"path\": \"file.txt\"}\n");
        assert_eq!(
            Command::FindImage {
                path: String::from("file.txt")
            },
            codec.decode(bytes).unwrap().unwrap()
        );
    }

    #[test]
    fn decodes_multiple_commands()
    {
        let mut codec = CommandsCodec::new();
        let bytes = &mut BytesMut::new();
        bytes.reserve(1000);
        bytes.put("{\"command\": \"find\", \"path\": \"file1.txt\"}\n");
        bytes.put("{\"command\": \"find\", \"path\": \"file2.txt\"}\n");

        let mut commands:Vec<Command> = vec![];
        commands.push(codec.decode(bytes).unwrap().unwrap());
        commands.push(codec.decode(bytes).unwrap().unwrap());

        assert_eq!(
            vec![
                Command::FindImage {
                    path: String::from("file1.txt")
                },
                Command::FindImage {
                    path: String::from("file2.txt")
                }
            ],
            commands
        );
    }

    #[test]
    fn propagates_decode_eof_into_line_codec () {
        let mut codec = CommandsCodec::new();
        let bytes = &mut BytesMut::new();
        bytes.reserve(1000);
        bytes.put("{\"command\": \"find\", \"path\": \"file1.txt\"}\n");
        bytes.put("{\"command\": \"find\", \"path\": \"file2.txt\"}");

        let mut commands:Vec<Command> = vec![];
        commands.push(codec.decode(bytes).unwrap().unwrap());
        commands.push(codec.decode_eof(bytes).unwrap().unwrap());

        assert_eq!(
            vec![
                Command::FindImage {
                    path: String::from("file1.txt")
                },
                Command::FindImage {
                    path: String::from("file2.txt")
                }
            ],
            commands
        );
    }

    #[test]
    fn encodes_event_as_json_line() {
        let mut codec = CommandsCodec::new();
        let mut buf = BytesMut::new();

        codec.encode(Event::found("file.jpg"), &mut buf).unwrap();
        assert_eq!("{\"event\":\"found\",\"path\":\"file.jpg\"}\n", buf);
    }
}
