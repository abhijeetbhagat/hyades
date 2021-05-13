use crate::chunk::{Init, InitAck};
use crate::error::SCTPError;
use crate::packet::Packet;
use crate::stream::Stream;
use rand::{rngs::ThreadRng, thread_rng, Rng};
use std::collections::VecDeque;
use std::net::SocketAddr;

pub struct Association {
    id: String,
    local_addr: SocketAddr,
    remote_addr: SocketAddr,
    stream: Stream,
    msg_queue: VecDeque<Packet>,
    rng: ThreadRng,
}

impl Association {
    pub async fn new(
        local_addr: impl AsRef<str>,
        remote_addr: impl AsRef<str>,
    ) -> Result<Self, SCTPError> {
        let local_sockaddr: SocketAddr = local_addr
            .as_ref()
            .parse()
            .map_err(|e| SCTPError::InvalidLocalAddress)?;
        let remote_sockaddr: SocketAddr = remote_addr
            .as_ref()
            .parse()
            .map_err(|e| SCTPError::InvalidRemoteAddress)?;
        let stream = Stream::new(remote_addr).await?;
        let mut association = Self {
            id: "nra".to_owned(),
            stream,
            rng: thread_rng(),
            msg_queue: VecDeque::new(),
            local_addr: local_sockaddr,
            remote_addr: remote_sockaddr,
        };
        association.start_4_way_handshake().await?;
        Ok(association)
    }

    async fn start_4_way_handshake(&mut self) -> Result<(), SCTPError> {
        self.send_init().await?;
        let init_ack = self.stream.recv().await?;
        self.send_cookie_echo().await?;
        let cookie_ack = self.stream.recv().await?;
        Ok(())
    }

    async fn send_init(&mut self) -> Result<(), SCTPError> {
        let ver_tag: u32 = self.rng.gen_range(1..=4294967295);
        let mut packet = Packet::new(self.local_addr.port(), self.remote_addr.port());
        let a_rwnd = 10000;
        packet.add_chunk(Box::new(Init::new(ver_tag, a_rwnd, 1, 1, None)));

        // self.stream.send(Init);
        todo!()
    }

    async fn send_cookie_echo(&self) -> Result<(), SCTPError> {
        // TODO abhi - send a cookie echo
        todo!()
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
