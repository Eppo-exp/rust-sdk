//! Sharder implementation.
use md5;

pub trait Sharder {
    fn get_shard(&self, input: impl AsRef<[u8]>, total_shards: u64) -> u64;
}

/// The default (and only) sharder.
pub struct Md5Sharder;

impl Sharder for Md5Sharder {
    fn get_shard(&self, input: impl AsRef<[u8]>, total_shards: u64) -> u64 {
        let hash = md5::compute(input);
        let value = u32::from_be_bytes(hash[0..4].try_into().unwrap());
        (value as u64) % total_shards
    }
}
