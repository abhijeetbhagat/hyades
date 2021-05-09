use thiserror::Error;

#[derive(Error, Debug)]
enum SCTPError {
    #[error("error setting up an association")]
    AssociationSetupError,
    #[error("error terminating association")]
    AssociationTerminationError,
    #[error("error binding local addr to socket")]
    SocketBindError,
    #[error("error connecting to remote addr")]
    SocketConnectError,
    #[error("error sending data to remote addr")]
    SocketSendError,
    #[error("error recving data from remote addr")]
    SocketRecvError,
}
