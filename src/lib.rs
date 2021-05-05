struct SCTPEndpoint {
    local_port: u16,
    stream: Stream,
}

impl SCTPEndpoint {
    pub fn initialize(local_port: u16) -> Self {}

    pub fn associate(&self, dst_addr: impl AsRef<str>, num_outbound_streams: u16) {}

    pub fn shutdown(&self) {}

    pub fn abort(&self) {}

    pub fn send(&self) {}

    pub fn set_primary(&self) {}

    pub fn receive(&self) {}

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
