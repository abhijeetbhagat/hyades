use crate::association::Association;
use crate::chunk::Init;
use crate::error::SCTPError;
use std::net::SocketAddr;
use tokio::net::UdpSocket;
use log::{debug, error, info};

pub struct SCTPEndpoint {
    local_addr: SocketAddr,
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
        local_addr: impl AsRef<str>,
        dst_addr: impl AsRef<str>,
        num_outbound_streams: u16,
    ) -> Result<Self, SCTPError> {
        let association = Association::new(local_addr, dst_addr).await?;
        todo!()
    }

    pub async fn listen(&self, local_addr: impl AsRef<str>) -> Result<(), SCTPError> {
        let local_addr: SocketAddr = local_addr
            .as_ref()
            .parse()
            .map_err(|_| SCTPError::InvalidLocalAddress)?;
        let socket = UdpSocket::bind(local_addr)
            .await
            .map_err(|_| SCTPError::SocketBindError)?;

        let mut buf = [0u8; 1024];
        socket
            .recv(&mut buf)
            .await
            .map_err(|_| SCTPError::SocketRecvError)?;
        info!("{:?}", Init::from(&buf));

        Ok(())
    }

    pub async fn shutdown(&self) {
        let _ = self.association.terminate().await;
    }

    pub async fn abort(&self) {
        let _ = self.association.abort().await;
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
