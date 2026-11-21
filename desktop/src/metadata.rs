//! MP3 metadata operations shared by the Windows interface and file-safety tests.

use id3::{frame::{Comment, Picture, PictureType}, Tag, TagLike, Version};
use std::{fs, io::{Read, Seek}, path::{Path, PathBuf}};

pub const MAX_AUDIO: u64 = 150 * 1024 * 1024;
pub const MAX_COVER: u64 = 3 * 1024 * 1024;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Fields {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub year: String,
    pub genre: String,
    pub comment: String,
}

pub struct Track {
    pub path: PathBuf,
    pub tag: Tag,
    pub fields: Fields,
    pub cover: Option<Picture>,
    pub size: u64,
}

pub fn load(path: &Path) -> Result<Track, String> {
    if !path.extension().is_some_and(|e| e.eq_ignore_ascii_case("mp3")) {
        return Err("Выберите аудиофайл в формате MP3.".into());
    }
    let size = fs::metadata(path).map_err(|e| e.to_string())?.len();
    if size > MAX_AUDIO { return Err("Максимальный размер MP3 — 150 МБ.".into()); }
    // Validate MPEG audio independently of the extension and presence of ID3 tags.
    let mut file = fs::File::open(path).map_err(|e| e.to_string())?;
    let mut header = [0u8; 10];
    file.read_exact(&mut header).map_err(|_| "Файл слишком короткий для MP3.".to_string())?;
    let start = if &header[..3] == b"ID3" {
        if header[6..10].iter().any(|v| *v > 127) { return Err("Повреждён заголовок ID3.".into()); }
        10 + header[6..10].iter().fold(0u64, |n, b| (n << 7) | *b as u64)
            + if header[3] == 4 && header[5] & 16 != 0 { 10 } else { 0 }
    } else { 0 };
    use std::io::{Seek, SeekFrom};
    file.seek(SeekFrom::Start(start)).map_err(|e| e.to_string())?;
    let mut audio = Vec::new();
    file.take(8192).read_to_end(&mut audio).map_err(|e| e.to_string())?;
    if !audio.windows(4).any(|b| b[0] == 255 && b[1] & 224 == 224 && b[1] & 24 != 8 && b[1] & 6 != 0 && b[2] & 240 != 240 && b[2] & 240 != 0 && b[2] & 12 != 12) {
        return Err("В файле не найдено корректное MP3-аудио.".into());
    }
    let tag = match Tag::read_from_path(path) {
        Ok(tag) => tag,
        Err(error) if matches!(error.kind, id3::ErrorKind::NoTag) => Tag::new(),
        Err(error) => return Err(format!("Не удалось прочитать теги: {error}")),
    };
    let fields = Fields {
        title: tag.title().unwrap_or_default().to_owned(),
        artist: tag.artist().unwrap_or_default().to_owned(),
        album: tag.album().unwrap_or_default().to_owned(),
        year: tag.year().or_else(|| tag.date_recorded().map(|d| d.year)).map(|v| v.to_string()).unwrap_or_default(),
        genre: tag.genre_parsed().map(|v| v.into_owned()).unwrap_or_default(),
        comment: tag.comments().next().map(|c| c.text.clone()).unwrap_or_default(),
    };
    let cover = tag.pictures().find(|p| p.picture_type == PictureType::CoverFront)
        .or_else(|| tag.pictures().next()).cloned();
    Ok(Track { path: path.to_owned(), tag, fields, cover, size })
}

pub fn cover_from_path(path: &Path) -> Result<Picture, String> {
    if fs::metadata(path).map_err(|e| e.to_string())?.len() > MAX_COVER {
        return Err("Обложка должна быть не больше 3 МБ.".into());
    }
    let data = fs::read(path).map_err(|e| e.to_string())?;
    let format = image::guess_format(&data).map_err(|_| "Выберите JPG или PNG.".to_string())?;
    let mime = match format {
        image::ImageFormat::Jpeg => "image/jpeg",
        image::ImageFormat::Png => "image/png",
        _ => return Err("Выберите JPG или PNG.".into()),
    };
    decode_cover(&data)?;
    Ok(Picture { mime_type: mime.into(), picture_type: PictureType::CoverFront, description: String::new(), data })
}

pub fn decode_cover(data: &[u8]) -> Result<image::DynamicImage, String> {
    let mut reader = image::ImageReader::new(std::io::Cursor::new(data))
        .with_guessed_format().map_err(|e| e.to_string())?;
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(8192);
    limits.max_image_height = Some(8192);
    limits.max_alloc = Some(128 * 1024 * 1024);
    reader.limits(limits);
    reader.decode().map_err(|_| "Не удалось прочитать обложку. Максимум 8192 × 8192 пикселя.".into())
}

