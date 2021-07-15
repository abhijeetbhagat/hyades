use crate::chunk::{Chunk, CookieAck, CookieEcho, Init, InitAck};
use crate::error::SCTPError;
use crc32c;
use std::convert::TryFrom;

/*
        0                   1                   2                   3
        0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
       |     Source Port Number        |     Destination Port Number   |
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
       |                      Verification Tag                         |
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
       |                           Checksum                            |
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
*/

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

/*
        0                   1                   2                   3
        0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
       |                        Common Header                          |
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
       |                          Chunk #1                             |
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
       |                           ...                                 |
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
       |                          Chunk #n                             |
       +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
*/

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

impl TryFrom<Vec<u8>> for Packet {
    type Error = SCTPError;

    fn try_from(raw_data: Vec<u8>) -> Result<Self, Self::Error> {
        // TODO abhi: convert raw bytes into a packet
        let header = CommonHeader {
            src_port: u16::from_be_bytes(<[u8; 2]>::try_from(&raw_data[0..2]).unwrap()),
            dst_port: u16::from_be_bytes(<[u8; 2]>::try_from(&raw_data[2..4]).unwrap()),
            ver_tag: u32::from_be_bytes(<[u8; 4]>::try_from(&raw_data[4..8]).unwrap()),
            checksum: u32::from_be_bytes(<[u8; 4]>::try_from(&raw_data[8..12]).unwrap()),
        };

        let mut offset = 12usize;
        let mut chunks: Vec<Box<dyn Chunk>> = vec![];

        while offset < raw_data.len() {
            match raw_data[offset] {
                0 => {
                    let len = u16::from_be_bytes(
                        <[u8; 2]>::try_from(&raw_data[offset + 3..=offset + 4]).unwrap(),
                    ) as usize;
                    chunks.push(Box::new(Data::from(&raw_data[offset..len])));
                    offset += len;
                }
                1 => {
                    let len = u16::from_be_bytes(
                        <[u8; 2]>::try_from(&raw_data[offset + 3..=offset + 4]).unwrap(),
                    ) as usize;
                    chunks.push(Box::new(Init::from(&raw_data[offset..len])));
                    offset += len;
                }
                2 => {
                    let len = u16::from_be_bytes(
                        <[u8; 2]>::try_from(&raw_data[offset + 3..=offset + 4]).unwrap(),
                    ) as usize;
                    chunks.push(Box::new(InitAck::from(&raw_data[offset..len])));
                    offset += len;
                }
                10 => {
                    let len = u16::from_be_bytes(
                        <[u8; 2]>::try_from(&raw_data[offset + 3..=offset + 4]).unwrap(),
                    ) as usize;
                    chunks.push(Box::new(CookieEcho::from(&raw_data[offset..len])));
                    offset += len;
                }
                11 => {
                    let len = u16::from_be_bytes(
                        <[u8; 2]>::try_from(&raw_data[offset + 3..=offset + 4]).unwrap(),
                    ) as usize;
                    chunks.push(Box::new(CookieAck::from(&raw_data[offset..len])));
                    offset += len;
                }
                _ => return Err(SCTPError::InvalidSCTPPacket),
            }
        }

        Ok(Packet { header, chunks })
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
