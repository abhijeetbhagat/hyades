#[derive(Clone, Debug)]
enum ChunkType {
    Init,
    InitAck,
    Data,
    CookieEcho,
}

trait Chunk {
    fn to_bytes(&self) -> &[u8];
}

struct ChunkHeader {
    chunk_type: u8,
    flags: u8,
    length: u16,
}

impl ChunkHeader {
    fn new(chunk_type: u8, flags: u8, length: u16) -> Self {
        Self {
            chunk_type,
            flags,
            length,
        }
    }
}

/*
        0                   1                   2                   3
        0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
       |   Type = 1    |  Chunk Flags  |      Chunk Length             |
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
       |                         Initiate Tag                          |
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
       |           Advertised Receiver Window Credit (a_rwnd)          |
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
       |  Number of Outbound Streams   |  Number of Inbound Streams    |
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
       |                          Initial TSN                          |
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
       \                                                               \
       /              Optional/Variable-Length Parameters              /
       \                                                               \
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
*/

#[derive(Clone, Debug)]
struct Init {
    header: ChunkHeader,
    init_tag: u32,
    a_rwnd: u32,
    num_ob_streams: u16,
    num_ib_streams: u16,
    init_tsn: u32,
    optional_params: Option<[u8]>,
}

impl Init {
    fn new(
        init_tag: u32,
        a_rwnd: u32,
        num_ob_streams: u16,
        num_ib_streams: u16,
        optional_params: Option<Vec<u8>>,
    ) -> Self {
        Self {
            header: ChunkHeader::new(1, 0, 20 + optional_params.map_or(0, |v| v.len())),
            init_tag,
            a_rwnd,
            num_ob_streams,
            num_ib_streams,
            init_tsn: init_tag,
            optional_params: None,
        }
    }
}

impl Chunk for Init {
    fn to_bytes(&self) -> &[u8] {
        todo!()
    }
}

#[derive(Clone, Debug)]
struct InitAck {
    header: ChunkHeader,
    init_tag: u32,
    a_rwnd: u32,
    num_ob_streams: u16,
    num_ib_streams: u16,
    init_tsn: u32,
    optional_params: Option<[u8]>,
}

impl InitAck {
    fn new(
        init_tag: u32,
        a_rwnd: u32,
        num_ob_streams: u16,
        num_ib_streams: u16,
        optional_params: Option<Vec<u8>>,
    ) -> Self {
        Self {
            header: ChunkHeader::new(1, 0, 20 + optional_params.map_or(0, |v| v.len())),
            init_tag,
            a_rwnd,
            num_ob_streams,
            num_ib_streams,
            init_tsn: init_tag,
            optional_params: None,
        }
    }
}

impl Chunk for InitAck {
    fn to_bytes(&self) -> &[u8] {
        todo!()
    }
}

