// One-time parser comparison on a garden whose text must stay out of this repository.
// Usage: node compare.mjs --scratch DIR [--lsdoc-repo PATH] [--logseq-repo PATH] [--lsg PATH]
// Every per-file output goes to DIR, which must lie outside this repository and the corpus.
// Only aggregate counts are written to results.json beside this script.
import { execFileSync, spawnSync } from 'node:child_process';
import { appendFileSync, copyFileSync, existsSync, mkdirSync, readFileSync, readdirSync, rmSync, writeFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { repairRunner, runnerRepair } from '../../openspec/changes/parser-comparison-evidence/experiments/parser-comparison/repair-runner.mjs';

const here = path.dirname(fileURLToPath(import.meta.url));
const repoRoot = execFileSync('git', ['-C', here, 'rev-parse', '--show-toplevel'], { encoding: 'utf8' }).trim();
const ogExperiment = path.join(repoRoot, 'openspec/changes/current-og-relationships/experiments/file-relationships');
const sources = JSON.parse(readFileSync(path.join(here, 'sources.json'), 'utf8'));

const args = process.argv.slice(2);
const opt = name => { const i = args.indexOf(name); return i >= 0 ? args[i + 1] : undefined; };
const ghq = repo => execFileSync('ghq', ['list', '--full-path', '--exact', repo], { encoding: 'utf8' }).trim();
const scratchArg = opt('--scratch');
if (!scratchArg) throw Error('Usage: node compare.mjs --scratch DIR [--lsdoc-repo PATH] [--logseq-repo PATH] [--lsg PATH]');
const scratch = path.resolve(scratchArg);
const corpusRepo = ghq(sources.corpus.repo);
if (!corpusRepo) throw Error(`Missing ghq checkout: ${sources.corpus.repo}`);
const inside = (child, parent) => { const r = path.relative(parent, child); return r === '' || (!r.startsWith('..') && !path.isAbsolute(r)); };
if (inside(scratch, repoRoot) || inside(scratch, corpusRepo)) throw Error('--scratch must be outside this repository and the corpus checkout');
const lsdocRepo = opt('--lsdoc-repo') || ghq(sources.lsdoc.repo);
const logseqRepo = opt('--logseq-repo') || ghq(sources.logseq.repo);
const lsg = opt('--lsg') || path.join(repoRoot, 'target/release/lsg');
for (const [name, dir] of [['lsdoc', lsdocRepo], ['logseq', logseqRepo]]) if (!dir) throw Error(`Missing source checkout for ${name}; pass --${name}-repo`);
if (!existsSync(lsg)) throw Error(`Missing lsg binary at ${lsg}; run cargo build --release --bin lsg or pass --lsg`);

const run = (cmd, argv, options = {}) => {
  const r = spawnSync(cmd, argv, { stdio: 'inherit', maxBuffer: 512 * 1024 * 1024, ...options });
  if (r.status !== 0) throw Error(`${cmd} ${argv.slice(0, 2).join(' ')} failed with status ${r.status}`);
  return r;
};
// Read-only extraction: git archive never changes the source checkout.
function extract(repo, revision, dest, paths = []) {
  execFileSync('git', ['-C', repo, 'cat-file', '-e', `${revision}^{commit}`]);
  rmSync(dest, { recursive: true, force: true });
  mkdirSync(dest, { recursive: true });
  const archive = execFileSync('git', ['-C', repo, 'archive', revision, ...paths], { maxBuffer: 512 * 1024 * 1024 });
  run('tar', ['-x', '-C', dest], { input: archive, stdio: ['pipe', 'inherit', 'inherit'] });
}
mkdirSync(scratch, { recursive: true });

// 1. Corpus: committed pages, journals and graph configuration at the pinned revision.
const corpusDir = path.join(scratch, 'corpus');
const corpusPaths = execFileSync('git', ['-C', corpusRepo, 'ls-tree', '--name-only', sources.corpus.revision, 'pages', 'journals', 'assets', 'logseq/config.edn'], { encoding: 'utf8' }).trim().split('\n').filter(Boolean);
extract(corpusRepo, sources.corpus.revision, corpusDir, corpusPaths);
const walk = dir => existsSync(dir) ? readdirSync(dir, { withFileTypes: true }).flatMap(e => e.isDirectory() ? walk(path.join(dir, e.name)) : /\.(md|org)$/i.test(e.name) ? [path.join(dir, e.name)] : []).sort() : [];
const files = ['pages', 'journals'].flatMap(d => walk(path.join(corpusDir, d)));
const corpus = files.map(file => {
  const id = path.relative(corpusDir, file);
  return { id, input: readFileSync(file, 'utf8'), format: /\.org$/i.test(id) ? 'org' : 'md' };
});
const hash = createHash('sha256');
for (const c of corpus) { hash.update(c.id); hash.update('\0'); hash.update(createHash('sha256').update(c.input).digest()); }
writeFileSync(path.join(scratch, 'corpus.json'), JSON.stringify(corpus));
const configText = readFileSync(path.join(corpusDir, 'logseq/config.edn'), 'utf8');
const configFlag = key => { const m = new RegExp(`^\\s*${key.replace(/[?/]/g, '\\$&')}\\s+(\\S+)`, 'm').exec(configText); return m ? m[1].replace(/}$/, '') : null; };

