use crate::chunk::{CookieAck, CookieEcho, Init, InitAck, ParamType, Parameter};
use crate::cookie::Cookie;
use crate::error::SCTPError;
use crate::packet::Packet;
use crate::stream::Stream;
use log::{debug, info};
use rand::{rngs::ThreadRng, thread_rng, Rng};
use std::collections::VecDeque;
use std::net::SocketAddr;
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
    msg_queue: VecDeque<Packet>,
    rng: ThreadRng,
    init_tag: u32,
    max_retries: u8,
    max_init_retries: u8,
    rto: u64
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

        let stream = Stream::new(local_addr).await?.connect(remote_addr).await?;

        let mut association = Self {
            id: "nra".to_owned(),
            stream,
            rng: thread_rng(),
            msg_queue: VecDeque::new(),
            local_addr: local_sockaddr,
            remote_addr: Some(remote_sockaddr),
            init_tag: 0,
            max_retries: ASSOCIATION_MAX_RETRANS,
            max_init_retries: MAX_INIT_RETRANSMITS,
            rto: RTO_INITIAL * 1000
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
            max_retries: ASSOCIATION_MAX_RETRANS,
            max_init_retries: MAX_INIT_RETRANSMITS,
            rto: RTO_INITIAL * 1000
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
            return Err(SCTPError::CookieMismatchError)
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

    /// Graceful termination of the association
    pub async fn terminate(&self) -> Result<(), SCTPError> {
        // TODO: abhi - send all pending msgs from local msg queue
        todo!()
    }

    /// Non-graceful termination of the association
    pub async fn abort(&self) {
        // TODO: abhi -
        // 1. destroy local msg queue
        // 2. send ABORT chunk to peer
    }
}
