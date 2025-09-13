use std::io::{Read, Write};
use std::net::TcpListener;
mod http;
use http::handle_handshake;
mod websocket;
use websocket::start_websocket_session;

fn main() -> std::io::Result<()> {
    let listener = TcpListener::bind("127.0.0.1:8080")?;
    println!("Server listening on port 8080");

    // accept connections
    for stream in listener.incoming() {
        if let Ok(mut stream) = stream {
            println!("New connection: {}", stream.peer_addr()?);
            let mut buffer = [0; 1024]; // 1kb

            match stream.read(&mut buffer) {
                Ok(0) => {
                    // Connection was closed by the client
                    println!("Client disconnected: {}", stream.peer_addr()?);
                    break;
                }
                Ok(n) => {
                    let result = handle_handshake(&buffer[..n]);

                    match result {
                        Ok(response) => {
                            stream.write_all(response.as_bytes())?;

                            // start websocket
                            start_websocket_session(&mut stream);
                        }
                        Err(response) => {
                            stream.write_all(response.as_bytes())?;
                            break;
                        }
                    }
                }
                Err(e) => {
                    println!("Failed to read from connection: {}", e);
                }
            }
        } else {
            println!("Connection failed");
        }
    }

    Ok(())
}
