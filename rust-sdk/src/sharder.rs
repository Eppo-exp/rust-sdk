use md5;

pub trait Sharder {
    fn get_shard(&self, input: &str, total_shards: u64) -> u64;
}

pub struct Md5Sharder;

impl Sharder for Md5Sharder {
    fn get_shard(&self, input: &str, total_shards: u64) -> u64 {
        let hash = md5::compute(input);
        let int_from_hash: u64 = (hash[0] as u64) << 24
            | (hash[1] as u64) << 16
            | (hash[2] as u64) << 8
            | (hash[3] as u64) << 0;
        int_from_hash % total_shards
    }
}

#[cfg(test)]
pub struct DeterministicSharder(std::collections::HashMap<String, u64>);

#[cfg(test)]
impl Sharder for DeterministicSharder {
    fn get_shard(&self, input: &str, total_shards: u64) -> u64 {
        self.0.get(input).copied().unwrap_or(0) % total_shards
    }
}

#[cfg(test)]
mod tests {
    use crate::sharder::{Md5Sharder, Sharder};

    #[test]
    fn test_md5_sharder() {
        assert_eq!(Md5Sharder.get_shard("test-input", 10_000), 5619);
        assert_eq!(Md5Sharder.get_shard("alice", 10_000), 3170);
        assert_eq!(Md5Sharder.get_shard("bob", 10_000), 7420);
        assert_eq!(Md5Sharder.get_shard("charlie", 10_000), 7497);
    }
}
