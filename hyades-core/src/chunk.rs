use std::convert::TryFrom;

#[derive(Clone, Debug)]
enum ChunkType {
    Init,
    InitAck,
    Data,
    CookieEcho,
}

pub trait Chunk {
    fn get_bytes(&self) -> Vec<u8>;
}

impl From<&Box<dyn Chunk>> for Vec<u8> {
    fn from(b: &Box<dyn Chunk>) -> Self {
        b.get_bytes()
    }
}

#[derive(Clone, Debug)]
pub struct ChunkHeader {
    chunk_type: u8,
    flags: u8,
    length: u16,
}

impl ChunkHeader {
    pub fn new(chunk_type: u8, flags: u8, length: u16) -> Self {
        Self {
            chunk_type,
            flags,
            length,
        }
    }
}

impl From<&ChunkHeader> for [u8; 4] {
    fn from(ch: &ChunkHeader) -> Self {
        [
            ch.chunk_type,
            ch.flags,
            ((ch.length | 0x0000) >> 8) as u8,
            (ch.length & 0x00ff) as u8,
        ]
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
pub struct Init {
    header: ChunkHeader,
    init_tag: u32,
    a_rwnd: u32,
    num_ob_streams: u16,
    num_ib_streams: u16,
    init_tsn: u32,
    optional_params: Option<Vec<u8>>,
}

impl Init {
    pub fn new(
        init_tag: u32,
        a_rwnd: u32,
        num_ob_streams: u16,
        num_ib_streams: u16,
        optional_params: Option<Vec<u8>>,
    ) -> Self {
        Self {
            header: ChunkHeader::new(1, 0, 20 + optional_params.map_or(0, |v| v.len() as u16)),
            init_tag,
            a_rwnd,
            num_ob_streams,
            num_ib_streams,
            init_tsn: init_tag,
            optional_params: None,
        }
    }
}

impl From<&[u8; 1024]> for Init {
    fn from(buf: &[u8; 1024]) -> Self {
        Self {
            header: ChunkHeader::new(
                buf[0],
                buf[1],
                u16::from_be_bytes(<[u8; 2]>::try_from(&buf[2..=3]).unwrap()),
            ),
            init_tag: u32::from_be_bytes(<[u8; 4]>::try_from(&buf[4..=7]).unwrap()),
            a_rwnd: u32::from_be_bytes(<[u8; 4]>::try_from(&buf[8..=11]).unwrap()),
            num_ob_streams: u16::from_be_bytes(<[u8; 2]>::try_from(&buf[12..=13]).unwrap()),
            num_ib_streams: u16::from_be_bytes(<[u8; 2]>::try_from(&buf[14..=15]).unwrap()),
            init_tsn: u32::from_be_bytes(<[u8; 4]>::try_from(&buf[16..=19]).unwrap()),
            optional_params: None,
        }
    }
}

impl Chunk for Init {
    fn get_bytes(&self) -> Vec<u8> {
        let mut v = vec![];
        v.extend(<[u8; 4]>::from(&self.header));
        v.extend(self.init_tag.to_be_bytes());
        v.extend(self.a_rwnd.to_be_bytes());
        v.extend(self.num_ob_streams.to_be_bytes());
        v.extend(self.num_ib_streams.to_be_bytes());
        v.extend(self.init_tsn.to_be_bytes());
        if let Some(params) = &self.optional_params {
            v.extend(params);
        }
        v
    }
}

#[derive(Clone, Debug)]
pub struct InitAck {
    header: ChunkHeader,
    init_tag: u32,
    a_rwnd: u32,
    num_ob_streams: u16,
    num_ib_streams: u16,
    init_tsn: u32,
    optional_params: Option<Vec<u8>>,
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
            header: ChunkHeader::new(1, 0, 20 + optional_params.map_or(0, |v| v.len() as u16)),
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
    fn get_bytes(&self) -> Vec<u8> {
        let mut v = vec![];
        v.extend(<[u8; 4]>::from(&self.header));
        v.extend(self.init_tag.to_be_bytes());
        v.extend(self.a_rwnd.to_be_bytes());
        v.extend(self.num_ob_streams.to_be_bytes());
        v.extend(self.num_ib_streams.to_be_bytes());
        v.extend(self.init_tsn.to_be_bytes());
        if let Some(params) = &self.optional_params {
            v.extend(params);
        }
        v
    }
}

pub struct Data {}

impl Chunk for Data {
    fn get_bytes(&self) -> Vec<u8> {
        todo!()
    }
}
