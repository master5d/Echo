---
name: echo-ask
description: Ask the human a question through Echo's floating panel (Windows). Use when a task needs user input, confirmation, or a periodic check-in, and you are running on the user's machine alongside Echo.
---

# Echo Ask

Echo (the dictation app) exposes a localhost Agent Bridge.

1. Read the token: `%APPDATA%\com.sovern.echo\agent_bridge_token`.
2. POST `http://127.0.0.1:4123/v1/ask` with `Authorization: Bearer <token>`:
   `{ "question": "...", "kind": "text"|"choice"|"confirm", "options": [...],
      "timeout_s": 300, "speak": true|false, "source": "<your-name>" }`
   The call BLOCKS until the human answers / dismisses / timeout:
   `{ "status": "answered"|"dismissed"|"timeout", "answer": "..." }`.
3. Fire-and-forget messages: POST `/v1/notify` `{ "message": "...", "speak": true }`.
4. Journal: GET `/v1/answers?since=<unix_ms>`.

Etiquette: one question at a time (server enforces it); short questions;
set `speak:true` only for time-sensitive asks; always set `source`.
CLI alternative (cron loops): `echo --ask "..." --ask-options "a,b" --ask-timeout 60`.
