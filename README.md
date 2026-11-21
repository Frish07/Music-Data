# Music Data

**English** | [Русский](README.ru.md)

An MP3 metadata editor with a browser version and a native **Rust desktop app for Windows 11 x64**. Edit titles, artists, albums, years, genres, comments, and cover art. Both interfaces are currently in Russian.

**[Open the web editor](https://frish07.github.io/Music-Data/)** · **[Download MusicData.exe](https://github.com/Frish07/Music-Data/releases/latest/download/MusicData.exe)** · **[Releases](https://github.com/Frish07/Music-Data/releases)**

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

Rust is installed **only on the temporary GitHub runner**, never on the end user's computer. Dependencies are pinned in `desktop/Cargo.lock` and builds use `--locked`. See [the desktop workflow](.github/workflows/desktop.yml).

## Web editor

**[Use Music Data on GitHub Pages](https://frish07.github.io/Music-Data/)**

The responsive web app processes files in your browser. Audio and cover art are not uploaded to a server. There are no accounts, analytics, CDN dependencies, or external fonts.

- Select or drop an MP3 up to 150 MB.
- Edit the same six metadata fields and JPG/PNG cover art up to 3 MB.
- Preview the updated tags and listen to the track.
- Download a separate `name — edited.mp3` copy; the original stays unchanged.
- Preserve audio bytes without re-encoding and retain unedited ID3v2 frames.

The web editor supports ordinary ID3v2.3 and ID3v2.4 tags, untagged MP3 files, and reading ID3v1. On saving, the legacy ID3v1 tag is replaced with a single ID3v2 tag. ID3v2.2, compressed, encrypted, grouped, and unsynchronised tags are rejected with an explanation. Other audio formats are not supported.

The year field stores four digits; a full TDRC date is replaced with the year. Replacing the cover replaces all APIC frames, and editing the comment replaces all COMM frames. The desktop app also replaces the cover and comment frames when saving.

### Run the web editor locally

For developers: Node.js 22 or newer is required. No package installation is needed.

```sh
node scripts/serve.mjs
```

Open [http://127.0.0.1:4173](http://127.0.0.1:4173), load an MP3, edit its fields, and save a copy.

```sh
node --test
node scripts/build.mjs
```

Static output is written to `dist/`. All asset paths are relative, so the website works under the `/Music-Data/` GitHub Pages path.

### GitHub Pages deployment

The [Pages workflow](.github/workflows/pages.yml) tests and builds the web app for pull requests. A push to `main` or a manual run also deploys it to GitHub Pages. No third-party secrets or npm dependencies are required.

The publishing source is **Settings → Pages → Build and deployment → Source → GitHub Actions**. Workflow permissions are `contents: read`, plus `pages: write` and `id-token: write` for deployment. Pull requests are not deployed.

## Source code

| Path | Purpose |
| --- | --- |
| `desktop/src/main.rs` | Rust application entry point |
| `desktop/src/windows.rs` | Native Windows interface and file dialogs |
| `desktop/src/metadata.rs` | MP3 tag operations and Rust tests |
| `desktop/.cargo/config.toml` | Static CRT configuration for Windows x64 |
| `desktop/scripts/check_portable.py` | Executable architecture and DLL checks |
| `index.html`, `styles.css` | Web interface |
| `src/app.js` | Browser file handling, forms, preview, and downloads |
| `src/id3.js` | Browser ID3 reader and writer |
| `tests/id3.test.js` | Web metadata and audio-preservation tests |
| `scripts/` | Web build and local preview scripts |
| `.github/workflows/` | Desktop builds, releases, Pages, and language verification |

### Build the desktop app from source

This is optional and intended for developers. End users should download the ready-made `.exe` instead. With Rust and the MSVC build tools already installed:

```sh
cd desktop
cargo check --locked --target x86_64-pc-windows-msvc
cargo test --locked --target x86_64-pc-windows-msvc
cargo build --locked --release --target x86_64-pc-windows-msvc
```

The executable is written to `desktop/target/x86_64-pc-windows-msvc/release/MusicData.exe`.

### Repository language detection

The Rust sources are tracked in `desktop/src/`. `.gitattributes` explicitly includes `.rs` files in GitHub Linguist statistics as maintained Rust code. The [language-verification workflow](.github/workflows/languages.yml) runs the official GitHub Linguist container, checks that all three Rust source files are counted, and publishes a detailed report in its job summary and artifact. GitHub refreshes its Languages sidebar separately; this workflow does not directly control the sidebar cache.
