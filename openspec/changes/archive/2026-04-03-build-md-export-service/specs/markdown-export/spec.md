## ADDED Requirements

### Requirement: Accept Markdown export request

The service SHALL expose a `POST /export` endpoint that accepts a JSON body with fields `markdown` (string, required), `format` ("pdf" or "docx", required), `filename` (string, optional), and `inline` (boolean, optional, default false).

#### Scenario: Valid PDF request

- **WHEN** a client sends `POST /export` with `{"markdown": "# Hello", "format": "pdf"}`
- **THEN** the service returns `200 OK` with `Content-Type: application/pdf` and a binary PDF body

#### Scenario: Valid DOCX request

- **WHEN** a client sends `POST /export` with `{"markdown": "# Hello", "format": "docx"}`
- **THEN** the service returns `200 OK` with `Content-Type: application/vnd.openxmlformats-officedocument.wordprocessingml.document` and a binary DOCX body

#### Scenario: Missing markdown field

- **WHEN** a client sends `POST /export` with `{"format": "pdf"}` (no markdown field)
- **THEN** the service returns `400` with `{"error": "invalid_request", "message": "..."}`

#### Scenario: Empty markdown string

- **WHEN** a client sends `POST /export` with `{"markdown": "", "format": "pdf"}`
- **THEN** the service returns `400` with `{"error": "invalid_request", "message": "..."}`

#### Scenario: Invalid format value

- **WHEN** a client sends `POST /export` with `{"markdown": "# Hi", "format": "html"}`
- **THEN** the service returns `400` with `{"error": "invalid_request", "message": "..."}`

### Requirement: Return file with correct Content-Disposition

The service SHALL set `Content-Disposition: attachment; filename="<filename>"` by default, using the `filename` field from the request if provided, or `"export.pdf"` / `"export.docx"` as the default. When `inline: true` is set, the service SHALL set `Content-Disposition: inline`.

#### Scenario: Default filename for PDF

- **WHEN** a request omits `filename` and `format` is `"pdf"`
- **THEN** the response includes `Content-Disposition: attachment; filename="export.pdf"`

#### Scenario: Custom filename

- **WHEN** a request sets `filename: "report.pdf"`
- **THEN** the response includes `Content-Disposition: attachment; filename="report.pdf"`

#### Scenario: Inline disposition

- **WHEN** a request sets `inline: true`
- **THEN** the response includes `Content-Disposition: inline`

### Requirement: Convert Markdown to PDF via Pandoc + Typst

For PDF requests, the service SHALL render the minijinja Typst template to a temp file, write the Markdown input to a temp file, and invoke Pandoc as:
`pandoc --from=markdown --to=pdf --pdf-engine=typst --template=<rendered_typst_tmp> -o <output_tmp> <input_tmp>`

Both temp files SHALL be held alive across the Pandoc await and automatically deleted after.

#### Scenario: Successful PDF conversion

- **WHEN** a valid PDF request is received and Pandoc exits with code 0
- **THEN** the service returns the output file bytes as the response body

#### Scenario: Pandoc failure

- **WHEN** Pandoc exits with a non-zero code
- **THEN** the service returns `422` with `{"error": "conversion_failed", "message": "<stderr output>"}`

#### Scenario: Pandoc timeout

- **WHEN** Pandoc does not complete within `CONVERSION_TIMEOUT_SECS` (default 30)
- **THEN** the process is killed and the service returns `422` with `{"error": "conversion_failed", "message": "timeout"}`

### Requirement: Convert Markdown to DOCX via Pandoc

For DOCX requests, the service SHALL write the Markdown input to a temp file and invoke Pandoc as:
`pandoc --from=markdown --to=docx --reference-doc=<REFERENCE_DOCX> -o <output_tmp> <input_tmp>`

#### Scenario: Successful DOCX conversion

- **WHEN** a valid DOCX request is received and Pandoc exits with code 0
- **THEN** the service returns the output file bytes as the response body

#### Scenario: DOCX Pandoc failure

- **WHEN** Pandoc exits with a non-zero code during DOCX conversion
- **THEN** the service returns `422` with `{"error": "conversion_failed", "message": "<stderr output>"}`
