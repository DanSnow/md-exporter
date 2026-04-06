## 1. Implementation

- [x] 1.1 In `src/converter.rs`, add `format!("--lua-filter={}", config.lua_filter)` to the args list in `convert_docx()`, matching the pattern used in `convert_pdf()` — Convert Markdown to DOCX via Pandoc

## 2. Verification

- [x] 2.1 Send a DOCX request with a Markdown table containing a short boolean/checkmark column and verify all columns render with auto width (no vertically stacked characters) — DOCX table with multiple columns
- [x] 2.2 Confirm existing DOCX conversion succeeds and PDF output is unchanged — Successful DOCX conversion
