pub const FIN_BITS: u8 = 0b1000_0000;
pub const OPCODE_BITS: u8 = 0b0000_1111;
pub const OPCODE_BINARY: u8 = 0b0000_0010;
pub const OPCODE_TEXT: u8 = 0b0000_0001;
pub const OPCODE_CONTINUE: u8 = 0b0000_0000;
pub const PAYLOAD_LENGTH_BITS: u8 = 0b0111_1111;
pub const MAGIC_STRING: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";
