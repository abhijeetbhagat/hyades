use crate::association::Association;
use crate::chunk::Init;
use crate::error::SCTPError;
use log::{debug, error, info};
use std::net::SocketAddr;
use tokio::net::UdpSocket;

/// An SCTP endpoint.
/// All methods inside this struct are meant to be called from the ULP.
pub struct SCTPEndpoint {
    local_addr: SocketAddr,
    dst_addr: Option<SocketAddr>,
    association: Association,
}

impl SCTPEndpoint {
    pub fn initialize(local_port: u16) -> Self {
        todo!()
    }

    /// Create an association from sender side
    pub async fn associate_send(
        local_addr: impl AsRef<str>,
        dst_addr: impl AsRef<str>,
        num_outbound_streams: u16,
    ) -> Result<Self, SCTPError> {
        let local_address: SocketAddr = local_addr
            .as_ref()
            .parse()
            .map_err(|_| SCTPError::InvalidLocalAddress)?;
        let dst_address: SocketAddr = dst_addr
            .as_ref()
            .parse()
            .map_err(|_| SCTPError::InvalidRemoteAddress)?;

        let association = Association::new_sender(local_addr, dst_addr).await?;

        Ok(Self {
            local_addr: local_address,
            dst_addr: Some(dst_address),
            association,
        })
    }

    /// Create an association from receiver side
    pub async fn associate_recv(local_addr: impl AsRef<str>) -> Result<Self, SCTPError> {
        let local_address: SocketAddr = local_addr
            .as_ref()
            .parse()
            .map_err(|_| SCTPError::InvalidLocalAddress)?;

        let association = Association::new_recvr(local_addr).await?;

        Ok(Self {
            local_addr: local_address,
            dst_addr: None,
            association,
        })
    }

    /// Shutdown an association
    pub async fn shutdown(&self) {
        let _ = self.association.terminate().await;
    }

    /// Abort an association
    pub async fn abort(&mut self) {
        let _ = self.association.abort().await;
    }

    /// Send data to the associated endpoint
    pub async fn send(&self, bytes: &[u8]) {}

    pub fn set_primary(&self) {
        // Not needed for WebRTC
    }

    pub async fn receive(&self) -> Vec<u8> {
        // TODO: abhi - pop message from association's queue
        // let packet = self.association.msg_queue.pop_front();
        todo!()
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
