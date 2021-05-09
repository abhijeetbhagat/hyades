use rand::{thread_rng, Rng};

struct Endpoint {
    association: Association,
    rng: ThreadRng,
}

impl Endpoint {
    fn new() -> Self {
        Self {
            association: Association,
            rng: thread_rng(),
        }
    }

    async fn initialization(&self) -> Result<(), std::io::Error> {
        let ver_tag: u32 = self.rng.gen_rage(1..=4294967295);
        let init_chunk = Chunk::new(ChunkType::Init);
        let init_ack = send(init_chunk).await?;
        Ok(())
    }
}

struct SCTPEndpoint {
    local_port: u16,
    remote_ip: u32,
    remote_port: u16,
    association: Association,
}

impl SCTPEndpoint {
    pub fn initialize(local_port: u16) -> Self {
        todo!()
    }

    pub async fn associate(
        &self,
        dst_addr: impl AsRef<str>,
        num_outbound_streams: u16,
    ) -> Result<Self, SCTPError> {
        let association = Association::start_4_way_handshake().await?;
        todo!()
    }

    pub fn shutdown(&self) {
        self.association.terminate();
    }

    pub fn abort(&self) {
        self.association.abort();
    }

    pub fn send(&self) {}

    pub fn set_primary(&self) {
        // Not needed for WebRTC
    }

    pub fn receive(&self) {
        // TODO: abhi - pop message from association's queue
        //
    }

    pub fn status(&self) {}

    pub fn change_heartbeat(&self) {}

    pub fn request_heartbeat(&self) {}

    pub fn get_srtt_report(&self) {}

    pub fn set_failure_threshold(&self) {}

    pub fn set_protocol_params(&self) {}

    pub fn recv_unsent_msg(&self) {}

    pub fn recv_unacked_msg(&self) {}

    pub fn destroy(&self) {}
}
