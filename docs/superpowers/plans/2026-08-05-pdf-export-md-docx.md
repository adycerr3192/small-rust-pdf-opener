# PDF Export to Markdown / Docx Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Export the open PDF’s embedded text to `.md` or `.docx` for agents (grep-friendly) and humans (Word/Pages), without auto-OCR.

**Architecture:** Collect per-page strings via existing `DocumentSession::extract_text`, then format with `write_markdown` / `write_docx` in a new `src/export/` module. UI mirrors Extract… (toolbar + dialog + `rfd` save).

**Tech Stack:** Rust, egui, MuPDF (`mupdf`), `docx-rs`, existing `page_range`.

## Global Constraints

- Formats: `.md` + `.docx` only
- No auto-OCR on export
- No CLI in v1
- Empty pages still emit page headings
- Sync on UI thread

---

## Task 0 — Spec artifacts

- [x] Design at `docs/superpowers/specs/2026-08-05-pdf-export-md-docx-design.md`
- [x] This plan under `docs/superpowers/plans/`

## Task 1 — Export core (TDD)

- [x] Add `src/export/mod.rs` with `PageText`, `format_markdown`, `collect_page_texts`, `write_markdown`
- [x] Unit test page headings + empty pages
- [x] Wire `mod export` in `main.rs`; drop `dead_code` on `extract_text`

## Task 2 — Docx writer

- [x] Add `docx-rs` dependency
- [x] `write_docx(title, pages, path) -> Result<()>`
- [x] Test: write temp file, assert non-empty / zip has `word/document.xml`

## Task 3 — UI Export dialog

- [x] `show_export`, `export_range_text`, `ExportFormat { Markdown, Docx }`
- [x] Toolbar **Export…** near Extract
- [x] `run_export` mirroring `run_extract`

## Task 4 — Polish

- [x] README one-liner
- [x] `cargo test` + `cargo build`
