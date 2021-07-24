use crate::chunk::{Abort, CookieAck, CookieEcho, Data, Init, InitAck, ParamType, Parameter};
use crate::cookie::Cookie;
use crate::error::SCTPError;
use crate::packet::Packet;
use crate::stream::Stream;
use herschel::pmtud::Pmtud;
use log::{debug, info};
use rand::{rngs::ThreadRng, thread_rng, Rng};
use std::convert::TryFrom;
use std::net::SocketAddr;
use std::{cmp, collections::VecDeque};
use tokio::time::{sleep, timeout, Duration};

const RTO_INITIAL: u64 = 3;
const RTO_MIN: u8 = 1;
const RTO_MAX: u8 = 60;
const MAX_BURST: u8 = 4;
// const  RTO_ALPHA = 1/8
// const  RTO_BETA = 1/4
const VALID_COOKIE_LIFE: u8 = 60;
const ASSOCIATION_MAX_RETRANS: u8 = 10;
const PATH_MAX_RETRANS: u8 = 5;
const MAX_INIT_RETRANSMITS: u8 = 8;
const HB_INTERVAL: u8 = 30;
const HB_MAX_BURST: u8 = 1;

/// An SCTP Association
pub struct Association {
    id: String,
    local_addr: SocketAddr,
    remote_addr: Option<SocketAddr>,
    stream: Stream,
    pub msg_queue: VecDeque<Packet>,
    rng: ThreadRng,
    init_tag: u32,
    stream_id: u16,
    stream_seq_no: u16,
    max_retries: u8,
    max_init_retries: u8,
    rto: u64,
    tsn: u32,
    largest_tsn: u32,
    remote_rwnd: u32,
    mtu: Option<u16>,
    cwnd: Option<u16>,
    ssthresh: Option<u16>,
}

impl Association {
    /// Creates a new sender endpoint
    pub async fn new_sender(
        local_addr: impl AsRef<str>,
        remote_addr: impl AsRef<str>,
    ) -> Result<Self, SCTPError> {
        let local_sockaddr: SocketAddr = local_addr
            .as_ref()
            .parse()
            .map_err(|_| SCTPError::InvalidLocalAddress)?;
        let remote_sockaddr: SocketAddr = remote_addr
            .as_ref()
            .parse()
            .map_err(|_| SCTPError::InvalidRemoteAddress)?;

        let (mtu, cwnd, ssthresh) = match Pmtud::new(
            local_addr.as_ref().parse().unwrap(),
            remote_addr.as_ref().parse().unwrap(),
        ) {
            Ok(mut pmtud) => match pmtud.discover() {
                Ok(mtu) => (
                    Some(mtu),
                    // 7.2.1: cwnd should be min(4*MTU, max (2*MTU, 4380 bytes))
                    Some(cmp::min(4 * mtu, cmp::max(2 * mtu, 4380))),
                    // 7.2.1; initial value of ssthreshold can be anything
                    Some(10000),
                ),
                _ => (Some(1500), Some(1500), Some(10000)),
            },
            _ => (None, None, None),
        };

        let stream = Stream::new(local_addr).await?.connect(remote_addr).await?;

        let mut association = Self {
            id: "nra".to_owned(),
            stream,
            rng: thread_rng(),
            msg_queue: VecDeque::new(),
            local_addr: local_sockaddr,
            remote_addr: Some(remote_sockaddr),
            init_tag: 0,
            // section 5.1.1: stream id can be between 0 to min(local OS, remote MIS)-1
            stream_id: 0,
            stream_seq_no: 0,
            max_retries: ASSOCIATION_MAX_RETRANS,
            max_init_retries: MAX_INIT_RETRANSMITS,
            rto: RTO_INITIAL * 1000,
            tsn: 0,
            largest_tsn: 0,
            remote_rwnd: 0,
            mtu,
            cwnd,
            ssthresh,
        };

        association.start_sender_4_way_handshake().await?;
        Ok(association)
    }

    /// Creates a new recvr endpoint
    pub async fn new_recvr(local_addr: impl AsRef<str>) -> Result<Self, SCTPError> {
        let local_sockaddr: SocketAddr = local_addr
            .as_ref()
            .parse()
            .map_err(|_| SCTPError::InvalidLocalAddress)?;

        let stream = Stream::new(local_addr).await?;

        let mut association = Self {
            id: "nra".to_owned(),
            stream,
            rng: thread_rng(),
            msg_queue: VecDeque::new(),
            local_addr: local_sockaddr,
            remote_addr: None,
            init_tag: 0,
            // section 5.1.1: stream id can be between 0 to min(local OS, remote MIS)-1
            stream_id: 0,
            stream_seq_no: 0,
            max_retries: ASSOCIATION_MAX_RETRANS,
            max_init_retries: MAX_INIT_RETRANSMITS,
            rto: RTO_INITIAL * 1000,
            tsn: 0,
            largest_tsn: 0,
            remote_rwnd: 0,
            mtu: None, // we dont know what this is yet!
            // 7.2.1: cwnd should be min(4*MTU, max (2*MTU, 4380 bytes))
            // but we dont know what the mtu is yet!
            cwnd: None,
            // 7.2.1; initial value of ssthreshold can be anything
            ssthresh: Some(10000),
        };

        association.start_recvr_4_way_handshake().await?;

        Ok(association)
    }

