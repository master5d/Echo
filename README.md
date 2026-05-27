<div align="center">

# Echo

**Local‑first, Russian‑primary bilingual dictation.**

Speak in Russian, English, or both at once — Echo transcribes it on‑device, in
real time, and types it straight into whatever app you're using. No cloud, no
account, no telemetry.

[![License: MIT](https://img.shields.io/badge/License-MIT-22c55e.svg?style=flat-square)](LICENSE)
![Platform](https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-475569?style=flat-square)
![Tauri](https://img.shields.io/badge/built%20with-Tauri%20%2B%20Rust-e76f51?style=flat-square)

</div>

---

## What it is

Echo is a tray‑first speech‑to‑text utility built on Tauri (Rust core, React
UI) and whisper.cpp / NVIDIA Parakeet. It's a fork of
[cjpais/Handy](https://github.com/cjpais/Handy), tuned for **Russian as the
primary language** with first‑class support for the whole Slavic family and for
mixing English into Russian speech.

Press a hotkey, talk, release — the text appears in your editor, terminal, chat,
or browser. Everything runs offline.

## Key capabilities

### Smart dictation

- 🗣️ **Bilingual code‑switching.** Say *"давай закоммитим этот pull request в
  main"* and English terms stay in Latin script while Russian stays Cyrillic —
  no transliteration to «бамп».
- 🌍 **Slavic family + English.** Auto‑detects Russian, Ukrainian, Polish,
  Czech, Slovak, Slovenian, Bulgarian, Croatian and 25 European languages.
- ✏️ **Auto‑punctuation & capitalization.** Bilingual heuristics add commas,
  sentence case, and question marks from intonation cues — toggleable.
- 🔁 **Self‑correction.** Restate yourself mid‑sentence (*"встречаемся в 2, нет,
  в 3 часа"*) and Echo keeps only the corrected version.
- 🔢 **Spoken lists.** Dictated enumerations (*"first… second… third…"* /
  *"первое… второе…"*) become a clean numbered list.
- 🧠 **Clean output.** Voice‑activity detection trims silence, and known Whisper
  "silence hallucinations" (subtitle credits, "thanks for watching", …) are
  filtered out so they never land in your text.

### Command Mode

Begin an utterance with a spoken instruction and Echo transforms the text in a
single step — no app switching:

- *"translate to English …"* / *"переведи на английский …"* → inserts the
  translation.
- *"make shorter …"* / *"сделай короче …"* → condenses the phrasing.
- *"make formal …"* / *"ответь формально …"* → rewrites in a professional tone.
- End with *"press enter"* / *"нажми ввод"* to submit after pasting.

Transform commands run through your configured post‑processing provider (a local
LLM via Ollama/llama.cpp, or any OpenAI‑compatible endpoint). Detection itself is
local; opt in under **Settings → Advanced → Voice commands**.

### Personalization

- 📓 **Custom dictionary.** Teach Echo unusual names, terms, and acronyms.
- ✂️ **Snippets.** Spoken trigger phrases expand into canned text — signatures,
  calendar links, boilerplate replies.
- 💬 **Live subtitles.** An optional caption under the recording orb streams the
  transcript as you speak, with adjustable size, length, and refresh rate.
- 🌐 **Interface language.** The UI itself ships in 20 languages.

### Developer features

- ⌨️ **Context‑aware formatting.** Echo detects code editors (VS Code, Cursor,
  Zed, JetBrains, …) and strips trailing punctuation automatically.
- 🐫 **camelCase trigger.** Say *"camel case my variable name"* to get
  `myVariableName`.
- 🧰 **Developer dictionary.** An opt‑in built‑in glossary steers transcription
  toward the correct spelling of common tooling (GitHub, Vercel, TypeScript,
  Kubernetes, …).

### Privacy & performance

- 🔒 **Truly local.** Your audio never leaves the machine. No cloud calls, no
  telemetry, no account, no word limits — free and open‑source (MIT).
- ⚡ **GPU accelerated.** Whisper runs on the GPU (Vulkan — NVIDIA / AMD / Intel,
  auto‑detected) or Parakeet on CPU/DirectML. Pick the device in Settings.
- 🫥 **Stealth UX.** A floating orb shows recording / preparing / transcribing
  state without ever stealing focus from the app you're typing into.
- 🪶 **Quiet at rest.** The model unloads after an idle timeout and the overlay
  only receives live data while visible, so Echo stays out of the way when not
  in use.

## Choosing a model

Echo ships several engines. For the bilingual Russian use case:

| Model | Best for | Speed | Notes |
| --- | --- | --- | --- |
| **Parakeet V3** *(recommended)* | RU + Slavic + EN, auto‑detect | ⚡⚡ fast (GPU/CPU) | Great all‑rounder; weaker on heavy intra‑sentence code‑switching |
| **Whisper Large v3 Turbo** | Heavy RU↔EN code‑switching | ⚡ (GPU) | Keeps English in Latin; needs GPU to be snappy |
| **GigaAM v3** | Pure Russian, max accuracy | ⚡⚡ | Russian‑only — no code‑switching or other languages |
| Whisper Small / Medium / Large | General multilingual | varies | Classic Whisper quality/size trade‑offs |

Pick and download models in **Settings → Models**.

## How it works

1. **Press** your shortcut (tap to lock, or hold to push‑to‑talk).
2. **Speak.** Silero VAD filters silence; the floating pill shows live state.
3. **Release.** The selected engine transcribes locally and refines the text.
4. **Paste.** Text is typed into the focused app (native Unicode input — works
   with Cyrillic regardless of keyboard layout), with code‑editor‑aware
   formatting.

## Practical scenarios

- **Bilingual code review.** In Cursor, dictate a comment mixing Russian prose
  with English identifiers — `pull request`, `main`, `useEffect` stay in Latin,
  trailing punctuation is stripped for the editor.
- **Instant translation.** Reply to a foreign colleague: Command Mode
  *"переведи на английский …"*, speak in Russian, the English text lands in the
  chat box.
- **Quick meeting notes.** Dictate *"first decisions, second action items,
  third owners"* and get a formatted numbered list in Notion.
- **Templated support reply.** A snippet `calendar link` expands to your full
  booking URL as you speak the trigger phrase.
- **Fast send.** End any dictation with *"press enter"* to fire the message
  without touching the keyboard.

## Limitations

- **Transform commands need an LLM provider.** Translate / shorten / formal run
  through a post‑processing provider; configure a local model (Ollama /
  llama.cpp) or an OpenAI‑compatible endpoint in **Settings → Advanced**. Plain
  dictation, snippets, self‑correction, lists, and the developer dictionary are
  fully offline.
- **Self‑correction is heuristic.** It collapses comma‑delimited restatements
  (*"…, no, …"*) and is off by default; complex corrections may pass through.
- **Live subtitles cost compute.** The streaming caption re‑transcribes the
  buffer periodically, so it's best paired with a fast engine (Parakeet /
  Whisper on GPU); it's off by default.

## Quick start

### Install

1. Download the latest build from the
   [releases page](https://github.com/master5d/Echo/releases).
2. Install and launch Echo, then grant microphone + accessibility permissions.
3. Open **Settings → Models**, download a model (Parakeet V3 to start).
4. Set your hotkey in **Settings → General**, and start dictating.

> On Windows the app installs to `%LOCALAPPDATA%/echo`.

### Build from source

Any package manager works (npm shown; pnpm / bun are fine):

```bash
npm install
npm run tauri dev      # development
npm run tauri build    # production bundle
```

Full instructions, including the GPU‑Whisper (Vulkan) toolchain on Windows, are
in [BUILD.md](BUILD.md). Contributors: see [CONTRIBUTING.md](CONTRIBUTING.md).

## GPU acceleration

Whisper can run on the GPU via Vulkan, which auto‑detects any vendor's card
(NVIDIA / AMD / Intel) and falls back to CPU when none is present. Choose the
accelerator and device in **Settings → Advanced → Hardware Acceleration**
(enable *Experimental Features* if the section is hidden). Parakeet uses
DirectML on Windows.

## Architecture

- **Frontend** — React 18 + TypeScript + Tailwind CSS.
- **Backend** — Rust core; a single‑threaded coordinator serialises the
  record → transcribe → paste pipeline to avoid races.
- **Key crates** — `whisper-rs` (whisper.cpp, GPU), `transcribe-rs` (Parakeet /
  GigaAM / Canary / SenseVoice), `vad-rs` (Silero VAD), `windows-rs` (Win32
  overlay integration).

## CLI

Echo accepts flags, both to control a running instance and to customise startup
(all platforms):

```bash
# control a running instance
echo --toggle-transcription      # toggle recording
echo --toggle-post-process       # toggle recording with post-processing
echo --cancel                    # cancel current operation

# startup
echo --start-hidden              # start with no window
echo --no-tray                   # start without the tray icon
echo --debug                     # verbose logging
echo --help                      # list all flags
```

> **macOS:** when installed as a bundle, call the binary directly, e.g.
> `/Applications/Echo.app/Contents/MacOS/Echo --toggle-transcription`.

Debug mode toggles with `Ctrl+Shift+D` (Windows/Linux) or `Cmd+Shift+D` (macOS).

## Platform support

macOS (Intel + Apple Silicon), x64 Windows, x64 Linux (Ubuntu 22.04 / 24.04).

- **Whisper models:** any modern GPU (NVIDIA / AMD / Intel), or CPU.
- **Parakeet V3:** CPU‑friendly (Intel Skylake / equivalent AMD or newer),
  ~5× real‑time on mid‑range hardware, automatic language detection.

## Linux notes

Text injection needs a display‑server tool:

| Display server | Tool | Install |
| --- | --- | --- |
| X11 | `xdotool` | `sudo apt install xdotool` |
| Wayland | `wtype` | `sudo apt install wtype` |
| Either | `dotool` | `sudo apt install dotool` (add user to `input` group) |

Without one, Echo falls back to `enigo`, which has limited Wayland support.

Echo links `gtk-layer-shell` for the overlay. If startup fails with
`libgtk-layer-shell.so.0`, install the runtime package
(`libgtk-layer-shell0` on Debian/Ubuntu, `gtk-layer-shell` on Fedora/Arch). The
overlay is off by default on Linux (`Overlay Position: None`) because some
compositors treat it as the active window and break paste‑back.

**Wayland global shortcuts** must be bound in your DE/WM to the CLI flags, e.g.
Sway/i3 `bindsym $mod+o exec echo --toggle-transcription`, or via Unix signals
(`pkill -USR2 -n echo` to toggle, `-USR1` for post‑processing).

If Echo crashes on launch, try `HANDY_NO_GTK_LAYER_SHELL=1 echo` (skip the layer
shell) or `WEBKIT_DISABLE_DMABUF_RENDERER=1 echo` (WebKit renderer). Make a
workaround permanent by prefixing your `.desktop` `Exec=` line.

## Manual model installation

If you're behind a proxy/firewall, download models by hand into the `models`
folder of your app‑data directory (shown in **Settings → About**):

- macOS `~/Library/Application Support/com.sovern.echo/models`
- Windows `%APPDATA%\com.sovern.echo\models`
- Linux `~/.config/com.sovern.echo/models`

**Whisper** (`.bin`, drop in directly):

```
https://blob.handy.computer/ggml-small.bin            # Small  (487 MB)
https://blob.handy.computer/whisper-medium-q4_1.bin   # Medium (492 MB)
https://blob.handy.computer/ggml-large-v3-turbo.bin   # Turbo  (1.6 GB)
https://blob.handy.computer/ggml-large-v3-q5_0.bin    # Large  (1.1 GB)
```

**Parakeet** (`.tar.gz`, extract the directory into `models/`, keep the exact
name `parakeet-tdt-0.6b-v3-int8`):

```
https://blob.handy.computer/parakeet-v3-int8.tar.gz   # V3 (478 MB)
```

Restart Echo; the models appear in **Settings → Models** as *Downloaded*. Custom
Whisper GGML `.bin` files dropped into `models/` are auto‑discovered under
*Custom Models*.

> Model files are hosted on the upstream Handy CDN (`blob.handy.computer`).

## Verify release signatures

Release artifacts are signed with Tauri's updater (minisign) key; the public key
lives in [`src-tauri/tauri.conf.json`](src-tauri/tauri.conf.json) under
`plugins.updater.pubkey`. Decode that key and the artifact's `.sig` from base64
and verify with `minisign` (not `gpg`). Releasing is documented in
[RELEASING.md](RELEASING.md).

## Links

- [Releases](https://github.com/master5d/Echo/releases) — download builds
- [Build from source](BUILD.md) — incl. the Windows GPU‑Whisper (Vulkan) toolchain
- [Contributing](CONTRIBUTING.md) · [Translations](CONTRIBUTING_TRANSLATIONS.md)
- [Releasing](RELEASING.md) — signing & publishing
- [Upstream: cjpais/Handy](https://github.com/cjpais/Handy) — the project Echo forks

## Contributing

Issues and PRs welcome at
[github.com/master5d/Echo](https://github.com/master5d/Echo/issues). Fork,
branch, test on your platform, and open a PR with a clear description. See
[CONTRIBUTING.md](CONTRIBUTING.md).

## Credits

Echo is a fork of **[Handy](https://github.com/cjpais/Handy)** by cjpais —
thank you for the foundation and for hosting the model CDN. Built on the work
of:

- **whisper.cpp / ggml** — fast cross‑platform Whisper inference
- **OpenAI Whisper** — the speech‑recognition models
- **NVIDIA Parakeet / Canary** and **Sber GigaAM** — multilingual & Russian ASR
- **Silero** — lightweight voice‑activity detection
- **Tauri** — the Rust app framework

## License

MIT — see [LICENSE](LICENSE).
</content>
