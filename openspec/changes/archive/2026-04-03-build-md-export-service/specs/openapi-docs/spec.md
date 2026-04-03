## ADDED Requirements

### Requirement: Serve OpenAPI 3.x spec at /openapi.json

The service SHALL expose `GET /openapi.json` returning the full OpenAPI 3.x specification as JSON, generated at compile time via `utoipa` proc macros.

#### Scenario: OpenAPI spec available

- **WHEN** a client sends `GET /openapi.json`
- **THEN** the service returns `200 OK` with `Content-Type: application/json` and a valid OpenAPI 3.x JSON body

### Requirement: Serve Swagger UI at /swagger

The service SHALL expose `GET /swagger` serving the Swagger UI for interactive API exploration, sourced from the `/openapi.json` endpoint.

#### Scenario: Swagger UI accessible

- **WHEN** a client navigates to `GET /swagger`
- **THEN** the service returns `200 OK` with an HTML page rendering the Swagger UI

### Requirement: All public endpoints documented in OpenAPI spec

All request/response structs for `POST /export` and `GET /health` SHALL derive `utoipa::ToSchema`. Each handler SHALL be annotated with `#[utoipa::path(...)]`. Routes SHALL be registered via `OpenApiRouter` so path metadata is collected automatically.

#### Scenario: Export endpoint in spec

- **WHEN** the OpenAPI spec is fetched from `/openapi.json`
- **THEN** it includes a `POST /export` path entry with request and response schemas

#### Scenario: Health endpoint in spec

- **WHEN** the OpenAPI spec is fetched from `/openapi.json`
- **THEN** it includes a `GET /health` path entry with response schema
