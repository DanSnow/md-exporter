use async_trait::async_trait;
use bytes::Bytes;
use moka::future::Cache;
use std::time::Duration;

use super::{CacheBackend, CacheResult};

pub struct MemoryCache {
    inner: Cache<u64, Bytes>,
}

impl MemoryCache {
    pub fn new(max_entries: u64, ttl_secs: u64) -> Self {
        let inner = Cache::builder()
            .max_capacity(max_entries)
            .time_to_live(Duration::from_secs(ttl_secs))
            .build();
        Self { inner }
    }
}

#[async_trait]
impl CacheBackend for MemoryCache {
    fn name(&self) -> &'static str {
        "memory"
    }

    async fn get(&self, key: u64) -> CacheResult {
        match self.inner.get(&key).await {
            Some(data) => CacheResult::Hit {
                backend: "memory",
                data,
            },
            None => CacheResult::Miss { backend: "memory" },
        }
    }

    async fn set(&self, key: u64, value: Bytes) {
        self.inner.insert(key, value).await;
    }
}
