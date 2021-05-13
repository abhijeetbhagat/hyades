use crate::error::SCTPError;
use std::net::SocketAddr;
use tokio::net::UdpSocket;

pub struct Stream {
    sock: UdpSocket,
}

impl Stream {
    /// Creates a new UDP stream
    pub async fn new(remote_addr: impl AsRef<str>) -> Result<Self, SCTPError> {
        let sockaddr: SocketAddr = remote_addr
            .as_ref()
            .parse()
            .map_err(|_| SCTPError::InvalidRemoteAddress)?;
        let sock = UdpSocket::bind(sockaddr)
            .await
            .map_err(|_| SCTPError::SocketBindError)?;

        // Set default remote addr to sent data to/recv data from
        sock.connect(sockaddr)
            .await
            .map_err(|_| SCTPError::SocketConnectError)?;

        Ok(Self { sock })
    }

    /// Send data to remote peer
    pub async fn send(&self, buf: &[u8]) -> Result<(), SCTPError> {
        self.sock
            .send(buf)
            .await
            .map_err(|_| SCTPError::SocketSendError)?;
        Ok(())
    }

    /// Recv data from remote peer
    pub async fn recv(&self) -> Result<Vec<u8>, SCTPError> {
        // TODO abhi - set min vec capacity from SCTP RFC
        let mut buf = Vec::new();
        self.sock
            .recv(&mut buf)
            .await
            .map_err(|_| SCTPError::SocketRecvError)?;
        Ok(buf)
    }
}
