use async_trait::async_trait;
use bytes::Bytes;
use fred::prelude::*;
use std::sync::Arc;

use super::{CacheBackend, memory::MemoryCache};

pub struct RedisCache {
    client: Client,
    ttl_secs: i64,
}

impl RedisCache {
    pub async fn new(url: &str, ttl_secs: u64) -> anyhow::Result<Self> {
        let config = Config::from_url(url)?;
        let client = Client::new(config, None, None, None);
        client.connect();
        client.wait_for_connect().await?;
        Ok(Self { client, ttl_secs: ttl_secs as i64 })
    }

    fn key(cache_key: u64) -> String {
        format!("md-export:{:x}", cache_key)
    }
}

#[async_trait]
impl CacheBackend for RedisCache {
    async fn get(&self, key: u64) -> Option<Bytes> {
        let result: Option<Vec<u8>> = self.client.get(Self::key(key)).await.ok()?;
        result.map(Bytes::from)
    }

    async fn set(&self, key: u64, value: Bytes) {
        let _: Result<(), _> = self
            .client
            .set(
                Self::key(key),
                value.to_vec(),
                Some(Expiration::EX(self.ttl_secs)),
                None,
                false,
            )
            .await;
    }
}

/// Two-layer Redis cache is a single struct, not trait composition.
/// Lookup order: Redis → memory → miss. Write to both on conversion.
pub struct TwoLayerCache {
    redis: RedisCache,
    memory: Arc<MemoryCache>,
}

impl TwoLayerCache {
    pub fn new(redis: RedisCache, memory: Arc<MemoryCache>) -> Self {
        Self { redis, memory }
    }
}

#[async_trait]
impl CacheBackend for TwoLayerCache {
    async fn get(&self, key: u64) -> Option<Bytes> {
        if let Some(val) = self.redis.get(key).await {
            return Some(val);
        }
        self.memory.get(key).await
    }

    async fn set(&self, key: u64, value: Bytes) {
        tokio::join!(
            self.redis.set(key, value.clone()),
            self.memory.set(key, value),
        );
    }
}
