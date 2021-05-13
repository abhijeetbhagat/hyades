use crate::chunk::Chunk;
use crc32c;

pub struct CommonHeader {
    src_port: u16,
    dst_port: u16,
    ver_tag: u32,
    checksum: u32,
}

impl Default for CommonHeader {
    fn default() -> Self {
        Self {
            src_port: 0,
            dst_port: 0,
            ver_tag: 0,
            checksum: 0,
        }
    }
}

/// An SCTP Packet
pub struct Packet {
    header: CommonHeader,
    chunks: Vec<Box<dyn Chunk>>,
}

impl From<&Packet> for Vec<u8> {
    fn from(p: &Packet) -> Self {
        let mut v = Vec::with_capacity(p.chunks.len());
        for chunk in &p.chunks {
            v.extend(Vec::<u8>::from(chunk));
        }
        v
    }
}

impl Packet {
    /// Creates a new `Packet`
    pub fn new(src_port: u16, dst_port: u16) -> Self {
        let mut header = CommonHeader::default();
        header.src_port = src_port;
        header.dst_port = dst_port;
        let mut packet = Self {
            header,
            chunks: Vec::new(),
        };
        let checksum = crc32c::crc32c(&Vec::<u8>::from(&packet));
        packet.header.checksum = checksum;
        packet
    }

    /// Add a chunk to this packet
    pub fn add_chunk(&mut self, chunk: Box<dyn Chunk>) {
        self.chunks.push(chunk);
    }
}
