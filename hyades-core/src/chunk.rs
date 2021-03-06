use crate::cookie::Cookie;
use rand::{thread_rng, Rng};
use std::convert::TryFrom;
use std::fmt::Debug;

#[derive(Clone, Debug)]
pub enum ChunkType {
    Data,
    Init,
    InitAck,
    Sack,
    Abort,
    Shutdown,
    CookieEcho,
    CookieAck,
    ShutdownComplete,
    ShutdownAck,
    Invalid,
}

impl From<u8> for ChunkType {
    fn from(value: u8) -> Self {
        match value {
            0 => ChunkType::Data,
            1 => ChunkType::Init,
            2 => ChunkType::InitAck,
            3 => ChunkType::Sack,
            6 => ChunkType::Abort,
            7 => ChunkType::Shutdown,
            8 => ChunkType::ShutdownAck,
            10 => ChunkType::CookieEcho,
            11 => ChunkType::CookieAck,
            14 => ChunkType::ShutdownComplete,
            _ => ChunkType::Invalid,
        }
    }
}

pub trait Chunk {
    fn get_bytes(&self) -> Vec<u8>;
    fn chunk_type(&self) -> ChunkType;
}

impl From<&Box<dyn Chunk>> for Vec<u8> {
    fn from(b: &Box<dyn Chunk>) -> Self {
        b.get_bytes()
    }
}

/*
        0                   1                   2                   3
        0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
       |   Chunk Type  | Chunk  Flags  |        Chunk Length           |
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
       \                                                               \
       /                          Chunk Value                          /
       \                                                               \
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
*/

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
       |          Parameter Type       |       Parameter Length        |
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
       \                                                               \
       /                       Parameter Value                         /
       \                                                               \
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
*/

#[derive(Clone, Debug, PartialEq)]
pub enum ParamType {
    StateCookie,
    HostNameAddr,
    Invalid, // TODO abhi - add other params as and when required
}

#[derive(Clone, Debug)]
pub struct Parameter {
    pub param_type: ParamType,
    len: u16,
    pub value: Vec<u8>,
}

impl Parameter {
    pub fn new(param_type: ParamType, value: Vec<u8>) -> Self {
        Self {
            param_type,
            len: value.len() as u16,
            value,
        }
    }
}

impl From<&ParamType> for u16 {
    fn from(param_type: &ParamType) -> Self {
        match param_type {
            ParamType::StateCookie => 7,
            ParamType::HostNameAddr => 11,
            ParamType::Invalid => 0,
        }
    }
}

impl From<u16> for ParamType {
    fn from(param_type: u16) -> Self {
        match param_type {
            7 => ParamType::StateCookie,
            11 => ParamType::HostNameAddr,
            _ => ParamType::Invalid,
        }
    }
}

impl From<&Parameter> for Vec<u8> {
    fn from(p: &Parameter) -> Self {
        let mut v = vec![];
        v.extend(u16::from(&p.param_type).to_be_bytes());
        v.extend(p.len.to_be_bytes());
        v.extend(&p.value);
        v
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
    pub init_tag: u32,
    pub a_rwnd: u32,
    num_ob_streams: u16,
    num_ib_streams: u16,
    init_tsn: u32,
    pub optional_params: Option<Vec<Parameter>>,
}

impl Init {
    pub fn new(
        init_tag: u32,
        a_rwnd: u32,
        num_ob_streams: u16,
        num_ib_streams: u16,
        optional_params: Option<Vec<Parameter>>,
    ) -> Self {
        Self {
            header: ChunkHeader::new(
                1,
                0,
                20 + optional_params.as_ref().map_or(0, |v| *(&v.len()) as u16),
            ),
            init_tag,
            a_rwnd,
            num_ob_streams,
            num_ib_streams,
            init_tsn: init_tag,
            optional_params,
        }
    }
}

impl From<Vec<u8>> for Init {
    fn from(buf: Vec<u8>) -> Self {
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
            optional_params: parse_optional_params(&buf, 20),
        }
    }
}

