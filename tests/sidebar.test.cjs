const { readFileSync } = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const test = require('node:test');
const assert = require('node:assert/strict');
const source = readFileSync(path.join(__dirname, '../crates/logseq-gardener-sdk/src/site/sidebar.js'), 'utf8');

const { Element, h } = require('./mini-dom.cjs');
Element.onFocus = el => { state.focused = el; };
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
  // A host that serves the site root at /gitpa/ and each page at a readable route, as Gitpa does.
  'gitpa/GitP/A/Session/26/10/29-Thu/': () => h('main', {},
    h('h1', {}, 'GitP/A/Session/26/10/29-Thu'),
    h('ul', { class: 'outline' },
      li(null, 'After ', h('span', { class: 'page-reference' }, h('a', { href: '/gitpa/GitP/A/Session/26/10/28-Wed/' }, 'GitP/A/Session/26/10/28-Wed'))),
      li(null, h('a', { class: 'page-ref', href: '/gitpa/GitP/A/' }, 'GitP/A'), h('a', { class: 'tag', href: '/gitpa/Microfreak/' }, '#Microfreak'), h('a', { href: '/gitpa/Other/' }, 'plain')),
      li(null, 'See ', h('a', { class: 'block-ref-link', href: '/gitpa/Terminal/#block-abc-1' }, 'ref')),
      h('li', { class: 'lsg-card' }, h('a', { class: 'bullet', href: '#' }), 'Front ', h('span', { class: 'page-reference' }, h('a', { href: '/gitpa/GitP/A/' }, 'GitP/A'))))),
  'gitpa/GitP/A/Session/26/10/28-Wed/': () => h('main', {},
    h('h1', {}, 'GitP/A/Session/26/10/28-Wed'),
    h('ul', { class: 'outline' }, li(null, 'Patch', h('ul', {}, li(null, 'Filter'))))),
  'gitpa/GitP/A/': () => h('main', {}, h('h1', {}, 'GitP/A'), h('ul', { class: 'outline' }, li(null, 'Sessions'))),
  'gitpa/Microfreak/': () => h('main', {}, h('h1', {}, 'Microfreak'), h('ul', { class: 'outline' }, li(null, 'Synth'))),
};
PAGES['gitpa/Terminal/'] = PAGES['p-terminal.html'];

