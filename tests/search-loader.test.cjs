const test = require('node:test');
const assert = require('node:assert/strict');
const path = require('node:path');
const { pathToFileURL } = require('node:url');
const loaded = import(pathToFileURL(path.join(__dirname, '../crates/logseq-gardener-sdk/src/site/search-ui.mjs')));
const revision = '0123456789abcdef';
const origin = 'https://garden.test';
const base = origin + '/published/nested/page/';
const manifestURL = origin + '/published/lsg-search-manifest-' + revision + '.json';
const link = (rev = revision, href = manifestURL) => ({ getAttribute: key => key === 'href' ? href : rev });
function fixture() {
  const source = JSON.stringify({ format_version: 1, revision, pages: [], content: [], note: 'é東京' });
  const manifest = { format_version: 1, revision, corpus: { href: `lsg-search-${revision}.json`, records: 0, bytes: Buffer.byteLength(source) }, engine: `lsg-search-engine-${revision}.mjs` };
  const calls = [], imports = [];
  const engine = { prepareIndex: (m, c, r) => ({ m, c, r }) };
  const fetchFile = async (url, options) => { calls.push({url, options}); return {ok: true, url, text: async () => url === manifestURL ? JSON.stringify(manifest) : source}; };
  const importEngine = async url => { imports.push(url); return engine; };
  return { source, manifest, calls, imports, fetchFile, importEngine };
}
test('no eager work; correct manifest-relative URLs; one cached publication load', async () => {
  const { createSearchLoader, publicationFromLink } = await loaded; const f = fixture();
  const loader = createSearchLoader(f);
  loader.setPublication(publicationFromLink(link(), base, origin));
  assert.deepEqual(f.calls, []); assert.deepEqual(f.imports, []);
  const [first, concurrent] = await Promise.all([loader.load(), loader.load()]);
  assert.equal(first, concurrent); assert.equal(await loader.load(), first);
  assert.deepEqual(f.calls.map(c => c.url), [manifestURL, origin + `/published/lsg-search-${revision}.json`]);
  assert.deepEqual(f.imports, [origin + `/published/lsg-search-engine-${revision}.mjs`]);
  assert.ok(f.calls.every(c => c.options.redirect === 'error'));
});
test('bad metadata, remote paths, versions and descriptor mismatch fail before evaluation', async () => {
  const { createSearchLoader, publicationFromLink } = await loaded;
  for (const bad of [link('bad'), link(revision, 'https://evil.test/' + `lsg-search-manifest-${revision}.json`), link(revision, manifestURL + '?other'), link(revision, 'javascript:alert(1)')]) {
    assert.throws(() => publicationFromLink(bad, base, origin));
  }
  for (const mutate of [f => f.manifest.format_version = 2, f => f.manifest.revision = 'fedcba9876543210', f => f.manifest.corpus.bytes++, f => f.manifest.engine = 'https://evil.test/engine.mjs', f => f.manifest.corpus.href = '../other.json']) {
    const f = fixture(); mutate(f); const loader = createSearchLoader(f); loader.setPublication(publicationFromLink(link(), base, origin));
    await assert.rejects(loader.load()); assert.deepEqual(f.imports, []);
  }
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
  await assert.rejects(loader.load()); bad = false; assert.ok(await loader.load());
  loader.setPublication(null); loader.setPublication(publicationFromLink(link(), base, origin));
  f.fetchFile = async () => ({ ok: true, text: async () => '{broken' });
  // A separate loader uses the altered dependency object.
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
  const current = await loader.load(); assert.equal(current.index.r, revision);
});
