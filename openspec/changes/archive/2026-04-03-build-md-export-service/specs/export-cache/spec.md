## ADDED Requirements

### Requirement: Cache conversion results in memory

The service SHALL cache conversion results using `moka::future::Cache` with a configurable TTL (default 3600s) and max entries (default 500). Cache values SHALL be `Bytes` (raw binary). The cache SHALL be stored in `AppState` and shared across requests.

#### Scenario: Cache miss triggers conversion

- **WHEN** a request arrives for content not in cache
- **THEN** the service runs Pandoc, stores the result in cache, and returns it with `X-Cache: MISS`

#### Scenario: Cache hit skips conversion

- **WHEN** a request arrives for content already in cache
- **THEN** the service returns the cached bytes without invoking Pandoc, with `X-Cache: HIT`

#### Scenario: TTL expiry

- **WHEN** a cached entry has been stored longer than `CACHE_TTL_SECS`
- **THEN** the next request for that content is treated as a cache miss

### Requirement: Compute cache key via xxh3 hash

The cache key SHALL be a `u64` computed as:
- PDF: `xxh3(markdown + "\0" + "pdf" + "\0" + typst_hash_hex)`
- DOCX: `xxh3(markdown + "\0" + "docx")`

`typst_hash` SHALL be computed once at startup by reading the Typst template file bytes and hashing them with xxh3. It SHALL be stored in `AppState`.

#### Scenario: Template change invalidates PDF cache

- **WHEN** the Typst template file is modified and the service is restarted
- **THEN** all previous PDF cache entries are effectively invalidated because the typst_hash changes

#### Scenario: DOCX key excludes template hash

- **WHEN** two DOCX requests arrive with identical markdown
- **THEN** they produce the same cache key regardless of the Typst template

### Requirement: Expose cache debug headers

Every response from `POST /export` SHALL include:
- `X-Cache: HIT` or `X-Cache: MISS`
- `X-Cache-Backend: memory` (or `redis` when Redis layer served the hit)

#### Scenario: Memory hit headers

- **WHEN** the in-memory cache serves a hit
- **THEN** response includes `X-Cache: HIT` and `X-Cache-Backend: memory`

#### Scenario: Miss headers

- **WHEN** no cache layer has the result
- **THEN** response includes `X-Cache: MISS` and `X-Cache-Backend: memory`

### Requirement: Optional Redis cache layer via feature flag

When built with the `redis-cache` Cargo feature AND `REDIS_URL` is set at runtime, the service SHALL use a two-layer cache: Redis checked first, then memory. On a miss in both, the result SHALL be written to both layers. The Redis key format SHALL be `md-export:<cache_key_hex>`. The Redis TTL SHALL match `CACHE_TTL_SECS`.

If `REDIS_URL` is absent at runtime (even with the feature compiled in), the service SHALL log a warning and fall back to memory-only cache.

#### Scenario: Redis hit

- **WHEN** Redis contains the key and `redis-cache` feature is active
- **THEN** response includes `X-Cache: HIT` and `X-Cache-Backend: redis`

#### Scenario: Redis miss, memory miss

- **WHEN** neither Redis nor memory has the key
- **THEN** service converts, writes to both Redis and memory, returns with `X-Cache: MISS`

#### Scenario: Redis miss, memory hit

- **WHEN** Redis does not have the key but in-memory cache does
- **THEN** response includes `X-Cache: HIT` and `X-Cache-Backend: memory`

#### Scenario: REDIS_URL absent with feature compiled

- **WHEN** the binary is compiled with `redis-cache` but `REDIS_URL` is not set
- **THEN** service starts successfully using memory-only cache and logs a warning
