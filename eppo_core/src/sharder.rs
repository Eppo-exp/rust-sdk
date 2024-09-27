//! Sharder implementation.
use md5;

/// Compute md5 shard for the set of inputs.
///
/// This function accepts an array of inputs to allow the caller to avoid allocating memory when
/// input is compound from multiple segments.
pub fn get_md5_shard(input: &[impl AsRef<[u8]>], total_shards: u32) -> u32 {
    let hash = {
        let mut hasher = md5::Context::new();
        for i in input {
            hasher.consume(i);
        }
        hasher.compute()
    };
    let value = u32::from_be_bytes(hash[0..4].try_into().unwrap());
    value % total_shards
}
