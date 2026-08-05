# PDF Export to Markdown / Docx — Design

**Date:** 2026-08-05  
**Status:** Approved

## Goal

Export the open PDF’s embedded text to `.md` or `.docx` so AI agents can grep/read it and people can open it in Word/Pages. No auto-OCR; scanned pages need **OCR…** first.

## Decisions

- Formats v1: `.md` + `.docx` only (no `.txt`, no CLI)
- Text source: native MuPDF `extract_text` only; user runs **OCR…** first for scans
- UI: in-app toolbar **Export…** → format + page range → save
- Empty pages still emit page headings so agents keep page boundaries
- Sync on UI thread for v1 (same as Extract)

## Out of scope

CLI, auto-OCR-on-export, layout-faithful tables/columns, `.txt`, replacing OCR toolbar

## User flow

1. Open a PDF (optionally run **OCR…** first for scans).
2. Toolbar **Export…** → dialog: format (Markdown / Word), page range (`1-3,5`), **Export…**.
3. Save dialog with the right extension; status line shows the path.
4. Empty/sparse pages export with page stubs — no silent OCR.

## Architecture

Shared text pipeline + dual writers:

1. Parse page range via existing `page_range`.
2. `collect_page_texts` → `DocumentSession::extract_text` per page.
3. `write_markdown` or `write_docx` from the same `Vec<PageText>`.

### Markdown shape

```markdown
# {source-filename}

## Page 1

{page text or empty}

## Page 2

...
```

### Docx shape

Title = source filename; each page = “Page N” heading + paragraph(s); page break between pages.

## Errors

Same patterns as Extract: busy guard, no session, range parse errors, MuPDF fail aborts whole export, IO on write, cancel save = no-op.

## Testing

- Unit tests for markdown string shape and docx non-empty zip contents
- Manual: digital PDF → md/docx; OCR’d scan → export; empty range / cancel / bad range
