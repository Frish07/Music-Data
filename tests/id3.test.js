import test from 'node:test';
import assert from 'node:assert/strict';
import { readMetadata, writeMetadata, validateCover } from '../src/id3.js';

const audio = new Uint8Array(2500);
audio.set([255, 251, 144, 100]);
for (let i = 4; i < audio.length; i++) audio[i] = i % 251;
const metadata = { title: 'Ночной город 🎵', artist: 'Исполнитель', album: 'Первый альбом', year: '2026', genre: 'Indie', comment: 'Заметка\nВторая строка' };
const png = new Uint8Array([137, 80, 78, 71, 13, 10, 26, 10, 0, 1, 2]);
for (const version of [3, 4]) {
  test(`ID3v2.${version}: Unicode, cover and audio round-trip`, () => {
    const source = { ...readMetadata(audio), version };
    const output = writeMetadata(source, metadata, { bytes: png, mime: 'image/png' });
    const parsed = readMetadata(output);
    assert.deepEqual(parsed.metadata, metadata);
    assert.equal(parsed.version, version);
    assert.deepEqual(parsed.cover.bytes, png);
    assert.deepEqual(output.subarray(parsed.audioStart), audio);
    const again = readMetadata(writeMetadata(parsed, { ...metadata, title: 'Новое название' }, null));
    assert.equal(again.metadata.title, 'Новое название');
    assert.equal(again.cover, null);
    assert.equal(again.frames.filter(f => f.id === 'TIT2').length, 1);
  });
  test(`ID3v2.${version}: unrelated frames survive byte-for-byte`, () => {
    const source = { ...readMetadata(audio), version };
    const unknown = new Uint8Array([84, 88, 88, 88, 0, 0, 0, 5, 0, 0, 0, 107, 0, 118, 0]);
    source.frames.push({ id: 'TXXX', raw: unknown });
    const parsed = readMetadata(writeMetadata(source, metadata));
    assert.deepEqual(parsed.frames.find(f => f.id === 'TXXX').raw, unknown);
  });
}
test('legacy ID3v1 is read and replaced to avoid stale duplicate tags', () => {
  const bytes = new Uint8Array(audio.length + 128);
  bytes.set(audio);
  bytes.set(new TextEncoder().encode('TAGLegacy title'), audio.length);
  const source = readMetadata(bytes);
  assert.equal(source.metadata.title, 'Legacy title');
  const output = writeMetadata(source, metadata);
  const parsed = readMetadata(output);
  assert.deepEqual(output.subarray(parsed.audioStart), audio);
});
test('bad files, truncated tags and invalid years fail safely', () => {
  assert.throws(() => readMetadata(new Uint8Array(100)), /MP3/);
  assert.throws(() => readMetadata(new Uint8Array([73, 68, 51])), /заголовок/);
  const output = writeMetadata(readMetadata(audio), metadata);
  assert.throws(() => readMetadata(output.subarray(0, 50)), /размер/);
  assert.throws(() => writeMetadata(readMetadata(audio), { ...metadata, year: '26' }), /четырёх/);
  const flagged = output.slice(); flagged[5] = 128;
  assert.throws(() => readMetadata(flagged), /флаги/);
});
test('cover size and real file signature are checked', () => {
  assert.equal(validateCover(png), 'image/png');
  assert.equal(validateCover(new Uint8Array([255, 216, 255, 224])), 'image/jpeg');
  assert.throws(() => validateCover(new Uint8Array([1, 2, 3])), /JPG/);
  assert.throws(() => validateCover(new Uint8Array(3 * 1024 * 1024 + 1)), /3 МБ/);
});
test('clearing fields removes the corresponding frames', () => {
  const source = readMetadata(writeMetadata(readMetadata(audio), metadata));
  const empty = Object.fromEntries(Object.keys(metadata).map(k => [k, '']));
  const result = readMetadata(writeMetadata(source, empty, null));
  assert.deepEqual(result.metadata, empty);
  assert.equal(result.frames.length, 0);
});
test('read-only edited frames cannot be silently overwritten', () => {
  const bytes = writeMetadata(readMetadata(audio), metadata);
  bytes[18] = 32;
  assert.throws(() => writeMetadata(readMetadata(bytes), metadata), /только для чтения/);
});
