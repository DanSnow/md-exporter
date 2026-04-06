use async_trait::async_trait;
use bytes::Bytes;
use xxhash_rust::xxh3::xxh3_64;

pub mod memory;

#[cfg(feature = "redis-cache")]
pub mod redis;

#[async_trait]
pub trait CacheBackend: Send + Sync {
    fn name(&self) -> &'static str;
    async fn get(&self, key: u64) -> CacheResult;
    async fn set(&self, key: u64, value: Bytes);
}

pub enum CacheResult {
    Hit { backend: &'static str, data: Bytes },
    Miss { backend: &'static str },
}

/// Compute cache key: u64 from xxh3
/// PDF:  xxh3(markdown + "\0" + "pdf"  + "\0" + typst_hash_hex)
/// DOCX: xxh3(markdown + "\0" + "docx")
pub fn compute_key(markdown: &str, format: &str, typst_hash: Option<u64>) -> u64 {
    let mut input = format!("{}\0{}", markdown, format);
    if let Some(hash) = typst_hash {
        input.push('\0');
        input.push_str(&format!("{:x}", hash));
    }
    xxh3_64(input.as_bytes())
}
