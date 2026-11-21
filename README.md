# Music Data

**English** | [Русский](README.ru.md)

A browser-based MP3 metadata editor. Edit titles, artists, albums, years, genres, comments, and cover art locally, without uploading your music. The interface is currently in Russian.

**[Open the web editor](https://frish07.github.io/Music-Data/)** · **[Rust desktop app — desktop branch](https://github.com/Frish07/Music-Data/tree/desktop)** · **[Download desktop app](https://github.com/Frish07/Music-Data/releases/latest/download/MusicData.exe)**

This `main` branch contains the website. The portable Rust application and its Windows build/release workflow are maintained separately in the [`desktop` branch](https://github.com/Frish07/Music-Data/tree/desktop).

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

- `index.html`, `styles.css` — web interface.
- `src/app.js` — files, forms, preview, and downloads.
- `src/id3.js` — browser ID3 reader and writer.
- `tests/id3.test.js` — metadata and audio-preservation tests.
- `scripts/` — build and local preview.
- `.github/workflows/pages.yml` — tests and GitHub Pages deployment.