let state;
// at is the page's address and root the folder that serves lsg-sidebar.js.
function load({ desktop = true, stored = {}, contents = null, defaults = null, at = 'https://garden.test/p-tmux.html', root = 'https://garden.test/' } = {}) {
  state = { focused: null, fetched: [], assigned: null };
  const storage = new Map(Object.entries(stored));
  const toggle = h('button', { 'data-lsg-right-toggle': '', ...(contents ? { 'data-lsg-contents': contents } : {}), ...(defaults ? { 'data-lsg-sidebar-default': defaults } : {}) });
  const main = PAGES[new URL(at).pathname.slice(1)]();
  const body = h('body', {}, h('header', {}, toggle), main);
  const document = makeDocument(body);
  const listeners = {};
  document.addEventListener = (type, callback) => (listeners[type] ||= []).push(callback);
  document.createElement = tag => new Element(tag);
  document.createElementNS = (_, tag) => new Element(tag);
  Object.defineProperty(document, 'activeElement', { get: () => state.focused });
  document.currentScript = { src: root + 'lsg-sidebar.js' };
  document.baseURI = at;
  const window = {
    innerWidth: 1280,
    matchMedia: () => ({ matches: desktop }),
    localStorage: { getItem: key => storage.get(key) ?? null, setItem: (key, value) => storage.set(key, String(value)) },
  };
  const context = {
    console, window, document, URL, JSON, Map, Set, Promise, Number, String, Math, Object, Array,
    location: { pathname: new URL(at).pathname, hash: '', href: at, assign: url => { state.assigned = String(url); } },
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

test('View as slides adds a Slides card for the current page and focuses its deck', async () => {
  const s = load();
  const decks = [];
  s.window.logseqGardenerSlides = outline => {
    const deck = h('div', { class: 'reveal', tabindex: '0' });
    decks.push([deck, outline]);
    return h('div', { class: 'lsg-slides-card' }, deck);
  };
  s.request({ type: 'slides' });
  await settle();
  const scripts = s.document.head.querySelectorAll('script');
  assert.deepEqual(scripts.map(script => script.src), ['https://garden.test/lsg-slides.js']);
  scripts[0].onload();
  await settle();
  // The deck reads the live outline of the current page; nothing is fetched.
  assert.deepEqual(state.fetched, []);
  assert.equal(decks.length, 1);
  assert.equal(decks[0][1], s.main.querySelector('ul.outline'));
  const card = s.cards()[0];
  const link = card.querySelector('.lsg-card-title a');
  assert.equal(link.getAttribute('href'), 'https://garden.test/p-tmux.html');
  assert.equal(link.textContent, 'tmux');
  const deck = card.querySelector('.reveal');
  assert.equal(deck, decks[0][0]);
  assert.equal(state.focused, deck);
  assert.deepEqual(s.saved().cards, [{ type: 'slides', route: 'p-tmux.html', collapsed: false }]);
  s.fire(card.querySelector('.lsg-card-menu'), 'click');
  const items = s.aside.querySelectorAll('[role=menuitem]');
  assert.deepEqual(items.map(item => item.textContent), ['Close', 'Collapse', 'Open as page']);
  s.fire(items.at(-1), 'click');
  assert.equal(state.assigned, 'https://garden.test/p-tmux.html');
  // Choosing it again moves the card to the top, focuses the deck and loads the script once.
  s.shift(s.main.querySelector('a[href^="p-"]'));
  await settle();
  state.focused = null;
  s.request({ type: 'slides' });
  await settle();
  assert.equal(s.cards()[0], card);
  assert.equal(state.focused, deck);
  assert.equal(s.document.head.querySelectorAll('script').length, 1);
});

test('a restored Slides card for another page reads that page file', async () => {
  const s = load({ stored: { 'logseq-gardener-right-sidebar': JSON.stringify({ open: true, width: 0, cards: [{ type: 'slides', route: 'p-pomodoro.html' }] }) } });
  const decks = [];
  s.window.logseqGardenerSlides = outline => { decks.push(outline); return h('div', {}, h('div', { class: 'reveal' })); };
  s.request('restore');
  await settle();
  s.document.head.querySelector('script').onload();
  await settle();
  assert.deepEqual(state.fetched, ['p-pomodoro.html']);
  assert.equal(decks[0].children.length, 2);
  assert.equal(s.titles()[0], 'tmux/Idea/Pomodoro');
  // A restored card does not take focus from the page.
  assert.equal(state.focused, null);
});

test('a host that rewrites page addresses keeps Shift-click, card links and saved cards', async () => {
  const root = 'https://garden.test/gitpa/';
  const s = load({ at: root + 'GitP/A/Session/26/10/29-Thu/', root });
  const [first, second, third, flashcard] = s.main.querySelectorAll('.outline>li');
  // A page reference, a query result and a tag add cards; a plain link adds none.
  s.shift(first.querySelector('.page-reference a'));
  s.shift(second.querySelector('a.page-ref'));
  s.shift(second.querySelector('a.tag'));
  s.shift(second.querySelectorAll('a').at(-1));
  await settle();
  assert.deepEqual(s.titles(), ['Microfreak', 'GitP/A', 'GitP/A/Session/26/10/28-Wed']);
  assert.deepEqual(s.saved().cards.map(card => card.route), ['Microfreak/', 'GitP/A/', 'GitP/A/Session/26/10/28-Wed/']);
  assert.deepEqual(state.fetched, ['gitpa/GitP/A/Session/26/10/28-Wed/', 'gitpa/GitP/A/', 'gitpa/Microfreak/']);
  // Open as page leaves for the page's own address.
  const wednesday = s.cards()[2];
  s.fire(wednesday.querySelector('.lsg-card-menu'), 'click');
  s.fire(s.aside.querySelectorAll('[role=menuitem]').at(-1), 'click');
  assert.equal(state.assigned, root + 'GitP/A/Session/26/10/28-Wed/');
  // A block card from inside that card links its crumbs to the same address.
  s.shift(wednesday.querySelectorAll('.outline li')[1].querySelector('.bullet'));
  await settle();
  assert.deepEqual(s.saved().cards[0], { type: 'block', route: 'GitP/A/Session/26/10/28-Wed/', block: '1.1', collapsed: false });
  const crumbs = s.cards()[0].querySelectorAll('.lsg-breadcrumb a');
  assert.deepEqual(crumbs.map(crumb => crumb.getAttribute('href')), [root + 'GitP/A/Session/26/10/28-Wed/', root + 'GitP/A/Session/26/10/28-Wed/#zoom/1']);
  s.shift(crumbs[1]);
  s.shift(crumbs[0]);
  assert.deepEqual(s.saved().cards.slice(0, 2).map(card => [card.type, card.route, card.block]), [['page', 'GitP/A/Session/26/10/28-Wed/', undefined], ['block', 'GitP/A/Session/26/10/28-Wed/', '1']]);
  // A bullet on the current page and a block reference keep their pages' routes.
  s.shift(first.querySelector('.bullet'));
  s.shift(third.querySelector('.block-ref-link'));
  await settle();
  assert.deepEqual(s.saved().cards.slice(0, 2).map(card => [card.type, card.route, card.block]), [['refs', 'Terminal/', 'abc-1'], ['block', 'GitP/A/Session/26/10/29-Thu/', '1']]);
  assert.match(s.cards()[1].querySelector('.lsg-card-body').textContent, /^After GitP\/A\/Session\/26\/10\/28-Wed/);
  // A link inside a flashcard block (li.lsg-card) moves its page's card to the top.
  s.shift(flashcard.querySelector('.page-reference a'));
  assert.equal(s.saved().cards[0].route, 'GitP/A/');
  // Another page restores the same cards from the site root.
  const titles = s.titles(), saved = s.saved();
  const again = load({ stored: Object.fromEntries(s.storage), at: root + 'Microfreak/', root });
  again.request('restore');
  await settle();
  assert.deepEqual(again.saved(), saved);
  assert.deepEqual(again.titles(), titles);
  assert.deepEqual([...new Set(state.fetched)].sort(), ['gitpa/GitP/A/', 'gitpa/GitP/A/Session/26/10/28-Wed/', 'gitpa/GitP/A/Session/26/10/29-Thu/', 'gitpa/Microfreak/', 'gitpa/Terminal/']);
});

test('phones keep the page: requests are ignored', () => {
  const s = load({ desktop: false });
  s.request('toggle');
  s.shift(s.main.querySelector('a[href^="p-"]'));
  assert.equal(s.aside.hidden, true);
  assert.deepEqual(state.fetched, []);
});
