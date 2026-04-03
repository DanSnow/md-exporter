# md-exporter

A lightweight HTTP service that converts Markdown documents to **PDF** or **DOCX** format via a simple REST API. Built with Rust, powered by [Pandoc](https://pandoc.org/) and [Typst](https://typst.app/).

## Features

- **PDF** export via Typst (customizable templates)
- **DOCX** export via Pandoc (customizable reference document)
- In-memory caching with TTL (default: 1 hour)
- Optional Redis cache for distributed deployments
- Auto-generated OpenAPI spec + Swagger UI at `/swagger`
- Health check endpoint with binary version info
- Docker-ready with multi-stage build

## API

### `POST /export`

Convert Markdown to a file.

**Request body** (`application/json`):

| Field      | Type                | Required | Description                                   |
|------------|---------------------|----------|-----------------------------------------------|
| `markdown` | string              | yes      | Markdown content to convert                   |
| `format`   | `"pdf"` \| `"docx"` | yes      | Output format                                 |
| `filename` | string              | no       | Suggested filename (default: `output.pdf/docx`) |
| `inline`   | boolean             | no       | Return inline instead of as attachment        |

**Response headers:**
- `X-Cache: HIT | MISS`
- `X-Cache-Backend: memory | redis`

**Example:**

```bash
curl -X POST http://localhost:8080/export \
  -H "Content-Type: application/json" \
  -d '{"markdown": "# Hello\n\nWorld", "format": "pdf"}' \
  --output output.pdf
```

### `GET /health`

Returns service status and binary versions.

```json
{
  "status": "ok",
  "pandoc_version": "3.9.0.2",
  "typst_version": "0.14.2",
  "cache_backend": "memory"
}
```

### `GET /swagger`

Interactive Swagger UI.

### `GET /openapi.json`

Raw OpenAPI 3.x specification.

## Running

### Prerequisites

- [Pandoc](https://pandoc.org/installing.html) in `PATH`
- [Typst](https://github.com/typst/typst#installation) in `PATH`
- Rust toolchain (for building from source)

### From source

```bash
cargo run --release
```

With Redis cache support:

```bash
cargo run --release --features redis-cache
```

### Docker

```bash
docker build -t md-exporter .
docker run -p 8080:8080 md-exporter
```

## Configuration

All settings are via environment variables:

| Variable                  | Default                      | Description                         |
|---------------------------|------------------------------|-------------------------------------|
| `PORT`                    | `8080`                       | HTTP listen port                    |
| `PANDOC_BIN`              | `pandoc` (from PATH)         | Path to Pandoc binary               |
| `TYPST_BIN`               | `typst` (from PATH)          | Path to Typst binary                |
| `TYPST_TEMPLATE`          | `templates/default.typ`      | Typst template for PDF output       |
| `REFERENCE_DOCX`          | `templates/reference.docx`   | Reference document for DOCX styling |
| `LUA_FILTER`              | `filters/table-auto-width.lua` | Pandoc Lua filter                 |
| `CACHE_TTL_SECS`          | `3600`                       | Cache entry TTL in seconds          |
| `CACHE_MAX_ENTRIES`       | `500`                        | Maximum in-memory cache entries     |
| `CONVERSION_TIMEOUT_SECS` | `30`                         | Pandoc/Typst process timeout        |
| `REDIS_URL`               | *(unset)*                    | Redis URL (requires `redis-cache` feature) |
| `LOG_LEVEL`               | `info`                       | Log level (`trace`, `debug`, `info`, `warn`, `error`) |

## Templates

### PDF (Typst)

The default template is at `templates/default.typ`. It uses [Minijinja](https://github.com/mitsuhiko/minijinja) syntax for variable substitution before being passed to Typst. Swap it out via `TYPST_TEMPLATE`.

### DOCX (reference document)

Pandoc uses `templates/reference.docx` as the style reference. Replace it with any `.docx` file that has the styles you want, then point `REFERENCE_DOCX` at it.

## License

Apache-2.0
