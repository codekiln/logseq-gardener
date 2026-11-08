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
    chapters: [],
  });
  const chapters = [{ anchor: 'source-block-9', seconds: 22, title: 'The Unstable', parent: null }];
  assert.deepEqual(episodeOf({ page, metadata: { recording: 'assets/a-1.mp3', chapters } }, base).chapters, chapters);
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

test('chaptersOf keeps well-formed chapters and parents that come earlier', async () => {
  const { chaptersOf } = await loaded;
  assert.deepEqual(chaptersOf(undefined), []);
  assert.deepEqual(chaptersOf({}), []);
  assert.deepEqual(chaptersOf([
    { anchor: 'source-block-1', seconds: 0, title: 'Enter', parent: null },
    { anchor: 'source-block-2', seconds: 0, title: 'Launchpad', parent: 0 },
    { anchor: 'source-block-3', seconds: -1, title: 'negative' },
    { anchor: 4, seconds: 5, title: 'anchor not a string' },
    { anchor: 'source-block-5', seconds: 1.5, title: 'fraction' },
    { anchor: 'source-block-6', seconds: 312, parent: 7 },
    { anchor: 'source-block-7', seconds: 1097, title: 'Self', parent: 3 },
  ]), [
    { anchor: 'source-block-1', seconds: 0, title: 'Enter', parent: null },
    { anchor: 'source-block-2', seconds: 0, title: 'Launchpad', parent: 0 },
    { anchor: 'source-block-6', seconds: 312, title: '', parent: null },
    { anchor: 'source-block-7', seconds: 1097, title: 'Self', parent: null },
  ]);
});

test('formatTime writes minutes and seconds, with hours only when needed', async () => {
  const { formatTime } = await loaded;
  assert.equal(formatTime(0), '0:00');
  assert.equal(formatTime(315), '5:15');
  assert.equal(formatTime(1097), '18:17');
  assert.equal(formatTime(3723), '1:02:03');
  assert.equal(formatTime(12.9), '0:12');
});

const sun = [
  { anchor: 'a', seconds: 0, title: 'Enter Infini Pad', parent: null },
  { anchor: 'b', seconds: 0, title: 'Launchpad playing', parent: 0 },
  { anchor: 'c', seconds: 312, title: 'Infini Pad as bass', parent: 0 },
  { anchor: 'd', seconds: 821, title: 'pointillist emergence', parent: 2 },
  { anchor: 'e', seconds: 1097, title: 'Enter 8bitfi', parent: null },
  { anchor: 'f', seconds: 1097, title: 'Same time, later in the page', parent: 4 },
  { anchor: 'g', seconds: 600, title: 'Out of order', parent: null },
];

test('currentChapter takes the latest start, and the later chapter on a tie', async () => {
  const { currentChapter } = await loaded;
  assert.equal(currentChapter([], 10), -1);
  assert.equal(currentChapter([{ seconds: 5 }], 4), -1);
  assert.equal(currentChapter(sun, 0), 1);
  assert.equal(currentChapter(sun, 311.9), 1);
  assert.equal(currentChapter(sun, 312), 2);
  assert.equal(currentChapter(sun, 700), 6);
  assert.equal(currentChapter(sun, 900), 3);
  assert.equal(currentChapter(sun, 5000), 5);
});

test('chapterTrail lists the enclosing chapters outermost first', async () => {
  const { chapterTrail } = await loaded;
  assert.deepEqual(chapterTrail(sun, 3).map((c) => c.anchor), ['a', 'c', 'd']);
  assert.deepEqual(chapterTrail(sun, 4).map((c) => c.anchor), ['e']);
  assert.deepEqual(chapterTrail(sun, -1), []);
  assert.deepEqual(chapterTrail([], 0), []);
});

test('timestampLength finds the written timestamp only when it is the chapter start', async () => {
  const { timestampLength } = await loaded;
  assert.equal(timestampLength('18:17 Enter', 1097), 5);
  assert.equal(timestampLength('5:15', 315), 4);
  assert.equal(timestampLength('01:00:23 Recording', 3623), 8);
  assert.equal(timestampLength('13:41 - pointillist', 821), 5);
  assert.equal(timestampLength('18:17 Enter', 1096), 0);
  assert.equal(timestampLength('Enter at 18:17', 1097), 0);
  assert.equal(timestampLength('', 0), 0);
});

test('the default export is a host API 1 plugin with the manifest id', async () => {
  const plugin = (await loaded).default;
  assert.equal(plugin.id, 'podcast-episode');
  assert.equal(plugin.apiVersion, 1);
  assert.equal(typeof plugin.setup, 'function');
});
