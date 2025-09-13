const HTTP_VERSION: f32 = 1.1;
const GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
const CODES: [(u16, &str); 3] = [
    (200, "Ok"),
    (101, "Switching protocols"),
    (404, "Bad Request"),
];

mod utils {
    use std::collections::HashMap;

    #[derive(Debug)]
    pub enum Method {
        Get,
        Other,
    }

    #[derive(Debug)]
    pub struct Request {
        uri: String,
        pub method: Method,
        pub headers: HashMap<String, String>,
    }

    pub fn parse_request(bytes: &[u8]) -> Request {
        let input_request = String::from_utf8(bytes.to_vec()).unwrap();
        let mut lines = input_request.lines();
        let mut request = Request {
            uri: String::new(),
            method: Method::Other,
            headers: HashMap::new(),
        };

        if let Some(line) = lines.next() {
            let lower_line = line.to_ascii_lowercase();
            let mut request_line_parts = lower_line.split(' ');

            let method = request_line_parts.next().unwrap();
            let uri = request_line_parts.next().unwrap();

            request.uri = uri.to_string();

            if method == "get" {
                request.method = Method::Get
            }
        }

        for line in lines {
            if line.is_empty() {
                break;
            }

            if let Some((key, value)) = line.split_once(':') {
                request
                    .headers
                    .insert(key.trim().to_ascii_lowercase(), value.trim().to_string());
            }
        }

        // body not useful
        request
    }

    pub struct ResponseBuilder {
        status_code: u16,
        headers: HashMap<String, String>,
        body: String,
    }

    impl ResponseBuilder {
        pub fn new() -> Self {
            ResponseBuilder {
                status_code: 200, // defaults to 200
                headers: HashMap::new(),
                body: String::new(),
            }
        }

        pub fn status(&mut self, status_code: u16) -> &mut Self {
            self.status_code = status_code;
            self
        }

        pub fn add_header(&mut self, key: &str, value: &str) -> &mut Self {
            self.headers
                .insert(key.to_ascii_lowercase(), value.to_string());
            self
        }

        pub fn set_body(&mut self, body: &str) -> &mut Self {
            self.body = body.to_string();
            self
        }

        fn stringify_headers(&self) -> String {
            let mut string_headers: String = String::new();

            for (key, val) in self.headers.iter() {
                let header_line = format!("{}: {}\r\n", key, val);
                string_headers.push_str(&header_line);
            }

            string_headers
        }

        pub fn to_string(&self) -> String {
            let status_text = super::CODES
                .iter()
                .find(|(code, _)| *code == self.status_code)
                .map(|(_, text)| *text)
                .unwrap_or("Unknown");

            let mut response = format!(
                "HTTP/{} {} {}\r\n",
                super::HTTP_VERSION,
                self.status_code,
                status_text
            );

            response.push_str(&self.stringify_headers());

            // Add Content-Length if we have a body
            if !self.body.is_empty() {
                response.push_str(&format!("Content-Length: {}\r\n", self.body.len()));
            }

            // Add blank line to separate headers from body
            response.push_str("\r\n");

            if !self.body.is_empty() {
                response.push_str(&self.body);
            }

            response
        }
    }
}

fn hash_websocket_key(key: &str, guid: &str) -> String {
    use base64::prelude::*;
    use sha1::{Digest, Sha1};
    let mut hasher = Sha1::new();
    let key_guid = format!("{}{}", key, guid);
    hasher.update(key_guid.as_bytes());
    let hash = hasher.finalize();
    BASE64_STANDARD.encode(&hash[..])
}

pub fn handle_handshake(buf: &[u8]) -> Result<String, String> {
    use utils::{Method, ResponseBuilder};

    let request = utils::parse_request(buf);
    println!("Request: {:#?}", request);

    let mut response_builder = ResponseBuilder::new();

    match request.method {
        Method::Get => {
            // check for upgrade and connection header
            let is_upgrade_request = request.headers.contains_key("upgrade")
                && request.headers.contains_key("connection");

            if is_upgrade_request {
                // if it is, send switch response
                if let Some(websocket_key) = request.headers.get("sec-websocket-key") {
                    let hash = hash_websocket_key(&websocket_key, GUID);

                    Ok(response_builder
                        .status(101)
                        .add_header("Upgrade", "websocket")
                        .add_header("Connection", "Upgrade")
                        .add_header("Sec-Websocket-Accept", &hash)
                        .to_string())
                } else {
                    Err(response_builder
                        .status(404)
                        .add_header("Server", "Deezy")
                        .to_string())
                }
            } else {
                // if not, send back ok
                Err(response_builder
                    .status(200)
                    .add_header("Server", "Deezy")
                    .add_header("Content-Type", "text/plain")
                    .set_body("Hello from Deezy server!")
                    .to_string())
            }
        }
        _ => Err(response_builder.add_header("Server", "Deezy").to_string()),
    }
}