impl From<&[u8]> for Init {
    fn from(buf: &[u8]) -> Self {
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
            optional_params: parse_optional_params(&buf, 20),
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
            for param in params {
                v.extend(Vec::<u8>::from(param));
            }
        }
        v
    }

    fn chunk_type(&self) -> ChunkType {
        self.header.chunk_type.into()
    }
}

/*
        0                   1                   2                   3
        0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
       |   Type = 2    |  Chunk Flags  |      Chunk Length             |
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
       |                         Initiate Tag                          |
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
       |              Advertised Receiver Window Credit                |
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
pub struct InitAck {
    header: ChunkHeader,
    init_tag: u32,
    a_rwnd: u32,
    num_ob_streams: u16,
    num_ib_streams: u16,
    init_tsn: u32,
    pub optional_params: Option<Vec<Parameter>>,
}

impl InitAck {
    pub fn new(init: Init) -> Self {
        Self {
            header: ChunkHeader::new(
                2,
                0,
                20 + init
                    .optional_params
                    .as_ref()
                    .map_or(0, |v| *(&v.len()) as u16),
            ),
            init_tag: init.init_tag,
            a_rwnd: init.a_rwnd,
            num_ob_streams: init.num_ob_streams,
            num_ib_streams: init.num_ib_streams,
            init_tsn: thread_rng().gen_range(0..=4294967295),
            optional_params: init.optional_params,
        }
    }

    pub fn add_param(&mut self, param: Parameter) {
        if let Some(params) = self.optional_params.as_mut() {
            params.push(param);
        } else {
            self.optional_params = Some(vec![param]);
        }
    }
}

impl From<Vec<u8>> for InitAck {
    fn from(buf: Vec<u8>) -> Self {
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
            optional_params: parse_optional_params(&buf, 20),
        }
    }
}

impl From<&[u8]> for InitAck {
    fn from(buf: &[u8]) -> Self {
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
            optional_params: parse_optional_params(&buf, 20),
        }
    }
}

/// Parses optional params and return them as `Option<Vec<Param>>`
fn parse_optional_params(buf: &[u8], start_offset: usize) -> Option<Vec<Parameter>> {
    // while we haven't reached the end of the buffer:
    //      parse the length of the param
    //      read length number of bytes from buf
    //      construct a param and push it into the optional_params vec
    //      repeat
    let mut offset = start_offset;
    if offset > buf.len() - 1 {
        None
    } else {
        let mut v = vec![];

        while offset < buf.len() {
            let param_type =
                u16::from_be_bytes(<[u8; 2]>::try_from(&buf[offset..=(offset + 1)]).unwrap())
                    .into();
            offset += 2;
            let len = u16::from_be_bytes(<[u8; 2]>::try_from(&buf[offset..=(offset + 1)]).unwrap());
            offset += 2;
            let value = &buf[offset..offset + len as usize];

            v.push(Parameter {
                param_type,
                len,
                value: value.to_vec(),
            });

            offset += len as usize;
        }

        Some(v)
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
            for param in params {
                v.extend(Vec::<u8>::from(param));
            }
        }
        v
    }

    fn chunk_type(&self) -> ChunkType {
        self.header.chunk_type.into()
    }
}

#[derive(Clone, Debug)]
pub struct CookieEcho {
    header: ChunkHeader,
    pub cookie: Cookie,
}

impl CookieEcho {
    pub fn new(cookie: Cookie) -> CookieEcho {
        Self {
            header: ChunkHeader::new(10, 0, 4 + cookie.len() as u16),
            cookie,
        }
    }
}

impl From<Vec<u8>> for CookieEcho {
    fn from(buf: Vec<u8>) -> Self {
        Self {
            header: ChunkHeader::new(
                buf[0],
                buf[1],
                u16::from_be_bytes(<[u8; 2]>::try_from(&buf[2..=3]).unwrap()),
            ),
            cookie: Cookie::from(&buf[4..]),
        }
    }
}

impl From<&[u8]> for CookieEcho {
    fn from(buf: &[u8]) -> Self {
        Self {
            header: ChunkHeader::new(
                buf[0],
                buf[1],
                u16::from_be_bytes(<[u8; 2]>::try_from(&buf[2..=3]).unwrap()),
            ),
            cookie: Cookie::from(&buf[4..]),
        }
    }
}

impl Chunk for CookieEcho {
    fn get_bytes(&self) -> Vec<u8> {
        let mut v = vec![];
        v.extend(<[u8; 4]>::from(&self.header));
        v.extend(Vec::<u8>::from(&self.cookie));
        v
    }

    fn chunk_type(&self) -> ChunkType {
        self.header.chunk_type.into()
    }
}

#[derive(Clone, Debug)]
pub struct CookieAck {
    header: ChunkHeader,
}

impl CookieAck {
    pub fn new() -> Self {
        Self {
            header: ChunkHeader::new(11, 0, 4),
        }
    }
}

impl From<Vec<u8>> for CookieAck {
    fn from(buf: Vec<u8>) -> Self {
        Self {
            header: ChunkHeader::new(
                buf[0],
                buf[1],
                u16::from_be_bytes(<[u8; 2]>::try_from(&buf[2..=3]).unwrap()),
            ),
        }
    }
}

impl From<&[u8]> for CookieAck {
    fn from(buf: &[u8]) -> Self {
        Self {
            header: ChunkHeader::new(
                buf[0],
                buf[1],
                u16::from_be_bytes(<[u8; 2]>::try_from(&buf[2..=3]).unwrap()),
            ),
        }
    }
}

impl Chunk for CookieAck {
    fn get_bytes(&self) -> Vec<u8> {
        <[u8; 4]>::from(&self.header).into()
    }

    fn chunk_type(&self) -> ChunkType {
        self.header.chunk_type.into()
    }
}

/*
        0                   1                   2                   3
        0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
       |   Type = 0    | Reserved|U|B|E|    Length                     |
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
       |                              TSN                              |
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
       |      Stream Identifier S      |   Stream Sequence Number n    |
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
       |                  Payload Protocol Identifier                  |
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
       \                                                               \
       /                 User Data (seq n of Stream S)                 /
       \                                                               \
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
*/

