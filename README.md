# websockets

A small, educational WebSocket server implemented in Rust using `smol` for async I/O.

This project implements a minimal WebSocket handshake and framing logic (RFC 6455) to demonstrate how WebSocket sessions, frames, and simple message echoing work.

## Highlights

- Minimal handshake implementation in `src/http.rs` (parses HTTP GET requests and performs the WebSocket upgrade).
- WebSocket frame parsing and building in `src/websocket/frame.rs` (supports text frames, continuation, ping/pong, and close frames).
- A simple server loop in `src/main.rs` that accepts TCP connections, performs the handshake, and spawns per-connection WebSocket sessions.
- No external WebSocket crates used — frame handling and masking are implemented by hand for learning purposes.

## Project layout

- `Cargo.toml` - crate metadata and dependencies (`smol`, `sha1`, `base64`).
- `src/main.rs` - TCP listener and handshake entry point; spawns WebSocket sessions.
- `src/http.rs` - HTTP request parsing and WebSocket handshake response generation.
- `src/websocket/mod.rs` - session loop that reads frames, handles control frames, and echoes messages.
- `src/websocket/frame.rs` - frame and message structs, frame parsing and serialization helpers, close payload builder.

## Building

Requires Rust toolchain (stable or nightly). From the project root run:

```powershell
# Build the project
cargo build --release
```

## Running

Run the server locally (binds to 127.0.0.1:8080):

```powershell
cargo run --release
```

You should see "Server listening on port 8080".

Use a WebSocket client (browser, `wscat`, or a simple JS snippet) to connect and test.

Example JS client (open in browser console and run):

```js
const ws = new WebSocket("ws://127.0.0.1:8080");
ws.onopen = () => console.log("open");
ws.onmessage = (ev) => console.log("message", ev.data);
ws.onclose = () => console.log("closed");
ws.onerror = (e) => console.error(e);
ws.send("Hello from browser");
```

Or use `wscat`:

```powershell
# Install wscat (requires Node.js)
npm install -g wscat
# Connect
wscat -c ws://127.0.0.1:8080
```

## Notes & Limitations

- This implementation is intentionally minimal and educational. It does not implement all aspects of the RFC (e.g., fragmented frames across multiple reads, strict validation of reserved bits, comprehensive error handling, and handling very large payloads efficiently).
- The server uses a fixed 1 KiB buffer for reading; for real-world usage you'd want a streaming implementation that can accumulate partial frames and handle larger payloads.
