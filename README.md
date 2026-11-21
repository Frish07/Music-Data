# Music Data Desktop

**English** | [Русский](README.ru.md)

Native offline MP3 metadata editor written in **Rust** for **Windows 11 x64**. This branch contains the portable desktop application. The interface is currently in Russian.

**[Download MusicData.exe](https://github.com/Frish07/Music-Data/releases/latest/download/MusicData.exe)** · **[Releases](https://github.com/Frish07/Music-Data/releases)** · **[Web version — main branch](https://github.com/Frish07/Music-Data/tree/main)**

## Windows desktop app

Download `MusicData.exe` from Releases and run it. The app is portable: no installer, administrator privileges, internet connection, Rust, Cargo, WebView2, external codecs, or Visual C++ Redistributable package is required.

The application uses native Windows controls. Its C runtime is linked statically, and all imported DLLs are included with Windows 11. It does not require OpenGL or Vulkan. You can keep just the `.exe`; the portable ZIP also includes instructions and a SHA-256 checksum.

### Features

- Open or drag in one MP3 file, up to 150 MB.
- Read and edit the title, artist, album, four-digit year, genre, and comment.
- Add, replace, or remove JPG/PNG cover art, up to 3 MB and 8192 × 8192 pixels.
- Preview metadata and reset edits to the original file's tags.
- Save a separate copy without re-encoding audio.
- Protect the original MP3 and existing files from being overwritten.
- Warn about unsaved changes before opening another file or closing.
- Work entirely offline, with no file uploads or account required.

Desktop preview displays metadata; it does not include an audio player. The target platform is **Windows 11 x64**. This executable is not a native app for macOS, Linux, Android, or iOS. An ARM64 build is not provided or tested. The executable is not code-signed, so Windows may show an unknown-publisher warning.

### Download from Actions

1. Open [Actions → Windows portable desktop](https://github.com/Frish07/Music-Data/actions/workflows/desktop.yml).
2. Select a successful run.
3. Download the **MusicData-Windows-x64-portable** artifact.
4. Extract it and launch `MusicData.exe`.

GitHub sign-in is required to download Actions artifacts. Artifacts are retained for 90 days. Published builds are also available directly in [Releases](https://github.com/Frish07/Music-Data/releases).

### Automated builds and releases

Every push to any branch or tag, pull request, and manual run performs:

1. `cargo check` for `x86_64-pc-windows-msvc`.
2. Metadata tests covering Unicode, cover art, unrelated tags, unchanged audio, invalid input, and overwrite protection.
3. An optimized Release build with `opt-level = 3`, LTO, one codegen unit, stripped symbols, and a static C runtime.
4. PE import-table inspection to verify x64, the GUI subsystem, and Windows system DLLs only.
5. A smoke test that starts and closes the actual `.exe`.
6. Upload of the portable executable and instructions to Actions Artifacts.

Pushing a new version tag, such as `v1.1.0`, additionally publishes the verified `.exe`, portable ZIP, and SHA-256 checksum to GitHub Releases. Tags can be created through the GitHub website. Pull requests do not publish releases.

Rust is installed **only on the temporary GitHub runner**, never on the end user's computer. Dependencies are pinned in `Cargo.lock` and builds use `--locked`. See [the desktop workflow](.github/workflows/desktop.yml).

## Source code

- `Cargo.toml`, `Cargo.lock` — Rust project and pinned dependencies.
- `src/main.rs` — application entry point.
- `src/windows.rs` — native Win32 interface and file dialogs.
- `src/metadata.rs` — tag editing and file-safety tests.
- `scripts/check_portable.py` — PE architecture and DLL checks.
- `.cargo/config.toml` — static C runtime configuration.
- `.github/workflows/desktop.yml` — Windows builds and releases.
- `.github/workflows/languages.yml` — official GitHub Linguist verification.

### Build the desktop app from source

This is optional and intended for developers. End users should download the ready-made `.exe` instead. With Rust and the MSVC build tools already installed:

```sh
cargo check --locked --target x86_64-pc-windows-msvc
cargo test --locked --target x86_64-pc-windows-msvc
cargo build --locked --release --target x86_64-pc-windows-msvc
```

The executable is written to `target/x86_64-pc-windows-msvc/release/MusicData.exe`.

New version tags must point to a commit on the `desktop` branch. The `main` branch contains the GitHub Pages website. Existing tags and releases are preserved.
