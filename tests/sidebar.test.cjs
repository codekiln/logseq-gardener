const { readFileSync } = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const test = require('node:test');
const assert = require('node:assert/strict');
const source = readFileSync(path.join(__dirname, '../crates/logseq-gardener-sdk/src/site/sidebar.js'), 'utf8');

// A small DOM: elements, text, the selectors the sidebar script uses, and listeners.
function compound(text) {
  const parts = { tag: null, classes: [], ids: [], attrs: [], scope: false };
  for (const [token] of text.matchAll(/:scope|\.[\w-]+|#[\w-]+|\[[^\]]+\]|[\w-]+/g)) {
    if (token === ':scope') parts.scope = true;
    else if (token[0] === '.') parts.classes.push(token.slice(1));
    else if (token[0] === '#') parts.ids.push(token.slice(1));
    else if (token[0] === '[') {
      const [, name, op, value] = token.match(/^\[([\w-]+)(?:(\^?=)"?([^"\]]*)"?)?\]$/);
      parts.attrs.push([name, op, value]);
    } else parts.tag = token.toUpperCase();
  }
  return parts;
}
function matchesCompound(el, parts, scope) {
  if (!el || el.nodeType !== 1) return false;
  if (parts.scope && el !== scope) return false;
  if (parts.tag && el.tagName !== parts.tag) return false;
  if (!parts.classes.every(c => el.classList.contains(c))) return false;
  if (!parts.ids.every(id => el.id === id)) return false;
  return parts.attrs.every(([name, op, value]) => {
    const actual = el.getAttribute(name);
    if (actual === null) return false;
    return !op || (op === '=' ? actual === value : actual.startsWith(value));
  });
}
function matches(el, selector, scope) {
  return selector.split(',').some(one => {
    const tokens = one.trim().replace(/\s*>\s*/g, ' > ').split(/\s+/);
    const step = (node, i) => {
      if (!matchesCompound(node, compound(tokens[i]), scope)) return false;
      if (i === 0) return true;
      if (tokens[i - 1] === '>') return step(node.parentElement, i - 2);
      for (let up = node.parentElement; up; up = up.parentElement) if (step(up, i - 1)) return true;
      return false;
    };
    return step(el, tokens.length - 1);
  });
}
class Node {
  constructor() { this.parentElement = null; this.childNodes = []; }
  remove() { if (this.parentElement) { const list = this.parentElement.childNodes; list.splice(list.indexOf(this), 1); this.parentElement = null; } }
  after(other) { const p = this.parentElement; other.remove(); p.childNodes.splice(p.childNodes.indexOf(this) + 1, 0, other); other.parentElement = p; }
}
class Text extends Node {
  constructor(text) { super(); this.nodeType = 3; this.text = text; }
  get textContent() { return this.text; }
  cloneNode() { return new Text(this.text); }
}
class Element extends Node {
  constructor(tag, attrs = {}) {
    super();
    this.nodeType = 1; this.tagName = tag.toUpperCase(); this.attrs = new Map(Object.entries(attrs)); this.listeners = {};
    const el = this;
    this.classList = {
      contains: c => (el.attrs.get('class') ?? '').split(' ').includes(c),
      add: c => { if (!el.classList.contains(c)) el.attrs.set('class', ((el.attrs.get('class') ?? '') + ' ' + c).trim()); },
      remove: c => el.attrs.set('class', (el.attrs.get('class') ?? '').split(' ').filter(x => x && x !== c).join(' ')),
      toggle: (c, force) => { const on = force ?? !el.classList.contains(c); if (on) el.classList.add(c); else el.classList.remove(c); return on; },
    };
  }
  get id() { return this.attrs.get('id') ?? ''; }
  get open() { return this.attrs.has('open'); }
  set open(value) { if (value) this.attrs.set('open', ''); else this.attrs.delete('open'); }
  get hidden() { return this.attrs.has('hidden'); }
  set hidden(value) { if (value) this.attrs.set('hidden', ''); else this.attrs.delete('hidden'); }
  get children() { return this.childNodes.filter(n => n.nodeType === 1); }
  get textContent() { return this.childNodes.map(n => n.textContent).join(''); }
  set textContent(text) { this.replaceChildren(text); }
  getAttribute(name) { return this.attrs.has(name) ? this.attrs.get(name) : null; }
  setAttribute(name, value) { this.attrs.set(name, String(value)); }
  removeAttribute(name) { this.attrs.delete(name); }
  append(...nodes) {
    for (let node of nodes) {
      if (typeof node === 'string') node = new Text(node);
      node.remove();
      node.parentElement = this;
      this.childNodes.push(node);
    }
  }
  prepend(node) { node.remove(); node.parentElement = this; this.childNodes.unshift(node); }
  insertBefore(node, before) {
    node.remove();
    node.parentElement = this;
    const at = before ? this.childNodes.indexOf(before) : -1;
    if (at < 0) this.childNodes.push(node); else this.childNodes.splice(at, 0, node);
  }
  replaceChildren(...nodes) { for (const node of [...this.childNodes]) node.remove(); this.append(...nodes); }
  descendants() { return this.children.flatMap(c => [c, ...c.descendants()]); }
  querySelectorAll(selector) { return this.descendants().filter(el => matches(el, selector, this)); }
  querySelector(selector) { return this.querySelectorAll(selector)[0] ?? null; }
  matches(selector) { return matches(this, selector, null); }
  closest(selector) { for (let el = this; el; el = el.parentElement) if (el.matches(selector)) return el; return null; }
  contains(other) { for (let el = other; el; el = el.parentElement) if (el === this) return true; return false; }
  cloneNode() {
    const copy = new Element(this.tagName, Object.fromEntries(this.attrs));
    for (const child of this.childNodes) copy.append(child.cloneNode(true));
    return copy;
  }
  addEventListener(type, callback) { (this.listeners[type] ||= []).push(callback); }
  removeEventListener(type, callback) { this.listeners[type] = (this.listeners[type] || []).filter(c => c !== callback); }
  focus() { state.focused = this; }
  getBoundingClientRect() { return { width: Number(this.style?.width ?? 512) }; }
}
// h('li', {class: 'x'}, child, 'text') builds markup.
const h = (tag, attrs = {}, ...children) => { const el = new Element(tag, attrs); el.append(...children); return el; };
const li = (anchor, ...content) => h('li', {}, ...(anchor ? [h('span', { id: 'block-' + anchor })] : []), h('a', { class: 'bullet', href: '#' }), ...content);

function makeDocument(body) {
  const html = h('html'), head = h('head');
  html.style = { setProperty: (name, value) => html.setAttribute('style-' + name, value) };
  html.append(head, body);
  return {
    documentElement: html, head, body,
    getElementById: id => html.querySelectorAll('#' + id)[0] ?? null,
    querySelector: s => html.querySelector(s),
    querySelectorAll: s => html.querySelectorAll(s),
  };
}
// Published pages the fetch stub serves, built fresh for every fetch as a parser would.
const PAGES = {
  'p-tmux.html': () => h('main', {},
    h('h1', {}, 'tmux'),
    h('ul', { class: 'outline' },
      li(null, 'See ', h('a', { href: 'p-pomodoro.html' }, 'Pomodoro')),
      li(null, 'Second ', h('a', { class: 'block-ref-link', href: 'p-terminal.html#block-abc-1' }, 'ref'), h('ul', {}, li(null, 'nested'))),
      li(null, h('a', { href: 'p-missing.html' }, 'gone')))),
  'p-pomodoro.html': () => h('main', {},
    h('h1', {}, 'tmux/Idea/Pomodoro'),
    h('ul', { class: 'outline' },
      li(null, h('h2', {}, 'Pomodoro in tmux'),
        h('ul', {}, li(null, 'first child'), li(null, 'second child'))),
      li(null, 'closing')),
    h('section', { class: 'references linked-references', 'data-refs': 'linked' }, h('details', { class: 'references-section', open: '' }, h('summary', {}, h('h2', {}, '2 Linked References')))),
    h('section', { class: 'references linked-references unlinked-references', 'data-refs': 'unlinked' }, h('details', {}, h('summary', {}, 'Unlinked References')))),
  'p-terminal.html': () => h('main', {},
    h('h1', {}, '01 Terminal'),
    h('ul', { class: 'outline' }, li('abc-1', 'Referenced block', h('ul', {}, li(null, 'its child'))))),
  'p-contents.html': () => h('main', {}, h('h1', {}, 'Contents'), h('ul', { class: 'outline' }, li(null))),
};

let state;
function load({ desktop = true, stored = {}, contents = null, defaults = null } = {}) {
  state = { focused: null, fetched: [], assigned: null };
  const storage = new Map(Object.entries(stored));
  const toggle = h('button', { 'data-lsg-right-toggle': '', ...(contents ? { 'data-lsg-contents': contents } : {}), ...(defaults ? { 'data-lsg-sidebar-default': defaults } : {}) });
  const main = PAGES['p-tmux.html']();
  const body = h('body', {}, h('header', {}, toggle), main);
  const document = makeDocument(body);
  const listeners = {};
  document.addEventListener = (type, callback) => (listeners[type] ||= []).push(callback);
  document.createElement = tag => new Element(tag);
  document.createElementNS = (_, tag) => new Element(tag);
  Object.defineProperty(document, 'activeElement', { get: () => state.focused });
  document.currentScript = { src: 'https://garden.test/lsg-sidebar.js' };
  const window = {
    innerWidth: 1280,
    matchMedia: () => ({ matches: desktop }),
    localStorage: { getItem: key => storage.get(key) ?? null, setItem: (key, value) => storage.set(key, String(value)) },
  };
  const context = {
    console, window, document, URL, JSON, Map, Set, Promise, Number, String, Math, Object, Array,
    location: { pathname: '/p-tmux.html', hash: '', href: 'https://garden.test/p-tmux.html', assign: url => { state.assigned = String(url); } },
    fetch: url => {
      const route = new URL(url).pathname.slice(1);
      state.fetched.push(route);
      return Promise.resolve(PAGES[route] ? { ok: true, text: () => Promise.resolve(route) } : { ok: false });
    },
    DOMParser: class { parseFromString(route) { return makeDocument(h('body', {}, PAGES[route]())); } },
  };
  vm.runInNewContext(source, context);
  const aside = body.querySelector('.lsg-right-sidebar');
  const fire = (node, type, event = {}) => {
    const all = { target: node, preventDefault() {}, stopPropagation() {}, ...event };
    for (let el = node; el; el = el.parentElement) for (const callback of el.listeners?.[type] || []) callback(all);
    for (const callback of listeners[type] || []) callback(all);
  };
  return {
    window, document, main, aside, toggle, storage, fire,
    request: value => window.logseqGardenerSidebar(value),
    shift: node => window.logseqGardenerSidebar({ target: node, shiftKey: true }),
    cards: () => aside.querySelectorAll('.lsg-card'),
    titles: () => aside.querySelectorAll('.lsg-card-title').map(title => title.textContent),
    saved: () => JSON.parse(storage.get('logseq-gardener-right-sidebar')),
  };
}
const settle = () => new Promise(resolve => setTimeout(resolve, 0));

test('nothing is fetched before use and an empty sidebar shows Contents', async () => {
  const s = load();
  assert.equal(s.aside.hidden, true);
  assert.deepEqual(state.fetched, []);
  s.request('toggle');
  await settle();
  assert.equal(s.document.documentElement.getAttribute('data-lsg-right-sidebar'), 'open');
  assert.equal(s.toggle.getAttribute('aria-expanded'), 'true');
  assert.deepEqual(s.titles(), ['Contents']);
  assert.equal(s.cards()[0].querySelectorAll('.outline .bullet').length, 1);
  assert.deepEqual(state.fetched, []);
  s.request('toggle');
  assert.equal(s.aside.hidden, true);
});

test('Shift-click adds page and block cards on top without leaving the page', async () => {
  const s = load();
  s.shift(s.main.querySelector('a[href^="p-"]'));
  await settle();
  const pageCard = s.cards()[0];
  assert.deepEqual(s.titles(), ['tmux/Idea/Pomodoro']);
  assert.equal(pageCard.querySelector('h1'), null);
  assert.equal(pageCard.querySelector('.unlinked-references'), null);
  assert.equal(pageCard.querySelector('.linked-references>details').open, false);
  assert.equal(pageCard.querySelector('.outline h2').textContent, 'Pomodoro in tmux');
  const second = s.main.querySelectorAll('.outline>li')[1];
  s.shift(second.querySelector('.bullet'));
  await settle();
  assert.equal(s.cards().length, 2);
  assert.equal(s.cards()[1], pageCard);
  const crumb = s.cards()[0].querySelector('.lsg-card-title .lsg-breadcrumb');
  assert.equal(crumb.querySelector('a').textContent, 'tmux');
  assert.deepEqual(s.saved().cards.map(card => [card.type, card.route, card.block]), [['block', 'p-tmux.html', '2'], ['page', 'p-pomodoro.html', undefined]]);
  assert.equal(state.assigned, null);
});

test('a bullet inside a card adds that block, and a block reference adds Block references', async () => {
  const s = load();
  s.shift(s.main.querySelector('a[href^="p-"]'));
  await settle();
  const child = s.cards()[0].querySelectorAll('.outline li')[2];
  assert.equal(child.textContent.trim(), 'second child');
  s.shift(child.querySelector('.bullet'));
  await settle();
  assert.deepEqual(s.saved().cards[0], { type: 'block', route: 'p-pomodoro.html', block: '1.2', collapsed: false });
  s.shift(s.main.querySelector('.block-ref-link'));
  await settle();
  assert.equal(s.titles()[0], 'Block references');
  assert.match(s.cards()[0].querySelector('.lsg-card-body').textContent, /Referenced block\s*its child/);
  assert.equal(s.cards()[0].querySelector('.lsg-breadcrumb a').textContent, '01 Terminal');
});

test('a card already present moves to the top, expanded', async () => {
  const s = load();
  const link = s.main.querySelector('a[href^="p-"]');
  s.shift(link);
  s.shift(s.main.querySelectorAll('.bullet')[2]);
  await settle();
  const pageCard = s.cards()[1];
  s.fire(pageCard.querySelector('.lsg-card-head'), 'click');
  assert.equal(pageCard.querySelector('.lsg-card-head').getAttribute('aria-expanded'), 'false');
  assert.equal(pageCard.querySelector('.lsg-card-body').children.length, 0);
  s.shift(link);
  await settle();
  assert.equal(s.cards()[0], pageCard);
  assert.equal(s.cards().length, 2);
  assert.equal(pageCard.querySelector('.lsg-card-head').getAttribute('aria-expanded'), 'true');
  assert.ok(pageCard.querySelector('.outline'));
});

test('the card menu lists Logseq items by card count and works from the keyboard', async () => {
  const s = load();
  s.shift(s.main.querySelector('a[href^="p-"]'));
  s.shift(s.main.querySelectorAll('.bullet')[0]);
  s.shift(s.main.querySelectorAll('.bullet')[1]);
  await settle();
  const button = s.cards()[0].querySelector('.lsg-card-menu');
  s.fire(button, 'click');
  const menu = s.aside.querySelector('[role=menu]');
  const labels = menu.querySelectorAll('[role=menuitem]').map(item => item.textContent);
  assert.deepEqual(labels, ['Close', 'Close others', 'Close all', 'Collapse', 'Collapse others', 'Collapse all', 'Expand all']);
  assert.equal(menu.querySelectorAll('hr').length, 1);
  assert.equal(state.focused.textContent, 'Close');
  s.fire(state.focused, 'keydown', { key: 'ArrowDown' });
  assert.equal(state.focused.textContent, 'Close others');
  s.fire(state.focused, 'keydown', { key: 'Escape' });
  assert.equal(s.aside.querySelector('[role=menu]'), null);
  assert.equal(state.focused, button);
  const pageButton = s.cards()[2].querySelector('.lsg-card-menu');
  s.fire(pageButton, 'click');
  const items = s.aside.querySelectorAll('[role=menuitem]');
  assert.equal(items.at(-1).textContent, 'Open as page');
  s.fire(items.find(item => item.textContent === 'Close all'), 'click');
  assert.equal(s.cards().length, 0);
  assert.equal(s.aside.hidden, true);
  assert.equal(state.focused, s.toggle);
});

test('a single card offers only its own items', async () => {
  const s = load();
  s.shift(s.main.querySelectorAll('.bullet')[0]);
  await settle();
  s.fire(s.cards()[0].querySelector('.lsg-card-menu'), 'click');
  assert.deepEqual(s.aside.querySelectorAll('[role=menuitem]').map(item => item.textContent), ['Close', 'Collapse']);
});

test('the sidebar survives a reload with its cards, order, collapsed flags and width', async () => {
  const s = load();
  s.shift(s.main.querySelector('a[href^="p-"]'));
  s.shift(s.main.querySelectorAll('.bullet')[1]);
  await settle();
  s.fire(s.cards()[1].querySelector('.lsg-card-head'), 'click');
  s.fire(s.aside.querySelector('[role=separator]'), 'keydown', { key: 'ArrowLeft' });
  const saved = s.saved();
  assert.equal(saved.width, 517);
  const again = load({ stored: Object.fromEntries(s.storage) });
  again.request('restore');
  await settle();
  assert.deepEqual(again.saved(), saved);
  assert.equal(again.cards()[1].querySelector('.lsg-card-head').getAttribute('aria-expanded'), 'false');
  assert.equal(again.document.documentElement.getAttribute('data-lsg-right-sidebar'), 'open');
});

test('configured start cards apply only without saved state', async () => {
  const s = load({ defaults: 'contents p-pomodoro.html', contents: 'p-contents.html' });
  s.request('restore');
  await settle();
  assert.deepEqual(s.titles(), ['Contents', 'tmux/Idea/Pomodoro']);
  assert.deepEqual(state.fetched, ['p-contents.html', 'p-pomodoro.html']);
  const stored = { 'logseq-gardener-right-sidebar': JSON.stringify({ open: false, width: 0, cards: [] }) };
  const t = load({ defaults: 'contents', stored });
  t.request('restore');
  assert.equal(t.aside.hidden, true);
});

test('alt+shift+c toggles the Contents card and Help lists only working shortcuts', async () => {
  const s = load({ contents: 'p-contents.html' });
  s.request('contents');
  await settle();
  assert.deepEqual(s.titles(), ['Contents']);
  s.request('contents');
  assert.equal(s.aside.hidden, true);
  s.fire(s.aside.querySelector('[data-add=help]'), 'click');
  await settle();
  const help = s.cards()[0];
  const keys = help.querySelectorAll('dt').map(dt => dt.textContent);
  assert.ok(keys.includes('tr') && keys.includes('altshiftc'));
  assert.ok(!keys.includes('gj') && !keys.includes('modk'));
  assert.equal(help.querySelector('a'), null);
});

test('an unpublished page says so and offers Close', async () => {
  const s = load();
  s.shift(s.main.querySelectorAll('a[href^="p-"]').at(-1));
  await settle();
  assert.match(s.cards()[0].textContent, /This page is not published\./);
  s.fire(s.cards()[0].querySelector('.lsg-card-dismiss'), 'click');
  assert.equal(s.cards().length, 0);
});

test('phones keep the page: requests are ignored', () => {
  const s = load({ desktop: false });
  s.request('toggle');
  s.shift(s.main.querySelector('a[href^="p-"]'));
  assert.equal(s.aside.hidden, true);
  assert.deepEqual(state.fetched, []);
});
