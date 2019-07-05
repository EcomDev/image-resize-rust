use serde_json::{Deserializer, Value as JsonValue, to_string, StreamDeserializer, Value};

use tokio::codec::{Decoder, Encoder};
use bytes::{BytesMut, BufMut};
use std::{io};
use serde_json::de::SliceRead;

#[derive(Debug, PartialEq)]
pub struct JsonCodec {

}

impl JsonCodec {
    pub fn new() -> Self {
        Self {}
    }
}

fn clear_until_new_line_or_eof(
    json_stream: StreamDeserializer<SliceRead, JsonValue>,
    bytes: &mut BytesMut
) {
    let new_line = bytes[..]
        .iter()
        .position(|b| *b == b'\n');

    match new_line {
        Some(pos) => bytes.advance(pos + 1),
        None => bytes.clear()
    }
}

fn advance_buffer(json_stream: StreamDeserializer<SliceRead, JsonValue>,
                  bytes: &mut BytesMut) {
    bytes.advance(json_stream.byte_offset());
}

impl Decoder for JsonCodec {
    type Item = JsonValue;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, io::Error> {

        let slice = src.to_vec();
        let mut json_stream = Deserializer::from_slice(&slice)
            .into_iter::<JsonValue>();

        let result = match json_stream.next() {
            Some(Ok(value)) => Ok(Some(value)),
            Some(Err(ref error)) if error.is_eof() => Ok(None),
            Some(Err(_))=> Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Error while parsing JSON structure"
            )),
            None => Ok(None)
        };

        match result {
            Err(_) => clear_until_new_line_or_eof(json_stream, &mut *src),
            Ok(_) => advance_buffer(json_stream,  &mut *src),
        };

        result
    }
}

impl Encoder for JsonCodec {
    type Item = JsonValue;
    type Error = io::Error;

    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), io::Error> {
        let json_string = to_string(&item)?;
        dst.reserve(json_string.len() + 1);
        dst.put(json_string);
        dst.put("\n");
        Ok(())
    }
}

#[cfg(test)]
fn create_codec_with_bytes() -> (BytesMut, JsonCodec) {
    (BytesMut::with_capacity(64), JsonCodec::new())
}

#[cfg(test)]
mod decoder
{
    use super::*;
    use tokio::io::ErrorKind;
    use serde_json::json;


    #[test]
    fn decodes_nothing_when_empty_buffer () {
        let (mut buffer, mut codec)  = create_codec_with_bytes();

        assert_eq!(None, codec.decode(&mut buffer).unwrap());
    }

    #[test]
    fn decodes_simple_array_from_json () {
        let (mut buffer, mut codec)  = create_codec_with_bytes();

        buffer.put("[1, 2, 3]");

        assert_eq!(Some(json!([1, 2, 3])), codec.decode(&mut buffer).unwrap());
    }

    #[test]
    fn advances_buffer_after_decoding_value () {
        let (mut buffer, mut codec)  = create_codec_with_bytes();

        buffer.put("[1, 2, 3]");
        buffer.put("[4, 5, 6]");

        codec.decode(&mut buffer);

        assert_eq!(json!([4, 5, 6]), codec.decode(&mut buffer).unwrap().unwrap());
    }

    #[test]
    fn ignores_whitespaces_in_buffer () {
        let (mut buffer, mut codec)  = create_codec_with_bytes();

        buffer.put("     [1, 2, 3]");

        assert_eq!(json!([1, 2, 3]), codec.decode(&mut buffer).unwrap().unwrap());
    }

    #[test]
    fn notifies_of_malformed_json() {

        let (mut buffer, mut codec)  = create_codec_with_bytes();

        buffer.put("{invalid}");

        assert_eq!("Error while parsing JSON structure", format!("{}", codec.decode(&mut buffer).err().unwrap()))
    }

    #[test]
    fn continues_parsing_json_after_errored_input()
    {
        let (mut buffer, mut codec)  = create_codec_with_bytes();

        buffer.put("{invalid}\n");
        buffer.put(r#"{"good":"data"}"#);

        codec.decode(&mut buffer);


        assert_eq!(json!({"good": "data"}), codec.decode(&mut buffer).unwrap().unwrap());
    }

    #[test]
    fn drains_buffer_from_errored_input()
    {
        let (mut buffer, mut codec)  = create_codec_with_bytes();

        buffer.put("{invalid}\n");
        buffer.put(r#"{"good":"data"}"#);

        codec.decode(&mut buffer);

        assert_eq!(r#"{"good":"data"}"#, buffer);
    }
}

#[cfg(test)]
mod encoder
{
    use super::*;
    use serde_json::json;

    #[test]
    fn writes_single_json_object() {
        let (mut buffer, mut codec) = create_codec_with_bytes();
        codec.encode(json!({"something": "happened"}), &mut buffer);

        assert_eq!(
            "{\"something\":\"happened\"}\n", buffer
        )
    }

    #[test]
    fn encodes_multiple_items () {
        let (mut buffer, mut codec) = create_codec_with_bytes();
        codec.encode(json!({"something": "happened1"}), &mut buffer);
        codec.encode(json!({"something": "happened2"}), &mut buffer);

        assert_eq!(
            "{\"something\":\"happened1\"}\n{\"something\":\"happened2\"}\n", buffer
        )
    }

    #[test]
    fn encodes_structures_larger_then_default_buffer_capacity () {
        let (mut buffer, mut codec) = create_codec_with_bytes();
        codec.encode(json!({"something": "happened1", "zdata": "that is too long"}), &mut buffer);
        codec.encode(json!({"something": "happened2", "zdata": "that is too long"}), &mut buffer);

        assert_eq!(
            "{\"something\":\"happened1\",\"zdata\":\"that is too long\"}\n\
            {\"something\":\"happened2\",\"zdata\":\"that is too long\"}\n",
            buffer
        )
    }
}