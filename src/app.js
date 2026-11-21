import { MAX_FILE_SIZE, readMetadata, writeMetadata, validateCover } from './id3.js';
const $ = id => document.getElementById(id);
const keys = ['title', 'artist', 'album', 'year', 'genre', 'comment'];
let source = null, file = null, cover = null, artUrl = null, audioUrl = null, dirty = false, busy = false;
const form = $('metadata-form');
function message(text, type = '') { $('message').textContent = text; $('message').className = `status-message ${type}`; }
function values() { return Object.fromEntries(keys.map(key => [key, $(key).value.trim()])); }
function markDirty(value = true) {
  dirty = value;
  $('editor-status').textContent = value ? 'Есть изменения' : 'Теги прочитаны';
  $('editor-status').classList.add('ready');
  $('comment-count').textContent = `${$('comment').value.length} / 5000`;
}
function updateCover() {
  if (artUrl) URL.revokeObjectURL(artUrl);
  artUrl = cover ? URL.createObjectURL(new Blob([cover.bytes], { type: cover.mime })) : null;
  for (const id of ['cover-image', 'preview-image']) {
    $(id).hidden = !artUrl;
    if (artUrl) $(id).src = artUrl; else $(id).removeAttribute('src');
  }
  $('record').style.backgroundImage = artUrl ? `url("${artUrl}")` : '';
  $('record').classList.toggle('has-cover', Boolean(artUrl));
  $('remove-cover').hidden = !cover;
}
function restore() {
  for (const key of keys) $(key).value = source.metadata[key];
  cover = source.cover;
  updateCover();
  markDirty(false);
}
function setBusy(value) {
  busy = value;
  $('dropzone').disabled = value;
  $('replace-file').disabled = value;
  $('editor-fields').disabled = value || !source;
  for (const id of ['save', 'reset', 'preview']) $(id).disabled = value || !source;
}
async function loadFile(nextFile) {
  if (!nextFile || busy) return;
  if (dirty && !window.confirm('Открыть другой файл? Несохранённые изменения будут потеряны.')) return;
  setBusy(true);
  message('Читаем метаданные…');
  try {
    if (!/\.mp3$/i.test(nextFile.name)) throw new Error('Выберите файл MP3. Другие форматы пока не поддерживаются.');
    if (nextFile.size > MAX_FILE_SIZE) throw new Error('Максимальный размер MP3 — 150 МБ.');
    const parsed = readMetadata(await nextFile.arrayBuffer());
    source = parsed;
    file = nextFile;
    restore();
    $('file-name').textContent = file.name;
    $('file-name').title = file.name;
    $('file-size').textContent = `${(file.size / 1024 / 1024).toLocaleString('ru-RU', { maximumFractionDigits: 2 })} МБ · ID3v2.${source.version}`;
    $('file-info').hidden = false;
    $('record-caption').textContent = source.metadata.artist || 'Готово к вашей следующей истории';
    if (audioUrl) URL.revokeObjectURL(audioUrl);
    audioUrl = URL.createObjectURL(file);
    $('audio-player').src = audioUrl;
    message('Теги прочитаны. Измените нужные поля и сохраните новую копию.', 'success');
  } catch (error) { message(error.message, 'error'); }
  finally { setBusy(false); $('audio-file').value = ''; }
}
function save() {
  if (!source || busy || !form.reportValidity()) return;
  try {
    const result = writeMetadata(source, values(), cover);
    const blob = new Blob([result], { type: 'audio/mpeg' });
    const url = URL.createObjectURL(blob);
    const link = document.createElement('a');
    link.href = url;
    link.download = file.name.replace(/\.mp3$/i, '') + ' — edited.mp3';
    document.body.append(link);
    link.click();
    link.remove();
    setTimeout(() => URL.revokeObjectURL(url), 60000);
    // The saved copy becomes the baseline for reset and subsequent edits.
    source = readMetadata(result);
    markDirty(false);
    message('Копия MP3 с новыми тегами готова к скачиванию. Исходный файл сохранён.', 'success');
    $('preview-dialog').close();
  } catch (error) { message(error.message, 'error'); $('preview-dialog').close(); }
}
$('dropzone').addEventListener('click', () => $('audio-file').click());
$('replace-file').addEventListener('click', () => $('audio-file').click());
$('audio-file').addEventListener('change', event => loadFile(event.target.files[0]));
for (const event of ['dragenter', 'dragover']) $('dropzone').addEventListener(event, e => { e.preventDefault(); $('dropzone').classList.add('dragover'); });
for (const event of ['dragleave', 'drop']) $('dropzone').addEventListener(event, e => { e.preventDefault(); $('dropzone').classList.remove('dragover'); });
$('dropzone').addEventListener('drop', event => {
  if (event.dataTransfer.files.length > 1) { message('Добавляйте по одному MP3 за раз.', 'error'); return; }
  loadFile(event.dataTransfer.files[0]);
});
window.addEventListener('dragover', e => e.preventDefault());
window.addEventListener('drop', e => e.preventDefault());
for (const id of ['choose-cover', 'cover-button']) $(id).addEventListener('click', () => $('cover-file').click());
$('cover-file').addEventListener('change', async event => {
  const image = event.target.files[0];
  if (!image || !source || busy) return;
  setBusy(true);
  try {
    if (image.size > 3 * 1024 * 1024) throw new Error('Обложка должна быть не больше 3 МБ.');
    const bytes = new Uint8Array(await image.arrayBuffer());
    const mime = validateCover(bytes);
    const bitmap = await createImageBitmap(new Blob([bytes], { type: mime }));
    bitmap.close();
    cover = { bytes, mime };
    updateCover(); markDirty(); message('Новая обложка добавлена. Сохраните MP3, чтобы записать её в файл.');
  } catch (error) { message(error.message === 'The source image could not be decoded.' ? 'Не удалось прочитать изображение.' : error.message, 'error'); }
  finally { event.target.value = ''; setBusy(false); }
});
$('remove-cover').addEventListener('click', () => { cover = null; updateCover(); markDirty(); message('Обложка будет удалена при сохранении MP3.'); });
form.addEventListener('input', () => markDirty());
form.addEventListener('submit', event => { event.preventDefault(); save(); });
$('reset').addEventListener('click', () => { restore(); message('Изменения сброшены к последней сохранённой версии.'); });
$('preview').addEventListener('click', () => {
  if (!source || !form.reportValidity()) return;
  const data = values();
  $('preview-title').textContent = data.title || file.name.replace(/\.mp3$/i, '');
  $('preview-artist').textContent = data.artist || 'Исполнитель не указан';
  const details = $('preview-details');
  details.replaceChildren();
  for (const [key, label] of [['album', 'Альбом'], ['year', 'Год'], ['genre', 'Жанр'], ['comment', 'Заметка']]) {
    if (!data[key]) continue;
    const dt = document.createElement('dt'), dd = document.createElement('dd');
    dt.textContent = label; dd.textContent = data[key]; details.append(dt, dd);
  }
  $('preview-dialog').showModal();
});
$('preview-save').addEventListener('click', save);
$('close-preview').addEventListener('click', () => $('preview-dialog').close());
$('preview-dialog').addEventListener('close', () => $('audio-player').pause());
for (const button of document.querySelectorAll('.help')) button.addEventListener('click', () => message(button.title));
window.addEventListener('beforeunload', event => { if (dirty) { event.preventDefault(); event.returnValue = ''; } });
