struct CommonHeader {
    src_port: u16,
    dst_port: u16,
    ver_tag: u32,
    checksum: u32,
}

struct Packet {
    header: CommonHeader,
    control_chunks: Option<Vec<Chunk>>,
    data_chunks: Vec<Chunk>,
}

impl Packet {
    fn new() -> Self {
        Self
    }
}
