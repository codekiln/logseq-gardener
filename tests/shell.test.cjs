const { readFileSync } = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const test = require('node:test');
const assert = require('node:assert/strict');
const source = readFileSync(path.join(__dirname, '../crates/logseq-gardener-sdk/src/site/shell.js'), 'utf8');
const { h } = require('./mini-dom.cjs');
class Events {
  constructor() { this.listeners = new Map(); }
  addEventListener(type, callback) {
    if (!this.listeners.has(type)) this.listeners.set(type, []);
    this.listeners.get(type).push(callback);
  }
  fire(type, event = {}) { for (const callback of this.listeners.get(type) || []) callback(event); }
}
function element(selectors, extra = {}) {
  const attributes = new Map();
  const node = {
    hidden: true, selectors, parent: null,
    setAttribute: (name, value) => attributes.set(name, String(value)),
    getAttribute: name => attributes.get(name) ?? null,
    removeAttribute: name => attributes.delete(name),
    matches: selector => selector.split(',').some(part => selectors.includes(part.trim())),
    closest(selector) { for (let at = this; at; at = at.parent) if (at.matches(selector)) return at; return null; },
    focus() { state.focused = this; },
    // A hidden item, or one CSS hides at phone width, has no boxes.
    getClientRects() { return this.hidden || this.phoneHidden?.() ? [] : [{}]; },
    ...extra,
  };
  return node;
}
let state;
function fixture({ desktop = true, stored = {}, blocked = false, appearance = 'light', choice = 'system', journals = false, feed = false, defaults = false, saved = null, graph = true, slides = false } = {}) {
  state = { focused: null, assigned: null };
  const values = new Map(Object.entries(stored));
  const root = element(['html']);
  const toggle = element(['[data-lsg-left-toggle]']);
  const moreBox = element(['.lsg-more']); moreBox.hidden = false;
  const more = element(['[data-lsg-more]']); more.parent = moreBox;
  const menu = element(['#lsg-more-menu']); menu.parent = moreBox;
  const themeItem = element(['[data-lsg-theme-toggle]', '[role="menuitem"]']); themeItem.hidden = false; themeItem.parent = menu;
  const systemItem = element(['[data-lsg-theme-system]', '[role="menuitem"]']); systemItem.parent = menu;
  const slidesItem = element(['[data-lsg-slides]', '[role="menuitem"]']); slidesItem.hidden = false; slidesItem.parent = menu;
  slidesItem.phoneHidden = () => !desktop;
  menu.querySelectorAll = () => slides ? [slidesItem, themeItem, systemItem] : [themeItem, systemItem];
  const shade = element(['[data-lsg-sidebar-shade]']);
  const sidebar = element(['.lsg-left-sidebar']);
  const link = element(['.lsg-left-sidebar a']); link.parent = sidebar;
  const allPages = element(['[data-lsg-all-pages]', '.lsg-left-sidebar a']); allPages.parent = sidebar; allPages.setAttribute('href', 'pages.html');
  const input = element(['input']);
  const page = element(['p']);
  const right = element(['[data-lsg-right-toggle]']);
  // A page reference at a readable address, as a host such as Gitpa rewrites it.
  const pageLink = h('a', { href: '/gitpa/GitP/A/Session/26/10/29-Thu/' });
  h('main', {}, h('span', { class: 'page-reference' }, pageLink));
  const all = [toggle, more, menu, themeItem, systemItem, shade, allPages, right];
  if (slides) all.push(slidesItem);
  if (defaults) { right.setAttribute('data-lsg-sidebar-default', 'contents'); all.push(element(['[data-lsg-sidebar-default]'])); }
  if (saved) root.setAttribute('data-lsg-right-sidebar', saved);
  if (journals) {
    const journalsLink = element(['[data-lsg-journals-link]', '.lsg-left-sidebar a']); journalsLink.parent = sidebar; journalsLink.setAttribute('href', 'journals.html');
    all.push(journalsLink);
  }
  if (graph) {
    const graphLink = element(['[data-lsg-graph-shortcut]', '.lsg-left-sidebar a']); graphLink.parent = sidebar; graphLink.setAttribute('href', 'graph.html');
    all.push(graphLink);
  }
  if (feed) {
    const list = element(['[data-lsg-journals]']); list.setAttribute('data-lsg-journals', 'lsg-journals.js');
    all.push(list);
  }
  const document = new Events();
  document.documentElement = root;
  document.readyState = 'loading';
  state.appended = [];
  document.createElement = tag => ({ tag });
  document.head = { append: node => state.appended.push(node) };
  document.querySelector = selector => all.find(node => node.matches(selector)) ?? null;
  document.querySelectorAll = selector => all.filter(node => node.matches(selector));
  Object.defineProperty(document, 'activeElement', { get: () => state.focused });
  const window = new Events();
  Object.defineProperty(window, 'localStorage', { get() {
    if (blocked) throw new Error('Storage unavailable');
    return { getItem: k => values.get(k) ?? null, setItem: (k, v) => values.set(k, v), removeItem: k => values.delete(k) };
  } });
  window.matchMedia = () => ({ matches: desktop });
  window.location = { assign: href => { state.assigned = href; } };
  const theme = { current: choice, shown: appearance };
  window.logseqGardenerTheme = {
    choice: () => theme.current,
    appearance: () => theme.shown,
    set: value => { theme.current = value; if (value !== 'system') theme.shown = value; },
  };
  state.sidebar = [];
  window.logseqGardenerSidebar = request => state.sidebar.push(request);
  vm.runInContext(source, vm.createContext({ window, document, Date, Promise }));
  const click = (target, extra = {}) => document.fire('click', { target, preventDefault() {}, stopPropagation() {}, ...extra });
  const key = (k, target = page, extra = {}) => document.fire('keydown', { key: k, target, preventDefault() {}, ...extra });
  return { slidesItem, right, pageLink, window, root, toggle, more, menu, themeItem, systemItem, shade, link, allPages, input, page, values, theme, click, key, document };
}
test('controls appear once the script runs and the sidebar state is saved', () => {
  const f = fixture();
  assert.equal(f.toggle.hidden, false); assert.equal(f.more.hidden, false);
  f.click(f.toggle);
  assert.equal(f.root.getAttribute('data-lsg-left-sidebar'), 'open');
  assert.equal(f.toggle.getAttribute('aria-expanded'), 'true');
  assert.equal(f.values.get('logseq-gardener-left-sidebar'), 'open');
  assert.equal(f.shade.hidden, true);
  f.key('t'); f.key('l');
  assert.equal(f.root.getAttribute('data-lsg-left-sidebar'), null);
  assert.equal(f.values.has('logseq-gardener-left-sidebar'), false);
});
test('g g opens the graph view unless the garden turns the shortcut off', () => {
  const f = fixture();
  f.key('g'); f.key('g'); assert.equal(state.assigned, 'graph.html');
  const off = fixture({ graph: false });
  off.key('g'); off.key('g'); assert.equal(state.assigned, null);
});
test('search commands run the shell actions', () => {
  const f = fixture();
  const { actions } = f.window.logseqGardenerShell;
  assert.deepEqual(Object.keys(actions).sort(), ['ga', 'gf', 'gg', 'gh', 'gj', 'tc', 'tl', 'tr', 'tt', 'tw']);
  actions.tl(); assert.equal(f.root.getAttribute('data-lsg-left-sidebar'), 'open');
  actions.tw(); assert.equal(f.root.getAttribute('data-lsg-wide'), 'true');
  assert.ok(Object.isFrozen(actions));
});
test('phone overlay closes from the shade, a link and Escape', () => {
  const f = fixture({ desktop: false });
  f.click(f.toggle); assert.equal(f.shade.hidden, false);
  f.click(f.shade); assert.equal(f.root.getAttribute('data-lsg-left-sidebar'), null); assert.equal(f.document.activeElement, f.toggle);
  f.click(f.toggle); f.click(f.link); assert.equal(f.root.getAttribute('data-lsg-left-sidebar'), null);
  f.click(f.toggle); f.key('Escape'); assert.equal(f.root.getAttribute('data-lsg-left-sidebar'), null);
});
test('More opens with focus on Toggle theme, moves with arrows and closes on Escape', () => {
  const f = fixture({ choice: 'dark', appearance: 'dark' });
  f.click(f.more);
  assert.equal(f.menu.hidden, false); assert.equal(f.more.getAttribute('aria-expanded'), 'true');
  assert.equal(f.systemItem.hidden, false); assert.equal(f.document.activeElement, f.themeItem);
  f.key('ArrowDown'); assert.equal(f.document.activeElement, f.systemItem);
  f.key('ArrowDown'); assert.equal(f.document.activeElement, f.themeItem);
  f.key('Escape'); assert.equal(f.menu.hidden, true); assert.equal(f.document.activeElement, f.more);
  f.click(f.more); f.click(f.page); assert.equal(f.menu.hidden, true);
});
test('View as slides comes first in More and asks the right sidebar for a Slides card', async () => {
  const f = fixture({ slides: true });
  f.click(f.more); assert.equal(f.document.activeElement, f.slidesItem);
  f.key('ArrowDown'); assert.equal(f.document.activeElement, f.themeItem);
  f.key('ArrowUp'); f.click(f.slidesItem);
  assert.equal(f.menu.hidden, true);
  // The deck takes focus, so focus does not return to More.
  assert.notEqual(f.document.activeElement, f.more);
  assert.ok(loaded());
  await settle();
  assert.equal(JSON.stringify(state.sidebar), '[{"type":"slides"}]');
});
test('below 640px More skips View as slides', () => {
  const f = fixture({ slides: true, desktop: false });
  f.click(f.more); assert.equal(f.document.activeElement, f.themeItem);
});
test('Toggle theme switches appearance and Use system theme clears the choice', () => {
  const f = fixture({ appearance: 'dark' });
  f.click(f.more); assert.equal(f.systemItem.hidden, true);
  f.click(f.themeItem); assert.equal(f.theme.current, 'light'); assert.equal(f.menu.hidden, true);
  f.click(f.more); assert.equal(f.systemItem.hidden, false);
  f.click(f.systemItem); assert.equal(f.theme.current, 'system');
  f.key('t'); f.key('t'); assert.equal(f.theme.current, 'dark');
});
test('shortcuts navigate, toggle wide mode and ignore fields and modifiers', () => {
  const f = fixture();
  f.key('t'); f.key('w'); assert.equal(f.root.getAttribute('data-lsg-wide'), 'true'); assert.equal(f.values.get('logseq-gardener-wide'), 'true');
  f.key('g'); f.key('a'); assert.equal(state.assigned, 'pages.html');
  f.key('g'); f.key('h'); assert.equal(state.assigned, 'index.html');
  state.assigned = null;
  f.key('g', f.input); f.key('a', f.input); assert.equal(state.assigned, null);
  f.key('g'); f.key('a', f.page, { metaKey: true }); assert.equal(state.assigned, null);
});
test('blocked storage keeps the state for the current page', () => {
  const f = fixture({ blocked: true });
  f.click(f.toggle); assert.equal(f.root.getAttribute('data-lsg-left-sidebar'), 'open');
  f.key('t'); f.key('w'); assert.equal(f.root.getAttribute('data-lsg-wide'), 'true');
});
test('g j opens the journals feed only when the sidebar links to one', () => {
  const f = fixture({ journals: true });
  f.key('g'); f.key('j'); assert.equal(state.assigned, 'journals.html');
  const plain = fixture();
  plain.key('g'); plain.key('j'); assert.equal(state.assigned, null);
});
test('the journals feed, outline and references scripts are fetched after the page loads', () => {
  const f = fixture({ feed: true });
  assert.deepEqual(state.appended, []);
  f.window.fire('load');
  assert.deepEqual(state.appended.map(({ tag, src }) => ({ tag, src })), [{ tag: 'script', src: 'lsg-journals.js' }, { tag: 'script', src: 'lsg-outline.js' }, { tag: 'script', src: 'lsg-references.js' }]);
  const plain = fixture();
  assert.deepEqual(state.appended, []);
  plain.window.fire('load');
  assert.deepEqual(state.appended.map(({ tag, src }) => ({ tag, src })), [{ tag: 'script', src: 'lsg-outline.js' }, { tag: 'script', src: 'lsg-references.js' }]);
});
const settle = () => new Promise(resolve => setTimeout(resolve, 0));
const loaded = () => { const script = state.appended.find(node => node.src === 'lsg-sidebar.js'); script?.onload(); return script; };
test('the right sidebar loads once, on its first use', async () => {
  const f = fixture();
  assert.equal(f.right.hidden, false);
  f.click(f.page, { shiftKey: true });
  f.click(f.pageLink);
  assert.deepEqual(state.appended, []);
  const shift = { target: f.pageLink, shiftKey: true, preventDefault() {}, stopPropagation() {} };
  f.document.fire('click', shift);
  assert.ok(loaded());
  f.click(f.right);
  f.key('t'); f.key('r');
  f.key('C', f.page, { code: 'KeyC', altKey: true, shiftKey: true });
  f.key('C', f.input, { code: 'KeyC', altKey: true, shiftKey: true });
  await settle();
  assert.deepEqual(state.sidebar, [shift, 'toggle', 'toggle', 'contents']);
  assert.equal(state.appended.filter(node => node.src === 'lsg-sidebar.js').length, 1);
});
test('phones never load the right sidebar', async () => {
  const f = fixture({ desktop: false, saved: 'open' });
  f.click(f.pageLink, { shiftKey: true });
  f.key('t'); f.key('r');
  f.window.fire('load');
  await settle();
  assert.equal(state.appended.some(node => node.src === 'lsg-sidebar.js'), false);
});
test('a saved open sidebar or configured start cards load the sidebar after the page loads', async () => {
  for (const options of [{ saved: 'open' }, { defaults: true }]) {
    fixture(options).window.fire('load');
    assert.ok(loaded());
    await settle();
    assert.deepEqual(state.sidebar, ['restore']);
  }
  for (const options of [{ saved: 'closed', defaults: true }, {}]) {
    fixture(options).window.fire('load');
    assert.equal(loaded(), undefined);
  }
});
test('Shift-click reaches the right sidebar for page links by their markup, whatever their address', async () => {
  const f = fixture();
  const at = '/gitpa/GitP/A/Session/26/10/29-Thu/';
  const wanted = [
    h('a', { href: 'p-tmux.html' }),
    h('a', { href: at + '#zoom/1.2' }),
    h('a', { class: 'block-ref-link', href: '/gitpa/Terminal/#block-abc-1' }),
    h('span', { class: 'page-reference' }, h('a', { href: at })),
    h('a', { class: 'page-ref', href: '/gitpa/GitP/A/' }),
    h('a', { class: 'tag', href: '/gitpa/Microfreak/' }),
    h('a', { class: 'bullet', href: '#' }),
  ];
  const others = [h('a', { href: '/gitpa/GitP/A/' }), h('a', { class: 'external-link', href: 'https://example.com/' })];
  h('main', {}, ...wanted, ...others);
  const left = h('aside', { class: 'lsg-left-sidebar' }, h('span', { class: 'page-reference' }, h('a', { href: at })));
  const card = h('aside', { class: 'lsg-right-sidebar' }, h('a', { class: 'page-ref', href: 'https://garden.test/gitpa/GitP/A/' }));
  const targets = [...wanted.map(node => node.tagName === 'SPAN' ? node.children[0] : node), card.children[0]];
  const stopped = [];
  const shift = target => f.click(target, { shiftKey: true, preventDefault() { stopped.push(target); } });
  for (const target of [...targets, ...others, left.querySelector('a')]) shift(target);
  for (const target of targets) f.click(target);
  assert.ok(loaded());
  await settle();
  assert.deepEqual(state.sidebar.map(event => event.target), targets);
  assert.deepEqual(stopped, targets);
});
