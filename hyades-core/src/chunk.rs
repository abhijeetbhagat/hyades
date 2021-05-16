use rand::{rngs::ThreadRng, thread_rng, Rng};
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
       |          Parameter Type       |       Parameter Length        |
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
       \                                                               \
       /                       Parameter Value                         /
       \                                                               \
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
*/

#[derive(Clone, Debug)]
pub struct Parameter {
    param_type: u16,
    len: u16,
    value: Vec<u8>,
}

impl From<&Parameter> for Vec<u8> {
    fn from(p: &Parameter) -> Self {
        let mut v = vec![];
        v.extend(p.param_type.to_be_bytes());
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
    optional_params: Option<Vec<Parameter>>,
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
            // TODO abhi - while we haven't reached the end of the buffer:
            //                  parse the length of the param
            //                  read length number of bytes from buf
            //                  construct a param and push it into the optional_params vec
            //                  repeat
            optional_params: {
                let mut offset = 20usize;
                let mut v = vec![];

                while offset < buf.len() {
                    let param_type =
                        u16::from_be_bytes(<[u8; 2]>::try_from(&buf[offset..=(offset + 1)]).unwrap());
                    offset += 2;
                    let len =
                        u16::from_be_bytes(<[u8; 2]>::try_from(&buf[offset..=(offset + 1)]).unwrap());
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
            },
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
}

#[derive(Clone, Debug)]
pub struct InitAck {
    header: ChunkHeader,
    init_tag: u32,
    a_rwnd: u32,
    num_ob_streams: u16,
    num_ib_streams: u16,
    init_tsn: u32,
    optional_params: Option<Vec<Parameter>>,
}

impl InitAck {
    pub fn new(init: Init) -> Self {
        Self {
            header: ChunkHeader::new(
                2,
                0,
                20 + init.optional_params.map_or(0, |v| v.len() as u16),
            ),
            init_tag: init.init_tag,
            a_rwnd: init.a_rwnd,
            num_ob_streams: init.num_ob_streams,
            num_ib_streams: init.num_ib_streams,
            init_tsn: thread_rng().gen_range(0..=4294967295),
            optional_params: None,
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
            for param in params {
                v.extend(Vec::<u8>::from(param));
            }
        }
        v
    }
}

#[derive(Clone, Debug)]
pub struct CookieEcho {}

impl CookieEcho {
    pub fn new() -> CookieEcho {
        Self {}
    }
}

impl From<Vec<u8>> for CookieEcho {
    fn from(buf: Vec<u8>) -> Self {
        Self {}
    }
}

impl Chunk for CookieEcho {
    fn get_bytes(&self) -> Vec<u8> {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub struct CookieAck {}

impl CookieAck {
    pub fn new() -> Self {
        Self {}
    }
}

impl From<Vec<u8>> for CookieAck {
    fn from(buf: Vec<u8>) -> Self {
        Self {}
    }
}

impl Chunk for CookieAck {
    fn get_bytes(&self) -> Vec<u8> {
        todo!()
    }
}

#[derive(Clone, Debug)]
pub struct Data {}

impl Chunk for Data {
    fn get_bytes(&self) -> Vec<u8> {
        todo!()
    }
}

#[test]
fn test_init_conversion() {
    let buf = vec![1u8, 1, 0, 1,
        0,0,0,1,
        0,0,0,1,
        0,1,
        0,1,
        0,0,0,1,
        // optional params
        0,7,0,4,
        0,1,0,1
    ];
    let chunk = Init::from(buf);
    assert!(chunk.num_ib_streams == 1);
    assert!(chunk.optional_params.is_some());
    let params = chunk.optional_params.unwrap();
    assert!(params.len() == 1);
    let param = &params[0];
    assert!(param.param_type == 7);
    assert!(param.len == 4);
    assert!(&param.value == &vec![0,1,0,1]);
}
