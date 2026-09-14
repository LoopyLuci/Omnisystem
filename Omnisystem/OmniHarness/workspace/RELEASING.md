# Releasing Omnisystem Workspace

This is the real, current release process for the Workspace desktop app — what
CI actually does today, what it does not do, and what a maintainer has to set
up before the auto-updater will work at all.

## Platform scope: Windows-only

The Workspace build (`Omnisystem/OmniHarness/workspace/src-tauri`) only ever
produces a Windows installer, and that is deliberate, not a gap to "get to
later." Every native dependency it pulls in — audio (`cpal`, `rodio`),
PTY (`portable-pty`), GPU (the Vulkan/`native` crate), screen capture
(`scrap`, `enigo`) — has only ever been built and exercised on Windows in
this project's history. Shipping an untested macOS or Linux installer would
be a false "enterprise grade" claim, not a real capability, so CI doesn't
attempt it. If macOS/Linux support is ever wanted, it needs its own build +
verification pass on those platforms first — this document does not promise
a path to that.

There is also no code-signing / notarization for the Windows installer. A
real Authenticode certificate costs money and isn't something that can be
provisioned as part of this kind of change — Windows SmartScreen will warn
on a fresh install until one is purchased and wired into the release
workflow. That's a known, real gap, not something this pass papers over.

## How to cut a release

1. Bump `"version"` in `Omnisystem/OmniHarness/workspace/src-tauri/tauri.conf.json`
   (and `src/package.json` if you want them to match — they're independent
   fields, only `tauri.conf.json`'s version is what the installer/updater
   actually use).
2. Commit, then tag and push:
   ```sh
   git tag v0.3.0
   git push origin v0.3.0
   ```
3. That tag push triggers `.github/workflows/release.yml`, which:
   - Builds the Workspace Tauri app on `windows-latest` via
     `tauri-apps/tauri-action@v1`, producing the `.msi`/`.exe` installer.
   - **Signs the update artifacts** and generates `latest.json` (via
     `includeUpdaterJson: true`) — see "Updater signing" below for what
     that requires.
   - Packages the VS Code extension as a `.vsix` on a separate Ubuntu job.
   - Attaches everything to a **draft** GitHub Release named
     `Workspace v0.3.0`. Nothing is published automatically — a human opens
     the draft release, edits the notes, and clicks "Publish" (or
     runs `gh release edit <tag> --draft=false`).
4. Once published, the release's assets include (at minimum):
   `Omnisystem.Workspace_0.3.0_x64-setup.exe` (or `.msi`), a `.sig` file
   next to each installer artifact, and `latest.json`.

`workflow_dispatch` is also enabled, so the same build can be triggered
manually from the Actions tab without a tag push (useful for testing the
build itself; it still needs a `tagName` context to produce a coherent
release, so prefer the tag-push path for anything you intend to actually
publish).

## Updater signing — what's already wired up, what you still need to do

`tauri-plugin-updater` requires every update artifact to carry a detached
Ed25519 (minisign-format) signature that the app verifies against a public
key baked into `tauri.conf.json`. That public key is already committed:

```json
// src-tauri/tauri.conf.json → plugins.updater.pubkey
"dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IENEMEYxMEVGQzVFQzZGODcKUldTSGIrekY3eEFQelRFeUdBcEU3UjFTVW9VemJJZEcyV2V0dCs1ZzR4NkFUL3NHRFhHaVhPTTMK"
```

The matching **private** key was generated with the real Tauri CLI
(`cargo tauri signer generate`) during this change, password-protected, and
was deliberately **not** committed and **not** pushed to GitHub Secrets by
the automation that made this change — a human needs to do that by hand.
See the handoff note this task's author left in the final report for the
actual key material and password; if you're reading this later and don't
have it, generate a fresh keypair yourself (this invalidates the pubkey
above, so update `tauri.conf.json` to match — see below) and store it
somewhere durable (password manager, HSM, vault), because **losing it means
you can never publish an update again and existing installs will only ever
be able to verify signatures from the original key.**

