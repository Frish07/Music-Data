// ID3v2.3 / ID3v2.4. Audio and unedited frames are copied without re-encoding.
const ascii = new TextDecoder('windows-1252');
const utf8 = new TextEncoder();
const ids = { title: 'TIT2', artist: 'TPE1', album: 'TALB', genre: 'TCON' };
export const MAX_FILE_SIZE = 150 * 1024 * 1024;
export const MAX_COVER_SIZE = 3 * 1024 * 1024;
const concat = (...parts) => {
  const out = new Uint8Array(parts.reduce((n, p) => n + p.length, 0));
  let offset = 0;
  for (const part of parts) { out.set(part, offset); offset += part.length; }
  return out;
};
const read32 = (b, p) => new DataView(b.buffer, b.byteOffset, b.byteLength).getUint32(p);
const readSync = (b, p) => {
  if (b.subarray(p, p + 4).some(v => v > 127)) throw new Error('Некорректный размер ID3-тега.');
  return b[p] * 2097152 + b[p + 1] * 16384 + b[p + 2] * 128 + b[p + 3];
};
const sizeBytes = (size, sync) => sync
  ? new Uint8Array([(size >>> 21) & 127, (size >>> 14) & 127, (size >>> 7) & 127, size & 127])
  : new Uint8Array([(size >>> 24) & 255, (size >>> 16) & 255, (size >>> 8) & 255, size & 255]);
