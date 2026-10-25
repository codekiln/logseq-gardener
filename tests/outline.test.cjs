const { readFileSync } = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const test = require('node:test');
const assert = require('node:assert/strict');
const source = readFileSync(path.join(__dirname, '../crates/logseq-gardener-sdk/src/site/outline.js'), 'utf8');

// A small DOM: enough elements, text and selectors for the outline script.
function compound(text) {
  const parts = { tag: null, classes: [], ids: [], prefixes: [], scope: false };
  for (const [token] of text.matchAll(/:scope|\.[\w-]+|#[\w-]+|\[[\w-]+\^="[^"]*"\]|[\w-]+/g)) {
    if (token === ':scope') parts.scope = true;
    else if (token[0] === '.') parts.classes.push(token.slice(1));
    else if (token[0] === '#') parts.ids.push(token.slice(1));
    else if (token[0] === '[') { const [, name, value] = token.match(/\[([\w-]+)\^="([^"]*)"\]/); parts.prefixes.push([name, value]); }
    else parts.tag = token.toUpperCase();
  }
  return parts;
}
function matchesCompound(el, parts, scope) {
  if (!el || el.nodeType !== 1) return false;
  if (parts.scope && el !== scope) return false;
  if (parts.tag && el.tagName !== parts.tag) return false;
  if (!parts.classes.every((c) => el.classList.contains(c))) return false;
  if (!parts.ids.every((id) => el.id === id)) return false;
  return parts.prefixes.every(([name, value]) => (el.getAttribute(name) ?? '').startsWith(value));
}
function matches(el, selector, scope) {
  return selector.split(',').some((one) => {
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
  get isConnected() { let n = this; while (n.parentElement) n = n.parentElement; return n === doc.root; }
  remove() { if (this.parentElement) { const list = this.parentElement.childNodes; list.splice(list.indexOf(this), 1); this.parentElement = null; } }
  replaceWith(other) { const p = this.parentElement, list = p.childNodes; other.remove(); list.splice(list.indexOf(this), 1, other); other.parentElement = p; this.parentElement = null; }
}
class Text extends Node { constructor(text) { super(); this.nodeType = 3; this.text = text; } get textContent() { return this.text; } cloneNode() { return new Text(this.text); } }
class Comment extends Node { constructor() { super(); this.nodeType = 8; } get textContent() { return ''; } cloneNode() { return new Comment(); } }
class Element extends Node {
  constructor(tag, attrs = {}) {
    super();
    this.nodeType = 1; this.tagName = tag.toUpperCase(); this.attrs = new Map(Object.entries(attrs)); this.open = !!attrs.open; this.innerHTML = '';
    const el = this;
    this.classList = {
      contains: (c) => (el.attrs.get('class') ?? '').split(' ').includes(c),
      add: (c) => { if (!el.classList.contains(c)) el.attrs.set('class', ((el.attrs.get('class') ?? '') + ' ' + c).trim()); },
      remove: (c) => el.attrs.set('class', (el.attrs.get('class') ?? '').split(' ').filter((x) => x && x !== c).join(' ')),
      toggle: (c, force) => { const on = force ?? !el.classList.contains(c); if (on) el.classList.add(c); else el.classList.remove(c); return on; },
    };
  }
  get id() { return this.attrs.get('id') ?? ''; }
  get className() { return this.attrs.get('class') ?? ''; }
  set className(v) { this.attrs.set('class', v); }
  get children() { return this.childNodes.filter((n) => n.nodeType === 1); }
  get textContent() { return this.childNodes.map((n) => n.textContent).join(''); }
  set textContent(v) { this.childNodes = []; this.append(String(v)); }
  getAttribute(n) { return this.attrs.has(n) ? this.attrs.get(n) : null; }
  setAttribute(n, v) { this.attrs.set(n, String(v)); }
  removeAttribute(n) { this.attrs.delete(n); }
  hasAttribute(n) { return this.attrs.has(n); }
  get href() { return this.getAttribute('href'); }
  set href(v) { this.setAttribute('href', v); }
  append(...nodes) { for (let n of nodes) { if (typeof n === 'string') n = new Text(n); n.remove(); n.parentElement = this; this.childNodes.push(n); } }
  prepend(...nodes) { for (let n of nodes.reverse()) { if (typeof n === 'string') n = new Text(n); n.remove(); n.parentElement = this; this.childNodes.unshift(n); } }
  contains(n) { for (; n; n = n.parentElement) if (n === this) return true; return false; }
  closest(s) { for (let n = this; n; n = n.parentElement) if (matches(n, s)) return n; return null; }
  descendants() { return this.children.flatMap((c) => [c, ...c.descendants()]); }
  querySelectorAll(s) { return this.descendants().filter((n) => matches(n, s, this)); }
  querySelector(s) { return this.querySelectorAll(s)[0] ?? null; }
  cloneNode() { const c = new Element(this.tagName, Object.fromEntries(this.attrs)); for (const n of this.childNodes) c.append(n.cloneNode(true)); return c; }
}
let doc;
const h = (tag, attrs, ...kids) => { const el = new Element(tag, attrs); el.append(...kids); return el; };
const bullet = () => h('a', { class: 'bullet', href: '#' });
const item = (text, ...kids) => h('li', {}, bullet(), text, ...(kids.length ? [h('ul', {}, ...kids)] : []));

function page(main, hash = '') {
  const listeners = { document: {}, window: {} };
  const root = h('html', {}, h('body', {}, main));
  const location = { hash, pathname: '/site/p-page.html', href: '' };
  const document = {
    createElement: (tag) => new Element(tag),
    createComment: () => new Comment(),
    addEventListener: (type, fn) => { listeners.document[type] = fn; },
    querySelector: (s) => root.querySelector(s),
    getElementById: (id) => root.querySelectorAll('span').find((n) => n.id === id) ?? null,
  };
  const window = { addEventListener: (type, fn) => { listeners.window[type] = fn; }, scrollTo() {} };
  doc = { root };
  vm.runInNewContext(source, { document, window, location });
  const go = (next) => { location.hash = next; listeners.window.hashchange(); };
  const click = (target, extra = {}) => { const event = { target, prevented: false, offsetX: 99, preventDefault() { this.prevented = true; }, ...extra }; listeners.document.click(event); return event; };
  const hover = (target) => listeners.document.mouseover({ target });
  return { main, location, go, click, hover };
}

function fixture() {
  const deep = item('Deep');
  const leaf = item('Leaf');
  const collapsed = h('li', { class: 'outline-collapsed' }, h('details', { class: 'outline-disclosure' }, h('summary', {}, bullet(), 'Closed'), h('ul', {}, item('Hidden child'))));
  const nested = item('Level four', deep);
  const b = h('li', {}, h('span', { id: 'block-6add3ffb-e52b-4e8f-908a-9958186e4dfb' }), bullet(), 'Level three', h('ul', {}, nested));
  const two = item('Level two', b);
  const one = item('One', two, leaf);
  const main = h('main', {}, h('h1', {}, 'rw-cli'), h('section', { class: 'linked-references' }), h('ul', { class: 'outline' }, one, collapsed));
  return { main, one, two, b, nested, deep, leaf, collapsed };
}

test('a bullet zooms by position path and the breadcrumb lists the page and nearest ancestors', () => {
  const f = fixture();
  const state = page(f.main);
  const event = state.click(f.deep.children[0]);
  assert.equal(event.prevented, true);
  assert.equal(state.location.hash, 'zoom/1.1.1.1.1');
  state.go('#zoom/1.1.1.1.1');
  assert.equal(f.main.hasAttribute('data-lsg-zoomed'), true);
  const view = f.main.querySelector('.lsg-zoom');
  assert.equal(view.querySelector('ul.outline').children[0], f.deep);
  const crumb = view.querySelector('.lsg-breadcrumb');
  assert.equal(crumb.textContent, 'rw-cli›⋯›Level two›Level three›Level four');
  const links = crumb.querySelectorAll('a').map((a) => a.href);
  assert.deepEqual(links, ['p-page.html', '#zoom/1.1', '#zoom/6add3ffb-e52b-4e8f-908a-9958186e4dfb', '#zoom/1.1.1.1']);
});

test('a block with an id:: anchor zooms by UUID and opens everything inside it', () => {
  const f = fixture();
  const state = page(f.main);
  f.nested.classList.add('folded');
  state.click(f.b.querySelector('.bullet'));
  assert.equal(state.location.hash, 'zoom/6add3ffb-e52b-4e8f-908a-9958186e4dfb');
  state.go('#zoom/6ADD3FFB-E52B-4E8F-908A-9958186E4DFB');
  assert.equal(f.main.querySelector('.lsg-zoom ul.outline').children[0], f.b);
  assert.equal(f.nested.classList.contains('folded'), false);
  state.go('');
  assert.equal(f.main.hasAttribute('data-lsg-zoomed'), false);
  assert.equal(f.main.querySelector('.lsg-zoom'), null);
  assert.equal(f.two.querySelector(':scope>ul').children[0], f.b);
  assert.equal(f.nested.classList.contains('folded'), true);
});

test('a page opened at a zoom address zooms, and an unknown address keeps the page view', () => {
  const f = fixture();
  page(f.main, '#zoom/2');
  assert.equal(f.main.querySelector('.lsg-zoom ul.outline').children[0], f.collapsed);
  assert.equal(f.collapsed.querySelector('details').open, true);
  const g = fixture();
  page(g.main, '#zoom/9.9');
  assert.equal(g.main.hasAttribute('data-lsg-zoomed'), false);
});

test('the arrow folds and unfolds ordinary and authored collapsed blocks', () => {
  const f = fixture();
  const state = page(f.main);
  state.hover(f.two);
  const arrow = f.two.querySelector('.lsg-arrow');
  assert.ok(arrow);
  assert.equal(arrow.getAttribute('aria-expanded'), 'true');
  state.click(arrow);
  assert.equal(f.two.classList.contains('folded'), true);
  assert.equal(arrow.getAttribute('aria-expanded'), 'false');
  state.click(arrow);
  assert.equal(f.two.classList.contains('folded'), false);
  state.hover(f.collapsed.querySelector('summary'));
  assert.equal(f.collapsed.querySelector('.lsg-arrow'), arrow);
  assert.equal(arrow.getAttribute('aria-expanded'), 'false');
  state.click(arrow);
  assert.equal(f.collapsed.querySelector('details').open, true);
  state.hover(f.leaf);
  assert.equal(f.leaf.querySelector('.lsg-arrow'), null);
});

test('the strip over a guide line toggles its block', () => {
  const f = fixture();
  const state = page(f.main);
  const list = f.one.querySelector(':scope>ul');
  state.click(list, { offsetX: 3 });
  assert.equal(f.one.classList.contains('folded'), true);
  state.click(list, { offsetX: 40 });
  assert.equal(f.one.classList.contains('folded'), true);
});

test('content swapped into main zooms without setup, and feed bullets open the journal page', () => {
  const f = fixture();
  const state = page(f.main);
  const replacement = fixture();
  f.main.replaceWith(replacement.main);
  state.go('#zoom/1.2');
  assert.equal(replacement.main.querySelector('.lsg-zoom ul.outline').children[0], replacement.leaf);
  const entry = item('Journal block');
  const article = h('article', { class: 'lsg-journal' }, h('h1', {}, h('a', { class: 'journal-title', href: 'p-day.html' }, 'Day')), h('ul', { class: 'outline' }, item('First'), entry));
  const feed = page(h('main', {}, article));
  feed.click(entry.querySelector('.bullet'));
  assert.equal(feed.location.href, 'p-day.html#zoom/2');
});
