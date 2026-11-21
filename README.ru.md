# Music Data Desktop

[English](README.md) | **Русский**

Нативный редактор метаданных MP3 на **Rust** для **Windows 11 x64**. Эта ветка содержит portable desktop-приложение с русским интерфейсом.

**[Скачать MusicData.exe](https://github.com/Frish07/Music-Data/releases/latest/download/MusicData.exe)** · **[Релизы](https://github.com/Frish07/Music-Data/releases)** · **[Веб-версия — ветка main](https://github.com/Frish07/Music-Data/tree/main)**

## Desktop для Windows 11 x64

В `` находится нативное приложение на Rust. Оно работает офлайн, использует стандартные окна Windows и распространяется одним portable `MusicData.exe`. Не требует установки, Rust, Cargo, WebView2, внешних кодеков или пакета Visual C++ Redistributable: C runtime статически включается в сборку. Остальные DLL входят в Windows 11. Графический интерфейс не требует OpenGL/Vulkan.

Для обычного использования скачайте готовый `.exe` из [Releases](https://github.com/Frish07/Music-Data/releases) и запустите его. Права администратора не нужны. Можно хранить только `MusicData.exe`; portable ZIP дополнительно содержит инструкцию и SHA-256.

Скачивание без инструментов разработки:

1. Откройте [Actions → Windows portable desktop](https://github.com/Frish07/Music-Data/actions/workflows/desktop.yml).
2. Выберите успешный запуск и скачайте artifact **MusicData-Windows-x64-portable**.
3. Распакуйте архив и запустите `MusicData.exe`. Для скачивания Artifacts нужен вход в GitHub.

При создании нового тега версии workflow автоматически публикует `.exe`, portable ZIP и SHA-256 в [GitHub Releases](https://github.com/Frish07/Music-Data/releases). Например, создайте тег `v1.1.0` в интерфейсе GitHub. Создавать локально тег или устанавливать Git/Rust не требуется. Вкладка Releases позволяет скачать опубликованный `.exe` напрямую. Artifacts хранятся 90 дней.

Каждый push (любая ветка и тег), pull request и ручной запуск выполняют `cargo check`, тесты чтения/записи тегов, оптимизированную Release-сборку x64, проверку зависимостей PE и пробный запуск готового `.exe`. Rust устанавливается только на временный runner GitHub. Для Release включены `opt-level = 3`, LTO, один codegen unit, удаление отладочных символов и статический CRT. Pull request не создаёт Release.

Зависимости зафиксированы в `Cargo.lock`, сборка выполняется с `--locked`. Тесты проверяют кириллицу, обложки, сохранность остальных тегов и аудиоданных, некорректные файлы и защиту от перезаписи. Конфигурация: [desktop.yml](.github/workflows/desktop.yml).

Desktop поддерживает MP3 до 150 МБ, название, исполнителя, альбом, четырёхзначный год, жанр, комментарий, обложки JPG/PNG до 3 МБ и 8192 × 8192 пикселей. Есть предпросмотр тегов, перетаскивание MP3, предупреждение о несохранённых изменениях и сохранение новой копии без перекодирования аудио. Существующие файлы и исходный MP3 не перезаписываются. Предпросмотр показывает метаданные, проигрыватель в desktop-версию не входит.

Целевая платформа — **Windows 11 x64**. Один Windows `.exe` не работает нативно на всех ОС и архитектурах. Для телефонов, macOS и Linux остаётся веб-версия. ARM64-версия отдельно не собирается. Приложение не подписано сертификатом издателя; это может вызывать предупреждение Windows об неизвестном издателе.

Для разработчиков: исходники в `src/`, конфигурация сборки в `.cargo/config.toml`, workflow — `.github/workflows/desktop.yml`. При установленном Rust/MSVC сборка выполняется из `` командой `cargo build --release --target x86_64-pc-windows-msvc`; пользователю программы это не нужно.

## Исходный код

- `Cargo.toml`, `Cargo.lock` — Rust-проект и зафиксированные зависимости.
- `src/main.rs` — точка входа.
- `src/windows.rs` — нативный интерфейс Win32 и диалоги выбора файлов.
- `src/metadata.rs` — работа с тегами и тесты сохранности файлов.
- `scripts/check_portable.py` — проверка архитектуры и DLL.
- `.cargo/config.toml` — статическая линковка C runtime.
- `.github/workflows/desktop.yml` — сборка Windows и Releases.
- `.github/workflows/languages.yml` — проверка GitHub Linguist.

## Сборка desktop из исходников

Этот раздел предназначен только для разработчиков. Пользователю приложения достаточно готового `.exe`. Если Rust и инструменты сборки MSVC уже установлены:

```sh
cargo check --locked --target x86_64-pc-windows-msvc
cargo test --locked --target x86_64-pc-windows-msvc
cargo build --locked --release --target x86_64-pc-windows-msvc
```

Результат: `target/x86_64-pc-windows-msvc/release/MusicData.exe`.

Новые теги версий должны указывать на коммиты ветки `desktop`. Ветка `main` содержит сайт GitHub Pages. Существующие теги и релизы сохранены.
