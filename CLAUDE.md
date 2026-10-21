# CLAUDE.md

Bridge (repo: `day-scheduler`) is a Tauri 2 desktop app that tracks work in flight and runs recurring
Claude jobs. Read [docs/SPEC.md](docs/SPEC.md) before making design decisions.

## Principles (non-negotiable)

- **Strict lints.** `cargo clippy --all-targets -- -D warnings` must pass. Lints are set in
  `src-tauri/Cargo.toml` `[lints]` and `src-tauri/clippy.toml`. Never add `#[allow(...)]` to silence a
  lint without a one-line comment saying why; prefer fixing the code.
- **No `unwrap`, `expect` or `panic!`** in non-test code. Return typed errors (`thiserror`).
- **Single responsibility, aggressively.** One concern per file, one domain per module or crate.
  Keep functions short (clippy enforces a line limit). If you want a section comment, split the file.
- **Behaviour lives in policy prompts, not code.** Decisions about *what* to do (intake, autonomy, labels,
  summaries) go in markdown files under `defaults/policies/` and the user's config dir. Code implements
  *how*. When asked to change behaviour, edit the policy first; only touch code if the policy can't express it.
- **Deterministic first.** Use app code for polling, timers and diffs. Spawn Claude only for judgement.

## Commands

```bash
npm run tauri dev                                         # run the app
cd src-tauri && cargo clippy --all-targets -- -D warnings # lint Rust
cd src-tauri && cargo fmt --check                         # format check
npx tsc --noEmit                                          # type-check frontend
```

## Layout

- `src/`: frontend (Vite + TypeScript, strict).
- `src-tauri/`: Rust core and Tauri shell.
- `docs/`: spec and design notes.
- `defaults/`: shipped policy and routine files (created in phase 2).
