import { mkdir, copyFile, cp, rm } from 'node:fs/promises';
const dist = new URL('../dist/', import.meta.url);
await rm(dist, { recursive: true, force: true });
await mkdir(dist, { recursive: true });
for (const name of ['index.html', 'styles.css', 'favicon.svg']) await copyFile(new URL(`../${name}`, import.meta.url), new URL(name, dist));
await cp(new URL('../src/', import.meta.url), new URL('src/', dist), { recursive: true });
console.log('Built dist/');
