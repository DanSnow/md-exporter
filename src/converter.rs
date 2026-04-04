use bytes::Bytes;
use std::io::Write;
use std::time::Duration;
use tempfile::NamedTempFile;
use tokio::process::Command;
use tokio::time::timeout;

use crate::config::Config;
use crate::error::AppError;

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize, utoipa::ToSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum ExportFormat {
    Pdf,
    Docx,
}

impl ExportFormat {
    pub fn as_str(self) -> &'static str {
        match self {
            ExportFormat::Pdf => "pdf",
            ExportFormat::Docx => "docx",
        }
    }

    pub fn content_type(self) -> &'static str {
        match self {
            ExportFormat::Pdf => "application/pdf",
            ExportFormat::Docx => {
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
            }
        }
    }

    pub fn default_filename(self) -> &'static str {
        match self {
            ExportFormat::Pdf => "export.pdf",
            ExportFormat::Docx => "export.docx",
        }
    }
}

pub struct ConvertRequest {
    pub markdown: String,
    pub format: ExportFormat,
}

pub async fn convert(
    req: ConvertRequest,
    config: &Config,
    typst_env: &minijinja::Environment<'_>,
) -> Result<Bytes, AppError> {
    match req.format {
        ExportFormat::Pdf => convert_pdf(req.markdown, config, typst_env).await,
        ExportFormat::Docx => convert_docx(req.markdown, config).await,
    }
}

async fn convert_pdf(
    markdown: String,
    config: &Config,
    typst_env: &minijinja::Environment<'_>,
) -> Result<Bytes, AppError> {
    let rendered = typst_env
        .get_template("default.typ")
        .and_then(|t| t.render(minijinja::context! {}))
        .map_err(|e| AppError::InternalError(anyhow::anyhow!("template render error: {}", e)))?;

    // Held alive across .await so pandoc can read it
    let typst_tmp = write_tempfile(rendered.as_bytes())?;

    run_pandoc(
        &markdown,
        config,
        &[
            "--to=pdf".to_string(),
            "--pdf-engine=typst".to_string(),
            format!("--template={}", typst_tmp.path().display()),
            format!("--lua-filter={}", config.lua_filter),
        ],
        &[&typst_tmp],
    )
    .await
}

async fn convert_docx(markdown: String, config: &Config) -> Result<Bytes, AppError> {
    run_pandoc(
        &markdown,
        config,
        &[
            "--to=docx".to_string(),
            format!("--reference-doc={}", config.reference_docx),
        ],
        &[],
    )
    .await
}

fn write_tempfile(content: &[u8]) -> Result<NamedTempFile, AppError> {
    let mut tmp = NamedTempFile::new()
        .map_err(|e| AppError::InternalError(anyhow::anyhow!("tempfile error: {}", e)))?;
    tmp.write_all(content)
        .map_err(|e| AppError::InternalError(anyhow::anyhow!("tempfile write error: {}", e)))?;
    Ok(tmp)
}

async fn run_pandoc(
    markdown: &str,
    config: &Config,
    extra_args: &[String],
    // Kept alive here so callers' temp files survive across .await
    _keep_alive: &[&NamedTempFile],
) -> Result<Bytes, AppError> {
    let input_tmp = write_tempfile(markdown.as_bytes())?;
    let output_tmp = NamedTempFile::new()
        .map_err(|e| AppError::InternalError(anyhow::anyhow!("tempfile error: {}", e)))?;

    let pandoc_bin = config.pandoc_bin.as_deref().unwrap_or("pandoc");
    let mut cmd = Command::new(pandoc_bin);
    cmd.arg("--from=markdown")
        .args(extra_args)
        .arg("-o")
        .arg(output_tmp.path())
        .arg(input_tmp.path())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped());

    let child = cmd
        .spawn()
        .map_err(|e| AppError::InternalError(anyhow::anyhow!("pandoc spawn error: {}", e)))?;

    let result = timeout(
        Duration::from_secs(config.conversion_timeout_secs),
        child.wait_with_output(),
    )
    .await;

    let output = match result {
        Ok(Ok(out)) => out,
        Ok(Err(e)) => {
            return Err(AppError::InternalError(anyhow::anyhow!(
                "pandoc error: {}",
                e
            )));
        }
        Err(_) => return Err(AppError::ConversionFailed("timeout".into())),
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
        return Err(AppError::ConversionFailed(stderr));
    }

    let bytes = tokio::fs::read(output_tmp.path())
        .await
        .map_err(|e| AppError::InternalError(anyhow::anyhow!("read output error: {}", e)))?;

    Ok(Bytes::from(bytes))
}
