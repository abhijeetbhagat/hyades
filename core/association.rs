use error::SCTPError;

struct Assoication {
    id: String,
    stream: Stream,
}

impl Association {
    fn new(id: String) -> Self {
        Self { id }
    }

    /// Graceful termination of the association
    fn terminate(&self) -> Result<(), SCTPError> {
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
