import { execFileSync, spawnSync } from 'node:child_process';
import { readFileSync, mkdirSync, rmSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const root = path.dirname(fileURLToPath(import.meta.url));
const sources = JSON.parse(readFileSync(path.join(root, 'sources.json')));
for (const [name, source] of Object.entries(sources)) {
  const repo = execFileSync('ghq', ['list', '--full-path', '--exact', source.repo], { encoding: 'utf8' }).trim();
  if (!repo) throw Error(`Missing ghq checkout: ${source.repo}`);
  const dest = path.join(root, 'node_modules/sources', name);
  rmSync(dest, { recursive: true, force: true });
  mkdirSync(dest, { recursive: true });
  const archive = execFileSync('git', ['-C', repo, 'archive', source.revision,
    'deps/graph-parser', 'deps/db', 'deps/common'], { maxBuffer: 128 * 1024 * 1024 });
  if (spawnSync('tar', ['-x', '-C', dest], { input: archive }).status !== 0) {
    throw Error(`Cannot extract ${name} source`);
  }
}
