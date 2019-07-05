use futures::stream::Stream;
use futures::future;

#[derive(Debug, PartialEq)]
pub enum Connection<T: Sized> {
    Input(T),
    Output(T),
}

#[derive(Debug, PartialEq)]
pub struct ConnectionError(());

pub fn merge_input_and_output_stream<S1, S2, T>(input: S1, output: S2)
                                                -> impl Stream<Item=Connection<T>, Error=ConnectionError>
    where S1: Stream<Item=T, Error=T> + Sized,
          S2: Stream<Item=T, Error=()> + Sized
{
    input.map(Connection::Input)
        .or_else(|v| future::ok(Connection::Output(v)))
        .select(
            output
                .map(|v| Connection::Output(v))
                .map_err(ConnectionError)
        )
}

#[cfg(test)]
mod stream_merger
{
    use futures::prelude::*;
    use futures::stream;
    use serde_json::{json, Value as JsonValue};

    use super::*;

    #[test]
    fn empty_stream() {
        let empty_stream = stream::empty::<JsonValue, JsonValue>();

        let result = merge_input_and_output_stream(
            empty_stream,
            stream::empty(),
        );

        assert_eq!(Ok(vec![]), result.collect().wait());
    }

    #[test]
    fn transforms_input_stream_values_into_connection_input() {
        let result = merge_input_and_output_stream(
            stream::iter_ok(vec![
                json!({"command": "command1"}),
                json!({"command": "command2"})
            ]),
            stream::empty(),
        );

        assert_eq!(
            Ok(
                vec![
                    Connection::Input(json!({"command": "command1"})),
                    Connection::Input(json!({"command": "command2"})),
                ]
            ),
            result.collect().wait()
        );
    }

    #[test]
    fn transforms_input_stream_errors_into_connection_output() {
        let vec: Vec<Result<JsonValue, JsonValue>> = vec![
            Err(json!({"error": "bad!"})),
            Err(json!({"error": "bad2!"})),
            Err(json!({"error": "bad3!"}))
        ];
        let result = merge_input_and_output_stream(
            stream::iter_result(vec),
            stream::empty(),
        );

        assert_eq!(
            Ok(
                vec![
                    Connection::Output(json!({"error": "bad!"})),
                    Connection::Output(json!({"error": "bad2!"})),
                    Connection::Output(json!({"error": "bad3!"})),
                ]
            ),
            result.collect().wait()
        );
    }

    #[test]
    fn transforms_output_stream_items_into_connection_output() {
        let result = merge_input_and_output_stream(
            stream::empty(),
            stream::iter_ok(vec![
                json!({"event": "event1"}),
                json!({"event": "event2"}),
                json!({"event": "event3"})
            ]),
        );

        assert_eq!(
            Ok(
                vec![
                    Connection::Output(json!({"event": "event1"})),
                    Connection::Output(json!({"event": "event2"})),
                    Connection::Output(json!({"event": "event3"})),
                ]
            ),
            result.collect().wait()
        );
    }

    #[test]
    fn round_robins_events_from_both_streams() {
        let result = merge_input_and_output_stream(
            stream::iter_result(vec![
                Ok(json!({"input": "one"})),
                Ok(json!({"input": "two"})),
                Err(json!({"error": "bad"})),
                Ok(json!({"input": "three"})),
            ]),
            stream::iter_ok(vec![
                json!({"event": "one"}),
                json!({"event": "two"}),
                json!({"event": "three"})
            ]),
        );

        assert_eq!(
            Ok(
                vec![
                    Connection::Input(json!({"input": "one"})),
                    Connection::Output(json!({"event": "one"})),
                    Connection::Input(json!({"input": "two"})),
                    Connection::Output(json!({"event": "two"})),
                    Connection::Output(json!({"error": "bad"})),
                    Connection::Output(json!({"event": "three"})),
                    Connection::Input(json!({"input": "three"})),
                ]
            ),
            result.collect().wait()
        );
    }
}