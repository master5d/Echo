---
name: echo-audit
description: Factual engineering notes and known state for Echo, a local-first bilingual (RU/EN) dictation app forked from cjpais/Handy. Use when working on Echo's architecture, build, or rebrand cleanup.
---

# Echo — Engineering Notes

Echo is a **fork of `cjpais/Handy`**, renamed and tuned for bilingual (RU/EN)
dictation. Stack: Tauri 2.x, Rust core (`src-tauri`), React/Vite frontend.
This file records what is actually true about the code, not aspirations.

## Architecture (as built)

- **Transcription**: `transcribe-rs` (Parakeet/Whisper) + Silero VAD, run locally.
  `TranscriptionCoordinator` (`src-tauri/src/transcription_coordinator.rs`)
  serializes access to the single inference engine so realtime subtitles and
  final dictation don't race.
- **Linguistic heuristics** (`src-tauri/src/heuristics.rs`): **best-effort**,
  wordlist-based auto-punctuation/capitalization and a `camel case` trigger.
  Matched per whitespace-split token — no grammar model. Gated behind the
  `auto_punctuate` / `auto_capitalize` settings (off by default). For anything
  beyond coarse assistance, prefer LLM post-processing.
- **Output** (`src-tauri/src/actions.rs`): probes the active window and, for
  code editors, strips trailing punctuation; `camel case <text>` converts to
  camelCase via `strip_prefix` (no byte slicing).
- **Overlay** (`src-tauri/src/overlay.rs`): non-focus-stealing floating pill.

## Build & infra

- **Package manager: bun** (canonical). The Nix build (`.nix/bun.nix`,
  `.nix/bun-lock-hash`) and `postinstall` depend on `bun.lock`. Do not
  reintroduce a pnpm lockfile.
- `transcribe-rs` is pinned to `0.3.8` across all platform targets in
  `src-tauri/Cargo.toml`.
- Updater endpoint points at `github.com/master5d/Echo/releases` — keep it on
  Echo's own repo, never upstream.

## Maintenance facts

- **Binary / lib**: bin `echo`, lib `echo_lib`.
- **Bundle identifier**: `com.sovern.echo`.
- **Logs**: `%LOCALAPPDATA%/com.sovern.echo/logs/echo.log` (Windows).
- **Models**: hosted on `blob.handy.computer` (upstream CDN — load-bearing,
  do not rename these URLs in `src-tauri/src/managers/model.rs`).

## Known rebrand residue (intentional / pending)

- Upstream attribution kept on purpose (MIT fork): "formerly Handy", the
  original `handy-cli`, `handy.computer`, sponsors, `blob.handy.computer`.
- Internal env flags still use the `HANDY_` prefix
  (`HANDY_FORCE_TRANSCRIPTION_FAILURE`, `HANDY_NO_GTK_LAYER_SHELL`) and the
  external `handy-keys` crate is unrelated to branding.
- Stale packaged copy `scripts/echo-audit.skill` (zip) predates this file.
