use tokio::prelude::*;
use futures::prelude::*;
use futures::stream::Stream;

#[derive(Debug, PartialEq)]
enum Connection<T: Sized,E: Sized>  {
    InputError(E),
    InputCommand(T)
}

#[derive(Debug, PartialEq)]
struct ConnectionError(());

fn create_merged_input_stream<S1: Stream + Sized>(input_stream: S1)
    -> impl Stream<Item=Connection<S1::Item, S1::Error>, Error=ConnectionError>
{
    input_stream.map(|v| Connection::InputCommand(v))
        .or_else( |e| future::ok(Connection::InputError(e)))
}

#[cfg(test)]
mod stream_merger
{
    use super::*;
    use serde_json::{Value as JsonValue, json};
    use futures::stream::Empty;

    #[test]
    fn test_it_merges_into_empty_stream() {
        let empty_stream = stream::empty::<JsonValue, ()>();

        let result = create_merged_input_stream(
            empty_stream
        );

        assert_eq!(Ok(vec![]), result.collect().wait());
    }

    #[test]
    fn returns_commands_wrapped_into_connection_input_command() {
        let result = create_merged_input_stream(
            stream::iter_ok::<_, ()>(vec![
                json!({"command": "command1"}),
                json!({"command": "command2"})
            ])
        );

        assert_eq!(
            Ok(
                vec![
                    Connection::InputCommand(json!({"command": "command1"})),
                    Connection::InputCommand(json!({"command": "command2"})),
                ]
            ),
            result.collect().wait()
        );
    }

    #[test]
    fn combines_errors_with_input_command_into_one_stream_of_data () {
        let vec: Vec<Result<JsonValue,()>> = vec![
            Ok(json!({"command": "command1"})),
            Err(()),
            Err(()),
            Ok(json!({"command": "command2"})),
            Err(()),
            Ok(json!({"command": "command3"})),
        ];
        let result = create_merged_input_stream(
            stream::iter_result(vec)
        );

        assert_eq!(
            Ok(
                vec![
                    Connection::InputCommand(json!({"command": "command1"})),
                    Connection::InputError(()),
                    Connection::InputError(()),
                    Connection::InputCommand(json!({"command": "command2"})),
                    Connection::InputError(()),
                    Connection::InputCommand(json!({"command": "command3"})),
                ]
            ),
            result.collect().wait()
        );
    }
}