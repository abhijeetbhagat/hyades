#[derive(Clone, Debug)]
enum ChunkType {
    Init,
    InitAck,
    Data,
    CookieEcho,
}

#[derive(Clone, Debug)]
struct Init {
    type: u8,
    flags: u8,
    length: u16,
    init_tag: u32,
    a_rwnd: u32,
    num_ob_streams: u16,
    num_ib_streams: u16,
    init_tsn: u32,
    optional_params: Option<[u8]>
}

impl Init {
    fn new() -> Self {
        Self { tsn: 0, chunk_type }
    }
}
