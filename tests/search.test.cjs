const test = require('node:test');
const assert = require('node:assert/strict');
const { pathToFileURL } = require('node:url');
const path = require('node:path');
const modulePromise = import(pathToFileURL(path.join(__dirname, '../crates/logseq-gardener-sdk/src/site/search.mjs')));
const href = n => `p-${String(n).padStart(16, '0')}.html`;
function fixture() {
  const pages = [
    { id: href(1), title: 'Needle', href: href(1), aliases: [] },
    { id: href(2), title: 'Alias title', href: href(2), aliases: ['Needle'] },
    { id: href(3), title: 'Needle path', href: href(3), aliases: [] },
    { id: href(4), title: 'A needle here', href: href(4), aliases: [] },
    { id: href(5), title: 'Café/東京', href: href(5), aliases: ['Coffee'] },
  ];
  const content = [
    { id: href(3) + '#source-block-40', page: href(3), href: href(3) + '#source-block-40', text: 'needle block later', ancestors: ['ancestor'], position: 40 },
    { id: href(3) + '#source-block-20', page: href(3), href: href(3) + '#source-block-20', text: 'needle block earlier', ancestors: [], position: 20 },
    { id: href(1) + '#source-block-1', page: href(1), href: href(1) + '#block-12345678-1234-1234-1234-123456789abc', text: 'needle <script> is plain text', ancestors: [], position: 1 },
    { id: href(5) + '#source-block-0', page: href(5), href: href(5) + '#source-block-0', text: 'CAFÉ 東京 path/name punctuation?! ' + '🙂'.repeat(300), ancestors: [], position: 0 },
  ];
  const corpus = { format_version: 1, revision: '0123456789abcdef', pages, content };
  const manifest = { format_version: 1, revision: corpus.revision, corpus: { href: 'lsg-search-' + corpus.revision + '.json', bytes: 1, records: pages.length + content.length }, engine: 'lsg-search-engine-' + corpus.revision + '.mjs' };
  return { manifest, corpus };
}
test('whole result order covers title, alias, prefix, all terms and source ties', async () => {
  const { prepareIndex, query } = await modulePromise;
  const { manifest, corpus } = fixture();
  const index = prepareIndex(manifest, corpus, corpus.revision);
  const expected = [href(1), href(2), href(3), href(4), corpus.content[2].href, corpus.content[1].href, corpus.content[0].href];
  assert.deepEqual(query(index, '  NEEDLE  ').results.map(r => r.href), expected);
  assert.deepEqual(query(index, 'needle needle').results.map(r => r.href), [href(4), href(2), href(1), href(3), corpus.content[2].href, corpus.content[1].href, corpus.content[0].href]);
  assert.deepEqual(query(index, 'needle', { limit: 2 }).results.map(r => r.href), expected.slice(0, 2));
  assert.deepEqual(query(index, 'unavailable').results, []);
  assert.equal(query(index, 'needle').results[4].text, 'needle <script> is plain text');
  assert.deepEqual(corpus, fixture().corpus, 'preparation/query preserve inputs');
});
test('Unicode normalization, CJK, punctuation, bounded text and repeated terms', async () => {
  const { prepareIndex, query } = await modulePromise;
  const { manifest, corpus } = fixture(); const index = prepareIndex(manifest, corpus, corpus.revision);
  assert.deepEqual(query(index, 'CAFÉ'), query(index, 'cafe\u0301'));
  assert.deepEqual(query(index, '東京').results.map(r => r.kind), ['page', 'block']);
  const result = query(index, 'path/name punctuation?!').results[0];
  assert.equal([...result.text].length, 240); assert.ok(result.text.endsWith('…'));
  assert.deepEqual(query(index, '東京 東京').results.map(r => r.href), query(index, '東京').results.map(r => r.href));
});
test('empty malformed oversized queries and limits return explicit bounded outcomes', async () => {
  const { prepareIndex, query } = await modulePromise;
  const { manifest, corpus } = fixture(); const index = prepareIndex(manifest, corpus, corpus.revision);
  assert.deepEqual(query(index, ' \n\t'), { error: null, results: [] });
  for (const value of [null, {}, 'a'.repeat(257), '\ud800']) assert.equal(query(index, value).error, 'invalid-query');
  assert.equal(query(index, Array.from({length: 17}, (_, i) => String(i)).join(' ')).error, 'too-many-terms');
  for (const limit of [0, 101, NaN, 1.1]) assert.equal(query(index, 'needle', {limit}).error, 'invalid-limit');
});
test('publication validation rejects versions, revisions, records and unsafe routes', async () => {
  const { prepareIndex } = await modulePromise;
  for (const mutate of [
    f => f.manifest.format_version = 2,
    f => f.corpus.format_version = 2,
    f => f.manifest.revision = 'fedcba9876543210',
    f => f.corpus.revision = 'fedcba9876543210',
    f => f.manifest.corpus.records++,
    f => f.manifest.corpus.href = '../elsewhere.json',
    f => f.corpus.pages[0].href = 'javascript:alert(1)',
    f => f.corpus.content[0].href = href(1) + '#source-block-40',
    f => f.corpus.content[0].text = {},
    f => f.corpus.content[0].ancestors = [null],
    f => f.corpus.content[0].id = f.corpus.content[1].id,
  ]) { const f = fixture(); mutate(f); assert.throws(() => prepareIndex(f.manifest, f.corpus, '0123456789abcdef'), /publication/); }
  const f = fixture(); assert.throws(() => prepareIndex(f.manifest, f.corpus, 'wrong'), /publication/);
});
test('host navigation adapts the page route and preserves UUID fragment', async () => {
  const { resolveResult } = await modulePromise; const result = { title: 'Public/Session', href: href(1) + '#block-12345678-1234-1234-1234-123456789abc' };
  assert.equal(resolveResult(result), result.href);
  assert.equal(resolveResult(result, title => '/' + title + '/'), '/Public/Session/#block-12345678-1234-1234-1234-123456789abc');
  assert.throws(() => resolveResult(result, () => '/bad/#other'), /resolver/);
});

test('long-block snippets explain a late match and preserve Unicode graphemes', async () => {
  const { prepareIndex, query } = await modulePromise;
  const f = fixture();
  f.corpus.content[0].text = '🙂e\u0301 '.repeat(300) + 'late-needle context ' + 'tail '.repeat(300);
  const index = prepareIndex(f.manifest, f.corpus, f.corpus.revision);
  const result = query(index, 'late-needle').results[0];
  assert.ok(result.text.includes('late-needle context'));
  assert.ok(result.text.startsWith('…'));
  assert.ok([...result.text].length <= 240);
  assert.ok(!result.text.includes('�'));
});