#[derive(Clone, Debug)]
pub struct Data {
    header: ChunkHeader,
    tsn: u32,
    stream_id: u16,
    // stream_seq_no will be the same for
    // fragments of the same msg
    stream_seq_no: u16,
    payload_proto_id: u32,
    pub data: Vec<u8>,
}

impl Data {
    pub fn new(
        tsn: u32,
        stream_id: u16,
        stream_seq_no: u16,
        payload_proto_id: u32,
        start: bool,
        end: bool,
        mut data: Vec<u8>,
    ) -> Self {
        let flag = match (start, end) {
            (true, false) => 6,
            (false, true) => 1,
            (true, true) => 7,
            _ => 0,
        };

        // pad with 0s if data len not multiple of 4
        let unpadded_data_len = data.len();
        let diff = unpadded_data_len % 4;
        let adjust = if diff == 0 { 0 } else { 4 - diff };
        data.extend(std::iter::repeat(0).take(adjust));

        Self {
            header: ChunkHeader::new(0, flag, 16 + unpadded_data_len as u16),
            tsn,
            stream_id,
            stream_seq_no,
            payload_proto_id,
            data,
        }
    }
}

impl From<&[u8]> for Data {
    fn from(buf: &[u8]) -> Self {
        Self {
            header: ChunkHeader::new(
                buf[0],
                buf[1],
                u16::from_be_bytes(<[u8; 2]>::try_from(&buf[2..=3]).unwrap()),
            ),
            tsn: u32::from_be_bytes(<[u8; 4]>::try_from(&buf[4..=7]).unwrap()),
            stream_id: u16::from_be_bytes(<[u8; 2]>::try_from(&buf[7..=8]).unwrap()),
            // stream_seq_no will be the same for
            // fragments of the same msg
            stream_seq_no: u16::from_be_bytes(<[u8; 2]>::try_from(&buf[9..=10]).unwrap()),
            payload_proto_id: u32::from_be_bytes(<[u8; 4]>::try_from(&buf[11..=14]).unwrap()),
            data: buf[15..].to_vec(),
        }
    }
}

