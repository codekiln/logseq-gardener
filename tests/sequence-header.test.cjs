// The sequence-header first-party plugin's entry reading. scripts/plugins/browser.py checks the
// header links in Chromium.
const test = require('node:test');
const assert = require('node:assert/strict');
const path = require('node:path');
const { pathToFileURL } = require('node:url');
const loaded = import(pathToFileURL(path.join(__dirname, '../crates/logseq-gardener-sdk/plugins/sequence-header/sequence-header.mjs')));

const base = 'https://garden.example.org/site/';

test('sequenceLinks resolves both neighbours against the site root', async () => {
  const { sequenceLinks } = await loaded;
  assert.deepEqual(sequenceLinks({
    prev: { url: 'p-1.html', title: 'Show/One' },
    next: { url: 'p-3.html', title: 'Show/<Three> & more' },
  }, base), {
    prev: { href: 'https://garden.example.org/site/p-1.html', title: 'Show/One' },
    next: { href: 'https://garden.example.org/site/p-3.html', title: 'Show/<Three> & more' },
  });
});

test('sequenceLinks keeps one side when the entry has one', async () => {
  const { sequenceLinks } = await loaded;
  assert.deepEqual(sequenceLinks({ prev: { url: 'p-1.html', title: 'One' } }, base),
    { prev: { href: 'https://garden.example.org/site/p-1.html', title: 'One' } });
});

test('sequenceLinks gives no links for a page without an entry or a malformed one', async () => {
  const { sequenceLinks } = await loaded;
  for (const metadata of [{}, undefined, null, { prev: null }, { prev: 'p-1.html' },
    { prev: { url: 'p-1.html' } }, { prev: { url: 'p-1.html', title: '' } }, { next: { url: 7, title: 'Seven' } },
    { next: { url: 'http://[bad', title: 'Bad' } }, { next: { url: 'https://elsewhere.example.com/p.html', title: 'Away' } },
    { next: { url: 'javascript:alert(1)', title: 'Script' } }]) {
    assert.deepEqual(sequenceLinks(metadata, base), {}, JSON.stringify(metadata));
  }
});
