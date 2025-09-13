use std::io::{Read, Write};
mod frame;
use frame::{Message, OpCode, StatusCode, build_close_frame_payload, build_message, parse_frame};

pub fn start_websocket_session<T: Read + Write>(mut stream: T) {
    let mut inbound_message = Message { frames: vec![] };
    let mut buf = [0; 1024]; // 1kb buffer

    loop {
        match stream.read(&mut buf) {
            Ok(0) => {
                println!("Client disconnected");
                break;
            }
            Ok(n) => {
                match parse_frame(&buf[..n]) {
                    Ok(frame) => {
                        match frame.opcode {
                            OpCode::Ping | OpCode::Pong => {
                                let mut msg = build_message(OpCode::Pong, &frame.payload);
                                let msg = msg.serialize();
                                let _ = stream.write_all(&msg);
                            }
                            OpCode::Close => {
                                // dispatch buffered message
                                let msg = inbound_message.serialize();
                                let _ = stream.write_all(&msg);
                                // echo close frame back to client
                                let msg = build_message(OpCode::Close, &frame.payload).serialize();
                                let _ = stream.write_all(&msg);
                                // break loop and end session
                                break;
                            }
                            OpCode::Unknown => {
                                println!("Unknown OpCode - terminating session");
                                // build close frame with encoded reason
                                let msg = build_message(
                                    OpCode::Close,
                                    &build_close_frame_payload(
                                        StatusCode::Unexpected,
                                        "Unknown OpCode",
                                    ),
                                )
                                .serialize();
                                // send back close frame
                                let _ = stream.write_all(&msg);
                                // break loop and end session
                                break;
                            }
                            _ => {
                                let is_fin = frame.fin;

                                // Add frame to message buffer
                                inbound_message.frames.push(frame);

                                // If this frame is fin process the complete message
                                if is_fin {
                                    let msg = inbound_message.serialize();
                                    // echo message back
                                    let _ = stream.write_all(&msg);
                                }
                            }
                        }
                    }
                    Err(e) => println!("Failed to parse frame: {}", e),
                }
            }
            Err(e) => {
                println!("Failed to read from connection: {}", e);
                break;
            }
        }
    }
}
