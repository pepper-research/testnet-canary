use reed_solomon_erasure::galois_8::ReedSolomon;
use thiserror::Error;

pub const SHARD_COUNT: usize = 11; // arbitary, still testing what's safe upper limit
pub const SHARD_SIZE: usize = 102; // 102 Bytes
pub const PARITY_SHARD_COUNT: usize = 4; // cieled value of 11 * 0.33

#[derive(Error, Debug)]
pub enum FecError {
    #[error("Reed-Solomon error: {0}")]
    ReedSolomonError(String),
}

pub struct FecCodec {
    reed_solomon: ReedSolomon,
}

impl FecCodec {
    pub fn new() -> Result<Self, FecError> {
        let reed_solomon = ReedSolomon::new(SHARD_COUNT, PARITY_SHARD_COUNT)
            .map_err(|e| FecError::ReedSolomonError(e.to_string()))?;
        Ok(Self { reed_solomon })
    }

    pub fn encode(&self, data: &[u8]) -> Result<Vec<u8>, FecError> {
        let mut shards: Vec<Vec<u8>> = data
            .chunks(SHARD_SIZE)
            .map(|chunk| {
                let mut shard = chunk.to_vec();
                shard.resize(SHARD_SIZE, 0);
                shard
            })
            .collect();

        shards.resize(SHARD_COUNT + PARITY_SHARD_COUNT, vec![0; SHARD_SIZE]);

        self.reed_solomon
            .encode(&mut shards)
            .map_err(|e| FecError::ReedSolomonError(e.to_string()))?;

        Ok(shards.into_iter().flatten().collect())
    }

    pub fn decode(&self, data: &[u8]) -> Result<Vec<u8>, FecError> {
        let mut shards: Vec<Option<Vec<u8>>> = data
            .chunks(SHARD_SIZE)
            .map(|chunk| Some(chunk.to_vec()))
            .collect();

        self.reed_solomon
            .reconstruct(&mut shards)
            .map_err(|e| FecError::ReedSolomonError(e.to_string()))?;

        Ok(shards
            .into_iter()
            .take(SHARD_COUNT)
            .filter_map(|s| s)
            .flatten()
            .collect())
    }
}