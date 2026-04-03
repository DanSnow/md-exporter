use axum::{
    Json,
    extract::State,
    http::{HeaderMap, HeaderValue, header},
    response::IntoResponse,
};
use serde::Deserialize;
use std::{borrow::Cow, sync::Arc};
use utoipa::ToSchema;

use crate::{
    AppState,
    cache::{CacheResult, compute_key},
    converter::{ConvertRequest, ExportFormat, convert},
    error::AppError,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct ExportRequest {
    pub markdown: String,
    pub format: ExportFormat,
    pub filename: Option<String>,
    pub inline: Option<bool>,
}

#[utoipa::path(
    post,
    path = "/export",
    request_body = ExportRequest,
    responses(
        (status = 200, description = "Converted file binary"),
        (status = 400, description = "Invalid request"),
        (status = 422, description = "Conversion failed"),
        (status = 500, description = "Internal error"),
    ),
    tag = "export"
)]
pub async fn export_handler(
    State(state): State<Arc<AppState>>,
    Json(req): Json<ExportRequest>,
) -> Result<impl IntoResponse, AppError> {
    if req.markdown.trim().is_empty() {
        return Err(AppError::InvalidRequest(
            "markdown must not be empty".into(),
        ));
    }

    let format = req.format;
    let markdown = req.markdown;

    let typst_hash_opt = match format {
        ExportFormat::Pdf => Some(state.typst_hash),
        ExportFormat::Docx => None,
    };
    let cache_key = compute_key(&markdown, format.as_str(), typst_hash_opt);

    let (body, cache_result) = match state.cache.get(cache_key).await {
        Some(cached) => (
            cached,
            CacheResult::Hit {
                backend: state.cache_backend_name(),
            },
        ),
        None => {
            let converted = convert(
                ConvertRequest { markdown, format },
                &state.config,
                &state.typst_env,
            )
            .await?;
            state.cache.set(cache_key, converted.clone()).await;
            (converted, CacheResult::Miss)
        }
    };

    let (x_cache, x_cache_backend) = match &cache_result {
        CacheResult::Hit { backend, .. } => ("HIT", *backend),
        CacheResult::Miss => ("MISS", state.cache_backend_name()),
    };

    let disposition = if req.inline.unwrap_or(false) {
        Cow::from("inline")
    } else {
        let filename = req.filename.as_deref().unwrap_or(format.default_filename());
        Cow::Owned(format!("attachment; filename=\"{}\"", filename))
    };

    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static(format.content_type()),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&disposition)
            .map_err(|e| AppError::InternalError(anyhow::anyhow!("header error: {}", e)))?,
    );
    headers.insert("X-Cache", HeaderValue::from_static(x_cache));
    headers.insert("X-Cache-Backend", HeaderValue::from_static(x_cache_backend));

    Ok((headers, body))
}
