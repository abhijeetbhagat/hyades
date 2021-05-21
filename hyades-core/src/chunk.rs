use crate::cookie::Cookie;
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

#[derive(Clone, Debug, PartialEq)]
pub enum ParamType {
    StateCookie,
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
            ParamType::Invalid => 0,
        }
    }
}

impl From<u16> for ParamType {
    fn from(param_type: u16) -> Self {
        match param_type {
            7 => ParamType::StateCookie,
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

            // while we haven't reached the end of the buffer:
            //      parse the length of the param
            //      read length number of bytes from buf
            //      construct a param and push it into the optional_params vec
            //      repeat
            optional_params: {
                let mut offset = 20usize;
                if offset == buf.len() - 1 {
                    None
                } else {
                    let mut v = vec![];

                    while offset < buf.len() {
                        let param_type = u16::from_be_bytes(
                            <[u8; 2]>::try_from(&buf[offset..=(offset + 1)]).unwrap(),
                        )
                        .into();
                        offset += 2;
                        let len = u16::from_be_bytes(
                            <[u8; 2]>::try_from(&buf[offset..=(offset + 1)]).unwrap(),
                        );
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
    pub optional_params: Option<Vec<Parameter>>,
}

impl InitAck {
    pub fn new(init: Init) -> Self {
        Self {
            header: ChunkHeader::new(2, 0, 20),
            init_tag: init.init_tag,
            a_rwnd: init.a_rwnd,
            num_ob_streams: init.num_ob_streams,
            num_ib_streams: init.num_ib_streams,
            init_tsn: thread_rng().gen_range(0..=4294967295),
            optional_params: None,
        }
    }

    pub fn add_param(&mut self, param: Parameter) {
        if let Some(mut params) = self.optional_params.as_mut() {
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
            optional_params: {
                let mut offset = 20usize;
                if offset == buf.len() - 1 {
                    None
                } else {
                    let mut v = vec![];

                    while offset < buf.len() {
                        let param_type = u16::from_be_bytes(
                            <[u8; 2]>::try_from(&buf[offset..=(offset + 1)]).unwrap(),
                        )
                        .into();
                        offset += 2;
                        let len = u16::from_be_bytes(
                            <[u8; 2]>::try_from(&buf[offset..=(offset + 1)]).unwrap(),
                        );
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
            },
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

impl Chunk for CookieEcho {
    fn get_bytes(&self) -> Vec<u8> {
        let mut v = vec![];
        v.extend(<[u8; 4]>::from(&self.header));
        v.extend(Vec::<u8>::from(&self.cookie));
        v
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

impl Chunk for CookieAck {
    fn get_bytes(&self) -> Vec<u8> {
        <[u8; 4]>::from(&self.header).into()
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
    let buf = vec![
        1u8, 1, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 1, 0, 1, 0, 0, 0, 1, // optional params
        0, 7, 0, 4, 0, 1, 0, 1,
    ];
    let chunk = Init::from(buf);
    assert!(chunk.num_ib_streams == 1);
    assert!(chunk.optional_params.is_some());
    let params = chunk.optional_params.unwrap();
    assert!(params.len() == 1);
    let param = &params[0];
    assert!(param.param_type == 7);
    assert!(param.len == 4);
    assert!(param.value == vec![0, 1, 0, 1]);
}

#[test]
fn test_init_conversion_2() {
    let buf = vec![
        1u8, 1, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 1, 0, 1, 0, 0, 0, 1,
        // optional params
        // param 1
        0, 7, 0, 4, 0, 1, 0, 1, // param 2
        0, 11, 0, 4, 0, 1, 0, 1,
    ];
    let chunk = Init::from(buf);
    assert!(chunk.num_ib_streams == 1);
    assert!(chunk.optional_params.is_some());
    let params = chunk.optional_params.unwrap();
    assert!(params.len() == 2);
    let param = &params[1];
    assert!(param.param_type == 11);
    assert!(param.len == 4);
    assert!(param.value == vec![0, 1, 0, 1]);
}

#[test]
fn test_init_conversion_with_no_params() {
    let buf = vec![
        1u8, 1, 0, 1, 0, 0, 0, 1, 0, 0, 0, 1, 0, 1, 0, 1, 0, 0, 0, 1,
        // no optional params
    ];
    let chunk = Init::from(buf);
    assert!(chunk.num_ib_streams == 1);
    assert!(chunk.optional_params.is_none());
}
