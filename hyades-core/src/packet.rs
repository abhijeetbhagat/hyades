use crc32c;
use self::chunk::Chunk;

struct CommonHeader {
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

struct Packet {
    header: CommonHeader,
    chunks: Vec<Box<dyn Chunk>>,
}

impl Packet {
    fn new(src_port: u16, dst_port: u16) -> Self {
        let mut header = CommonHeader::default();
        header.src_port = src_port;
        header.dst_port = dst_port;
        let mut packet = Self {
            header,
            chunks: Vec::new(),
        };
        let checksum = crc32c::crc32c(packet.to_bytes());
        packet.header.checksum = checksum;
        packet
    }

    pub fn to_bytes(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                (&self as *const Packet) as *const u8,
                std::mem::size_of::<Packet>(),
            )
        }
    }

    pub fn add_chunk(&mut self, chunk: Chunk) {
        self.chunks.push(chunk);
    }
}
