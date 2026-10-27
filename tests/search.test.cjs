const test = require('node:test');
const assert = require('node:assert/strict');
const { readFileSync } = require('node:fs');
const path = require('node:path');
const site = path.join(__dirname, '../crates/logseq-gardener-sdk/src/site');
// The engine as the publisher writes it: search.mjs with the shared fuzzy score spliced in.
const source = readFileSync(path.join(site, 'search.mjs'), 'utf8')
  .replace('// __LSG_FUZZY__', () => readFileSync(path.join(site, 'fuzzy.js'), 'utf8'));
const modulePromise = import('data:text/javascript;base64,' + Buffer.from(source).toString('base64'));
const fixture = JSON.parse(readFileSync(path.join(__dirname, 'search-fixture.json'), 'utf8'));
const { files, manifest: manifestName } = fixture.published;

async function load() {
  const engine = await modulePromise;
  const manifest = engine.prepareManifest(JSON.parse(files[manifestName]), JSON.parse(files[manifestName]).revision);
  const index = engine.createIndex(manifest, files[engine.titlesFile(manifest)]);
  const fetchShards = input => { for (const key of engine.missingShards(index, input)) engine.addShard(index, key, files[engine.shardFile(manifest, key)]); };
  const fetchChunks = blocks => { for (const n of engine.missingChunks(index, blocks)) engine.addChunk(index, n, files[engine.chunkFile(manifest, n)]); };
  return { engine, manifest, index, fetchShards, fetchChunks };
}

test('the shared fixture returns the same results as the Rust query', async () => {
  const { engine, index, fetchShards, fetchChunks } = await load();
  for (const entry of fixture.queries) {
    fetchShards(entry.query);
    const current = entry.current_page ? index.pages.find(page => page.title === entry.current_page).id : null;
    const outcome = engine.query(index, entry.query, { limit: entry.limit ?? 100, currentPage: current });
    if (outcome.error) { assert.deepEqual({ error: outcome.error }, entry.expected, entry.query); continue; }
    const blocks = group => {
      fetchChunks(group.results.map(result => result.block));
      return { count: group.count, hits: group.results.map(result => { const row = engine.blockRow(index, result.block); return `${row.title}: ${row.text}`; }) };
    };
    assert.deepEqual({
      error: null,
      pages: { count: outcome.pages.count, hits: outcome.pages.results.map(r => r.alias ? `${r.title} ← ${r.alias}` : r.title) },
      blocks: blocks(outcome.blocks),
      current_page: blocks(outcome.currentPage),
    }, entry.expected, entry.query);
  }
});

test('a query fetches only the shards its words need, and none for one character', async () => {
  const { engine, index } = await load();
  assert.deepEqual(engine.shardsFor(index, 'tmux'), ['tm']);
  assert.deepEqual(engine.shardsFor(index, 'tm'), ['tm']);
  assert.deepEqual(engine.shardsFor(index, 'term').sort(), ['terminal', 'terminals']);
  assert.deepEqual(engine.shardsFor(index, 'terminals'), ['terminals']);
  assert.deepEqual(engine.shardsFor(index, 'set g').sort(), ['g', 'set']);
  assert.deepEqual(engine.shardsFor(index, 'g'), []);
  assert.equal(engine.query(index, 'tmux').error, 'shards-not-loaded');
  assert.deepEqual(engine.shardsFor(index, 'zzz'), []);
  assert.equal(engine.query(index, 'zzz').error, null, 'a word no shard can hold needs no fetch');
});

test('Unicode normalization, bounded input and unchanged inputs', async () => {
  const { engine, index, fetchShards } = await load();
  fetchShards('CAFÉ'); fetchShards('café');
  assert.deepEqual(engine.query(index, 'CAFÉ'), engine.query(index, 'café'));
  assert.deepEqual(engine.query(index, ' \n\t'), { error: null, pages: { count: 0, results: [] }, blocks: { count: 0, results: [] }, currentPage: { count: 0, results: [] } });
  for (const value of [null, {}, 'a'.repeat(257), '\ud800']) assert.equal(engine.query(index, value).error, 'invalid-query');
  assert.equal(engine.query(index, Array.from({ length: 17 }, (_, i) => 'w' + i).join(' ')).error, 'too-many-terms');
  for (const limit of [0, 101, NaN, 1.1]) assert.equal(engine.query(index, 'tmux', { limit }).error, 'invalid-limit');
  assert.deepEqual(engine.words(engine.normalize('tmux.conf: set -g mouse on')), ['tmux', 'conf', 'set', 'g', 'mouse', 'on']);
});

