## MODIFIED Requirements

### Requirement: Convert Markdown to DOCX via Pandoc

For DOCX requests, the service SHALL write the Markdown input to a temp file and invoke Pandoc as:
`pandoc --from=markdown --to=docx --reference-doc=<REFERENCE_DOCX> --lua-filter=<LUA_FILTER> -o <output_tmp> <input_tmp>`

The lua filter SHALL be the same `table-auto-width.lua` filter used in the PDF conversion path, applied to reset explicit column widths to `ColWidthDefault` so Word uses automatic column sizing.

#### Scenario: Successful DOCX conversion

- **WHEN** a valid DOCX request is received and Pandoc exits with code 0
- **THEN** the service returns the output file bytes as the response body

#### Scenario: DOCX Pandoc failure

- **WHEN** Pandoc exits with a non-zero code during DOCX conversion
- **THEN** the service returns `422` with `{"error": "conversion_failed", "message": "<stderr output>"}`

#### Scenario: DOCX table with multiple columns

- **WHEN** a DOCX request contains a Markdown table with a narrow column (e.g., a boolean/checkmark column)
- **THEN** the output DOCX SHALL render all columns with automatic width sizing, with no column so narrow that characters stack vertically
