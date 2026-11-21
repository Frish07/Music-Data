# Music Data

[English](README.md) | **Русский**

Бесплатный редактор метаданных MP3 на русском языке. Тёмный адаптивный интерфейс по предоставленному референсу, без регистрации, серверов и внешних зависимостей.

**[Открыть веб-редактор](https://frish07.github.io/Music-Data/)** · **[Скачать MusicData.exe](https://github.com/Frish07/Music-Data/releases/latest/download/MusicData.exe)** · **[Релизы](https://github.com/Frish07/Music-Data/releases)**

## Desktop для Windows 11 x64

В `desktop/` находится нативное приложение на Rust. Оно работает офлайн, использует стандартные окна Windows и распространяется одним portable `MusicData.exe`. Не требует установки, Rust, Cargo, WebView2, внешних кодеков или пакета Visual C++ Redistributable: C runtime статически включается в сборку. Остальные DLL входят в Windows 11. Графический интерфейс не требует OpenGL/Vulkan.

Для обычного использования скачайте готовый `.exe` из [Releases](https://github.com/Frish07/Music-Data/releases) и запустите его. Права администратора не нужны. Можно хранить только `MusicData.exe`; portable ZIP дополнительно содержит инструкцию и SHA-256.

Скачивание без инструментов разработки:

1. Откройте [Actions → Windows portable desktop](https://github.com/Frish07/Music-Data/actions/workflows/desktop.yml).
2. Выберите успешный запуск и скачайте artifact **MusicData-Windows-x64-portable**.
3. Распакуйте архив и запустите `MusicData.exe`. Для скачивания Artifacts нужен вход в GitHub.

При создании нового тега версии workflow автоматически публикует `.exe`, portable ZIP и SHA-256 в [GitHub Releases](https://github.com/Frish07/Music-Data/releases). Например, создайте тег `v1.1.0` в интерфейсе GitHub. Создавать локально тег или устанавливать Git/Rust не требуется. Вкладка Releases позволяет скачать опубликованный `.exe` напрямую. Artifacts хранятся 90 дней.

Каждый push (любая ветка и тег), pull request и ручной запуск выполняют `cargo check`, тесты чтения/записи тегов, оптимизированную Release-сборку x64, проверку зависимостей PE и пробный запуск готового `.exe`. Rust устанавливается только на временный runner GitHub. Для Release включены `opt-level = 3`, LTO, один codegen unit, удаление отладочных символов и статический CRT. Pull request не создаёт Release.

Зависимости зафиксированы в `desktop/Cargo.lock`, сборка выполняется с `--locked`. Тесты проверяют кириллицу, обложки, сохранность остальных тегов и аудиоданных, некорректные файлы и защиту от перезаписи. Конфигурация: [desktop.yml](.github/workflows/desktop.yml).

Desktop поддерживает MP3 до 150 МБ, название, исполнителя, альбом, четырёхзначный год, жанр, комментарий, обложки JPG/PNG до 3 МБ и 8192 × 8192 пикселей. Есть предпросмотр тегов, перетаскивание MP3, предупреждение о несохранённых изменениях и сохранение новой копии без перекодирования аудио. Существующие файлы и исходный MP3 не перезаписываются. Предпросмотр показывает метаданные, проигрыватель в desktop-версию не входит.

Целевая платформа — **Windows 11 x64**. Один Windows `.exe` не работает нативно на всех ОС и архитектурах. Для телефонов, macOS и Linux остаётся веб-версия. ARM64-версия отдельно не собирается. Приложение не подписано сертификатом издателя; это может вызывать предупреждение Windows об неизвестном издателе.

Для разработчиков: исходники в `desktop/src/`, конфигурация сборки в `desktop/.cargo/config.toml`, workflow — `.github/workflows/desktop.yml`. При установленном Rust/MSVC сборка выполняется из `desktop/` командой `cargo build --release --target x86_64-pc-windows-msvc`; пользователю программы это не нужно.

## Веб-версия

**[Открыть Music Data на GitHub Pages](https://frish07.github.io/Music-Data/)**

- Выбор или перетаскивание MP3 до 150 МБ.
- Чтение и запись названия, исполнителя, альбома, года, жанра и комментария.
- Чтение и замена обложки JPG/PNG до 3 МБ, удаление обложки.
- Предпросмотр тегов и прослушивание трека.
- Скачивание отдельной копии `имя — edited.mp3`. Исходный файл не изменяется.
- Аудиоданные копируются без перекодирования. Неизменяемые сторонние ID3v2-кадры сохраняются.
- Обработка в браузере: аудио и обложки не отправляются на сервер. Нет аналитики, внешних шрифтов или CDN.

Поддерживаются обычные ID3v2.3 и ID3v2.4, MP3 без тегов и чтение ID3v1. При сохранении старый ID3v1 заменяется единым ID3v2-тегом. ID3v2.2, сжатые, зашифрованные, сгруппированные и unsynchronised теги отклоняются с сообщением. Другие аудиоформаты пока не поддерживаются. Поле года хранит четыре цифры; полная дата TDRC заменяется годом. Новая обложка заменяет все APIC-кадры; комментарий — все COMM-кадры.

## Локальный запуск

Нужен Node.js 22 или новее. Установка пакетов не требуется.

```sh
node scripts/serve.mjs
```

Откройте http://127.0.0.1:4173. Для редактирования загрузите свой MP3, измените поля и нажмите «Сохранить MP3».

```sh
node --test
node scripts/build.mjs
```

Статический результат создаётся в `dist/`. Все пути относительные: сайт работает в подпапке `/Music-Data/` на GitHub Pages.

## GitHub Actions и публикация

Workflow `.github/workflows/pages.yml` проверяет чтение/запись метаданных и собирает сайт для pull request. Push в `main` и ручной запуск дополнительно публикуют сайт на GitHub Pages. Сторонние секреты и npm-зависимости не нужны.

В репозитории откройте **Settings → Pages → Build and deployment → Source → GitHub Actions**. После успешного workflow адрес будет https://frish07.github.io/Music-Data/.

Права workflow: `contents: read`, а у deploy — `pages: write` и `id-token: write`. Pull request не публикуется.

## Структура

- `index.html`, `styles.css` — интерфейс.
- `src/app.js` — файлы, формы, предпросмотр и скачивание.
- `src/id3.js` — чтение и запись тегов.
- `tests/id3.test.js` — проверки кириллицы, обложек, повреждённых тегов и сохранности аудиоданных.
- `scripts/` — сборка и локальный сервер.
- `desktop/src/main.rs` — точка входа Rust-приложения.
- `desktop/src/windows.rs` — нативный интерфейс Windows и диалоги выбора файлов.
- `desktop/src/metadata.rs` — работа с тегами MP3 и Rust-тесты.
- `desktop/scripts/check_portable.py` — проверка архитектуры и DLL-зависимостей `.exe`.
- `.github/workflows/` — сборки desktop, Releases, GitHub Pages и проверка языков.

## Сборка desktop из исходников

Этот раздел предназначен только для разработчиков. Пользователю приложения достаточно готового `.exe`. Если Rust и инструменты сборки MSVC уже установлены:

```sh
cd desktop
cargo check --locked --target x86_64-pc-windows-msvc
cargo test --locked --target x86_64-pc-windows-msvc
cargo build --locked --release --target x86_64-pc-windows-msvc
```

Результат: `desktop/target/x86_64-pc-windows-msvc/release/MusicData.exe`.

## Определение языков репозитория

Исходники Rust хранятся в `desktop/src/`. `.gitattributes` явно включает `.rs`-файлы в статистику GitHub Linguist как поддерживаемый Rust-код. [Workflow проверки языков](.github/workflows/languages.yml) запускает официальный контейнер GitHub Linguist, проверяет учёт всех трёх Rust-файлов и публикует подробный отчёт в сводке запуска и Artifacts. Панель Languages GitHub пересчитывает отдельно; workflow не управляет её кешем напрямую.
