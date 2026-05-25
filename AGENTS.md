# AGENTS.md

This file provides guidance to AI coding assistants working with code in this repository (**Project Echo**).

## Development Commands

**Prerequisites:**

- [Rust](https://rustup.rs/) (latest stable)
- [Node.js / pnpm](https://pnpm.io/) (Current standard package manager)

**Core Development:**

```bash
# Install dependencies
pnpm install

# Run in development mode
pnpm tauri dev

# Build for production
pnpm tauri build

# Frontend only development
pnpm run dev        # Start Vite dev server
pnpm run build      # Build frontend (TypeScript + Vite)
```

**Linting and Formatting (run before committing):**

```bash
pnpm run lint              # ESLint for frontend (includes i18n checks)
pnpm run format            # Prettier + cargo fmt
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
- `actions.rs` - Global shortcut handlers and context-aware formatting logic.
- `overlay.rs` - Stealth UX management (`WS_EX_NOACTIVATE`).

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