    /// Starts a 4 way handshake from the sender side
    async fn start_sender_4_way_handshake(&mut self) -> Result<(), SCTPError> {
        let mut num_init_retries = 0;
        while num_init_retries < self.max_init_retries {
            debug!("sending init ...");
            self.send_init().await?;

            match timeout(Duration::from_millis(self.rto), self.stream.recv()).await {
                Ok(bytes) => {
                    let init_ack = InitAck::from(bytes?);
                    debug!("recvd: {:?}", init_ack);

                    match init_ack.optional_params {
                        Some(params) => {
                            let param = params
                                .iter()
                                .find(|param| param.param_type == ParamType::StateCookie)
                                .ok_or(SCTPError::NoCookieError)?;
                            let cookie: Cookie = (&param.value).into();

                            self.attempt_cookie_echo_and_ack(cookie).await?;
                            break;
                        }
                        _ => return Err(SCTPError::NoCookieError),
                    }
                }
                _ => {
                    num_init_retries += 1;
                    continue;
                }
            }
        }

        Ok(())
    }

    /// Attempts to perform the last two steps of the sender's handshake
    async fn attempt_cookie_echo_and_ack(&self, cookie: Cookie) -> Result<(), SCTPError> {
        let mut num_retries = 0;

        let mut packet = Packet::new(
            self.local_addr.port(),
            self.remote_addr.as_ref().unwrap().port(),
        );
        packet.add_chunk(Box::new(CookieEcho::new(cookie)));

        while num_retries < self.max_retries {
            debug!("sending cookie echo {}", num_retries);
            self.send_cookie_echo(&packet).await?;

            // TODO abhi - timeout duration of cookie timer isn't hardcoded;
            // it is calculated from RTO (which is 3 secs to begin with according to the RFC)
            match timeout(Duration::from_millis(self.rto), self.stream.recv()).await {
                Ok(bytes) => {
                    let _ = CookieAck::from(bytes?);
                    return Ok(());
                }
                _ => {
                    num_retries += 1;
                    continue;
                }
            }
        }

        // TODO abhi - our cookie echoing has failed. do something instead of Ok(())
        Ok(())
    }

    /// Starts a 4 way handshake from the recvr side
    async fn start_recvr_4_way_handshake(&mut self) -> Result<(), SCTPError> {
        debug!("waiting ...");
        let (bytes, remote_addr) = self.stream.recv_from().await?;
        let init = Init::from(bytes);
        self.remote_addr = Some(remote_addr);
        debug!("recvd: {:?}", init);
        let cookie = self.send_init_ack(init).await?;
        let cookie_echo = CookieEcho::from(self.stream.recv().await?);

        if cookie == cookie_echo.cookie {
            self.send_cookie_ack().await?;
        } else {
            //TODO abhi - 'silently discard the packet'
            return Err(SCTPError::CookieMismatchError);
        }

        sleep(Duration::from_millis(2000)).await;
        Ok(())
    }

    /// Sends a packet with init chunk
    async fn send_init(&mut self) -> Result<(), SCTPError> {
        let ver_tag: u32 = self.rng.gen_range(1..=4294967295);
        let mut packet = Packet::new(
            self.local_addr.port(),
            self.remote_addr.as_ref().unwrap().port(),
        );

        // TODO abhi - figure what the buffer size should be
        let a_rwnd = 10000;
        packet.add_chunk(Box::new(Init::new(ver_tag, a_rwnd, 1, 1, None)));

        self.stream.send(&Vec::<u8>::from(&packet)).await?;
        Ok(())
    }

    /// Sends a packet with init ack chunk
    async fn send_init_ack(&mut self, init: Init) -> Result<Cookie, SCTPError> {
        debug!("sending init ack");
        let mut packet = Packet::new(
            self.local_addr.port(),
            self.remote_addr.as_ref().unwrap().port(),
        );
        let mut init_ack = InitAck::new(init);
        let cookie = Cookie::new();
        init_ack.add_param(Parameter::new(ParamType::StateCookie, (&cookie).into()));
        packet.add_chunk(Box::new(init_ack));

        self.stream.send(&Vec::<u8>::from(&packet)).await?;
        Ok(cookie)
    }

