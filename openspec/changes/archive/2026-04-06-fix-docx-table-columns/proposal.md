## Why

Docx table output has extremely narrow columns (characters stacking vertically) because the `table-auto-width.lua` filter that resets explicit column widths to auto is only applied in the PDF conversion path, not the docx path.

## What Changes

- Apply `--lua-filter` with `table-auto-width.lua` to the docx conversion in `convert_docx()` in `src/converter.rs`

## Non-Goals

- Modifying `reference.docx` table styles (test first; only needed if the filter alone is insufficient)
- Changing PDF behavior (already working correctly)

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `markdown-export`: Docx output now uses the lua filter for auto-width column sizing, matching PDF behavior

## Impact

- Affected specs: `markdown-export`
- Affected code: `src/converter.rs` (`convert_docx` function)
