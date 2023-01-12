use std::fmt;
use std::fmt::Formatter;

struct WebsocketError;

impl fmt::Display for WebsocketError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Websocket error")
    }
}

#[derive(Debug)]
pub enum DecodePayloadErrors {
    IoError,
    ParseError,
    
}

#[derive(Debug)]
pub enum HandshakeErrors {
    InvalidMethodError,
    ParseError,
    InvalidHeaders,
    IoError
}
