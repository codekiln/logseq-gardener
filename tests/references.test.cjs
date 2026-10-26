const { readFileSync } = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const test = require('node:test');
const assert = require('node:assert/strict');
const raw = readFileSync(path.join(__dirname, '../crates/logseq-gardener-sdk/src/site/references.js'), 'utf8');
// The same trimming the publisher applies before writing lsg-references.js.
let source = '';
for (const line of raw.split('\n').map((l) => l.trim())) {
  if (!line || line.startsWith('//')) continue;
  if (source && !';{},'.includes(source.at(-1))) source += '\n';
  source += line;
}

// A small DOM with an HTML reader, enough for the script's selectors and fragment files.
const VOID = new Set(['input', 'br', 'img', 'meta', 'base']);
class Node {
  constructor(tag, attributes = {}) {
    this.tagName = tag.toUpperCase();
    this.attributes = attributes;
    this.childNodes = [];
    this.parentElement = null;
    this.hidden = 'hidden' in attributes;
    this.value = '';
    this.text = '';
    this.dataset = new Proxy({}, {
      get: (_, key) => this.attributes[`data-${key.replace(/[A-Z]/g, (c) => '-' + c.toLowerCase())}`],
      set: (_, key, value) => { this.attributes[`data-${key.replace(/[A-Z]/g, (c) => '-' + c.toLowerCase())}`] = String(value); return true; },
    });
  }
  get className() { return this.attributes.class || ''; }
  set className(value) { this.attributes.class = value; }
  get firstChild() { return this.childNodes[0] ?? null; }
  get lastChild() { return this.childNodes.at(-1) ?? null; }
  get textContent() { return this.text + this.childNodes.map((c) => c.textContent).join(''); }
  set textContent(value) { this.text = String(value); this.childNodes = []; }
  setAttribute(name, value) { this.attributes[name] = String(value); }
  getAttribute(name) { return this.attributes[name] ?? null; }
  append(...nodes) {
    for (const node of nodes) {
      if (node.parentElement) node.parentElement.childNodes = node.parentElement.childNodes.filter((c) => c !== node);
      node.parentElement = this;
      this.childNodes.push(node);
    }
  }
  prepend(node) { this.append(node); this.childNodes.unshift(this.childNodes.pop()); }
  replaceChildren(...nodes) { for (const c of this.childNodes) c.parentElement = null; this.childNodes = []; this.append(...nodes.filter(Boolean)); }
  insertAdjacentHTML(_, html) { this.append(...parse(html).childNodes); }
  matches(selector) {
    let rest = selector;
    if (rest.endsWith(':not([hidden])')) { if (this.hidden) return false; rest = rest.slice(0, -14); }
    const attr = rest.match(/^\[([\w-]+)(?:="?([^"\]]*)"?)?\]$/);
    if (attr) return attr[1] in this.attributes && (attr[2] === undefined || this.attributes[attr[1]] === attr[2]);
    if (rest.startsWith('.')) return this.className.split(' ').includes(rest.slice(1));
    return this.tagName === rest.toUpperCase();
  }
  closest(selector) { for (let n = this; n; n = n.parentElement) if (n.matches(selector)) return n; return null; }
  querySelectorAll(selector) { return this.childNodes.flatMap((c) => [...(c.matches(selector) ? [c] : []), ...c.querySelectorAll(selector)]); }
  querySelector(selector) { return this.querySelectorAll(selector)[0] ?? null; }
  showModal() { this.open = true; }
  close() { this.open = false; }
  focus() { focused = this; }
}
let focused = null;
function parse(html) {
  const root = new Node('#root');
  let current = root;
  for (const [, close, tag, attrs, text] of html.matchAll(/<(\/?)([\w!-]+)([^>]*)>|([^<]+)/g)) {
    if (text !== undefined) { const t = new Node('#text'); t.text = text; current.append(t); continue; }
    if (tag.startsWith('!')) continue;
    if (close) { current = current.parentElement || root; continue; }
    const attributes = {};
    for (const [, name, value] of attrs.matchAll(/([\w-]+)(?:="([^"]*)")?/g)) attributes[name] = value ?? '';
    const node = new Node(tag, attributes);
    current.append(node);
    if (tag === 'script') {
      const end = html.indexOf('</script>', html.indexOf(attrs) + attrs.length);
      node.text = html.slice(html.indexOf('>', html.indexOf(attrs)) + 1, end);
      continue;
    }
    if (!VOID.has(tag)) current = node;
  }
  // Script text was read above; drop the text node the tokenizer also made for it.
  for (const script of root.querySelectorAll('script')) script.childNodes = [];
  return root;
}

function fragment(body, data) {
  const json = data ? `<script type="application/json" class="references-data">${JSON.stringify(data)}</script>` : '';
  return `<!doctype html><html><head><base href="../">${json}</head><body><main><div class="references-fragment">${body}</div></main></body></html>`;
}
const group = (title, blocks) => `<div class="references-blocks-item"><details class="references-group" open><summary class="foldable-title"><a href="${title}.html">${title}</a></summary>${blocks}</details></div>`;
const block = (n, child = '') => `<div class="references-block"><div class="blocks-container"><ul class="refs-outline"><li data-ref-block="${n}">B${n}${child}</li></ul></div></div>`;

// Pages 0 Alpha, 1 Beta, 2 Gamma. Block 1 is inside block 0, so it adds no filter option of
// its own; block 0's descendants name Beta and Alpha.
const DATA = {
  count: 3,
  files: ['t-linked-1.html', 't-linked-2.html'],
  pages: [['Alpha', 'a.html'], ['Beta', 'b.html'], ['Gamma', 'g.html']],
  blocks: [[0, [0], [1, 0]], [0, [0, 1]], [1, [2], []]],
  filter: [[], []],
};
const FILES = {
  '_refs/t-linked-1.html': (data) => fragment(group('One', block(0, `<ul><li data-ref-block="1">B1</li></ul>`)), data),
  '_refs/t-linked-2.html': () => fragment(group('Two', block(2))),
  '_refs/t-unlinked-1.html': () => fragment(group('Three', '<div class="references-block">U</div>'), { count: 1, files: ['t-unlinked-1.html'] }),
};

function setup({ data = DATA, open = false, storage = new Map(), throwing = false, unlinkedSrc = true } = {}) {
  const page = parse(`<section class="references" data-refs="linked" data-src="_refs/t-linked-1.html"><details class="references-section"><summary><h2>3 Linked References</h2><button type="button" class="filter" data-state="none" hidden></button></summary><div class="references-body"><p role="status"></p></div></details></section><section class="references" data-refs="unlinked"${unlinkedSrc ? ' data-src="_refs/t-unlinked-1.html"' : ''}><details class="references-section"><summary><h2>Unlinked References</h2></summary><div class="references-body"></div></details></section>`);
  const [linked, unlinked] = page.querySelectorAll('section');
  linked.querySelector('details').open = open;
  const fetched = [];
  const observers = [];
  const document = Object.assign(page, { baseURI: 'https://site.test/', createElement: (tag) => new Node(tag) });
  const store = {
    getItem: (key) => { if (throwing) throw new Error('blocked'); return storage.get(key) ?? null; },
    setItem: (key, value) => { if (throwing) throw new Error('blocked'); storage.set(key, value); },
  };
  const window = {};
  const context = {
    window, document, URL, JSON, Promise, Set, Map, Error,
    location: { pathname: '/garden/t.html', hash: '' },
    sessionStorage: store,
    fetch: async (url) => {
      const key = String(url).replace('https://site.test/', '');
      fetched.push(key);
      const body = key === '_refs/t-linked-1.html' ? FILES[key](data) : FILES[key]();
      return { ok: true, text: async () => body };
    },
    DOMParser: class { parseFromString(text) { return parse(text); } },
    IntersectionObserver: class {
      constructor(callback) { this.callback = callback; observers.push(this); }
      observe() {}
      disconnect() { this.done = true; }
    },
  };
  window.IntersectionObserver = context.IntersectionObserver;
  vm.runInNewContext(source, context);
  const tick = () => new Promise((resolve) => setImmediate(resolve));
  const settle = async () => { for (let i = 0; i < 10; i++) await tick(); };
  const toggle = async (section) => {
    const details = section.querySelector('details');
    details.open = !details.open;
    details.ontoggle?.();
    await settle();
  };
  return { window, linked, unlinked, fetched, observers, storage, settle, toggle, h2: (s) => s.querySelector('h2').textContent };
}
const shown = (section) => section.querySelectorAll('[data-ref-block]').filter((b) => !b.hidden).map((b) => b.dataset.refBlock);
const chips = (section) => section.querySelector('dialog').querySelectorAll('div')[2].childNodes.map((c) => c.textContent);
async function openDialog(t) {
  t.linked.querySelector('.filter').onclick({ preventDefault() {} });
  await t.settle();
  return t.linked.querySelector('dialog');
}

test('opening a section fetches only its first file and shows the filter control', async () => {
  const t = setup();
  assert.equal(t.linked.querySelector('.filter').hidden, false);
  assert.deepEqual(t.fetched, []);
  await t.toggle(t.linked);
  assert.deepEqual(t.fetched, ['_refs/t-linked-1.html']);
  assert.equal(t.h2(t.linked), '3 Linked References');
  assert.deepEqual(shown(t.linked), ['0', '1']);
  // The next file loads when the reader nears the end.
  t.observers[0].callback([{ isIntersecting: true }]);
  await t.settle();
  assert.deepEqual(t.fetched, ['_refs/t-linked-1.html', '_refs/t-linked-2.html']);
  assert.deepEqual(shown(t.linked), ['0', '1', '2']);
});

test('a starting filter sets the count, the funnel and keeps ancestors of matching blocks', async () => {
  const t = setup({ data: { ...DATA, filter: [[1], []] }, open: true });
  await t.settle();
  assert.equal(t.h2(t.linked), '1 of 3 Linked References');
  assert.equal(t.linked.querySelector('.filter').dataset.state, 'include');
  // Block 1 matches; block 0 stays as its ancestor; file 2 holds nothing that matches.
  assert.deepEqual(shown(t.linked), ['0', '1']);
  assert.deepEqual(t.fetched, ['_refs/t-linked-1.html']);
});

test('chips include, exclude and remove, with counts from the kept blocks', async () => {
  const t = setup({ open: true });
  await t.settle();
  const dialog = await openDialog(t);
  assert.equal(dialog.open, true);
  assert.deepEqual(chips(t.linked), ['Alpha2', 'Beta1', 'Gamma1']);
  const alpha = dialog.querySelector('[data-p="0"]');
  assert.equal(alpha.getAttribute('aria-label'), 'Alpha, 2: not filtered');
  dialog.onclick({ target: alpha, shiftKey: false });
  await t.settle();
  assert.equal(t.h2(t.linked), '2 of 3 Linked References');
  assert.deepEqual(chips(t.linked), ['Beta1']);
  assert.equal(dialog.querySelector('[data-p="0"]').getAttribute('aria-label'), 'Alpha: included');
  assert.equal(focused, dialog.querySelector('[data-p="0"]'));
  // Shift+Enter on Beta excludes it: the funnel turns to the mixed state.
  let prevented = false;
  dialog.onkeydown({ key: 'Enter', shiftKey: true, target: dialog.querySelector('[data-p="1"]'), preventDefault() { prevented = true; } });
  await t.settle();
  assert.ok(prevented);
  assert.equal(t.linked.querySelector('.filter').dataset.state, 'mixed');
  assert.equal(t.h2(t.linked), '1 of 3 Linked References');
  assert.deepEqual(shown(t.linked), ['0']);
  // Activating an active chip removes it.
  dialog.onclick({ target: dialog.querySelector('[data-p="0"]'), shiftKey: false });
  dialog.onclick({ target: dialog.querySelector('[data-p="1"]'), shiftKey: false });
  await t.settle();
  assert.equal(t.h2(t.linked), '3 Linked References');
  assert.equal(t.linked.querySelector('.filter').dataset.state, 'none');
});

test('no matching block shows a status and hides every group', async () => {
  const t = setup({ open: true });
  await t.settle();
  const dialog = await openDialog(t);
  dialog.onclick({ target: dialog.querySelector('[data-p="0"]'), shiftKey: true });
  dialog.onclick({ target: dialog.querySelector('[data-p="2"]'), shiftKey: true });
  await t.settle();
  assert.equal(t.linked.querySelector('.filter').dataset.state, 'exclude');
  assert.equal(t.h2(t.linked), '0 of 3 Linked References');
  assert.equal(t.linked.querySelector('[role=status]').textContent, 'No linked references match these filters.');
  assert.ok(t.linked.querySelectorAll('.references-blocks-item').every((g) => g.hidden));
});

test('search narrows chips by fuzzy match', async () => {
  const t = setup({ open: true });
  await t.settle();
  const dialog = await openDialog(t);
  dialog.querySelector('input').value = 'gma';
  dialog.oninput();
  assert.deepEqual(chips(t.linked), ['Gamma1']);
});

test('filter changes last for the visit and survive blocked storage', async () => {
  const storage = new Map();
  const t = setup({ open: true, storage });
  await t.settle();
  const dialog = await openDialog(t);
  dialog.onclick({ target: dialog.querySelector('[data-p="2"]'), shiftKey: false });
  await t.settle();
  assert.deepEqual(JSON.parse(storage.get('lsg-refs:/garden/t.html')), { i: ['g.html'], e: [] });
  // A later visit to the page restores the filter before the section opens.
  const again = setup({ storage });
  await again.settle();
  assert.equal(again.h2(again.linked), '1 of 3 Linked References');
  const blocked = setup({ open: true, throwing: true });
  await blocked.settle();
  const d = await openDialog(blocked);
  d.onclick({ target: d.querySelector('[data-p="2"]'), shiftKey: false });
  await blocked.settle();
  assert.equal(blocked.h2(blocked.linked), '1 of 3 Linked References');
});

test('unlinked references show their count once opened, and zero without a file', async () => {
  const t = setup();
  assert.equal(t.h2(t.unlinked), 'Unlinked References');
  await t.toggle(t.unlinked);
  assert.equal(t.h2(t.unlinked), '1 Unlinked Reference');
  assert.deepEqual(t.fetched, ['_refs/t-unlinked-1.html']);
  const none = setup({ unlinkedSrc: false });
  await none.toggle(none.unlinked);
  assert.equal(none.h2(none.unlinked), '0 Unlinked References');
  assert.deepEqual(none.fetched, []);
});

test('initializing again after a content swap registers nothing twice', async () => {
  const t = setup();
  t.window.logseqGardenerInitReferences();
  t.window.logseqGardenerInitBacklinks(t.linked);
  await t.toggle(t.linked);
  assert.deepEqual(t.fetched, ['_refs/t-linked-1.html']);
  assert.equal(t.linked.querySelectorAll('[data-ref-block="0"]').length, 1);
});

test('the shipped script stays within its 6 KB budget', () => {
  assert.ok(Buffer.byteLength(source) <= 6000, `${Buffer.byteLength(source)} bytes`);
});