test('validation rejects version 1, revision and digest mismatch, and malformed records', async () => {
  const { engine, manifest, index } = await load();
  const raw = JSON.parse(files[manifestName]);
  for (const mutate of [
    m => m.format_version = 1,
    m => m.revision = 'fedcba9876543210',
    m => m.chunks.pop(),
    m => m.chunk_starts[0] = 1,
    m => m.chunk_starts[2] = m.chunk_starts[1],
    m => m.chunk_starts[m.chunk_starts.length - 1] = m.blocks,
    m => { m.chunks = []; m.chunk_starts = []; },
    m => m.engine = 'https://evil.test/engine.mjs',
    m => m.shards['../x'] = m.titles,
  ]) {
    const copy = structuredClone(raw); mutate(copy);
    assert.throws(() => engine.prepareManifest(copy, raw.revision), /publication/);
  }
  assert.throws(() => engine.prepareManifest(raw, 'fedcba9876543210'), /publication/);
  const titles = files[engine.titlesFile(manifest)];
  assert.throws(() => engine.createIndex(manifest, titles + ' '), /publication/, 'digest mismatch');
  const shard = files[engine.shardFile(manifest, 'tm')];
  assert.throws(() => engine.addShard(index, 'tm', shard.replace('tmux', 'tmuy')), /publication/);
  assert.throws(() => engine.addShard(index, 'mo', shard), /publication/, 'a shard under another key');
  const chunk = files[engine.chunkFile(manifest, 0)];
  assert.throws(() => engine.addChunk(index, 1, chunk), /publication/);
  // A record that is well formed but names the wrong page fails even with a matching digest.
  const forged = { ...manifest, chunks: [engine.digest(chunk.replace('p-43ff06c325b57047', 'p-0000000000000000')), ...manifest.chunks.slice(1)] };
  const other = engine.createIndex(forged, titles);
  assert.throws(() => engine.addChunk(other, 0, chunk.replace('p-43ff06c325b57047', 'p-0000000000000000')), /publication/);
});

test('chunks of different lengths map each block to the chunk that starts at or before it', async () => {
  const { engine, manifest, index, fetchChunks } = await load();
  const starts = manifest.chunkStarts;
  assert.ok(new Set(starts.map((start, at) => (starts[at + 1] ?? manifest.blocks) - start)).size > 1, 'the fixture has chunks of different lengths');
  for (let block = 0; block < manifest.blocks; block++) {
    const number = engine.chunkOf(manifest, block);
    assert.ok(starts[number] <= block && block < (starts[number + 1] ?? manifest.blocks), `block ${block}`);
  }
  const all = Array.from({ length: manifest.blocks }, (_, block) => block);
  assert.deepEqual(engine.missingChunks(index, all), starts.map((_, number) => number));
  fetchChunks(all);
  assert.deepEqual(engine.missingChunks(index, all), []);
  for (const block of all) assert.equal(engine.blockRow(index, block).page, engine.pageOf(index, block).id);
  // A chunk with a record more or fewer than its range is rejected.
  const raw = JSON.parse(files[engine.chunkFile(manifest, 0)]);
  raw.blocks.pop();
  const short = JSON.stringify(raw);
  const forged = engine.createIndex({ ...manifest, chunks: [engine.digest(short), ...manifest.chunks.slice(1)] }, files[engine.titlesFile(manifest)]);
  assert.throws(() => engine.addChunk(forged, 0, short), /publication/);
});

test('the digest matches the publisher and All pages', async () => {
  const { engine } = await load();
  assert.equal(engine.digest(''), 'cbf29ce484222325');
  assert.equal(engine.digest('tmux'), '424036ef4353ec8f');
  assert.equal(engine.digest('Café/東京'), '0c0168ce3c4520c8');
});

test('block rows carry breadcrumbs and destinations; highlights mark word starts', async () => {
  const { engine, index, fetchShards, fetchChunks } = await load();
  fetchShards('tmux.conf.local');
  const outcome = engine.query(index, 'tmux.conf.local');
  assert.deepEqual(outcome.pages.results, [{ kind: 'page', page: 'p-75119bb74a8ca2e2.html', title: 'tmux/Config/local', href: 'p-75119bb74a8ca2e2.html', alias: 'tmux.conf.local' }]);
  const [hit] = outcome.blocks.results;
  assert.equal(engine.blockRow(index, hit.block), null, 'no row before its chunk loads');
  fetchChunks([hit.block]);
  const row = engine.blockRow(index, hit.block);
  assert.deepEqual(row.parents, ['Layers', 'Local', 'Deepest']);
  assert.match(row.href, /^p-75119bb74a8ca2e2\.html#source-block-4$/);
  assert.deepEqual(engine.highlight(row.text, 'tmux.conf.local'), [[0, 4], [5, 9], [10, 15], [26, 30], [31, 35]]);
  assert.deepEqual(engine.highlight('Tmux Config', 'tm con'), [[0, 2], [5, 8]]);
  assert.deepEqual(engine.highlight('tmux/Config', 'config', 'page'), [[5, 11]]);
  assert.deepEqual(engine.highlight('tmux', 'tmx', 'page'), [[0, 2], [3, 4]]);
});

test('host navigation adapts the page route and preserves UUID fragment', async () => {
  const { resolveResult } = await modulePromise;
  const result = { title: 'Public/Session', href: 'p-0000000000000001.html#block-12345678-1234-1234-1234-123456789abc' };
  assert.equal(resolveResult(result), result.href);
  assert.equal(resolveResult(result, title => '/' + title + '/'), '/Public/Session/#block-12345678-1234-1234-1234-123456789abc');
  assert.throws(() => resolveResult(result, () => '/bad/#other'), /resolver/);
});

test('long-block snippets explain a late match and preserve Unicode graphemes', async () => {
  const { snippet } = await modulePromise;
  const value = '🙂é '.repeat(300) + 'late-needle context ' + 'tail '.repeat(300);
  const result = snippet(value, 'late-needle');
  assert.ok(result.includes('late-needle context'));
  assert.ok(result.startsWith('…'));
  assert.ok([...result].length <= 240);
  assert.ok(!result.includes('�'));
  assert.equal(snippet('short text', 'short'), 'short text');
});