    /// Sends a packet with cookie echo chunk
    async fn send_cookie_echo(&self, packet: &Packet) -> Result<(), SCTPError> {
        self.stream.send(&Vec::<u8>::from(packet)).await?;

        Ok(())
    }

    /// Sends a packet with cookie ack chunk
    async fn send_cookie_ack(&self) -> Result<(), SCTPError> {
        debug!("sending cookie ack");
        let mut packet = Packet::new(
            self.local_addr.port(),
            self.remote_addr.as_ref().unwrap().port(),
        );

        packet.add_chunk(Box::new(CookieAck::new()));

        self.stream.send(&Vec::<u8>::from(&packet)).await?;
        Ok(())
    }

    /// Sends user data
    pub async fn send(&mut self, user_data: &[u8]) -> Result<(), SCTPError> {

        // section 6.1 rule A)
        if self.remote_rwnd == 0 {
            // TODO abhi: send a zero probe
            return Err(SCTPError::RemoteBufferFull);
        }

        // if self.cwnd.as_ref().unwrap() > 0

        // section 6 note 1)
        let mtu = *self.mtu.as_ref().unwrap() as usize;

        if user_data.len() > mtu {
            // TODO abhi - fragment data into multiple chunks
            let mtu_sized_chunks = user_data.chunks(mtu);
            let len = mtu_sized_chunks.len();

            for (i, mtu_sized_chunk) in mtu_sized_chunks.enumerate() {
                let mut packet = Packet::new(
                    self.local_addr.port(),
                    self.remote_addr.as_ref().unwrap().port(),
                );
                packet.add_chunk(Box::new(Data::new(
                    self.tsn,
                    self.stream_id,
                    self.stream_seq_no,
                    0,
                    i == 0,
                    i == len - 1,
                    mtu_sized_chunk.to_vec(),
                )));

                self.stream.send(&Vec::<u8>::from(&packet)).await?;
            }
        } else {
            // we can send the entire user data in a single data chunk
                let mut packet = Packet::new(
                    self.local_addr.port(),
                    self.remote_addr.as_ref().unwrap().port(),
                );
                packet.add_chunk(Box::new(Data::new(
                    self.tsn,
                    self.stream_id,
                    self.stream_seq_no,
                    0,
                    true,
                    true,
                    user_data.to_vec(),
                )));
        }

        self.stream_seq_no += 1 % 65535;

        // 6.3.3 handle T3-rtx-Expiration
        match timeout(Duration::from_millis(self.rto), self.stream.recv()).await {
            Ok(bytes) => {}
            _ => {
                // 6.3.3.  Handle T3-rtx Expiration E1)
                self.ssthresh = Some(cmp::max(self.cwnd.unwrap() / 2, 4 * self.mtu.unwrap()));
                self.cwnd = self.mtu;

                // 6.3.3.  Handle T3-rtx Expiration E2)
                self.rto = self.rto * 2;

                // 6.3.3.  Handle T3-rtx Expiration E3)
                // TODO abhi: handle this case
            }
        }

        Ok(())
    }

    /// Recvs user data
    pub async fn recv(&mut self) {
        // this is the association recving function. it can recv any kind of a chunk.
        // it can recv data/error/abort/whatever. so every packet recvd should be
        // checked for the chunk type and then appropriate action should be taken.
        //
        // TODO abhi: when the recvr wnd is 0, drop any new incoming DATA chunk with
        // TSN larger than the largest TSN recvd so far.
        if let Ok(bytes) = self.stream.recv().await {
            if let Ok(packet) = Packet::try_from(bytes) {
                for chunk in packet.chunks {
                    match chunk.chunk_type() {
                        Data => {}
                        Init => {}
                        InitAck => {}
                        Sack => {}
                        Abort => {}
                        Shutdown => {}
                        CookieAck => {}
                        CookieEcho => {}
                        ShutdownComplete => {}
                        ShutdownAck => {}
                        Invalid => {}
                    }
                }
            }
        }
    }

    /// Graceful termination of the association
    pub async fn terminate(&self) -> Result<(), SCTPError> {
        // TODO: abhi - send all pending msgs from local msg queue
        todo!()
    }

    /// Non-graceful termination of the association
    pub async fn abort(&mut self) -> Result<(), SCTPError> {
        // TODO: abhi -
        // 1. destroy local msg queue
        let _ = self.msg_queue.drain(..);
        // 2. send ABORT chunk to peer

        let mut packet = Packet::new(
            self.local_addr.port(),
            self.remote_addr.as_ref().unwrap().port(),
        );

        // TODO abhi - pass a list of errors when creating the ABORT chunk
        packet.add_chunk(Box::new(Abort::new(None)));

        self.stream.send(&Vec::<u8>::from(&packet)).await?;
        Ok(())
    }
}
