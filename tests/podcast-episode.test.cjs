// The podcast-episode first-party plugin's pure functions. scripts/plugins/browser.py checks the
// player in Chromium.
const test = require('node:test');
const assert = require('node:assert/strict');
const path = require('node:path');
const { pathToFileURL } = require('node:url');
const loaded = import(pathToFileURL(path.join(__dirname, '../crates/logseq-gardener-sdk/plugins/podcast-episode/podcast-episode.mjs')));

const base = 'https://garden.example.org/site/';

test('recordingURL resolves against the site root and drops the fragment', async () => {
  const { recordingURL } = await loaded;
  assert.equal(recordingURL('assets/a-1.mp3', base), 'https://garden.example.org/site/assets/a-1.mp3');
  assert.equal(recordingURL('https://cdn.example.com/full.mp3#t=12', base), 'https://cdn.example.com/full.mp3');
  assert.equal(recordingURL('../../assets/a-1.mp3', 'https://garden.example.org/site/GitP/A/'), 'https://garden.example.org/site/assets/a-1.mp3');
  assert.equal(recordingURL('http://[bad', base), null);
});

test('clampTime keeps a seek within a known duration', async () => {
  const { clampTime } = await loaded;
  assert.equal(clampTime(90, 60), 60);
  assert.equal(clampTime(30, 60), 30);
  assert.equal(clampTime(90, NaN), 90);
  assert.equal(clampTime(90, Infinity), 90);
});

test('requestedTime accepts only whole seconds for this recording', async () => {
  const { requestedTime } = await loaded;
  const source = 'https://cdn.example.com/full.mp3';
  assert.equal(requestedTime({ source, seconds: 75 }, source), 75);
  assert.equal(requestedTime({ source, seconds: 0 }, source), 0);
  assert.equal(requestedTime({ source: 'https://cdn.example.com/other.mp3', seconds: 75 }, source), null);
  for (const seconds of [-1, 1.5, '75', undefined, Number.MAX_SAFE_INTEGER + 1]) {
    assert.equal(requestedTime({ source, seconds }, source), null, String(seconds));
  }
  assert.equal(requestedTime({ seconds: 4 }, source), null);
  assert.equal(requestedTime(undefined, source), null);
});

test('episodeOf reads the entry, and a page without one is not an episode', async () => {
  const { episodeOf } = await loaded;
  const page = { id: 'gitp/a/session/26/10/24-sat', title: 'GitP/A/Session/26/10/24-Sat', url: 'GitP/A/Session/26/10/24-Sat/' };
  assert.deepEqual(episodeOf({ page, metadata: { recording: 'assets/a-1.mp3', title: 'The Unstable over USB' } }, base), {
    source: 'https://garden.example.org/site/assets/a-1.mp3',
    title: 'The Unstable over USB',
    url: 'https://garden.example.org/site/GitP/A/Session/26/10/24-Sat/',
  });
  assert.equal(episodeOf({ page, metadata: { recording: 'assets/a-1.mp3' } }, base).title, page.title);
  assert.equal(episodeOf({ page, metadata: {} }, base), null);
  assert.equal(episodeOf({ page, metadata: { recording: 42 } }, base), null);
});

test('inlineAudio finds the page recording outside references and embeds', async () => {
  const { inlineAudio } = await loaded;
  const audio = (src, inside = null) => ({
    getAttribute: (name) => (name === 'src' ? src : null),
    closest: (selector) => (inside && selector.includes(inside) ? {} : null),
  });
  const embedded = audio('assets/a-1.mp3', '.embed-block');
  const other = audio('assets/a-2.mp3');
  const own = audio('assets/a-1.mp3#t=0');
  const main = { querySelectorAll: () => [embedded, other, own] };
  assert.equal(inlineAudio(main, 'https://garden.example.org/site/assets/a-1.mp3', base), own);
  assert.equal(inlineAudio(main, 'https://garden.example.org/site/assets/a-3.mp3', base), null);
  assert.equal(inlineAudio(null, 'https://garden.example.org/site/assets/a-1.mp3', base), null);
});

test('the default export is a host API 1 plugin with the manifest id', async () => {
  const plugin = (await loaded).default;
  assert.equal(plugin.id, 'podcast-episode');
  assert.equal(plugin.apiVersion, 1);
  assert.equal(typeof plugin.setup, 'function');
});
