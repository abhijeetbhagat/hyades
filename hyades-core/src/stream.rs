use error::SCTPError;
use tokio::net::UdpSocket;

struct Stream {
    // TODO abhi: will have an UDP socket here
    sock: UdpSocket,
}

impl Stream {
    /// Creates a new UDP stream
    async fn new(remote_addr: impl AsRef<str>) -> Result<Self, SCTPError> {
        let sock = UdpSocket::bind(remote_addr.parse::<SocketAddr>().unwrap())
            .await
            .map_err(|e| SCTPError::SocketBindError)?;
        // Set default remote addr to sent data to/recv data from
        sock.connect(remote_addr.parse::<SocketAddr>().unwrap())
            .await
            .map_err(|e| SCTPError::SocketConnectError)?;

        Ok(Self { sock })
    }

    /// Send data to remote peer
    async fn send(&self, buf: &[u8]) -> Result<(), SCTPError> {
        self.sock
            .send(buf)
            .await
            .map_err(|e| SCTPError::SocketSendError)?
    }

    /// Recv data from remote peer
    async fn recv(&self) -> Result<Vec<u8>, SCTPError> {
        // TODO abhi - set min vec capacity from SCTP RFC
        let mut buf = Vec::new();
        self.sock
            .recv(&mut buf)
            .await
            .map_err(|e| SCTPError::SocketRecvError)?;
        buf
    }
}
