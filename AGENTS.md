# AGENTS.md

This file provides guidance to AI coding assistants working with code in this repository (**Project Echo**).

## Development Commands

**Prerequisites:**

- [Rust](https://rustup.rs/) (latest stable)
- [Node.js](https://nodejs.org/) 20+ (ships with `npm`)

**Package manager: any of npm / pnpm / bun works.** Scripts and the Tauri
`beforeDevCommand`/`beforeBuildCommand` use `npm` (always present with Node),
and `postinstall` is plain Node, so a fresh clone installs and runs without
extra tooling. CI and the Nix build use **bun** and pin `bun.lock`; npm/pnpm
developers resolve from `package.json` ranges. Substitute your PM below.

**Core Development:**

```bash
# Install dependencies
npm install

# Run in development mode
npm run tauri dev

# Build for production
npm run tauri build

# Frontend only development
npm run dev        # Start Vite dev server (127.0.0.1:1420)
npm run build      # Build frontend (TypeScript + Vite)
```

**Linting and Formatting (run before committing):**

```bash
npm run lint              # ESLint for frontend (includes i18n checks)
npm run format            # Prettier + cargo fmt
```

## Architecture Overview

**Echo** is a cross-platform desktop speech-to-text application built with Tauri 2.x. It is designed for ultra-low latency bilingual (RU/EN) transcription.

### Backend Structure (src-tauri/src/)

- `lib.rs` - Main entry point, Tauri setup, manager initialization.
- `managers/` - Core business logic:
  - `audio_recording.rs` - Audio stream management with `cpal`.
  - `transcription.rs` - Orchestrates ML inference (Whisper/Parakeet).
- `heuristics.rs` - Linguistic intelligence:
  - `to_camel_case()` - Developer-centric text transformation.
  - `auto_punctuate()` - Bilingual punctuation rules (comma before "но", "которой", etc.).
- `actions.rs` - Global shortcut handlers, the `process_transcription_output`
  pipeline, and context-aware formatting logic.
- `voice_commands.rs` - Wispr-inspired transforms: Command Mode detection
  (`detect_prefix_command`, `strip_submit_command`), snippet expansion,
  self-correction, spoken-list formatting, and the built-in developer dictionary.
- `platform/` - OS/UX peripherals grouped by module: `overlay.rs` (stealth UX,
  `WS_EX_NOACTIVATE`), `clipboard.rs` (paste + auto-submit), `input.rs`,
  `tray.rs`, `signal_handle.rs`, `audio_feedback.rs`.

### Key Architecture Patterns

**Wait-and-Notify Concurrency:** The `TranscriptionCoordinator` manages the singleton inference engine, allowing real-time subtitles and final dictation tasks to share resources without race conditions.

**Context-Aware Formatting:** The backend probes the active window (e.g., VS Code, Cursor) and automatically adjusts output (stripping trailing periods, applying camelCase).

**Stealth UX:** The floating pill overlay is designed to never steal window focus, providing feedback in the peripheral vision.

### Technology Stack

- `whisper-rs` - GPU-accelerated Whisper inference.
- `transcribe-rs` - Parakeet V3 engine.
- `ort-directml` - ONNX acceleration for Windows.
- `windows-rs` - Deep Win32 integration.

## Internationalization (i18n)

All user-facing strings must use `i18next`. Hardcoded strings in JSX are disallowed by ESLint.
New elite-edition strings (e.g., "Sovern Elite Edition") live in `src/i18n/locales/en/translation.json` under `settings.about`.
