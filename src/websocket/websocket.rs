use crate::consts::frames::FIN_BITS;
use std::future::Future;

use crate::errors::status_codes::StatusCodes;
use crate::errors::websocket_error::DecodePayloadErrors;
use crate::parsers::websocket_server::{
    decode_payload, get_mask_bit, get_message_length, get_opcode, PayloadLengthFrame,
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub struct WebSocket {
    stream: TcpStream,
    buffer: [u8; 1024],
}

impl WebSocket {
    pub fn new(stream: TcpStream) -> WebSocket {
        WebSocket {
            stream,
            buffer: [0; 1024],
        }
    }

    async fn write_to_stream(&mut self, buffer: &[u8]) {
        self.stream.write_all(buffer).await.unwrap();
    }

    async fn read_stream(&mut self) -> usize {
        self.stream.read(&mut self.buffer).await.unwrap()
    }

    pub async fn send_message(&mut self, message: &[u8]) {
        let len = message.len();
        let payload_header = WebSocket::get_payload_header(len).await;

        let first_byte = [0b1000_0010];
        self.stream.write_all(&first_byte).await.unwrap();
        self.stream.write_all(&payload_header).await.unwrap();
        self.stream.write_all(message).await.unwrap();
    }
    pub async fn send_message_as_text(&mut self, message: &str) {
        let len = message.len();
        let payload_header = WebSocket::get_payload_header(len).await;

        let first_byte = [0b1000_0001];
        self.stream.write_all(&first_byte).await.unwrap();
        self.stream.write_all(&payload_header).await.unwrap();
        self.stream.write_all(message.as_bytes()).await.unwrap();
    }

    async fn get_payload_header(message_length: usize) -> Vec<u8> {
        let payload_length_frame = get_message_length(message_length);

        match payload_length_frame {
            PayloadLengthFrame::PayloadData => {
                vec![message_length as u8]
            }
            PayloadLengthFrame::ExtensionData => {
                let first_byte: u8 = 126;
                let payload_frame = (message_length as u16).to_be_bytes().to_vec();
                [vec![first_byte], payload_frame].concat()
            }
            PayloadLengthFrame::ApplicationData => {
                let first_byte: u8 = 127;
                let payload_frame = message_length.to_be_bytes().to_vec();
                [vec![first_byte], payload_frame].concat()
            }
        }
    }

    async fn close_connection(&mut self, close_reason: StatusCodes) {
        self.write_to_stream(&[0b1000_1000]).await;
        let code = close_reason as u16;
        let header_payload = WebSocket::get_payload_header(code.to_be_bytes().len()).await;
        self.write_to_stream(&header_payload).await;
        self.write_to_stream(&code.to_be_bytes()).await
    }

    // on_message: fn(&[u8]
    pub async fn listen_for_messages<F, T>(&mut self, on_message: &F)
    where
        F: Fn(Vec<u8>) -> T,
        T: Future,
    {
        loop {
            let size = self.read_stream().await;
            if size == 0 {
                continue;
            }
            let first_byte = self.buffer[0];
            let opcode = get_opcode(first_byte);
            if opcode.is_none() {
                self.close_connection(StatusCodes::PROTOCOL_ERROR).await;
                break;
            }

            let second_byte = self.buffer[1];
            let mask_bit = get_mask_bit(second_byte);
            if mask_bit != FIN_BITS {
                self.close_connection(StatusCodes::PROTOCOL_ERROR).await;
                break;
            }

            let decoded = decode_payload(&self.buffer);
            match decoded {
                Ok(data) => {
                    let text = std::str::from_utf8(&data).unwrap();
                    println!("{}", text);
                    on_message(data.to_vec()).await;
                    self.send_message_as_text("hello").await;
                }
                Err(err) => match err {
                    DecodePayloadErrors::IoError => {
                        self.close_connection(StatusCodes::SERVER_DOWN).await;
                    }
                    DecodePayloadErrors::ParseError => {
                        self.close_connection(StatusCodes::PROTOCOL_ERROR).await;
                    }
                },
            }
        }
    }
}
