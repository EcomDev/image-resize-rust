use serde_json::{Deserializer, Value as JsonValue, json};

use tokio::codec::{Decoder, Encoder, Framed};
use bytes::{BytesMut, BufMut};
use tokio::prelude::*;
use std::{io};

#[derive(Debug, PartialEq)]
pub struct JsonCodec {

}

impl JsonCodec {
    pub fn new() -> Self {
        Self {}
    }
}

impl Decoder for JsonCodec {
    type Item = JsonValue;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, io::Error> {
        let mut json_stream = Deserializer::from_slice(&src[..])
            .into_iter::<JsonValue>();

        let result = match json_stream.next() {
            Some(Ok(value)) => Ok(Some(value)),
            Some(Err(_)) => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Error while parsing JSON structure"
            )),
            _ => Ok(None)
        };

        src.advance(json_stream.byte_offset());

        result
    }
}

impl Encoder for JsonCodec {
    type Item = JsonValue;
    type Error = io::Error;

    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<(), io::Error> {
        unimplemented!()
    }
}


#[cfg(test)]
mod decoder
{
    use super::*;
    use tokio::io::ErrorKind;

    fn create_codec_with_bytes() -> (BytesMut, JsonCodec) {
        (BytesMut::new(), JsonCodec::new())
    }


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

    fn continues_parsing_json_after_errored_input()
    {
        let (mut buffer, mut codec)  = create_codec_with_bytes();

        buffer.put("{invalid} ");
        buffer.put(r#"{"good":"data"}"#);

        codec.decode(&mut buffer);


        assert_eq!(json!({"good": "data"}), codec.decode(&mut buffer).unwrap().unwrap());
    }
}

#[cfg(test)]
mod encoder
{
    #[test]
    fn something () {

    }
}