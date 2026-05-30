# Releasing Echo

Echo ships auto-updates via Tauri's updater. The app checks
`github.com/master5d/Echo/releases/latest` and verifies each release against the
public key in `src-tauri/tauri.conf.json` (`plugins.updater.pubkey`). Releases
must be signed with the matching private key.

## One-time setup

A signing keypair was generated (empty password). The **public** key is already
committed in `tauri.conf.json`. The **private** key lives only on the maintainer
machine at:

```
C:\Users\sasha\.tauri\echo-updater.key
```

Add it as a GitHub Actions secret so CI can sign releases. **Never commit it.**

1. Open the file's contents (it's a short base64 blob).
2. In GitHub: `master5d/Echo` -> Settings -> Secrets and variables -> Actions ->
   New repository secret:
   - Name: `TAURI_SIGNING_PRIVATE_KEY`
   - Value: the full contents of `echo-updater.key`
3. Add a second secret (the key has no password):
   - Name: `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`
   - Value: _(leave empty)_

> Back up `echo-updater.key` somewhere safe. If it's lost, existing installs can
> no longer auto-update (you'd have to ship a new pubkey + reinstall).

OS code-signing (Apple notarization, Windows Authenticode) is **off** by default
(`sign-binaries: false` in `release.yml`) since it needs paid certificates.
Auto-updates still work; users just see an "unknown publisher" prompt on first
install. To enable it later, add the `APPLE_*` / `AZURE_*` secrets and flip
`sign-binaries: true`.

## Cutting a release

1. Bump the version in **all three** (keep them in sync):
   - `src-tauri/tauri.conf.json` -> `version`
   - `package.json` -> `version`
   - `src-tauri/Cargo.toml` -> `[package] version`
2. Commit and push to `main`.
3. GitHub -> Actions -> **Release** -> _Run workflow_ (it's `workflow_dispatch`).
4. It creates a **draft** release `v<version>`, builds the matrix
   (macOS/Windows/Linux), signs the updater artifacts, and uploads them plus
   `latest.json`.
5. Review the draft release, then **Publish** it.
6. Installed apps' "Check for updates" (footer / tray) will now find it.

The build matrix in `release.yml` covers 7 targets with `fail-fast: false`, so a
single platform failing won't block the others. Trim the matrix if you only need
specific platforms (e.g. Windows x64).
