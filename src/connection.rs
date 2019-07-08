use futures::stream::Stream;
use futures::future;
use tokio::prelude::*;
use std::io;

#[derive(Debug, PartialEq)]
pub enum Message<T: Sized> {
    Input(T),
    Output(T),
}

#[derive(Debug, PartialEq)]
pub struct SinkError(String);

#[derive(Debug, PartialEq)]
pub struct RoutedSink<T, I, O>
    where I: Sink<SinkItem=T, SinkError=SinkError> + Sized,
          O: Sink<SinkItem=T, SinkError=SinkError> + Sized
{
    input: I,
    output: O,
    state: RoutedSinkState<T>
}

#[derive(Debug, PartialEq)]
enum RoutedSinkState<T>
{
    Ready,
    Send(Message<T>),
    WaitInput,
    WaitOutput
}

impl<T, I, O> RoutedSink<T, I, O>
    where I: Sink<SinkItem=T, SinkError=SinkError> + Sized,
          O: Sink<SinkItem=T, SinkError=SinkError> + Sized
{
    pub fn new(input: I, output: O) -> Self
    {
        RoutedSink {
            input, output, state: RoutedSinkState::Ready
        }
    }

    pub fn send(self, message: Message<T>) -> Self {
        Self {
            input: self.input,
            output: self.output,
            state: RoutedSinkState::Send(message)
        }
    }
}

impl<T, I, O> Future for RoutedSink<T, I, O>
    where I: Sink<SinkItem=T, SinkError=SinkError> + Sized,
          O: Sink<SinkItem=T, SinkError=SinkError> + Sized
{

    type Item = Self;
    type Error = ();

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        unimplemented!()
    }
}

pub fn combine_stream<S1, S2, T>(input: S1, output: S2)
                                 -> impl Stream<Item=Message<T>, Error=()>
    where S1: Stream<Item=T, Error=T> + Sized,
          S2: Stream<Item=T, Error=()> + Sized
{
    input.map(Message::Input)
        .or_else(|v| future::ok(Message::Output(v)))
        .select(
            output
                .map(|v| Message::Output(v))
                .map_err(| _ | ())
        )
}

#[cfg(test)]
mod stream_combinator
{
    use futures::prelude::*;
    use futures::stream;
    use serde_json::{json, Value as JsonValue};

    use super::*;

    #[test]
    fn empty_stream() {
        let empty_stream = stream::empty::<JsonValue, JsonValue>();

        let result = combine_stream(
            empty_stream,
            stream::empty(),
        );

        assert_eq!(Ok(vec![]), result.collect().wait());
    }

    #[test]
    fn transforms_input_stream_values_into_connection_input() {
        let result = combine_stream(
            stream::iter_ok(vec![
                json!({"command": "command1"}),
                json!({"command": "command2"})
            ]),
            stream::empty(),
        );

        assert_eq!(
            Ok(
                vec![
                    Message::Input(json!({"command": "command1"})),
                    Message::Input(json!({"command": "command2"})),
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
        let result = combine_stream(
            stream::iter_result(vec),
            stream::empty(),
        );

        assert_eq!(
            Ok(
                vec![
                    Message::Output(json!({"error": "bad!"})),
                    Message::Output(json!({"error": "bad2!"})),
                    Message::Output(json!({"error": "bad3!"})),
                ]
            ),
            result.collect().wait()
        );
    }

    #[test]
    fn transforms_output_stream_items_into_connection_output() {
        let result = combine_stream(
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
                    Message::Output(json!({"event": "event1"})),
                    Message::Output(json!({"event": "event2"})),
                    Message::Output(json!({"event": "event3"})),
                ]
            ),
            result.collect().wait()
        );
    }

    #[test]
    fn round_robins_events_from_both_streams() {
        let result = combine_stream(
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
                    Message::Input(json!({"input": "one"})),
                    Message::Output(json!({"event": "one"})),
                    Message::Input(json!({"input": "two"})),
                    Message::Output(json!({"event": "two"})),
                    Message::Output(json!({"error": "bad"})),
                    Message::Output(json!({"event": "three"})),
                    Message::Input(json!({"input": "three"})),
                ]
            ),
            result.collect().wait()
        );
    }
}

#[cfg(test)]
mod combined_sink
{
    use super::*;

    use tokio::sync::mpsc::*;
    


    #[test]
    fn writes_to_input_sink_when_input_message_is_send () {

        let (input_sink, input_stream) = unbounded_channel::<String>();
        let (output_sink, output_stream) = channel::<String>(100);

        let input_sink = input_sink.sink_map_err(|e| SinkError(e.to_string()));
        let output_sink = output_sink.sink_map_err(|e| SinkError(e.to_string()));

        let mut sink = RoutedSink::new(
            input_sink,
            output_sink
        );


    }
}
