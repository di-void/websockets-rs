mod http;
use http::handle_handshake;
mod websocket;
use smol::{io, net::TcpListener, prelude::*};
use websocket::start_websocket_session;

fn main() -> io::Result<()> {
    smol::block_on(async {
        let listener = TcpListener::bind("127.0.0.1:8080").await?;
        println!("Server listening on port 8080");
        let mut incoming = listener.incoming();

        // accept connections
        while let Some(stream) = incoming.next().await {
            let mut stream = stream?;
            println!("New Connection: {}", stream.peer_addr()?);
            let mut buffer = [0u8; 1024]; // 1kb

            match stream.read(&mut buffer).await {
                Ok(0) => {
                    // Connection was closed by the client
                    println!("Client disconnected: {}", stream.peer_addr()?);
                }
                Ok(n) => {
                    let result = handle_handshake(&buffer[..n]);

                    match result {
                        Ok(response) => {
                            stream.write_all(response.as_bytes()).await?;

                            // start websocket session
                            let task = smol::spawn(async move {
                                println!("Starting websocket session..");
                                start_websocket_session(stream).await;
                                println!("Websocket session has ended!");
                            });

                            task.detach();
                        }
                        Err(response) => {
                            stream.write_all(response.as_bytes()).await?;
                        }
                    }
                }
                Err(e) => {
                    println!("Failed to read from connection: {}", e);
                }
            }
        }

        Ok(())
    })
}