// 2. lsdoc at the pinned revision, with the Org reference repair used by the earlier comparison.
const lsdoc = path.join(scratch, 'lsdoc');
extract(lsdocRepo, sources.lsdoc.revision, lsdoc);
repairRunner(path.join(lsdoc, 'tools/graph-check.mjs'));
appendFileSync(path.join(lsdoc, 'Cargo.toml'), '\n[workspace]\n');
run('cargo', ['build', '--locked', '--release', '--bin', 'lsdoc-parse'], { cwd: lsdoc });
run('npm', ['ci', '--ignore-scripts', '--no-audit', '--no-fund'], { cwd: path.join(lsdoc, 'harness') });

// 3. Syntax and reference comparison: lsdoc's garden runner, fresh mldoc process per file.
const report = path.join(scratch, 'syntax-report.md');
run(process.execPath, [path.join(lsdoc, 'tools/graph-check.mjs'), corpusDir, '--mode', 'both', '--no-pathological', '--jobs', '4', '--out', report]);
const reportText = readFileSync(report, 'utf8');
const findingTypes = {};
for (const m of reportText.matchAll(/^### Finding \d+: (\S+)/gm)) findingTypes[m[1]] = (findingTypes[m[1]] || 0) + 1;
const num = re => { const m = re.exec(reportText); return m ? Number(m[1]) : null; };
const syntax = {
  runner: `lsdoc tools/graph-check.mjs, mode both, journals on; ${runnerRepair}`,
  matchedFiles: num(/^- Matched files: (\d+)/m),
  skippedOver8MiB: num(/^- Skipped files over 8 MB: (\d+)/m),
  findings: num(/^(\d+) finding\(s\)/m) ?? 0,
  findingTypes,
  bestTotalParseMs: [...reportText.matchAll(/^- Best total: ([\d.]+) ms/gm)].map(m => Number(m[1])),
};
syntax.bestTotalParseMs = { lsdoc: syntax.bestTotalParseMs[0], mldoc: syntax.bestTotalParseMs[1] };

// 4. Embedded media and publication properties, counted in both parsers' projections.
const { canon } = await import(pathToFileURL(path.join(lsdoc, 'harness/lib/compare.mjs')).href);
run(path.join(lsdoc, 'target/release/lsdoc-parse'), [path.join(scratch, 'corpus.json'), path.join(scratch, 'lsdoc.json')]);
const lsdocOut = JSON.parse(readFileSync(path.join(scratch, 'lsdoc.json'), 'utf8'));
const MEDIA_MACROS = new Set(['video', 'youtube', 'vimeo', 'bilibili', 'youtube-timestamp', 'video-timestamp', 'pdf', 'tweet', 'twitter', 'renderer']);
const PUBLICATION_KEYS = new Set(['public', 'title', 'alias', 'tags', 'exclude-from-graph-view']);
function census(projection) {
  const c = {};
  const add = k => { c[k] = (c[k] || 0) + 1; };
  const props = [];
  const keys = [];
  const visit = (v, topIndex) => {
    if (Array.isArray(v)) { v.forEach(x => visit(x)); return; }
    if (!v || typeof v !== 'object') return;
    if (v.k === 'link') {
      const target = JSON.stringify(v.url);
      const local = /assets\//.test(target);
      if (v.image) add(`media.image.${local ? 'local-asset' : /https?:/.test(target) ? 'remote' : 'other'}`);
      else if (local) add('media.asset-link');
    }
    if (v.k === 'macro') {
      const name = String(v.name || '').toLowerCase();
      if (name === 'embed') add(`media.embed.${String(v.args?.[0] || '').trim().startsWith('((') ? 'block' : 'page'}`);
      else if (MEDIA_MACROS.has(name)) add(`media.macro.${name}`);
    }
    if ((v.kind === 'raw_html' || v.k === 'inline_html') && /<(iframe|video|audio|img|embed|object)\b/i.test(v.text || '')) add('media.html');
    if (v.kind === 'properties') {
      const scope = topIndex === 0 ? 'page' : 'block';
      add(`properties.${scope}-blocks`);
      for (const [key, value] of v.props || []) {
        const k = String(key).toLowerCase();
        add(`properties.${scope}.${PUBLICATION_KEYS.has(k) ? k : 'other'}`);
        if (k === 'public') add(`properties.public=${String(value).trim().toLowerCase()}`);
        props.push(String(value).toLowerCase());
        keys.push(k);
      }
    }
    for (const [key, x] of Object.entries(v)) if (key !== 'url') visit(x);
  };
  (projection.blocks || []).forEach((b, i) => visit(b, i));
  return { counts: c, props, keys };
}
const mldocProjections = [];
const censusTotals = { lsdoc: {}, mldoc: {} };
const filesWith = {};
let censusAgree = 0;
const censusDisagreements = [];
const inputPath = path.join(scratch, 'worker-input.json');
corpus.forEach((item, i) => {
  writeFileSync(inputPath, JSON.stringify(item));
  const mldoc = JSON.parse(execFileSync(process.execPath, [path.join(here, 'mldoc-worker.mjs'), lsdoc, inputPath], { encoding: 'utf8', maxBuffer: 256 * 1024 * 1024 }));
  mldocProjections.push(mldoc);
  const a = census(canon(lsdocOut[i].projection)).counts;
  const b = census(mldoc).counts;
  for (const [k, n] of Object.entries(a)) censusTotals.lsdoc[k] = (censusTotals.lsdoc[k] || 0) + n;
  for (const [k, n] of Object.entries(b)) { censusTotals.mldoc[k] = (censusTotals.mldoc[k] || 0) + n; filesWith[k] = (filesWith[k] || 0) + 1; }
  if (JSON.stringify(canon(a)) === JSON.stringify(canon(b))) censusAgree++;
  else censusDisagreements.push({ file: item.id, lsdoc: a, mldoc: b });
});
const sortKeys = o => Object.fromEntries(Object.entries(o).sort(([x], [y]) => x.localeCompare(y)));
const constructs = {
  method: 'Counts from canonical lsdoc and mldoc projections; mldoc runs in a fresh process per file',
  filesCompared: corpus.length,
  filesWithEqualCounts: censusAgree,
  totals: { lsdoc: sortKeys(censusTotals.lsdoc), mldoc: sortKeys(censusTotals.mldoc) },
  filesContaining: sortKeys(filesWith),
  graphConfig: { allPagesPublicLegacyKey: configFlag(':all-pages-public?'), allPagesPublic: configFlag(':publishing/all-pages-public?'), fileNameFormat: configFlag(':file/name-format') },
};

// 5. Logseq OG relationships: current file-graph parser on the same notes and the corpus's own config.
const og = path.join(scratch, 'og');
mkdirSync(og, { recursive: true });
for (const f of ['package.json', 'package-lock.json']) copyFileSync(path.join(ogExperiment, f), path.join(og, f));
run('npm', ['ci', '--ignore-scripts', '--no-audit', '--no-fund'], { cwd: og });
extract(logseqRepo, sources.logseq.revision, path.join(og, 'sources'), ['deps/graph-parser', 'deps/db', 'deps/common']);
writeFileSync(path.join(og, 'nbb.edn'), `{:paths ${JSON.stringify(['graph-parser', 'db', 'common'].map(p => path.join(og, 'sources/deps', p, 'src')))}}\n`);
// nbb reads nbb.edn beside the script, so the shared graph script runs from a copy in the scratch directory.
copyFileSync(path.join(ogExperiment, 'graph.cljs'), path.join(og, 'graph.cljs'));
run(path.join(og, 'node_modules/.bin/nbb-logseq'), [path.join(og, 'graph.cljs'), corpusDir, path.join(scratch, 'corpus.json'), path.join(scratch, 'og-graph.json')], { cwd: og });
const graph = JSON.parse(readFileSync(path.join(scratch, 'og-graph.json'), 'utf8'));
const UUID = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i;
const norm = s => String(s).trim().toLowerCase();
const pageNames = new Set(graph.pages.map(p => p.name));
const pageByFile = new Map(graph.pages.filter(p => p.file).map(p => [p.file, p.name]));
const refsByPage = new Map();
for (const b of graph.blocks) {
  const set = refsByPage.get(b.page) || new Set();
  for (const r of b.refs) if (pageNames.has(r)) set.add(r);
  refsByPage.set(b.page, set);
}
const DATE = /\b\d{4}[/_-]\d{2}[/_-]\d{2}\b|\b(jan|feb|mar|apr|may|jun|jul|aug|sep|oct|nov|dec)[a-z]* \d{1,2}(st|nd|rd|th)?, \d{4}\b/;
const MEDIA_FILE = /assets\/|\.(png|jpe?g|gif|svg|webp|pdf|mp4|mov|webm|mp3|m4a|excalidraw)$/i;
const TASK_MARKERS = new Set(['todo', 'doing', 'done', 'later', 'now', 'wait', 'waiting', 'canceled', 'cancelled', 'in-progress', 'started']);
const relation = { filesCompared: 0, filesWithoutGraphPage: 0, filesWithEqualPageRefs: 0, graphOnly: {}, syntaxOnly: {} };
const relationDetails = [];
corpus.forEach((item, i) => {
  const page = pageByFile.get(item.id);
  if (!page) { relation.filesWithoutGraphPage++; return; }
  relation.filesCompared++;
  const { props, keys } = census(canon(lsdocOut[i].projection));
  const syntaxRefs = new Set(canon(lsdocOut[i].projection).refs.page.map(norm).filter(r => !UUID.test(r)));
  const graphRefs = refsByPage.get(page) || new Set();
  const all = [...syntaxRefs, ...graphRefs];
  const inProps = r => props.some(v => v.includes(r));
  const kind = (r, syntaxSide) => {
    if (syntaxSide && MEDIA_FILE.test(r)) return 'media-or-file-target';
    if (!syntaxSide && TASK_MARKERS.has(r)) return 'task-marker';
    if (!syntaxSide && keys.includes(r)) return 'property-name';
    if (!syntaxSide && UUID.test(r)) return 'block-uuid';
    if (!syntaxSide && all.some(o => o.startsWith(`${r}/`))) return 'namespace-parent';
    if (DATE.test(r)) return 'date';
    if (inProps(r)) return 'property-value';
    return 'other';
  };
  const graphOnly = [...graphRefs].filter(r => !syntaxRefs.has(r));
  const syntaxOnly = [...syntaxRefs].filter(r => !graphRefs.has(r));
  if (!graphOnly.length && !syntaxOnly.length) relation.filesWithEqualPageRefs++;
  for (const r of graphOnly) { const k = kind(r, false); relation.graphOnly[k] = (relation.graphOnly[k] || 0) + 1; }
  for (const r of syntaxOnly) { const k = kind(r, true); relation.syntaxOnly[k] = (relation.syntaxOnly[k] || 0) + 1; }
  if (graphOnly.length || syntaxOnly.length) relationDetails.push({ file: item.id, graphOnly: graphOnly.map(r => [r, kind(r, false)]), syntaxOnly: syntaxOnly.map(r => [r, kind(r, true)]) });
});
const relationships = {
  method: 'Per file: page names referenced by the file\'s blocks in the Logseq OG graph, compared with lsdoc syntax page references (lowercased, UUIDs dropped)',
  pages: graph.pages.length,
  fileBackedPages: pageByFile.size,
  blocks: graph.blocks.length,
  pagesWithPublicProperty: graph.pages.filter(p => p.properties && 'public' in p.properties).length,
  ...relation,
  graphOnly: sortKeys(relation.graphOnly),
  syntaxOnly: sortKeys(relation.syntaxOnly),
};

// 6. Gardener SDK end to end: publish every root namespace and every journal.
const roots = new Set();
corpus.forEach((item, i) => {
  if (!item.id.startsWith('pages/')) return;
  const first = canon(lsdocOut[i].projection).blocks?.[0];
  const title = first?.kind === 'properties' ? (first.props.find(([k]) => k.toLowerCase() === 'title') || [])[1] : null;
  for (const name of [title, decodeURIComponent(path.basename(item.id).replace(/\.(md|org)$/i, '')).split('___').join('/')]) {
    if (name && name.trim()) roots.add(name.trim().split('/')[0].trim());
  }
});
const site = path.join(scratch, 'site');
rmSync(site, { recursive: true, force: true });
const publish = spawnSync(lsg, ['publish', '--graph', corpusDir, '--output', site, '--filename-format', configFlag(':file/name-format') === ':triple-lowbar' ? 'triple-lowbar' : 'legacy',
  '--journals', '--format', 'json', ...[...roots].filter(Boolean).flatMap(r => ['--include', r])], { encoding: 'utf8', maxBuffer: 256 * 1024 * 1024 });
writeFileSync(path.join(scratch, 'publish-stderr.txt'), publish.stderr);
writeFileSync(path.join(scratch, 'publish-stdout.json'), publish.stdout);
const published = publish.status === 0 ? JSON.parse(publish.stdout) : {};
const count = v => Array.isArray(v) ? v.length : typeof v === 'number' ? v : null;
const diagnosticKinds = {};
for (const line of publish.stderr.split('\n').filter(Boolean)) {
  const k = /Unsupported drawer/.test(line) ? 'drawer kept as text'
    : /Unsupported (inline )?(HTML|Hiccup)/.test(line) ? 'raw HTML or Hiccup kept as text'
    : /Unsupported macro|Renderer macro omitted/.test(line) ? 'unsupported macro or renderer'
    : /Unresolved or excluded (tag|page reference)/.test(line) ? 'link to a page without a file'
    : /block reference/.test(line) ? 'unavailable block reference'
    : /Query failed/.test(line) ? 'advanced query'
    : /asset|media/i.test(line) ? 'asset or media'
    : /Journal skipped/.test(line) ? 'journal file name is not a date'
    : /page-title-format/.test(line) ? 'unsupported journal title token'
    : /not in a git repository/.test(line) ? 'no git history in extracted copy'
    : /Favorite/.test(line) ? 'favorite names no published page' : 'other';
  diagnosticKinds[k] = (diagnosticKinds[k] || 0) + 1;
}
const sdk = {
  command: 'lsg publish --journals with every root namespace included',
  exitStatus: publish.status,
  pages: count(published.pages), journals: count(published.journals), assets: count(published.assets),
  withheldPages: count(published.withheld_pages), skippedJournals: count(published.skipped_journals),
  diagnosticCount: published.diagnostic_count ?? null,
  diagnosticLinesByKind: sortKeys(diagnosticKinds),
};

writeFileSync(path.join(scratch, 'details.json'), JSON.stringify({ censusDisagreements, relationDetails }, null, 2));
const byDir = {};
for (const c of corpus) { const k = `${c.id.split('/')[0]}.${c.format}`; byDir[k] = (byDir[k] || 0) + 1; }
const results = {
  note: 'Aggregate counts only. Per-file output stays in the scratch directory and is not committed.',
  sources,
  runtime: { node: process.version, mldocSyntax: '1.5.9 (lsdoc harness lock)', mldocGraph: JSON.parse(readFileSync(path.join(og, 'node_modules/mldoc/package.json'), 'utf8')).version,
    nbbLogseq: JSON.parse(readFileSync(path.join(og, 'node_modules/@logseq/nbb-logseq/package.json'), 'utf8')).version },
  corpus: { files: corpus.length, emptyFiles: corpus.filter(c => !c.input.trim()).length, byFolderAndFormat: sortKeys(byDir), sha256: hash.digest('hex'), selection: 'Tracked Markdown and Org files under pages/ and journals/ at the pinned revision' },
  syntax, constructs, relationships, sdk,
};
writeFileSync(opt('--results') || path.join(here, 'results.json'), JSON.stringify(results, null, 2) + '\n');
console.log(`Wrote aggregate results. Per-file output is in ${scratch}.`);
