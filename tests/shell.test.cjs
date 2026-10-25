const { readFileSync } = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const test = require('node:test');
const assert = require('node:assert/strict');
const source = readFileSync(path.join(__dirname, '../crates/logseq-gardener-sdk/src/site/shell.js'), 'utf8');
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
    ...extra,
  };
  return node;
}
let state;
function fixture({ desktop = true, stored = {}, blocked = false, appearance = 'light', choice = 'system', journals = false, feed = false } = {}) {
  state = { focused: null, assigned: null };
  const values = new Map(Object.entries(stored));
  const root = element(['html']);
  const toggle = element(['[data-lsg-left-toggle]']);
  const moreBox = element(['.lsg-more']); moreBox.hidden = false;
  const more = element(['[data-lsg-more]']); more.parent = moreBox;
  const menu = element(['#lsg-more-menu']); menu.parent = moreBox;
  const themeItem = element(['[data-lsg-theme-toggle]', '[role="menuitem"]']); themeItem.hidden = false; themeItem.parent = menu;
  const systemItem = element(['[data-lsg-theme-system]', '[role="menuitem"]']); systemItem.parent = menu;
  menu.querySelectorAll = () => [themeItem, systemItem];
  const shade = element(['[data-lsg-sidebar-shade]']);
  const sidebar = element(['.lsg-left-sidebar']);
  const link = element(['.lsg-left-sidebar a']); link.parent = sidebar;
  const allPages = element(['[data-lsg-all-pages]', '.lsg-left-sidebar a']); allPages.parent = sidebar; allPages.setAttribute('href', 'pages.html');
  const input = element(['input']);
  const page = element(['p']);
  const all = [toggle, more, menu, themeItem, systemItem, shade, allPages];
  if (journals) {
    const journalsLink = element(['[data-lsg-journals-link]', '.lsg-left-sidebar a']); journalsLink.parent = sidebar; journalsLink.setAttribute('href', 'journals.html');
    all.push(journalsLink);
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
  vm.runInContext(source, vm.createContext({ window, document, Date }));
  const click = target => document.fire('click', { target });
  const key = (k, target = page, extra = {}) => document.fire('keydown', { key: k, target, preventDefault() {}, ...extra });
  return { window, root, toggle, more, menu, themeItem, systemItem, shade, link, allPages, input, page, values, theme, click, key, document };
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
test('the journals feed loader is fetched after the page loads', () => {
  const f = fixture({ feed: true });
  assert.deepEqual(state.appended, []);
  f.window.fire('load');
  assert.deepEqual(state.appended, [{ tag: 'script', src: 'lsg-journals.js' }]);
  fixture();
  assert.deepEqual(state.appended, []);
});
