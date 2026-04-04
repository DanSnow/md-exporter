use axum::{Json, extract::State, response::IntoResponse};
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
    ),
    tag = "export"
)]
pub async fn health_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    Json(HealthResponse {
        status: "ok".into(),
        pandoc_version: state.pandoc_version.clone(),
        typst_version: state.typst_version.clone(),
        cache_backend: state.cache_backend_name().into(),
    })
    .into_response()
}

pub(crate) async fn probe_version(bin: &str) -> Result<String, String> {
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
