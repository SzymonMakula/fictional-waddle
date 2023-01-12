use crate::consts::frames::{
    FIN_BITS, OPCODE_BINARY, OPCODE_BITS, OPCODE_CONTINUE, OPCODE_TEXT, PAYLOAD_LENGTH_BITS,
};
use crate::errors::websocket_error::{DecodePayloadErrors, HandshakeErrors};
use crate::handlers::http_server::encode_accept_key;
use crate::parsers::http_parsers::{get_http_headers, get_socket_key};
use std::io::ErrorKind;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub enum OPCODES {
    TEXT,
    CONTINUE,
    BINARY,
}

pub fn get_opcode(byte: u8) -> Option<OPCODES> {
    let opcode = byte & OPCODE_BITS;
    match opcode {
        OPCODE_TEXT => Some(OPCODES::TEXT),
        OPCODE_BINARY => Some(OPCODES::BINARY),
        OPCODE_CONTINUE => Some(OPCODES::CONTINUE),
        _ => None,
    }
}

pub fn get_fin_bit(byte: u8) -> u8 {
    byte & FIN_BITS
}

pub fn get_mask_bit(byte: u8) -> u8 {
    byte & FIN_BITS
}

pub fn get_payload_length_and_offset(buffer: &[u8]) -> Result<(u64, usize), DecodePayloadErrors> {
    let second_byte = buffer[1];
    let payload_length_frame = second_byte & PAYLOAD_LENGTH_BITS;

    match payload_length_frame {
        0..=125 => Ok((payload_length_frame as u64, 2)),
        126 => {
            let payload_frames: [u8; 2] = buffer[2..4].try_into().unwrap();
            let payload_length = u16::from_be_bytes(payload_frames);
            Ok((payload_length as u64, 4))
        }
        _ => {
            let payload_frames: [u8; 8] = buffer[2..10].try_into().unwrap();

            let payload_length = u64::from_be_bytes(payload_frames);
            if (buffer[2] & FIN_BITS) != FIN_BITS {
                return Err(DecodePayloadErrors::ParseError);
            }
            Ok((payload_length, 10))
        }
    }
}

pub fn decode_payload(buffer: &[u8]) -> Result<Vec<u8>, DecodePayloadErrors> {
    let (payload_length, payload_offset) = get_payload_length_and_offset(&buffer)?;

    let mask: [u8; 4] = buffer[payload_offset..4 + payload_offset]
        .try_into()
        .or(Err(DecodePayloadErrors::IoError))?;
    let encoded = buffer[payload_offset..payload_length as usize + payload_offset + 4].to_vec();

    let decoded: Vec<u8> = encoded
        .iter()
        .enumerate()
        .map(|(index, &value)| value ^ mask[index % 4])
        .skip(4)
        .collect();
    Ok(decoded)
}

pub enum PayloadLengthFrame {
    PayloadData,
    ExtensionData,
    ApplicationData,
}

pub fn get_message_length(length: usize) -> PayloadLengthFrame {
    match length {
        v if v as u8 as usize == v => PayloadLengthFrame::PayloadData,
        v if v as u16 as usize == v => PayloadLengthFrame::ExtensionData,
        _ => PayloadLengthFrame::ApplicationData,
    }
}

pub async fn handshake(stream: &mut TcpStream) -> Result<(), HandshakeErrors> {
    let mut buffer = [0; 1024];
    stream
        .read(&mut buffer)
        .await
        .or(Err(HandshakeErrors::IoError))?;
    if !buffer.starts_with(b"GET / HTTP/1.1\r\n") {
        return Err(HandshakeErrors::InvalidMethodError);
    }

    let string_data = std::str::from_utf8(&buffer).or(Err(HandshakeErrors::ParseError))?;

    let headers = get_http_headers(string_data);
    let socket_key = get_socket_key(headers).ok_or(HandshakeErrors::InvalidHeaders)?;
    let accept_socket_key = encode_accept_key(&socket_key);

    let response = format!("HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: {}\r\n\r\n", accept_socket_key);
    stream
        .write_all(response.as_bytes())
        .await
        .or(Err(HandshakeErrors::IoError))?;
    Ok(())
}
