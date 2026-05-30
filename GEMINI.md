# ECHO: Sovereign Bilingual Transcription Engine

## 1. Project Identity

**Echo** is a high-performance, local-first dictation utility designed for elite bilingual (Russian/English) users. It merges the ultra-low-latency Rust engine of **Handy** with the linguistic intelligence of **WhisperDesk** and the UX polish of **Wispr Flow**.

## 2. Core Pillars (Elite Edition)

### **A. Linguistic Intelligence (`heuristics.rs`)**

Unlike standard ASR tools, Echo understands the nuances of RU/EN sentence structure.

- **Auto-Punctuation**: Intelligent comma insertion before conjunctions (_но, а, который, because, although_).
- **Question Logic**: Automatic `?` detection based on sentence-starting tokens (_кто, почему, how, when_).
- **CamelCase Trigger**: Explicitly convert text to `camelCase` by starting dictation with "camel case" when in a code editor.
- **Optimized Lookups**: Uses Regex-pre-filtered $O(1)$ HashSet lookups for zero-latency subtitle streaming.

### **B. NAUTILUS Integration (Semantic Smoothing)**

Echo is designed to be the "voice" of your personal knowledge mesh.

- **LiteLLM Gateway**: Connects to `localhost:4000` for offline post-processing.
- **Speech-to-Clean-Thought**: Removes disfluencies (_um, uh, э-э_) while strictly preserving the original language.

### **C. OS Synergy**

- **Context-Awareness**: Detects active windows (_VS Code, IntelliJ, Cursor, Neovim_) to adjust formatting rules (e.g., stripping periods in code editors).
- **Stealth UX**: A non-focus-stealing floating pill (`WS_EX_NOACTIVATE`) with deep backdrop blur and high-contrast subtitles.
- **Smart Toggle**: Unified hotkey logic (Tap for Lock, Hold for PTT).

## 3. Engineering Standards

- **Concurrency**: Implements a `Wait-and-Notify` pattern (`Condvar`) to manage the singleton inference engine across real-time subtitles and final dictation tasks.
- **Safe Win32**: Modern `windows-rs` integration for window probing with explicit memory safety checks.
- **Aura**: Uses the **MinimalistUI** sound collection for high-fidelity tactile feedback.

## 4. Maintenance & Operations

- **Binary**: `target/release/echo.exe`
- **Logs**: `%LOCALAPPDATA%/com.sovern.echo/logs/handy.log`
- **Models**: `%APPDATA%/echo/models` (Parakeet V3 & Silero VAD)

---

_Created by Gemini CLI for the SOVRN v3.3 Architecture._
