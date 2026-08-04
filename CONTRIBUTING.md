# Contributing

Thanks for helping improve **Small Rust PDF Opener**.

## Dev setup

1. Install Rust (1.85+) and clang / a C toolchain.
2. `cargo run --release`
3. Optional: `./packaging/build-dmg.sh` on macOS.

## Guidelines

- Keep the app **simple and local-first** — avoid cloud services and heavy suites.
- Prefer small, focused PRs.
- Match existing Rust style; run `cargo check` / `cargo clippy` when practical.
- Document user-facing changes in the PR description.
- Preserve AGPL-3.0 and update [NOTICE](NOTICE) if you add significant dependencies.

## Good first issues

- Windows / Linux packaging
- More OCR languages / model backends
- Thumbnail strip polish
- Better certificate appearance streams
- Automated smoke tests for open/save/compress

## Code of conduct

Be respectful. Harassment and discrimination are not welcome.
