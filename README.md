# Small Rust PDF Opener

<p align="center">
  <img src="docs/images/promo.gif" alt="Promo: local-first PDF viewer — no uploads, scroll, edit, sign, OCR" width="100%">
</p>

> Fast, local-first PDF viewer and light editor in **Rust** — open, scroll, edit pages, compress, sign, and OCR **without uploading your documents**.

[![License: AGPL v3](https://img.shields.io/badge/License-AGPL%20v3-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org/)
[![Release](https://img.shields.io/github/v/release/will702/small-rust-pdf-opener)](https://github.com/will702/small-rust-pdf-opener/releases)
[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-lightgrey.svg)](#build--run)

**Keywords:** `rust` `pdf` `pdf-viewer` `pdf-editor` `egui` `mupdf` `ocr` `local-ocr` `pdf-sign` `pkcs12` `lightweight` `desktop-app` `privacy` `offline`

---

## Why this exists

Most PDF tools are either heavy suites or cloud uploaders. **Small Rust PDF Opener** is a single native binary that stays on your machine: view quickly, do a few useful edits, merge/split, compress, stamp a signature, optionally certify with a PKCS#12 key, and run OCR after you choose to download local models.

## Architecture at a glance

<p align="center">
  <img src="docs/images/architecture-diagram.png" alt="Architecture: egui UI, Document Session, MuPDF, OCR, Sign" width="90%">
</p>

### Runtime data flow

```mermaid
flowchart TB
  User[User] --> UI[egui Desktop UI]
  UI --> Session[DocumentSession]
  Session --> MuPDF[MuPDF Engine]
  MuPDF --> Render[Page RGBA textures]
  Render --> Scroll[Continuous scroll viewer]
  Session --> Edit[Delete Rotate Reorder Crop]
  Session --> MergeSplit[Merge Append Split Extract]
  Session --> Export[Export md docx]
  Session --> Compress[Compress presets]
  UI --> OCR[OCR module]
  OCR -->|opt-in download| Models[Local ocrs models cache]
  OCR --> TextLayer[Invisible searchable text]
  OCR --> Overlay[Bounding box overlays]
  UI --> VisualSign[Visual signature stamp]
  UI --> CertSign[PKCS12 to PKCS7]
  CertSign --> Lopdf[lopdf AcroForm Sig]
  Edit --> MuPDF
  MergeSplit --> MuPDF
  Compress --> MuPDF
  TextLayer --> MuPDF
  VisualSign --> MuPDF
```

### Typical user journeys

```mermaid
flowchart LR
  subgraph view [View]
    A1[Open PDF] --> A2[Scroll / zoom / search]
  end
  subgraph edit [Edit]
    B1[Select page] --> B2[Delete or rotate or crop]
    B2 --> B3[Save As]
  end
  subgraph mergeSplit [Merge Split]
    M1[Merge or Append] --> M2[Split or Extract ranges]
  end
  subgraph sign [Sign]
    C1[Draw or import mark] --> C2[Place on page]
    C3[Load P12] --> C4[Drag signature rect]
    C4 --> C5[Write signed PDF]
  end
  subgraph ocrFlow [OCR]
    D1[Download models once] --> D2[OCR page range or all]
    D2 --> D3[Boxes plus text overlay]
    D3 --> D4[Searchable text layer]
  end
  view --> edit
  edit --> mergeSplit
  edit --> sign
  edit --> ocrFlow
```

### Build / package pipeline

```mermaid
flowchart LR
  Src[Rust sources] --> Cargo["cargo build --release"]
  Cargo --> Bin[pdf-opener binary]
  Bin --> AppBundle["PDF Opener.app"]
  Icon[AppIcon.icns] --> AppBundle
  AppBundle --> Dmg[DMG installer]
  Dmg --> AppsFolder["Applications folder"]
```

## Features

| Area | What you get |
|------|----------------|
| **View** | Fast MuPDF rendering, continuous scroll, zoom / fit-width, text search, keyboard nav |
| **Edit** | Delete page, rotate 90°, reorder (move up/down), crop (CropBox), Save / Save As |
| **Merge / Split** | Merge several PDFs, append into the open doc, split ranges to files, extract pages |
| **Export** | Export embedded text to Markdown (`.md`) or Word (`.docx`); run OCR… first for scans |
| **Compress** | Fast / Balanced / Small write presets (deflate, image compress, garbage collect) |
| **Visual sign** | Draw a signature or import PNG/JPEG and stamp onto a page |
| **Cert sign** | Import `.p12` / `.pfx`, place a field, embed PKCS#7 detached signature (OpenSSL) |
| **OCR** | Opt-in local [ocrs](https://github.com/robertknight/ocrs) models; page / range / whole-doc; bounding boxes + text overlays; invisible searchable layer |
| **Privacy** | Documents never leave your computer; OCR models download only when you ask |

Out of scope (on purpose): full annotation suites, forms designer, PDF/A conversion, cloud sync.

## Install (macOS)

Apple Silicon, via [Homebrew](https://brew.sh):

```bash
brew tap will702/tap
brew install --cask pdf-opener
```

Or grab the DMG from [Releases](https://github.com/will702/small-rust-pdf-opener/releases).

## Try it from source

See [Build & run](#build--run) below. Quickest path with Rust installed:

```bash
cargo run --release -- ./testdata/hello.pdf
```

## Requirements

| Dependency | Why |
|------------|-----|
| **Rust** 1.85+ (edition 2021) | Build toolchain |
| **clang** / C toolchain | Builds [`mupdf-sys`](https://crates.io/crates/mupdf-sys) (MuPDF C library) |
| **macOS** | Xcode Command Line Tools (`xcode-select --install`) |
| **Linux** | Typical `build-essential` + clang; fontconfig often needed for system fonts |
| **Windows** | MSVC Build Tools + clang for MuPDF |

## Build & run

```bash
git clone https://github.com/will702/small-rust-pdf-opener.git
cd small-rust-pdf-opener
cargo run --release
# or:
cargo run --release -- ./testdata/hello.pdf
```

### Compile notes

- First build compiles MuPDF via `mupdf-sys` — expect several minutes.
- Release profile uses LTO + stripped binary ([`Cargo.toml`](Cargo.toml)).
- OCR models are **not** in the binary; use **OCR… → Download models** in the app (~tens of MB to your cache dir).

### macOS `.app` + DMG

```bash
./packaging/build-dmg.sh
```

Produces `dist/PDF Opener.app` and `dist/PDF-Opener-<version>.dmg`. Drag into Applications. If Gatekeeper blocks: right-click → **Open** (adhoc-signed local build).

**GitHub Actions:** every `v*` tag / published Release (and manual **Release DMG** workflow dispatch) builds the DMG on `macos-latest` and uploads it to the release assets.

## Usage cheatsheet

| Action | How |
|--------|-----|
| Open / Save | Toolbar or ⌘/Ctrl+O, ⌘/Ctrl+S, ⌘⇧S |
| Scroll pages | Mouse wheel / trackpad (continuous strip) |
| Jump page | ◀ ▶ or ← → / PgUp / PgDn |
| Crop | Mode **Crop**, drag a rectangle on a page |
| Visual sign | Mode **Sign**, draw/import, **Place on page**, click |
| Cert sign | Mode **Cert sign**, load PKCS#12, **Place & sign**, drag rect |
| Compress | **Compress…** → preset → save new file |
| Merge | **Merge…** → add 2+ PDFs → **Save merged…** |
| Append | **Append PDF…** → pick files to add to the open document |
| Split | **Split…** → ranges like `1-2,4` → each range becomes a file |
| Extract | **Extract…** → page range → one new PDF (optional reopen) |
| Export | **Export…** → Markdown or Word → page range → `.md` / `.docx` (optional `--features anydoc-export` for structured Markdown via [anydoc](https://github.com/firecrawl/anydoc)) |
| OCR | **OCR…** → download models once → this page / all / range; boxes + text overlay; Cancel supported |

## Project layout

```
src/
  main.rs          eframe entry + Dock icon
  app.rs           egui UI, continuous scroll viewer
  page_range.rs    parse `1-3,5` page ranges
  pdf/             MuPDF document session (render/edit/merge/split/save)
  export/          Markdown / Word text export from embedded PDF text
  ocr/             model download + ocrs recognition
  sign/            visual pad + PKCS#12 / PKCS#7 signing
packaging/         Info.plist + build-dmg.sh
assets/            app icons
docs/images/       README banner + architecture art
testdata/          sample PDF
```

## Stack & attributions

This project stands on open-source libraries. **Cite and respect their licenses.**

| Component | Project | Role | License (upstream) |
|-----------|---------|------|--------------------|
| PDF engine | [Artifex MuPDF](https://mupdf.com/) via [`mupdf`](https://crates.io/crates/mupdf) / [`mupdf-rs`](https://github.com/messense/mupdf-rs) | Render, page ops, compress write options | **AGPL-3.0** |
| UI | [`egui`](https://github.com/emilk/egui) / [`eframe`](https://github.com/emilk/egui) | Immediate-mode desktop UI | MIT / Apache-2.0 |
| OCR | [`ocrs`](https://github.com/robertknight/ocrs) + [`rten`](https://github.com/robertknight/rten) | Local neural OCR; models from [ocrs-models](https://github.com/robertknight/ocrs-models) | MIT / Apache-2.0 |
| PDF COS surgery | [`lopdf`](https://crates.io/crates/lopdf) | Signature dictionary / AcroForm wiring | MIT |
| Crypto | [`openssl`](https://crates.io/crates/openssl) (vendored) | PKCS#12 parse, PKCS#7 detached sign | Apache-2.0 |
| Dialogs / images | [`rfd`](https://crates.io/crates/rfd), [`image`](https://crates.io/crates/image) | File pickers, PNG/JPEG | MIT / Apache-2.0 |
| Markdown export (optional) | [`anydoc`](https://github.com/firecrawl/anydoc) | Structured GFM from PDF (`anydoc-export` feature) | MIT |

Full third-party notice: [NOTICE](NOTICE). Formal citation: [CITATION.cff](CITATION.cff).

Because MuPDF is AGPL-3.0, **this repository is licensed under AGPL-3.0** — see [LICENSE](LICENSE). If you distribute a modified version (including as a network service that users interact with), you must provide corresponding source under AGPL-3.0.

## Security & privacy

- PDFs are processed locally.
- Certificate signing uses keys you import; passwords are not stored.
- OCR model download uses HTTPS to the ocrs model host; disable by never clicking Download.
- Report vulnerabilities privately — see [SECURITY.md](SECURITY.md).

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md). Ideas welcome: more OCR languages, better cert appearance streams, Linux/Windows packaging.

## Discoverability (for humans & AI)

- **One-liner:** Local-first Rust PDF viewer/editor with scroll, crop, merge/split, compress, visual + PKCS#12 signing, and opt-in offline OCR with visible results.
- **Topics:** `rust`, `pdf`, `pdf-viewer`, `pdf-editor`, `egui`, `mupdf`, `ocr`, `offline`, `privacy`, `desktop-app`, `agpl`
- Machine-readable summary: [llms.txt](llms.txt)

## License

[GNU Affero General Public License v3.0](LICENSE)
