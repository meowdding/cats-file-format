pub const MAGIC_NUMBER: [u8; 4] = [0x43, 0x41, 0x54, 0x53];

#[derive(Debug)]
pub struct Header {
    pub version: u8,
    pub entries: Vec<Entry>,
}

#[derive(Debug, Clone)]
pub enum Entry {
    Directory {
        name: String,
        entries: Vec<Entry>,
    },
    File {
        name: String,
        offset: u32,
        size: u32,
        compression: Compression,
    },
}

#[derive(Debug, Clone)]
pub enum Compression {
    Gzip = 0xFE,
    None = 0xFF,
}
