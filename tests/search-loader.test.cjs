const test = require('node:test');
const assert = require('node:assert/strict');
const path = require('node:path');
const { readFileSync } = require('node:fs');
const { pathToFileURL } = require('node:url');
const loaded = import(pathToFileURL(path.join(__dirname, '../crates/logseq-gardener-sdk/src/site/search-ui.mjs')));
const site = path.join(__dirname, '../crates/logseq-gardener-sdk/src/site');
const engineSource = readFileSync(path.join(site, 'search.mjs'), 'utf8')
  .replace('// __LSG_FUZZY__', () => readFileSync(path.join(site, 'fuzzy.js'), 'utf8'));
const realEngine = import('data:text/javascript;base64,' + Buffer.from(engineSource).toString('base64'));
// The shared fixture's published files, served beside a nested page.
const { files, manifest: manifestName } = JSON.parse(readFileSync(path.join(__dirname, 'search-fixture.json'), 'utf8')).published;
const revision = JSON.parse(files[manifestName]).revision;
const origin = 'https://garden.test';
const base = origin + '/published/nested/page/';
const published = origin + '/published/';
const manifestURL = published + manifestName;
const link = (rev = revision, href = manifestURL) => ({ getAttribute: key => key === 'href' ? href : rev });
function fixture() {
  const served = { ...files };
  const calls = [], imports = [];
  const fetchFile = async (url, options) => {
    calls.push({ url, options });
    const name = url.slice(published.length);
    return url.startsWith(published) && name in served ? { ok: true, url, text: async () => served[name] } : { ok: false, url, text: async () => 'missing' };
  };
  const importEngine = async url => { imports.push(url); return realEngine; };
  return { served, calls, imports, fetchFile, importEngine };
}
const names = calls => calls.map(c => c.url.slice(published.length).replace(/-[a-f0-9]{16}\.json$/u, ''));

test('no eager work; manifest, engine and title index load once; shards per query, chunks per row', async () => {
  const { createSearchLoader, publicationFromLink } = await loaded; const f = fixture();
  const loader = createSearchLoader(f);
  loader.setPublication(publicationFromLink(link(), base, origin));
  assert.deepEqual(f.calls, []); assert.deepEqual(f.imports, []);
  const [first, concurrent] = await Promise.all([loader.load(), loader.load()]);
  assert.equal(first, concurrent); assert.equal(await loader.load(), first);
  assert.deepEqual(names(f.calls), ['lsg-search-manifest', 'lsg-search-titles']);
  assert.deepEqual(f.imports, [published + 'lsg-search-engine-0000000000000000.mjs']);
  await Promise.all([loader.shards('tm'), loader.shards('tmux')]);
  await loader.shards('tmux');
  assert.deepEqual(names(f.calls).slice(2), ['lsg-search-shard'], 'tm then tmux fetch the tm shard once');
  const outcome = first.engine.query(first.index, 'tmux');
  await loader.chunks(outcome.blocks.results.map(r => r.block));
  assert.ok(outcome.blocks.results.every(r => first.engine.blockRow(first.index, r.block)));
  assert.ok(f.calls.every(c => c.options.redirect === 'error'));
});

test('bad metadata, remote paths, version 1 and digest mismatch fail before evaluation', async () => {
  const { createSearchLoader, publicationFromLink } = await loaded;
  for (const bad of [link('bad'), link(revision, 'https://evil.test/' + manifestName), link(revision, manifestURL + '?other'), link(revision, 'javascript:alert(1)')]) {
    assert.throws(() => publicationFromLink(bad, base, origin));
  }
  const manifest = () => JSON.parse(files[manifestName]);
  for (const mutate of [
    s => s[manifestName] = JSON.stringify({ ...manifest(), format_version: 1 }),
    s => s[manifestName] = JSON.stringify({ ...manifest(), revision: 'fedcba9876543210' }),
    s => s[manifestName] = JSON.stringify({ ...manifest(), engine: 'https://evil.test/engine.mjs' }),
    s => { const name = Object.keys(s).find(n => n.startsWith('lsg-search-titles-')); s[name] += ' '; },
  ]) {
    const f = fixture(); mutate(f.served); const loader = createSearchLoader(f); loader.setPublication(publicationFromLink(link(), base, origin));
    await assert.rejects(loader.load());
  }
  const f = fixture();
  const shard = Object.keys(f.served).find(n => n.startsWith('lsg-search-shard-') && f.served[n].includes('"tmux"'));
  f.served[shard] = f.served[shard].replace('tmux', 'tmuy');
  const loader = createSearchLoader(f); loader.setPublication(publicationFromLink(link(), base, origin));
  await loader.load();
  await assert.rejects(loader.shards('tmux'), /publication/);
});

test('close aborts and stale completions cannot populate a later publication', async () => {
  const { createSearchLoader, publicationFromLink } = await loaded;
  const f = fixture(); let release;
  const deferred = new Promise(resolve => release = resolve);
  const normal = f.fetchFile;
  f.fetchFile = async (url, options) => { const result = await normal(url, options); await deferred; return result; };
  const loader = createSearchLoader(f); loader.setPublication(publicationFromLink(link(), base, origin));
  const old = loader.load(); loader.cancel();
  assert.equal(f.calls[0].options.signal.aborted, true);
  release(); await assert.rejects(old, error => error.name === 'AbortError');
  const fresh = await loader.load(); assert.equal(fresh.publication.revision, revision);
  assert.equal(loader.setPublication(null), true); await assert.rejects(loader.load(), /unavailable/);
});

test('failed request can retry and corrupt JSON never reaches engine', async () => {
  const { createSearchLoader, publicationFromLink } = await loaded;
  const f = fixture(); let bad = true; const normal = f.fetchFile;
  f.fetchFile = async (...args) => bad ? { ok: false, text: async () => 'missing' } : normal(...args);
  const loader = createSearchLoader(f); loader.setPublication(publicationFromLink(link(), base, origin));
  await assert.rejects(loader.load()); bad = false; const data = await loader.load();
  bad = true; await assert.rejects(loader.shards('tmux'));
  bad = false; await loader.shards('tmux'); assert.equal(data.engine.missingShards(data.index, 'tmux').length, 0);
  f.fetchFile = async url => ({ ok: true, url, text: async () => '{broken' });
  const corrupt = createSearchLoader(f); corrupt.setPublication(publicationFromLink(link(), base, origin));
  await assert.rejects(corrupt.load());
});

test('changed revision invalidates data even while an engine import completes', async () => {
  const { createSearchLoader, publicationFromLink } = await loaded;
  const f = fixture(); let release, started;
  const ready = new Promise(resolve => started = resolve);
  const gate = new Promise(resolve => release = resolve);
  const normal = f.importEngine;
  f.importEngine = async url => { started(); await gate; return normal(url); };
  const loader = createSearchLoader(f);
  loader.setPublication(publicationFromLink(link(), base, origin));
  const old = loader.load(); await ready;
  const changed = 'fedcba9876543210';
  assert.equal(loader.setPublication(publicationFromLink(link(changed, manifestURL.replace(revision, changed)), base, origin)), true);
  release(); await assert.rejects(old, error => error.name === 'AbortError');
  assert.equal(loader.setPublication(publicationFromLink(link(), base, origin)), true);
  const current = await loader.load(); assert.equal(current.index.manifest.revision, revision);
});
