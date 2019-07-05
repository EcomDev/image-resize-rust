use tokio::prelude::*;
use futures::prelude::*;
use serde_json::Value as JsonValue;
use std::io;
use tokio::codec::Framed;
use crate::json_codec::JsonCodec;
use serde::de::Error;
use tokio::sync::mpsc::Receiver;

enum Connection {
    InputError(io::Error),
    InputCommand(JsonValue),
    OutputEvent(JsonValue)
}

fn create_stream<S: Stream + Sized, U: Sized>(
    read_stream: S<Item=JsonValue, Error=io::Error>,
    event_stream: S<Item=JsonValue, Error=()>) -> S<Item=Connection, Error=()>
{
    stream::iter(vec![])
}

#[cfg(test)]
mod stream_merger
{
    use super::*;

    #[test]
    fn test_it_works() {
        let stream = create_stream(
           stream::iter(
               vec![]
           ),
           stream::iter(
               vec![]
           )
        );

        stream.wait().collect();
    }
}