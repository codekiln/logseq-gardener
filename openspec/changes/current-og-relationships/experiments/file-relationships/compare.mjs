import { execFileSync } from 'node:child_process';
import { readFileSync, writeFileSync, readdirSync, mkdirSync, rmSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { fileURLToPath } from 'node:url';
import path from 'node:path';

const root = path.dirname(fileURLToPath(import.meta.url));
process.chdir(root);
if (process.argv.length > 3 || (process.argv.length === 3 && process.argv[2] !== '--update')) {
  throw Error('Usage: node compare.mjs [--update]');
}
const sources = JSON.parse(readFileSync('sources.json'));
const runtime = {
  mldoc: JSON.parse(readFileSync('node_modules/mldoc/package.json')).version,
  nbb: JSON.parse(readFileSync('node_modules/@logseq/nbb-logseq/package.json')).version,
};
const baseline = path.resolve(root, '../../../parser-comparison-baseline/experiments/mldoc-baseline/fixtures/garden');
const evidence = path.resolve(root, '../../../parser-comparison-evidence/experiments/parser-comparison');
function walk(dir) {
  return readdirSync(dir, { withFileTypes: true }).flatMap(e => e.isDirectory()
    ? walk(path.join(dir, e.name)) : e.name.endsWith('.md') ? [path.join(dir, e.name)] : []).sort();
}
const corpus = [baseline, path.join(evidence, 'fixtures')].flatMap(dir => walk(dir)
  .map(file => ({ id: path.relative(dir, file), input: readFileSync(file, 'utf8') })))
  .sort((a, b) => a.id.localeCompare(b.id, 'en'));
const inputs = corpus.map(({ id, input }) => ({ file: id, sha256: createHash('sha256').update(input).digest('hex') }));
mkdirSync('node_modules/results', { recursive: true });
writeFileSync('node_modules/results/corpus.json', JSON.stringify(corpus));
function canonical(value) {
  if (Array.isArray(value)) return value.map(canonical);
  if (value && typeof value === 'object') return Object.fromEntries(Object.keys(value).sort().map(k => [k, canonical(value[k])]));
  return value;
}
function equal(a, b) { return JSON.stringify(canonical(a)) === JSON.stringify(canonical(b)); }
const prior = JSON.parse(readFileSync(path.join(evidence, 'fixtures.json')));
const configurations = [];
for (const [name, config, priorKey] of [
  ['default', '{}', 'graph'],
  ['triple-lowbar', '{:file/name-format :triple-lowbar}', 'configuredGraph'],
]) {
  const garden = path.resolve(root, 'node_modules/results', name);
  rmSync(garden, { recursive: true, force: true });
  mkdirSync(path.join(garden, 'logseq'), { recursive: true });
  writeFileSync(path.join(garden, 'logseq/config.edn'), config);
  for (const item of corpus) {
    mkdirSync(path.dirname(path.join(garden, item.id)), { recursive: true });
    writeFileSync(path.join(garden, item.id), item.input);
  }
  const graphs = {};
  for (const parser of Object.keys(sources)) {
    const paths = ['graph-parser', 'db', 'common'].map(p => `node_modules/sources/${parser}/deps/${p}/src`);
    writeFileSync('nbb.edn', `{:paths ${JSON.stringify(paths)}}\n`);
    const output = `node_modules/results/${name}-${parser}.json`;
    execFileSync('node_modules/.bin/nbb-logseq', ['graph.cljs', garden, 'node_modules/results/corpus.json', output], { stdio: 'inherit' });
    graphs[parser] = JSON.parse(readFileSync(output));
  }
  const historicalPriorFields = {
    pages: graphs.historical.pages.map(({ 'journal?': journal, 'journal-day': day, ...page }) => page),
    blocks: graphs.historical.blocks,
  };
  configurations.push({ name, config, equal: equal(graphs.current, graphs.historical),
    historicalMatchesPriorSnapshot: equal(historicalPriorFields, { pages: prior[priorKey].pages, blocks: prior[priorKey].blocks }),
    ...graphs });
}
const result = JSON.stringify(canonical({ sources, runtime, inputs, configurations }), null, 2) + '\n';
if (process.argv[2] === '--update') {
  writeFileSync('relationships.json', result);
  console.log('Updated current and historical relationship evidence.');
} else if (result !== readFileSync('relationships.json', 'utf8')) {
  throw Error('Relationship evidence differs. Run mise run parser:og-update and review the input and relationship changes.');
} else console.log('Current and historical relationship evidence matches.');