impl Chunk for Data {
    fn get_bytes(&self) -> Vec<u8> {
        let mut v = vec![];
        v.extend(<[u8; 4]>::from(&self.header));
        v.extend(&self.tsn.to_be_bytes());
        v.extend(&self.stream_id.to_be_bytes());
        v.extend(&self.stream_seq_no.to_be_bytes());
        v.extend(&self.payload_proto_id.to_be_bytes());
        v.extend(&self.data);
        v
    }

    fn chunk_type(&self) -> ChunkType {
        self.header.chunk_type.into()
    }
}

/*
        0                   1                   2                   3
        0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
       |   Type = 3    |Chunk  Flags   |      Chunk Length             |
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
       |                      Cumulative TSN Ack                       |
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
       |          Advertised Receiver Window Credit (a_rwnd)           |
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
       | Number of Gap Ack Blocks = N  |  Number of Duplicate TSNs = X |
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
       |  Gap Ack Block #1 Start       |   Gap Ack Block #1 End        |
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
       /                                                               /
       \                              ...                              \
       /                                                               /
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
       |   Gap Ack Block #N Start      |  Gap Ack Block #N End         |
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
       |                       Duplicate TSN 1                         |
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
       /                                                               /
       \                              ...                              \
       /                                                               /
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
       |                       Duplicate TSN X                         |
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
*/

#[derive(Clone, Debug)]
pub struct Sack {
    header: ChunkHeader,
    pub cumulative_tsn_ack: u32,
    a_rwnd: u32,
    num_gap_ack_blocks: u16,
    num_dup_tsns: u16,
    gap_ack_blk_starts_ends: Option<Vec<(u16, u16)>>,
    dup_tsns: Option<Vec<u32>>,
}

impl Sack {
    pub fn new(
        cumulative_tsn_ack: u32,
        a_rwnd: u32,
        num_gap_ack_blocks: u16,
        num_dup_tsns: u16,
        gap_ack_blk_starts_ends: Option<Vec<(u16, u16)>>,
        dup_tsns: Option<Vec<u32>>,
    ) -> Self {
        Self {
            header: ChunkHeader::new(
                3,
                0,
                16 + gap_ack_blk_starts_ends
                    .as_ref()
                    .map_or(0, |v| (*(&v.len()) * 4) as u16)
                    + dup_tsns.as_ref().map_or(0, |v| (*(&v.len()) * 4) as u16),
            ),
            cumulative_tsn_ack,
            a_rwnd,
            num_gap_ack_blocks,
            num_dup_tsns,
            gap_ack_blk_starts_ends,
            dup_tsns,
        }
    }
}

impl Chunk for Sack {
    fn get_bytes(&self) -> Vec<u8> {
        let mut v = vec![];
        v.extend(<[u8; 4]>::from(&self.header));
        v.extend(&self.cumulative_tsn_ack.to_be_bytes());
        v.extend(&self.a_rwnd.to_be_bytes());
        v.extend(&self.num_gap_ack_blocks.to_be_bytes());
        v.extend(&self.num_dup_tsns.to_be_bytes());

        if let Some(gap_ack_blk_starts_ends) = self.gap_ack_blk_starts_ends.as_ref() {
            for (start, end) in gap_ack_blk_starts_ends {
                v.extend(start.to_be_bytes());
                v.extend(end.to_be_bytes());
            }
        }

        if let Some(dup_tsns) = self.dup_tsns.as_ref() {
            for dup_tsn in dup_tsns {
                v.extend(dup_tsn.to_be_bytes());
            }
        }
        v
    }

    fn chunk_type(&self) -> ChunkType {
        self.header.chunk_type.into()
    }
}

