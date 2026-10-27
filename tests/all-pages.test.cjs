const { readFileSync } = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const test = require('node:test');
const assert = require('node:assert/strict');
const site = path.join(__dirname, '../crates/logseq-gardener-sdk/src/site');
const raw = readFileSync(path.join(site, 'all-pages.js'), 'utf8').replace('// __LSG_FUZZY__', () => readFileSync(path.join(site, 'fuzzy.js'), 'utf8'));
// The same splicing and trimming the publisher applies before writing lsg-all-pages.js.
let source = '';
for (const line of raw.split('\n').map((l) => l.trim())) {
  if (!line || line.startsWith('//')) continue;
  if (source && !';{},'.includes(source.at(-1))) source += '\n';
  source += line;
}

// A small DOM: elements with classes, attributes, children and listeners, and the
// selectors the script uses (tag, .class, tag.class, tag[attr]).
class Element {
  constructor(tag) {
    this.tagName = tag.toUpperCase();
    this.attributes = {};
    this.children = [];
    this.listeners = {};
    this.textContent = '';
    this.className = '';
    this.value = '';
    this.dataset = {};
  }
  append(...nodes) { for (const n of nodes) { if (typeof n === 'string') this.textContent += n; else { n.parent = this; this.children.push(n); } } }
  replaceChildren(...nodes) { this.children = []; this.append(...nodes); }
  remove() { if (this.parent) this.parent.children = this.parent.children.filter((c) => c !== this); }
  setAttribute(k, v) { this.attributes[k] = String(v); }
  getAttribute(k) { return this.attributes[k] ?? null; }
  removeAttribute(k) { delete this.attributes[k]; }
  addEventListener(type, f) { (this.listeners[type] ||= []).push(f); }
  dispatch(type, event) { let n = this; event.target ||= this; while (n) { for (const f of n.listeners[type] || []) f(event); n = n.parent; } }
  closest(selector) { let n = this; while (n && !matches(n, selector)) n = n.parent; return n || null; }
  all() { return this.children.flatMap((c) => [c, ...c.all()]); }
  querySelectorAll(selector) { return this.all().filter((n) => matches(n, selector)); }
  querySelector(selector) { return this.querySelectorAll(selector)[0] || null; }
  get text() { return this.textContent + this.children.map((c) => c.text).join(''); }
}
function matches(node, selector) {
  const m = /^([a-z]*)(?:\.([\w-]+))?(?:\[([\w-]+)\])?$/.exec(selector);
  if (!m) throw new Error(`selector ${selector}`);
  const [, tag, cls, attr] = m;
  if (tag && node.tagName !== tag.toUpperCase()) return false;
  if (cls && !node.className.split(' ').includes(cls)) return false;
  if (attr === 'class') return !!node.className;
  if (attr && !(attr in node.attributes)) return false;
  return true;
}
const make = (tag, props = {}, ...children) => { const n = Object.assign(new Element(tag), props); n.append(...children); return n; };

// Logseq's search on lower-cased names: `mise` itself, children, and a looser match.
const LIST = [
  ['mise', null, 'p', 598, 100, 900],
  ['mise/x', null, 'p', 3, 100, 800],
  ['Microfreak', null, 'p', 7662, 100, 700],
  ['Logseq', null, 'p', 6429, 100, 600],
  ['Logseq/Entity', null, 'n', 5902, null, null],
  ['2026-10-23 Fri', 'p-journal.html', 'j', 2, 100, 1000],
  ['minister', null, 'p', 1, 100, 500, '🧭'],
];

function setup() {
  const box = make('div', { className: 'all-pages' });
  box.dataset = { listSrc: '_pages/list.json', sort: 'updated-at', desc: 'true' };
  const header = (cls) => make('th', { className: cls }, make('button', { className: 'sort', dataset: { sort: cls } }));
  box.append(
    make('p', { className: 'total' }),
    make('input', { className: 'search' }),
    make('button', { className: 'journal-toggle' }),
    make('nav', { className: 'paging' }),
    make('table', {}, make('thead', {}, make('tr', {}, make('th', {}), header('name'), header('backlinks'), header('created-at'), header('updated-at'))), make('tbody')),
    make('nav', { className: 'paging' }),
  );
  const root = make('main');
  root.append(box);
  const fetched = [];
  const window = {};
  const context = vm.createContext({
    window,
    document: { baseURI: 'http://site/pages.html', querySelectorAll: (s) => root.querySelectorAll(s), createElement: (tag) => new Element(tag) },
    fetch: (url) => { fetched.push(String(url)); return Promise.resolve({ json: () => Promise.resolve(LIST) }); },
    URL, Date, Math, String, Object, Promise, BigInt, TextEncoder, scrollTo: () => {},
  });
  vm.runInContext(source, context);
  const names = () => box.querySelector('tbody').children.map((tr) => tr.children[1].text);
  const settle = () => new Promise((resolve) => setTimeout(resolve, 0));
  return { box, window, fetched, names, settle, total: () => box.querySelector('.total').textContent };
}