To make the release workflow actually sign builds, add two **repository**
secrets (Settings → Secrets and variables → Actions → New repository secret)
on `LoopyLuci/Omnisystem`:

| Secret name                           | Value                                              |
|----------------------------------------|-----------------------------------------------------|
| `TAURI_SIGNING_PRIVATE_KEY`             | The full contents of the private key file (the `.key` file `cargo tauri signer generate` produced — a minisign-format encrypted private key, not the raw bytes) |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`    | The password used to generate that key |

`.github/workflows/release.yml` already references
`secrets.TAURI_SIGNING_PRIVATE_KEY` / `secrets.TAURI_SIGNING_PRIVATE_KEY_PASSWORD`
and passes them to `tauri-apps/tauri-action@v1` as env vars — that action
signs the built artifacts and writes `latest.json` automatically when both
are present. **Until those two secrets are set, tag-push builds still
succeed and still produce an installer — they just come out unsigned, with
no `latest.json`, so the in-app updater will silently find nothing to
install.** There's no separate step to fail loudly about this today; if you
want a release to hard-fail without signing configured, that's a real
follow-up gap.

### Rotating the signing key

If the key ever needs to change (compromise, lost password, deliberate
rotation):

```sh
cd Omnisystem/OmniHarness/workspace
cargo tauri signer generate -w /somewhere/outside/the/repo/new-key.key --password "<new password>"
```

Then:
1. Replace `plugins.updater.pubkey` in `tauri.conf.json` with the new
   `.key.pub` file's contents.
2. Update the two GitHub secrets with the new private key + password.
3. Every install running a version signed with the **old** key will not
   trust updates signed with the **new** key (that's the point of pinning
   `pubkey`) — so a key rotation needs one more release built and signed
   with the *old* key first, whose only job is to ship the new `pubkey`,
   before you can cut further releases under the new key. Plan accordingly;
   this isn't automated.

## In-app update flow (what actually ships)

- On launch (desktop only — the updater plugin has no Android/iOS build),
  `App.svelte` runs a background `check()` via
  `src/lib/stores/updater.ts`. If a newer version is found it shows a toast;
  it never auto-installs anything.
- The toolbar has a "🔄 Updates" button (next to Settings) that opens
  `src/lib/panels/UpdatePanel.svelte`, which shows the installed version,
  lets the user trigger a check, shows the available version + release notes
  (`body` from `latest.json`) if one exists, downloads with a real progress
  bar, and offers a "Restart to Apply" button that calls
  `@tauri-apps/plugin-process`'s `relaunch()`.
- `plugins.updater.dialog` is intentionally **not set** in `tauri.conf.json`
  — that key doesn't exist in the Tauri v2 updater plugin (it's a
  leftover-sounding v1 concept). The v2 plugin never shows an OS-native
  dialog on its own; `check()`/`downloadAndInstall()` are just JS calls, so
  the custom `UpdatePanel.svelte` flow above *is* the update UI, not an
  alternative to some built-in one.

## What's verified vs. what isn't

Verified locally as part of this change:
- `cargo check` on `src-tauri` (full workspace, all crates) — passes.
- `cargo test --lib --no-run` on `src-tauri` — compiles clean.
- `npm run check` (svelte-check) on the frontend — 0 errors (pre-existing
  a11y warnings in unrelated files only).
- `npx vitest run` — 61/61 tests pass, including new coverage for
  `src/lib/utils/version.ts` (semver comparison) and
  `src/lib/stores/updater.ts` (the check → available → downloading →
  ready-to-restart state machine, using a fake `check()`/`downloadAndInstall`
  so no real network or signing is involved).
- `npm run build` (`vite build`) — succeeds, `UpdatePanel` code-splits into
  its own chunk like the other lazy panels.

**Not** verified, and not verifiable without publishing a real tagged
release: the actual signed build → `latest.json` → in-app `check()` →
download → install → relaunch round trip against a live GitHub Release.
That needs the two secrets above configured and an actual tag pushed; treat
the first real release after this change as the real end-to-end test, and
watch the Actions run for the `tauri-action` signing step specifically.
