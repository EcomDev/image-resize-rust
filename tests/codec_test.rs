extern crate image_resize_rust;
extern crate tokio_codec;

use image_resize_rust::{CommandsCodec, Command};
use bytes::{BytesMut, BufMut};
use tokio_codec::{Decoder, Encoder};

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