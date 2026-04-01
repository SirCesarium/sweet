import * as fs from 'node:fs';
import * as path from 'node:path';
import { get } from 'node:https';
import { chmodSync, existsSync, mkdirSync } from 'node:fs';

const VERSION = '3.1.1';
const REPO = 'SirCesarium/sweet';
const BIN_DIR = path.join(__dirname, '..', 'bin');

const targets = [
  { src: `sweet-lsp_x86_64-unknown-linux-gnu`, dest: 'sweet-lsp-linux' },
  { src: `sweet-lsp_aarch64-unknown-linux-gnu`, dest: 'sweet-lsp-linux-arm64' },
  { src: `sweet-lsp_x86_64-pc-windows-msvc.exe`, dest: 'sweet-lsp-win.exe' },
  { src: `sweet-lsp_x86_64-apple-darwin`, dest: 'sweet-lsp-macos-arm64' },
  { src: `sweet-lsp_aarch64-apple-darwin`, dest: 'sweet-lsp-macos' },
];

async function download(url: string, dest: string): Promise<void> {
  return new Promise((resolve, reject) => {
    get(url, (res) => {
      if (res.statusCode === 302 || res.statusCode === 301) {
        download(res.headers.location!, dest).then(resolve).catch(reject);
      } else if (res.statusCode === 200) {
        const file = fs.createWriteStream(dest);
        res.pipe(file);
        file.on('finish', () => {
          file.close();
          if (!dest.endsWith('.exe')) chmodSync(dest, '755');
          resolve();
        });
      } else {
        reject(new Error(`Failed to download: ${res.statusCode}`));
      }
    }).on('error', reject);
  });
}

async function main() {
  if (!existsSync(BIN_DIR)) mkdirSync(BIN_DIR);

  for (const target of targets) {
    const url = `https://github.com/${REPO}/releases/download/v${VERSION}/${target.src}`;
    const destPath = path.join(BIN_DIR, target.dest);
    console.log(`Downloading ${target.src}...`);
    await download(url, destPath);
  }
  console.log('All binaries downloaded successfully.');
}

main().catch(console.error);
