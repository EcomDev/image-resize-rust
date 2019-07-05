use tokio;
use tempfile::Builder;
use tokio::prelude::*;
use futures::prelude::*;
use tokio::net::UnixListener;
use tokio::codec::Decoder;
use serde_json::{Value, json};
use lib::*;

fn main() {
    let dir = Builder::new().prefix("image-server").tempdir().unwrap();
    let sock_path = dir.path().join("connect.sock");

    println!("{}", sock_path.to_str().unwrap());

    let server = UnixListener::bind(&sock_path).unwrap();

    tokio::run(
        server.incoming()
            .map_err( |_| ())
            .for_each(
            |socket| {
                let (writer, reader) = JsonCodec::new().framed(socket).split();

                let reader = reader.map_err(|e| {
                    let message = format!("{}", e);
                    json!({"error": message})
                });

                let stream = combine_stream(reader, stream::empty());

                tokio::spawn(
                    stream.fold(writer, |writer, message| {
                        let response = match message {
                            Connection::Input(v) => v,
                            Connection::Output(v) => v
                        };

                        writer.send(response)
                            .and_then(|writer| future::ok(writer))
                            .map_err(|_| () )
                    })
                        .map_err(|_| ())
                        .map(| _ | ())
                )
            }
        )
        .then(|_| Ok(()))

    )
}