---
name: echo-audit
description: Findings and strategic plan for the Echo (formerly WhisperHandy) bilingual transcription engine audit, covering architecture, engineering standards, and product design.
---

# Echo Audit & Strategic Plan

## Overview
This skill contains the definitive findings and forward-looking strategic plan for **Echo**, a high-performance, local-first dictation utility for elite Russian/English users.

## 1. Engineering Audit Findings

### Architecture & Core Logic
- **Concurrency Model**: Robust `Wait-and-Notify` pattern via `TranscriptionCoordinator` to prevent race conditions.
- **Linguistic Intelligence**: $O(1)$ HashSet lookups for auto-punctuation and bilingual sentence structure.
- **OS Synergy**: Stealth UX using `WS_EX_NOACTIVATE` and `HWND_TOPMOST` for non-focus-stealing overlays.

### Performance & Safety
- **Latency**: Audio level smoothing (0.7/0.3 EMA) ensures fluid visual feedback.
- **Stability**: Background threads use `catch_unwind` to maintain app uptime during pipeline failures.

## 2. Product Design Audit Findings

### Visuals & UX
- **Floating Pill**: Minimalist design that follows the cursor, reducing cognitive load.
- **State Clarity**: Distinct visual cues for Recording, Transcribing, and Processing.
- **Smart Toggle**: Tap-to-Lock / Hold-for-PTT logic supports diverse workflow needs.

## 3. Strategic Plan (Superpowers)

### Phase 1: Infrastructure Cleanup (Immediate)
- **Unified Lockfiles**: Delete `package-lock.json` and `pnpm-lock.yaml`; standardize on `bun.lock` for faster installs.
- **Tauri Upstream Audit**: Evaluate the `cjpais/tauri` fork and identify paths to return to the main branch or upstream critical patches.

### Phase 2: UX Enhancements (Next 2 Weeks)
- **Subtitle Contrast**: Implement dynamic backdrop blur or a subtle shadow for real-time subtitles to improve legibility on varied backgrounds.
- **Context-Aware Refinement**: Expand the `heuristics.rs` rules to include more IDE-specific formatting (e.g., camelCase conversion for code editors).

### Phase 3: Knowledge Mesh Integration (Long Term)
- **NAUTILUS Synergy**: Deepen the "Speech-to-Clean-Thought" pipeline via LiteLLM for better semantic smoothing and disfluency removal.

## 4. Maintenance Standards
- **Binary**: `target/release/echo.exe`
- **Logs**: `%LOCALAPPDATA%/com.sovern.echo/logs/handy.log`
- **Models**: Parakeet V3 & Silero VAD.
