const { readFileSync } = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const test = require('node:test');
const assert = require('node:assert/strict');
const source = readFileSync(path.join(__dirname, '../crates/logseq-gardener-sdk/src/site/journals.js'), 'utf8');
const flush = async () => { for (let i = 0; i < 5; i++) await new Promise(setImmediate); };
function node(selectors, attributes = {}) {
  const values = new Map(Object.entries(attributes));
  return {
    selectors, isConnected: true, replacedWith: null,
    getAttribute: name => values.get(name) ?? null,
    setAttribute: (name, value) => values.set(name, String(value)),
    removeAttribute: name => values.delete(name),
    hasAttribute: name => values.has(name),
    matches: selector => selector.split(',').some(part => selectors.includes(part.trim())),
  };
}
function fixture(pages) {
  const feed = [];
  const fetched = [];
  const link = node(['[data-lsg-journals-earlier]'], { href: 'journals-2.html' });
  link.before = article => feed.push(article.name);
  link.remove = () => { link.isConnected = false; };
  const observed = new Set();
  let callback;
  const listeners = {};
  const document = {
    addEventListener: (type, fn, capture) => { listeners[type] = { fn, capture }; },
    querySelectorAll: selector => (selector === '[data-lsg-journals-earlier]' && link.isConnected ? [link] : []),
    adoptNode: value => value,
  };
  const initialized = [];
  const window = {
    IntersectionObserver: class {
      constructor(fn, options) { callback = fn; this.options = options; }
      observe(target) { observed.add(target); }
      unobserve(target) { observed.delete(target); }
    },
    logseqGardenerInitBacklinks: root => initialized.push(root),
  };
  window.IntersectionObserver.prototype.options = null;
  const fetch = async href => {
    fetched.push(href);
    return href in pages ? { ok: true, text: async () => href } : { ok: false, status: 404 };
  };
  class DOMParser { parseFromString(text) { return pages[text]; } }
  vm.runInContext(source, vm.createContext({ window, document, fetch, DOMParser, Error }));
  const intersect = async () => { if (observed.has(link)) callback([{ isIntersecting: true, target: link }]); await flush(); };
  return { feed, fetched, link, observed, intersect, listeners, initialized, window };
}
function batch(names, next) {
  return {
    querySelectorAll: selector => (selector === 'article.lsg-journal' ? names.map(name => ({ name })) : []),
    querySelector: selector => (selector === '[data-lsg-journals-earlier]' && next ? node([], { href: next }) : null),
  };
}
test('later batches append near the bottom until the last one', async () => {
  const f = fixture({
    'journals-2.html': batch(['2026-10-21 Wed', '2026-10-20 Tue'], 'journals-3.html'),
    'journals-3.html': batch(['2026-10-19 Mon'], null),
  });
  assert.ok(f.observed.has(f.link));
  await f.intersect();
  assert.deepEqual(f.feed, ['2026-10-21 Wed', '2026-10-20 Tue']);
  assert.equal(f.link.getAttribute('href'), 'journals-3.html');
  await f.intersect();
  assert.deepEqual(f.feed, ['2026-10-21 Wed', '2026-10-20 Tue', '2026-10-19 Mon']);
  assert.equal(f.link.isConnected, false);
  assert.equal(f.observed.has(f.link), false);
});
test('a failed batch leaves the Earlier link in place', async () => {
  const f = fixture({});
  await f.intersect();
  assert.deepEqual(f.fetched, ['journals-2.html']);
  assert.equal(f.link.isConnected, true);
  assert.equal(f.link.getAttribute('href'), 'journals-2.html');
  assert.deepEqual(f.feed, []);
});
function references(route) {
  const details = node(['details[data-lsg-journal-refs]'], { 'data-lsg-journal-refs': route });
  details.open = true;
  details.replaceWith = value => { details.replacedWith = value; };
  return details;
}
test('Linked References load from the journal page when opened', async () => {
  const disclosure = node(['details.backlinks-disclosure']);
  const section = node(['[data-lsg-backlinks]'], { id: 'linked-references' });
  section.parentElement = { name: 'article' };
  section.querySelector = selector => (selector === 'details.backlinks-disclosure' ? disclosure : null);
  const f = fixture({ 'p-1.html': { querySelector: selector => (selector === '[data-lsg-backlinks]' ? section : null) } });
  assert.equal(f.listeners.toggle.capture, true);
  const details = references('p-1.html');
  await f.listeners.toggle.fn({ target: details });
  assert.equal(details.replacedWith, section);
  assert.equal(disclosure.hasAttribute('open'), true);
  assert.equal(section.hasAttribute('id'), false);
  assert.deepEqual(f.initialized, [section.parentElement]);
});
test('a failed reference fetch keeps the fallback link and can retry', async () => {
  const f = fixture({});
  const details = references('p-missing.html');
  await f.listeners.toggle.fn({ target: details });
  assert.equal(details.replacedWith, null);
  assert.equal(details.hasAttribute('data-lsg-loading'), false);
  details.open = false;
  await f.listeners.toggle.fn({ target: details });
  assert.deepEqual(f.fetched, ['p-missing.html']);
});
test('hosts can watch replaced feed content again', () => {
  const f = fixture({});
  f.observed.clear();
  f.window.logseqGardenerInitJournals();
  assert.ok(f.observed.has(f.link));
});
