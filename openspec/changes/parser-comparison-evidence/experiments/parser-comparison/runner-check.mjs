import { execFileSync } from 'node:child_process';
import { readFileSync, mkdirSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

const root = path.dirname(fileURLToPath(import.meta.url));
process.chdir(root);
mkdirSync('node_modules/results', { recursive: true });
const report = 'node_modules/results/runner-report.md';
execFileSync(process.execPath, [
  'node_modules/sources/lsdoc/tools/graph-check.mjs',
  path.join(root, 'runner-fixtures'),
  '--mode', 'diff', '--no-pathological', '--jobs', '1', '--out', report,
], { stdio: 'inherit' });
const text = readFileSync(report, 'utf8');
if (!text.includes('- Matched files: 2\n') ||
    !text.includes('No divergences, crashes, or timeouts found.')) {
  throw Error('Public-garden runner disagrees on the Markdown/Org link fixtures. Inspect node_modules/results/runner-report.md.');
}
console.log('Public-garden runner agrees on Markdown and Org link references.');
