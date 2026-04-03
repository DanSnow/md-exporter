use utoipa::OpenApi;

use crate::routes::{export::ExportRequest, health::HealthResponse};

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routes::export::export_handler,
        crate::routes::health::health_handler,
    ),
    components(schemas(ExportRequest, HealthResponse)),
    tags((name = "export", description = "Markdown export API"))
)]
pub struct ApiDoc;