pub fn updated_tag(track: &Track, fields: &Fields, cover: &Option<Picture>) -> Result<Tag, String> {
    if !fields.year.is_empty() && (fields.year.len() != 4 || !fields.year.bytes().all(|b| b.is_ascii_digit())) {
        return Err("Укажите год из четырёх цифр или оставьте поле пустым.".into());
    }
    let mut tag = track.tag.clone();
    for id in ["TIT2", "TPE1", "TALB", "TYER", "TDRC", "TCON", "COMM", "APIC"] { tag.remove(id); }
    if !fields.title.is_empty() { tag.set_title(&fields.title); }
    if !fields.artist.is_empty() { tag.set_artist(&fields.artist); }
    if !fields.album.is_empty() { tag.set_album(&fields.album); }
    if !fields.genre.is_empty() { tag.set_genre(&fields.genre); }
    if !fields.year.is_empty() { tag.set_year(fields.year.parse().map_err(|_| "Некорректный год.".to_string())?); }
    if !fields.comment.is_empty() { tag.add_frame(Comment { lang: "rus".into(), description: String::new(), text: fields.comment.clone() }); }
    if let Some(picture) = cover { tag.add_frame(picture.clone()); }
    Ok(tag)
}

pub fn save_copy(track: &Track, fields: &Fields, cover: &Option<Picture>, destination: &Path) -> Result<(), String> {
    let tag = updated_tag(track, fields, cover)?;
    let source = fs::canonicalize(&track.path).map_err(|e| e.to_string())?;
    if destination.exists() && fs::canonicalize(destination).map_err(|e| e.to_string())? == source {
        return Err("Выберите другое имя. Исходный MP3 не перезаписывается.".into());
    }
    if destination.exists() { return Err("Файл с таким именем уже существует. Выберите новое имя.".into()); }
    // create_new also protects against a file appearing after the existence check.
    let mut output = fs::OpenOptions::new().read(true).write(true).create_new(true).open(destination).map_err(|e| e.to_string())?;
    let result = (|| -> Result<(), String> {
        let mut input = fs::File::open(&source).map_err(|e| e.to_string())?;
        std::io::copy(&mut input, &mut output).map_err(|e| e.to_string())?;
        output.rewind().map_err(|e| e.to_string())?;
        tag.write_to_file(&mut output, Version::Id3v24).map_err(|e| e.to_string())?;
        output.sync_all().map_err(|e| e.to_string())?;
        Ok(())
    })();
    drop(output);
    if result.is_err() { let _ = fs::remove_file(destination); }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    fn temp() -> PathBuf {
        let dir = std::env::temp_dir().join(format!("music-data-test-{}-{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed)));
        fs::create_dir_all(&dir).unwrap(); dir
    }
    #[test]
    fn unicode_roundtrip_and_original_untouched() {
        let dir = temp(); let source = dir.join("источник.mp3"); let destination = dir.join("копия.mp3");
        let original = include_bytes!("../tests/fixtures/tone.mp3");
        fs::write(&source, original).unwrap();
        let track = load(&source).unwrap();
        let fields = Fields { title: "Ночной город 🎵".into(), artist: "Артист".into(), album: "Альбом".into(), year: "2026".into(), genre: "Indie".into(), comment: "Моя заметка\nВторая строка".into() };
        save_copy(&track, &fields, &None, &destination).unwrap();
        assert_eq!(load(&destination).unwrap().fields, fields);
        assert_eq!(fs::read(&source).unwrap(), original);
        let mut a = std::io::Cursor::new(original.as_slice()); Tag::skip(&mut a).unwrap();
        let result = fs::read(&destination).unwrap();
        let mut b = std::io::Cursor::new(result.as_slice()); Tag::skip(&mut b).unwrap();
        assert!(&original[a.position() as usize..] == &result[b.position() as usize..], "MP3 audio bytes changed or duplicate ID3 headers were inserted");
        assert!(save_copy(&track, &fields, &None, &source).is_err());
        assert!(save_copy(&track, &fields, &None, &destination).is_err());
        fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn invalid_input_and_year_are_rejected() {
        let dir = temp(); let path = dir.join("invalid.mp3"); fs::write(&path, b"not music at all").unwrap();
        assert!(load(&path).is_err());
        fs::write(&path, include_bytes!("../tests/fixtures/tone.mp3")).unwrap();
        let track = load(&path).unwrap();
        assert!(updated_tag(&track, &Fields { year: "26".into(), ..Fields::default() }, &None).is_err());
        fs::remove_dir_all(dir).unwrap();
    }
    #[test]
    fn picture_and_unrelated_frames_survive() {
        let dir = temp(); let path = dir.join("track.mp3"); let out = dir.join("out.mp3");
        fs::write(&path, include_bytes!("../tests/fixtures/tone.mp3")).unwrap();
        let mut track = load(&path).unwrap(); track.tag.set_album_artist("Other artist");
        let mut png = std::io::Cursor::new(Vec::new());
        image::DynamicImage::new_rgb8(2, 2).write_to(&mut png, image::ImageFormat::Png).unwrap();
        let image_path = dir.join("cover.png"); fs::write(&image_path, png.into_inner()).unwrap();
        let cover = Some(cover_from_path(&image_path).unwrap());
        save_copy(&track, &track.fields, &cover, &out).unwrap();
        let saved = load(&out).unwrap();
        assert_eq!(saved.cover.unwrap().data, cover.unwrap().data);
        assert_eq!(saved.tag.album_artist(), Some("Other artist"));
        fs::remove_dir_all(dir).unwrap();
    }
}
