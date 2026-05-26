# Read This

Read This is a small Windows tray app that reads highlighted text aloud with natural Edge TTS voices.

It is built for the very ordinary moment where a chat reply, email, document, or long response is worth hearing instead of staring at. Highlight the text, press `Ctrl+Alt+R`, and let the app read it once from beginning to end.

## Features

- **Read highlighted text anywhere** using the current selection and a reliable clipboard-copy capture path.
- **Natural neural voices** through Edge TTS, with local Microsoft TTS available only as an optional fallback.
- **Tray-first workflow** with quick actions for Open Panel, Read Highlighted Text, Stop, Test Voice, and Quit.
- **Compact desktop UI** for voice, speed, volume, startup, and fallback settings.
- **Global shortcut support** with a keyboard watcher fallback if another app already owns the hotkey.
- **GitHub Actions build** that produces a downloadable Windows artifact.

## Quick Start

### Run From Source

```powershell
git clone <your-repo-url>
cd Read_This
python -m pip install edge-tts -t python_deps
cargo run --manifest-path .\src-tauri\Cargo.toml
```

The app starts in the system tray. Open the tray menu, choose **Open Panel**, then test the voice.

### Use The Shortcut

1. Highlight text in any app.
2. Press `Ctrl+Alt+R`.
3. Read This fetches Edge TTS audio and plays it once.
4. Use **Stop** from the tray or panel if you need to interrupt long text.

## Downloading CI Builds

This repo includes a GitHub Actions workflow that builds the Windows executable on every push, pull request, and manual run.

To download a build from GitHub:

1. Open the repository on GitHub.
2. Go to **Actions**.
3. Open the latest successful **Windows Build** run.
4. Download the `read-this-windows-x64` artifact.
5. Extract the artifact and run `read-this.exe`.

The artifact includes `python_deps` because the current Edge TTS path uses the maintained Python `edge-tts` client at runtime.

## Build Locally

### Requirements

- Windows 10 or newer
- Rust stable toolchain
- Python 3.10 or newer
- Microsoft Edge WebView2 runtime

### Commands

```powershell
python -m pip install edge-tts -t python_deps
cargo check --manifest-path .\src-tauri\Cargo.toml
cargo build --release --manifest-path .\src-tauri\Cargo.toml
```

The release executable is created at:

```text
src-tauri\target\release\read-this.exe
```

## Architecture

Read This is split into a lightweight frontend and a Rust desktop backend.

```text
app/                  Plain HTML, CSS, and JavaScript control panel
src-tauri/            Tauri v2 + Rust desktop application
vendor/msedge-tts/    Local patched Rust Edge TTS crate retained as fallback research
python_deps/          Local runtime folder for the working edge-tts client
ReadThis.ps1          Original PowerShell prototype
```

The normal read flow is:

```text
Hotkey or tray action
-> capture highlighted text
-> request Edge TTS audio
-> play MP3 audio locally
-> stop naturally at the end
```

## Notes On Latency

Edge TTS is network-backed. Short text usually starts quickly, but longer selections can take a moment because the app has to request and receive synthesized audio before playback begins.

## Troubleshooting

### Nothing happens when I press `Ctrl+Alt+R`

Another app may already own the global shortcut. Read This has a polling fallback, so restart the app and try again. You can also use **Read Highlighted Text** from the tray menu.

### Edge TTS fails

Make sure the artifact includes `python_deps`, or reinstall the runtime dependency:

```powershell
python -m pip install edge-tts -t python_deps
```

If your network blocks Microsoft's speech endpoint, enable the local fallback in the app settings.

### The clipboard changes briefly

Read This captures selected text by temporarily sending `Ctrl+C`, reading the copied text, and restoring text clipboard content. Rich clipboard formats are a future improvement.

## GitHub Actions

The workflow lives at `.github/workflows/windows-build.yml`.

It performs these steps:

- Checks out the repository.
- Installs Python and the `edge-tts` runtime dependency.
- Installs Rust stable.
- Runs `cargo check`.
- Builds `read-this.exe` in release mode.
- Stages the executable with `python_deps`.
- Uploads a downloadable Windows artifact.

## Legacy Prototype

The original tray prototype is still available:

```powershell
powershell.exe -NoProfile -ExecutionPolicy Bypass -STA -File .\ReadThis.ps1
```
