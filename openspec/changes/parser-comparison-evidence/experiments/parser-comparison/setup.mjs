import { execFileSync, spawnSync } from 'node:child_process';
import { mkdirSync, readFileSync, writeFileSync, appendFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import path from 'node:path';
import { repairRunner } from './repair-runner.mjs';
const root = path.dirname(fileURLToPath(import.meta.url));
const sources = JSON.parse(readFileSync(path.join(root, 'sources.json')));
for (const [name, source] of Object.entries(sources)) {
  const repo = execFileSync('ghq', ['list', '--full-path', '--exact', source.repo], {encoding:'utf8'}).trim();
  if (!repo) throw Error(`Missing ghq checkout: ${source.repo}`);
  const dest = path.join(root, 'node_modules', 'sources', name);
  mkdirSync(dest, {recursive:true});
  const archive = execFileSync('git', ['-C', repo, 'archive', source.revision, ...(source.paths || [])], {maxBuffer:128*1024*1024});
  const unpack = spawnSync('tar', ['-x', '-C', dest], {input:archive, stdio:['pipe','inherit','inherit']});
  if (unpack.status !== 0) throw Error(`Cannot unpack ${name}`);
  if (name === 'lsdoc') {
    repairRunner(path.join(dest, 'tools/graph-check.mjs'));
    appendFileSync(path.join(dest, 'Cargo.toml'), '\n[workspace]\n');
    const build = spawnSync('cargo', ['build','--locked','--release','--bin','lsdoc-parse'], {cwd:dest, stdio:'inherit'});
    if (build.status !== 0) throw Error('lsdoc build failed');
  }
}
writeFileSync(path.join(root, 'nbb.edn'), '{:paths ["node_modules/sources/logseq/deps/graph-parser/src" "node_modules/sources/logseq/deps/db/src" "node_modules/sources/logseq/deps/common/src" "node_modules/sources/validator/src"]}\n');
