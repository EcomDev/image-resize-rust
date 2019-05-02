#![deny(warnings)]

extern crate tokio;
extern crate tokio_codec;

use tokio::codec::Decoder;
use tokio::net::TcpListener;
use tokio::prelude::*;
use tokio_codec::LinesCodec;
use image_resize_rust::parse_json_string;

use std::env;
use std::net::SocketAddr;

fn main() -> Result<(), Box<std::error::Error>> {

    let addr = env::args().nth(1).unwrap_or("127.0.0.1:9090".to_string());
    let addr = addr.parse::<SocketAddr>()?;


    let socket = TcpListener::bind(&addr)?;
    println!("Listening on: {}", addr);


    let done = socket
        .incoming()
        .map_err(|e| println!("failed to accept socket; error = {:?}", e))
        .for_each(move |socket| {

            let framed = LinesCodec::new().framed(socket);
            let (_writer, reader) = framed.split();

            let processor = reader
                .for_each(|line| {
                    let command = parse_json_string(line);
                    println!("Received: {:?}", command);
                    Ok(())
                })
                .and_then(|()| {
                    Ok(())
                })
                .then(|_| {
                    Ok(())
                });

            tokio::spawn(processor)
        });

    tokio::run(done);
    Ok(())
}