test('score ports Logseq: exact and shorter matches first, missing letters score 0', () => {
  const { window } = setup();
  const { score } = window.logseqGardenerAllPages;
  assert.ok(score('mise', 'mise') > score('mise', 'mise/x'));
  assert.ok(score('mise', 'mise/x') > score('mise', 'minister'));
  assert.ok(score('mise', 'minister') > 0);
  assert.equal(score('mise', 'tmux'), 0);
  // Consecutive matches: 1+2+3+4, then length similarity 1, then the exact substring.
  assert.equal(score('mise', 'mise'), 10 + 1 + 1000);
});

test('digest matches the publisher\'s FNV-1a route digest', () => {
  const { digest } = setup().window.logseqGardenerAllPages;
  assert.equal(digest('mise'), 'da5911a2a798df4f');
  assert.equal(digest('Café/Ünï'), 'eb6395408434ffb3');
});

test('no list request before a sort, search or toggle', async () => {
  const { fetched, settle } = setup();
  await settle();
  assert.deepEqual(fetched, []);
});

test('sorting by Page name then Backlinks flips one shared direction', async () => {
  const { box, names, settle, fetched } = setup();
  const sort = (column) => box.querySelector('th.' + column).querySelector('.sort').dispatch('click', {});
  sort('name');
  await settle();
  assert.deepEqual(fetched, ['http://site/_pages/list.json']);
  assert.deepEqual(names(), ['Logseq', 'Logseq/Entity', 'Microfreak', 'minister', 'mise', 'mise/x']);
  sort('backlinks');
  await settle();
  assert.deepEqual(names().slice(0, 3), ['Microfreak', 'Logseq', 'Logseq/Entity']);
  assert.equal(box.querySelector('th.backlinks').getAttribute('aria-sort'), 'descending');
  assert.equal(box.querySelector('th.name').getAttribute('aria-sort'), null);
  assert.equal(fetched.length, 1);
});

test('updated newest first keeps undated rows last', async () => {
  const { box, names, settle } = setup();
  box.querySelector('th.updated-at').querySelector('.sort').dispatch('click', {});
  await settle();
  // The click flips the default newest-first to oldest first; undated rows stay last.
  assert.deepEqual(names(), ['minister', 'Logseq', 'Microfreak', 'mise/x', 'mise', 'Logseq/Entity']);
});

test('search runs on Enter, ranks by score and clears on Escape', async () => {
  const { box, names, settle, total } = setup();
  const input = box.querySelector('.search');
  input.value = 'Mise';
  input.dispatch('keydown', { key: 'Enter' });
  await settle();
  assert.deepEqual(names(), ['mise', 'mise/x', 'minister']);
  assert.equal(total(), 'Total 3 pages');
  input.dispatch('keydown', { key: 'Escape' });
  await settle();
  assert.equal(input.value, '');
  assert.equal(total(), 'Total 6 pages');
});

test('the journals toggle adds journals', async () => {
  const { box, names, settle, total } = setup();
  const toggle = box.querySelector('.journal-toggle');
  toggle.dispatch('click', {});
  await settle();
  assert.equal(total(), 'Total 7 pages');
  assert.equal(names()[0], '2026-10-23 Fri');
  assert.equal(toggle.getAttribute('aria-pressed'), 'true');
});

test('rows keep namespace style, icons and routes', async () => {
  const { box, settle } = setup();
  box.querySelector('.journal-toggle').dispatch('click', {});
  await settle();
  const rows = box.querySelector('tbody').children;
  const entity = rows.find((tr) => tr.children[1].text === 'Logseq/Entity').children[1].children[0];
  assert.equal(entity.className, 'page-ref namespace-link');
  assert.equal(entity.getAttribute('aria-label'), 'Logseq/Entity (Namespace)');
  assert.equal(entity.href, 'p-1d703d4fa50f254a.html');
  const journal = rows.find((tr) => tr.children[1].text === '2026-10-23 Fri').children[1].children[0];
  assert.equal(journal.href, 'p-journal.html');
  const insight = rows.find((tr) => tr.children[1].text === 'minister');
  assert.equal(insight.children[0].textContent, '🧭');
});

test('paging shows 40 rows and moves between pages', async () => {
  const { box, settle, names } = setup();
  for (let i = 0; i < 50; i++) LIST.push([`page ${String(i).padStart(2, '0')}`, null, 'p', 0, 1, 1]);
  try {
    box.querySelector('.journal-toggle').dispatch('click', {});
    await settle();
    assert.equal(names().length, 40);
    const nav = box.querySelector('.paging');
    assert.equal(nav.querySelector('.page-number').textContent, '1/2');
    nav.children.find((b) => b.textContent === 'Next ▸').onclick();
    assert.equal(names().length, 17);
    assert.equal(box.querySelector('.paging').querySelector('.page-number').textContent, '2/2');
  } finally {
    LIST.splice(7);
  }
});