impl From<Vec<u8>> for Sack {
    fn from(buf: Vec<u8>) -> Self {
        let header = ChunkHeader::new(
            buf[0],
            buf[1],
            u16::from_be_bytes(<[u8; 2]>::try_from(&buf[2..=3]).unwrap()),
        );
        let cumulative_tsn_ack = u32::from_be_bytes(<[u8; 4]>::try_from(&buf[4..=7]).unwrap());
        let a_rwnd = u32::from_be_bytes(<[u8; 4]>::try_from(&buf[8..=11]).unwrap());
        let num_gap_ack_blocks = u16::from_be_bytes(<[u8; 2]>::try_from(&buf[12..=13]).unwrap());
        let num_dup_tsns = u16::from_be_bytes(<[u8; 2]>::try_from(&buf[14..=15]).unwrap());
        let mut offset = 16usize;
        let gap_ack_blk_starts_ends = if num_gap_ack_blocks > 0 {
            let mut gaps = vec![];
            for _ in 0..num_gap_ack_blocks {
                gaps.push((
                    u16::from_be_bytes(<[u8; 2]>::try_from(&buf[offset..=offset + 1]).unwrap()),
                    u16::from_be_bytes(<[u8; 2]>::try_from(&buf[offset + 2..=offset + 3]).unwrap()),
                ));
                offset += 4;
            }
            Some(gaps)
        } else {
            None
        };

        let dup_tsns = if num_dup_tsns > 0 {
            let mut tsns = vec![];
            for _ in 0..num_dup_tsns {
                tsns.push(u32::from_be_bytes(
                    <[u8; 4]>::try_from(&buf[offset..=offset + 4]).unwrap(),
                ));
                offset += 4;
            }
            Some(tsns)
        } else {
            None
        };

        Self {
            header,
            cumulative_tsn_ack,
            a_rwnd,
            num_gap_ack_blocks,
            num_dup_tsns,
            gap_ack_blk_starts_ends,
            dup_tsns,
        }
    }
}
/*
        0                   1                   2                   3
        0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
       |   Type = 9    | Chunk  Flags  |           Length              |
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
       \                                                               \
       /                    one or more Error Causes                   /
       \                                                               \
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
*/

pub trait Cause: Debug {
    fn get_bytes(&self) -> Vec<u8>;
}

#[derive(Clone, Debug)]
pub struct CauseHeader {
    code: u16,
    len: u16,
}

#[derive(Clone, Debug)]
pub struct InvalidStreamId {
    header: CauseHeader,
    id: u16,
    reserved: u16,
}

