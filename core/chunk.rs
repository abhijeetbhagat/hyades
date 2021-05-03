#[derive(Clone, Debug)]
enum ChunkType {
    Init,
    InitAck,
    Data,
    CookieEcho,
}

#[derive(Clone, Debug)]
struct Chunk {
    tsn: u32,
    chunk_type: ChunkType,
}

impl Chunk {
    fn new(chunk_type: ChunkType) -> Self {
        Self { tsn: 0, chunk_type }
    }
}