function decode(bytes, encoding) {
  const name = ['windows-1252', 'utf-16le', 'utf-16be', 'utf-8'][encoding];
  if (!name) return '';
  if (encoding === 1 && bytes[0] === 254 && bytes[1] === 255) return decode(bytes.subarray(2), 2);
  if (encoding === 1 && bytes[0] === 255 && bytes[1] === 254) bytes = bytes.subarray(2);
  return new TextDecoder(name).decode(bytes).replace(/\0+$/g, '');
}
function terminator(bytes, offset, encoding) {
  const step = encoding === 1 || encoding === 2 ? 2 : 1;
  for (let i = offset; i < bytes.length; i += step) {
    if (bytes[i] === 0 && (step === 1 || bytes[i + 1] === 0)) return i + step;
  }
  return bytes.length;
}
function encodeText(text, version) {
  if (version === 4) return concat(new Uint8Array([3]), utf8.encode(text));
  const data = new Uint8Array(text.length * 2 + 3);
  data.set([1, 255, 254]);
  for (let i = 0; i < text.length; i++) { data[3 + i * 2] = text.charCodeAt(i) & 255; data[4 + i * 2] = text.charCodeAt(i) >>> 8; }
  return data;
}
export function validateCover(bytes) {
  if (bytes.length > MAX_COVER_SIZE) throw new Error('Обложка должна быть не больше 3 МБ.');
  if (bytes.length >= 8 && [137, 80, 78, 71, 13, 10, 26, 10].every((v, i) => bytes[i] === v)) return 'image/png';
  if (bytes.length >= 3 && bytes[0] === 255 && bytes[1] === 216 && bytes[2] === 255) return 'image/jpeg';
  throw new Error('Выберите изображение в формате JPG или PNG.');
}
export function readMetadata(input) {
  const bytes = input instanceof Uint8Array ? input : new Uint8Array(input);
  if (bytes.length > MAX_FILE_SIZE) throw new Error('Максимальный размер MP3 — 150 МБ.');
  const metadata = { title: '', artist: '', album: '', year: '', genre: '', comment: '' };
  let version = 3, audioStart = 0, cover = null;
  const frames = [];
  if (ascii.decode(bytes.subarray(0, 3)) === 'ID3') {
    if (bytes.length < 10) throw new Error('Повреждён заголовок ID3.');
    version = bytes[3];
    if (version !== 3 && version !== 4) throw new Error('Поддерживаются теги ID3v2.3 и ID3v2.4. Конвертируйте старые теги ID3v2.2.');
    const flags = bytes[5];
    if (flags & (version === 4 ? 0xaf : 0xbf)) throw new Error('Этот MP3 содержит неподдерживаемые флаги ID3. Исходный файл не изменён.');
    const end = 10 + readSync(bytes, 6);
    audioStart = end + (version === 4 && flags & 16 ? 10 : 0);
    if (audioStart > bytes.length) throw new Error('Повреждён размер ID3-тега.');
    let pos = 10;
    if (flags & 64) {
      if (pos + 4 > end) throw new Error('Повреждён расширенный заголовок ID3.');
      const extendedSize = version === 4 ? readSync(bytes, pos) : read32(bytes, pos) + 4;
      if (extendedSize < (version === 4 ? 6 : 10) || pos + extendedSize > end) throw new Error('Повреждён расширенный заголовок ID3.');
      pos += extendedSize;
    }
    while (pos < end && bytes[pos] !== 0) {
      if (pos + 10 > end) throw new Error('Повреждён заголовок кадра ID3.');
      const id = ascii.decode(bytes.subarray(pos, pos + 4));
      if (!/^[A-Z0-9]{4}$/.test(id)) throw new Error('Некорректный кадр ID3.');
      const size = version === 4 ? readSync(bytes, pos + 4) : read32(bytes, pos + 4);
      if (pos + 10 + size > end) throw new Error('Повреждены данные кадра ID3.');
      // Compression, encryption, grouping and per-frame unsynchronisation need separate decoding.
      if (bytes[pos + 9] !== 0) throw new Error('Сжатые, зашифрованные и unsynchronised ID3-теги пока не поддерживаются.');
      const data = bytes.slice(pos + 10, pos + 10 + size);
      frames.push({ id, raw: bytes.slice(pos, pos + 10 + size) });
      for (const [key, frameId] of Object.entries(ids)) if (id === frameId) metadata[key] = decode(data.subarray(1), data[0]);
      if (id === 'TYER' || id === 'TDRC') metadata.year = decode(data.subarray(1), data[0]).slice(0, 4);
      if (id === 'COMM' && data.length >= 4) metadata.comment = decode(data.subarray(terminator(data, 4, data[0])), data[0]);
      if (id === 'APIC' && data.length > 4) {
        const mimeEnd = data.indexOf(0, 1);
        if (mimeEnd > 0 && mimeEnd + 1 < data.length) {
          const imageStart = terminator(data, mimeEnd + 2, data[0]);
          const mime = ascii.decode(data.subarray(1, mimeEnd));
          if (['image/jpeg', 'image/png'].includes(mime) && (!cover || data[mimeEnd + 1] === 3)) cover = { mime, bytes: data.slice(imageStart) };
        }
      }
      pos += 10 + size;
    }
  }
  let audioEnd = bytes.length;
  if (bytes.length >= 128 && ascii.decode(bytes.subarray(bytes.length - 128, bytes.length - 125)) === 'TAG') {
    const tag = bytes.subarray(bytes.length - 128);
    for (const [key, start, end] of [['title', 3, 33], ['artist', 33, 63], ['album', 63, 93], ['year', 93, 97], ['comment', 97, tag[125] === 0 ? 125 : 127]]) {
      if (!metadata[key]) metadata[key] = decode(tag.subarray(start, end), 0).trim();
    }
    audioEnd -= 128;
  }
  // Check the first audio bytes for a legal MPEG audio frame header.
  let hasAudio = false;
  for (let p = audioStart; p < Math.min(audioStart + 8192, audioEnd - 3); p++) {
    if (bytes[p] === 255 && (bytes[p + 1] & 224) === 224 && (bytes[p + 1] & 24) !== 8 && (bytes[p + 1] & 6) !== 0 && (bytes[p + 2] & 240) !== 240 && (bytes[p + 2] & 240) !== 0 && (bytes[p + 2] & 12) !== 12) { hasAudio = true; break; }
  }
  if (!hasAudio) throw new Error('Не удалось найти MP3-аудио. Выберите корректный файл .mp3.');
  return { bytes, metadata, cover, version, frames, audioStart, audioEnd };
}
export function writeMetadata(source, metadata, cover = source.cover) {
  const version = source.version;
  if (metadata.year && !/^\d{4}$/.test(metadata.year)) throw new Error('Укажите год из четырёх цифр.');
  const replaced = new Set([...Object.values(ids), 'TYER', 'TDRC', 'COMM', 'APIC']);
  // Read-only frames cannot safely be changed or discarded.
  if (source.frames.some(f => replaced.has(f.id) && (f.raw[8] & (version === 4 ? 16 : 32)))) throw new Error('Изменяемый тег помечен как доступный только для чтения.');
  const frames = source.frames.filter(f => !replaced.has(f.id)).map(f => f.raw);
  const frame = (id, data) => concat(utf8.encode(id), sizeBytes(data.length, version === 4), new Uint8Array(2), data);
  for (const [key, id] of Object.entries(ids)) if (metadata[key]) frames.push(frame(id, encodeText(metadata[key], version)));
  if (metadata.year) frames.push(frame(version === 4 ? 'TDRC' : 'TYER', encodeText(metadata.year, version)));
  if (metadata.comment) {
    const text = encodeText(metadata.comment, version);
    frames.push(frame('COMM', concat(text.subarray(0, 1), utf8.encode('eng'), new Uint8Array(version === 4 ? 1 : 2), text.subarray(1))));
  }
  if (cover) {
    const mime = validateCover(cover.bytes);
    frames.push(frame('APIC', concat(new Uint8Array([0]), utf8.encode(mime), new Uint8Array([0, 3, 0]), cover.bytes)));
  }
  const body = concat(...frames, new Uint8Array(1024));
  return concat(utf8.encode('ID3'), new Uint8Array([version, 0, 0]), sizeBytes(body.length, true), body, source.bytes.subarray(source.audioStart, source.audioEnd));
}
