use figment2::{Figment, providers::Env};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default = "default_port")]
    pub port: u16,

    pub pandoc_bin: Option<String>,

    #[serde(default = "default_typst_bin")]
    pub typst_bin: String,

    #[serde(default = "default_typst_template")]
    pub typst_template: String,

    #[serde(default = "default_reference_docx")]
    pub reference_docx: String,

    #[serde(default = "default_cache_ttl_secs")]
    pub cache_ttl_secs: u64,

    #[serde(default = "default_cache_max_entries")]
    pub cache_max_entries: u64,

    #[serde(default = "default_conversion_timeout_secs")]
    pub conversion_timeout_secs: u64,

    #[cfg(feature = "redis-cache")]
    pub redis_url: Option<String>,

    #[serde(default = "default_lua_filter")]
    pub lua_filter: String,

    #[serde(default = "default_log_level")]
    pub log_level: String,
}

fn default_port() -> u16 { 8080 }
fn default_typst_bin() -> String { "typst".into() }
fn default_typst_template() -> String { "templates/default.typ".into() }
fn default_reference_docx() -> String { "templates/reference.docx".into() }
fn default_cache_ttl_secs() -> u64 { 3600 }
fn default_cache_max_entries() -> u64 { 500 }
fn default_conversion_timeout_secs() -> u64 { 30 }
fn default_lua_filter() -> String { "filters/table-auto-width.lua".into() }
fn default_log_level() -> String { "info".into() }

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let config = Figment::new()
            .merge(Env::raw())
            .extract()?;
        Ok(config)
    }
}
