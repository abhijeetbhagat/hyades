use crate::error::SCTPError;
use log::info;
use std::net::SocketAddr;
use tokio::net::UdpSocket;

pub struct Stream {
    sock: UdpSocket,
}

impl Stream {
    /// Creates a new UDP stream
    pub async fn new(local_addr: impl AsRef<str>) -> Result<Self, SCTPError> {
        let local_sockaddr: SocketAddr = local_addr
            .as_ref()
            .parse()
            .map_err(|_| SCTPError::InvalidLocalAddress)?;

        let sock = UdpSocket::bind(local_sockaddr)
            .await
            .map_err(|_| SCTPError::SocketBindError)?;

        Ok(Self { sock })
    }

    pub async fn connect(self, remote_addr: impl AsRef<str>) -> Result<Stream, SCTPError> {
        let remote_sockaddr: SocketAddr = remote_addr
            .as_ref()
            .parse()
            .map_err(|_| SCTPError::InvalidRemoteAddress)?;

        // Set default remote addr to sent data to/recv data from
        self.sock
            .connect(remote_sockaddr)
            .await
            .map_err(|_| SCTPError::SocketConnectError)?;

        Ok(self)
    }

    /// Send data to remote peer
    pub async fn send(&self, buf: &[u8]) -> Result<(), SCTPError> {
        self.sock
            .send(buf)
            .await
            .map_err(|_| SCTPError::SocketSendError)?;
        Ok(())
    }

    /// Recv data from remote peer this stream is already connected to
    pub async fn recv(&self) -> Result<Vec<u8>, SCTPError> {
        // TODO abhi - set min vec capacity from SCTP RFC
        let mut buf = [0u8; 1024]; // Vec::new();

        let len = self
            .sock
            .recv(&mut buf)
            .await
            .map_err(|_| SCTPError::SocketRecvError)?;
        Ok(buf[..len].to_vec())
    }

    /// Recv data from a remote peer this stream isn't connected to
    pub async fn recv_from(&self) -> Result<(Vec<u8>, SocketAddr), SCTPError> {
        // TODO abhi - set min vec capacity from SCTP RFC
        let mut buf = [0u8; 1024]; // Vec::new();
        let (len, addr) = self
            .sock
            .recv_from(&mut buf)
            .await
            .map_err(|_| SCTPError::SocketRecvError)?;

        self.sock
            .connect(addr)
            .await
            .map_err(|_| SCTPError::SocketConnectError)?;

        Ok((buf[..len].to_vec(), addr))
    }
}
