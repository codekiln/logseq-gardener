const { readFileSync } = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const test = require('node:test');
const assert = require('node:assert/strict');
const script = readFileSync(path.join(__dirname, '../crates/logseq-gardener-sdk/src/site/install.js'), 'utf8');

// A page with the More menu and the hidden Install app item, run with the real script.
function page({ item = true, standalone = false, iosStandalone = false } = {}) {
  const listeners = { window: {}, item: {} };
  const state = { focused: null, menuHidden: false, expanded: 'true', itemHidden: true, requests: 0 };
  const installItem = item && {
    get hidden() { return state.itemHidden; },
    set hidden(value) { state.itemHidden = value; },
    addEventListener: (type, fn) => { listeners.item[type] = fn; },
  };
  const more = {
    setAttribute: (name, value) => { if (name === 'aria-expanded') state.expanded = value; },
    focus: () => { state.focused = 'more'; },
  };
  const menu = { get hidden() { return state.menuHidden; }, set hidden(value) { state.menuHidden = value; } };
  const document = {
    querySelector: (selector) => ({ '[data-lsg-install]': installItem || null, '[data-lsg-more]': more })[selector] ?? null,
    getElementById: (id) => (id === 'lsg-more-menu' ? menu : null),
  };
  const window = { addEventListener: (type, fn) => { listeners.window[type] = fn; } };
  const context = vm.createContext({
    window, document,
    navigator: iosStandalone ? { standalone: true } : {},
    matchMedia: (query) => ({ matches: standalone && query === '(display-mode: standalone)' }),
    fetch: () => { state.requests += 1; },
    XMLHttpRequest: function () { state.requests += 1; },
  });
  vm.runInContext(script, context);
  return { listeners, state };
}

function offer() {
  const event = { prevented: false, prompts: 0, preventDefault() { this.prevented = true; }, prompt() { this.prompts += 1; return Promise.resolve({ outcome: 'accepted' }); } };
  return event;
}

test('the item stays hidden until the browser offers installation, and a browser that never does leaves it hidden', () => {
  const { listeners, state } = page();
  assert.equal(state.itemHidden, true);
  assert.equal(typeof listeners.window.beforeinstallprompt, 'function');
  assert.equal(state.requests, 0);
});

test('the offer is kept, its default prompt is suppressed and the item appears', () => {
  const { listeners, state } = page();
  const event = offer();
  listeners.window.beforeinstallprompt(event);
  assert.equal(event.prevented, true);
  assert.equal(state.itemHidden, false);
  assert.equal(event.prompts, 0, 'the browser dialog opens only when the reader activates the item');
});

test('activating the item prompts once, hides it, closes the menu and returns focus to More', () => {
  const { listeners, state } = page();
  const event = offer();
  listeners.window.beforeinstallprompt(event);
  listeners.item.click();
  assert.equal(event.prompts, 1);
  assert.equal(state.itemHidden, true);
  assert.equal(state.menuHidden, true);
  assert.equal(state.expanded, 'false');
  assert.equal(state.focused, 'more');
  listeners.item.click();
  assert.equal(event.prompts, 1, 'a second activation cannot prompt again');
});

test('an installed window never shows the item', () => {
  for (const options of [{ standalone: true }, { iosStandalone: true }]) {
    const { listeners, state } = page(options);
    const event = offer();
    listeners.window.beforeinstallprompt(event);
    assert.equal(event.prevented, true);
    assert.equal(state.itemHidden, true);
  }
});

test('a page without the item raises no error and listens for nothing', () => {
  const { listeners } = page({ item: false });
  assert.deepEqual(listeners.window, {});
});

test('a rejected prompt raises no unhandled rejection', async () => {
  const { listeners } = page();
  const event = offer();
  event.prompt = () => Promise.reject(new Error('dismissed'));
  listeners.window.beforeinstallprompt(event);
  listeners.item.click();
  await new Promise((resolve) => setImmediate(resolve));
});

test('the script reads and writes no storage or cookie and sends no request', () => {
  for (const word of ['localStorage', 'sessionStorage', 'indexedDB', 'cookie', 'caches', 'serviceWorker', 'fetch(', 'XMLHttpRequest', 'sendBeacon']) {
    assert.equal(script.includes(word), false, word);
  }
});

test('the minified script fits its size budget', () => {
  const minified = script.split('\n').map((l) => l.trim()).filter((l) => l && !l.startsWith('//')).join('\n');
  assert.ok(minified.length <= 1500, `${minified.length} bytes`);
});
