use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;
use std::sync::Arc;
use tokio::process::Command;
use utoipa::ToSchema;

use crate::AppState;

#[derive(Serialize, ToSchema)]
pub struct HealthResponse {
    pub status: String,
    pub pandoc_version: String,
    pub typst_version: String,
    pub cache_backend: String,
}


#[utoipa::path(
    get,
    path = "/health",
    responses(
        (status = 200, description = "Service healthy", body = HealthResponse),
        (status = 500, description = "Binary missing or not executable"),
    ),
    tag = "export"
)]
pub async fn health_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let pandoc_bin = state.config.pandoc_bin.as_deref().unwrap_or("pandoc");
    let typst_bin = &state.config.typst_bin;

    let (pandoc_result, typst_result) =
        tokio::join!(probe_version(pandoc_bin), probe_version(typst_bin));

    let pandoc_version = match pandoc_result {
        Ok(v) => v,
        Err(msg) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "status": "error", "message": msg })),
            )
                .into_response();
        }
    };

    let typst_version = match typst_result {
        Ok(v) => v,
        Err(msg) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "status": "error", "message": msg })),
            )
                .into_response();
        }
    };

    Json(HealthResponse {
        status: "ok".into(),
        pandoc_version,
        typst_version,
        cache_backend: state.cache_backend_name().into(),
    })
    .into_response()
}

async fn probe_version(bin: &str) -> Result<String, String> {
    let output = Command::new(bin)
        .arg("--version")
        .output()
        .await
        .map_err(|_| format!("{} not found", bin))?;

    if !output.status.success() {
        return Err(format!("{} not executable", bin));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let version = stdout.lines().next().unwrap_or("unknown").to_string();
    Ok(version)
}