impl Cause for InvalidStreamId {
    fn get_bytes(&self) -> Vec<u8> {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub struct MissingMandatoryParam {
    header: CauseHeader,
    num: u32,
    params: Vec<ParamType>,
}

impl Cause for MissingMandatoryParam {
    fn get_bytes(&self) -> Vec<u8> {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub struct StateCookieError {
    header: CauseHeader,
    staleness_measure: u32,
}

impl Cause for StateCookieError {
    fn get_bytes(&self) -> Vec<u8> {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub struct OutOfResource {
    header: CauseHeader,
}

impl Cause for OutOfResource {
    fn get_bytes(&self) -> Vec<u8> {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub struct UnresolvableAddr {
    header: CauseHeader,
}

impl Cause for UnresolvableAddr {
    fn get_bytes(&self) -> Vec<u8> {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub struct UnrecognizedChunkType {
    header: CauseHeader,
}

impl Cause for UnrecognizedChunkType {
    fn get_bytes(&self) -> Vec<u8> {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub struct InvalidMandatoryParam {
    header: CauseHeader,
}

impl Cause for InvalidMandatoryParam {
    fn get_bytes(&self) -> Vec<u8> {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub struct UnrecognizedParams {
    header: CauseHeader,
}

impl Cause for UnrecognizedParams {
    fn get_bytes(&self) -> Vec<u8> {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub struct NoUserData {
    header: CauseHeader,
    tsn: u32,
}

impl Cause for NoUserData {
    fn get_bytes(&self) -> Vec<u8> {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub struct CookieRcvdWhileShuttingDown {
    header: CauseHeader,
}

impl Cause for CookieRcvdWhileShuttingDown {
    fn get_bytes(&self) -> Vec<u8> {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub struct AssocRestartWithNewAddrs {
    header: CauseHeader,
}

impl Cause for AssocRestartWithNewAddrs {
    fn get_bytes(&self) -> Vec<u8> {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub struct UserInitiatedAbort {
    header: CauseHeader,
}

impl Cause for UserInitiatedAbort {
    fn get_bytes(&self) -> Vec<u8> {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub struct ProtocolViolation {
    header: CauseHeader,
}

impl Cause for ProtocolViolation {
    fn get_bytes(&self) -> Vec<u8> {
        todo!()
    }
}

#[derive(Debug)]
pub struct Error {
    header: ChunkHeader,
    errors: Vec<Box<dyn Cause>>,
}

impl Chunk for Error {
    fn get_bytes(&self) -> Vec<u8> {
        todo!()
    }

    fn chunk_type(&self) -> ChunkType {
        self.header.chunk_type.into()
    }
}

/*
        0                   1                   2                   3
        0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
       |   Type = 6    |Reserved     |T|           Length              |
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
       \                                                               \
       /                   zero or more Error Causes                   /
       \                                                               \
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
*/

#[derive(Clone, Debug)]
pub struct Abort {
    header: ChunkHeader,
    // errors: Option<Vec<Error>>,
}

impl Abort {
    pub fn new(errors: Option<Vec<Error>>) -> Self {
        Self {
            header: ChunkHeader::new(6, 1, 4 + errors.as_ref().map_or(0, |v| *(&v.len()) as u16)),
            // errors: None,
        }
    }
}

impl Chunk for Abort {
    fn get_bytes(&self) -> Vec<u8> {
        let mut v = vec![];
        v.extend(<[u8; 4]>::from(&self.header));
        todo!()
    }

    fn chunk_type(&self) -> ChunkType {
        self.header.chunk_type.into()
    }
}

/*
        0                   1                   2                   3
        0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
       |   Type = 7    | Chunk  Flags  |      Length = 8               |
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
       |                      Cumulative TSN Ack                       |
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
*/

#[derive(Clone, Debug)]
pub struct Shutdown {
    header: ChunkHeader,
    cumulative_tsn_ack: u32,
}

impl Shutdown {
    pub fn new(cumulative_tsn_ack: u32) -> Self {
        Self {
            header: ChunkHeader::new(7, 0, 8),
            cumulative_tsn_ack,
        }
    }
}

impl Chunk for Shutdown {
    fn get_bytes(&self) -> Vec<u8> {
        let mut v = vec![];
        v.extend(<[u8; 4]>::from(&self.header));
        v.extend(self.cumulative_tsn_ack.to_be_bytes());
        v
    }

    fn chunk_type(&self) -> ChunkType {
        self.header.chunk_type.into()
    }
}

/*
        0                   1                   2                   3
        0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
       |   Type = 8    |Chunk  Flags   |      Length = 4               |
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
*/

#[derive(Clone, Debug)]
pub struct ShutdownAck {
    header: ChunkHeader,
}

impl ShutdownAck {
    pub fn new() -> Self {
        Self {
            header: ChunkHeader::new(8, 0, 4),
        }
    }
}

impl Chunk for ShutdownAck {
    fn get_bytes(&self) -> Vec<u8> {
        <[u8; 4]>::from(&self.header).into()
    }

    fn chunk_type(&self) -> ChunkType {
        self.header.chunk_type.into()
    }
}

/*
        0                   1                   2                   3
        0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
       |   Type = 14   |Reserved     |T|      Length = 4               |
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
*/

#[derive(Clone, Debug)]
pub struct ShutdownComplete {
    header: ChunkHeader,
}

impl ShutdownComplete {
    pub fn new() -> Self {
        Self {
            header: ChunkHeader::new(14, 1, 4),
        }
    }
}

impl Chunk for ShutdownComplete {
    fn get_bytes(&self) -> Vec<u8> {
        let mut v = vec![];
        v.extend(<[u8; 4]>::from(&self.header));
        v
    }

    fn chunk_type(&self) -> ChunkType {
        self.header.chunk_type.into()
    }
}
