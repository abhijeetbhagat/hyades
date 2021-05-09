use self::packet::Packet;
use error::SCTPError;
use std::collections::VecDeque;

struct Association {
    id: String,
    stream: Stream,
    msg_queue: VecDeque<Packet>,
}

impl Association {
    pub async fn new(id: String, remote_addr: impl AsRef<str>) -> Result<Self, SCTPError> {
        let stream = Stream::new(remote_addr).await?;
        let association = Self { id, stream };
        association.start_4_way_handshake().await?;
        Ok(association)
    }

    async fn start_4_way_handshake(&self) -> Result<Self, SCTPError> {
        self.send_init().await?;
        let init_ack = self.stream.recv().await?;
        self.send_cookie_echo().await?;
        let cookie_ack = self.stream.recv().await?;
    }

    async fn send_init(&self) {
        let mut packet = Packet::new();
        packet.add(Box::new(Init::new()));

        // self.stream.send(Init);
    }

    async fn send_cookie_echo(&self) {
        // TODO abhi - send a cookie echo
    }

    /// Graceful termination of the association
    pub async fn terminate(&self) -> Result<(), SCTPError> {
        // TODO: abhi - send all pending msgs from local msg queue
        todo!()
    }

    /// Non-graceful termination of the association
    fn abort(&self) {
        // TODO: abhi -
        // 1. destroy local msg queue
        // 2. send ABORT chunk to peer
    }
}
