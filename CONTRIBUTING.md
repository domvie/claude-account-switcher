# Contributing

Thanks for your interest! This is a small project — no heavy process.

## Development setup

Prerequisites: [Rust](https://rustup.rs), Node 22+, and the
[Tauri v2 system dependencies](https://v2.tauri.app/start/prerequisites/) for
your platform.

```sh
npm install
npm run tauri dev
```

## Before opening a PR

Run the same checks CI runs:

```sh
npx tsc --noEmit
cd src-tauri
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test
```

## Notes for contributors

- **Never touch credentials.** The app reads login tokens read-only for usage
  display and must never write, refresh, copy, or log them. Profile isolation
  works precisely because each tool manages its own credentials.
- **Slugs and config-dir paths are immutable** once a profile exists — the
  macOS Keychain entry name is derived from the exact path string.
- Platform-specific code lives in `src-tauri/src/platform/`; keep the shared
  modules platform-free.
- Windows and Linux support are stubs so far (see the milestones in the README
  status note) — contributions welcome.
