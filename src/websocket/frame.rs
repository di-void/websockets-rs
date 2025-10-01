// https://datatracker.ietf.org/doc/html/rfc6455#section-5
// https://websocket.org/guides/websocket-protocol/#data-framing

// Websocket frame:
// [FIN][RSV1][RSV2][RSV3][OpCode(4)][MASK][PayloadLen(7)][ExtendedLen(16/64?)][MaskingKey(32)?][PayloadData...]

pub struct Message {
    pub frames: Vec<Frame>,
}

impl Message {
    pub fn serialize(&mut self) -> Vec<u8> {
        let mut output = vec![];

        for frame in &mut self.frames {
            output.push(frame.serialize());
        }

        output.into_iter().flatten().collect::<Vec<u8>>()
    }
}

// #[derive(Clone)]
pub struct Frame {
    pub fin: bool,
    pub opcode: OpCode,
    pub mask: bool,
    pub payload_len: u64,
    pub payload: Vec<u8>,
}

impl Frame {
    // Websocket frame:
    // [FIN][RSV1][RSV2][RSV3][OpCode(4)][MASK][PayloadLen(7)][ExtendedLen(16/64?)][MaskingKey(32)?][PayloadData...]
    fn serialize(&mut self) -> Vec<u8> {
        use std::u16;

        let mut serialized_frame = vec![];
        let fin = self.fin as u8;
        let opcode = serialize_opcode(self.opcode);
        let header = (fin << 7) | opcode;
        serialized_frame.push(header);

        let mask = false as u8;
        let payload_len = if self.payload_len <= 125 {
            self.payload_len as u8
        } else {
            match u16::try_from(self.payload_len) {
                Ok(_) => 126,
                _ => 127,
            }
        };

        let mut extended_len_bytes = Vec::with_capacity(8);

        if payload_len == 126 {
            extended_len_bytes.extend(
                u16::try_from(self.payload_len)
                    .unwrap()
                    .to_be_bytes()
                    .iter(),
            )
        } else if payload_len == 127 {
            extended_len_bytes.extend(self.payload_len.to_be_bytes().iter())
        }

        let footer = mask | payload_len;
        serialized_frame.push(footer);
        serialized_frame.append(&mut extended_len_bytes);
        serialized_frame.append(&mut self.payload);

        serialized_frame
    }
}

#[derive(Copy, Clone)]
pub enum OpCode {
    Text,
    Continuation,
    Ping,
    Pong,
    Close,
    Unknown,
}

pub fn parse_frame(buf: &[u8]) -> Result<Frame, String> {
    if buf.len() < 2 {
        return Err("Buffer too small for header".to_string());
    }

    let header = &buf[..2];
    let part_one = header[0];
    let part_two = header[1];

    let is_fin = (part_one & 0b1000_0000) != 0;
    let op_code = part_one & 0b0000_1111;
    let is_masked = (part_two & 0b1000_0000) != 0;
    let mut payload_len = (part_two & 0b0111_1111) as u64;

    let mut offset = 2;

    if payload_len == 126 {
        if buf.len() < offset + 2 {
            return Err("Buffer too small for extended payload length (126)".to_string());
        }
        payload_len = u16::from_be_bytes([buf[offset], buf[offset + 1]]) as u64;
        offset += 2;
    } else if payload_len == 127 {
        if buf.len() < offset + 8 {
            return Err("Buffer too small for extended payload length (127)".to_string());
        }
        let bytes: [u8; 8] = buf[offset..offset + 8].try_into().unwrap();
        payload_len = u64::from_be_bytes(bytes);
        offset += 8;
    }

    let masking_key: Vec<u8> = if is_masked {
        if buf.len() < offset + 4 {
            return Err("Buffer too small for masking key".to_string());
        }

        let mut key = Vec::with_capacity(4);
        key.extend(buf[offset..offset + 4].iter());
        offset += 4;
        key
    } else {
        vec![]
    };

    if buf.len() < offset + payload_len as usize {
        return Err("Buffer too small for payload".to_string());
    }

    let mut payload: Vec<u8> = Vec::with_capacity(payload_len as usize);
    // unmask payload
    for i in 0..payload_len as usize {
        let mut byte = buf[offset + i];
        if is_masked {
            byte ^= masking_key[i % 4];
            payload.push(byte);
        } else {
            payload.push(byte);
        }
    }

    Ok(Frame {
        fin: is_fin,
        opcode: deserialize_opcode(op_code),
        mask: is_masked,
        payload_len,
        payload,
    })
}

fn deserialize_opcode(op: u8) -> OpCode {
    match op {
        0 => OpCode::Continuation,
        1 => OpCode::Text,
        8 => OpCode::Close,
        9 => OpCode::Ping,
        10 => OpCode::Pong,
        _ => OpCode::Unknown,
    }
}

fn serialize_opcode(opcode: OpCode) -> u8 {
    match opcode {
        OpCode::Continuation => 0,
        OpCode::Text => 1,
        OpCode::Close => 8,
        OpCode::Ping => 9,
        OpCode::Pong => 10,
        _ => unreachable!("Unknown OpCode"),
    }
}

pub fn build_message(opcode: OpCode, payload: &[u8]) -> Message {
    // payload length
    let payload_len = payload.len();
    let mut frames = vec![];
    let frame = Frame {
        fin: true,
        opcode,
        mask: false,
        payload_len: payload_len.try_into().unwrap(),
        payload: payload.to_vec(),
    };

    frames.push(frame);

    Message { frames }
}

pub enum StatusCode {
    Unexpected,
}

impl StatusCode {
    fn serialize(&self) -> u16 {
        match self {
            Self::Unexpected => 1011,
        }
    }
}

pub fn build_close_frame_payload(code: StatusCode, reason: &str) -> Vec<u8> {
    let mut payload = vec![];
    let status_code = code.serialize();
    payload.append(&mut status_code.to_be_bytes().to_vec());
    payload.append(&mut reason.as_bytes().to_vec());

    payload
}

// https://datatracker.ietf.org/doc/html/rfc6455#section-6.1
