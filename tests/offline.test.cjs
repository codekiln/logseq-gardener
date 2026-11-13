const { readFileSync } = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const test = require('node:test');
const assert = require('node:assert/strict');
const script = readFileSync(path.join(__dirname, '../crates/logseq-gardener-sdk/src/site/offline.js'), 'utf8');

const tick = () => new Promise((resolve) => setTimeout(resolve, 80));

function element(state, name) {
  const listeners = {};
  return {
    listeners,
    get hidden() { return state[name]; },
    set hidden(value) { state[name] = value; },
    addEventListener: (type, fn) => { listeners[type] = fn; },
  };
}

// A page with the More menu and the two hidden items, run with the real script.
function page({ href = 'https://example.org/gitpa/Notes/', base = 'https://example.org/gitpa/', items = true, worker = true,
  registered = null, controller = null, online = true, timing = [], copies = {}, refuse = false } = {}) {
  const state = { save: true, remove: true, menuHidden: false, expanded: 'true', focused: null, calls: [], deleted: [], registrations: [] };
  const save = element(state, 'save'), remove = element(state, 'remove');
  let registration = registered && { scope: registered, unregister: async () => { state.calls.push('unregister'); return true; } };
  const main = { first: null, prepend(node) { this.first = node; } };
  const more = { setAttribute: (name, value) => { if (name === 'aria-expanded') state.expanded = value; }, focus: () => { state.focused = 'more'; } };
  const menu = { get hidden() { return state.menuHidden; }, set hidden(value) { state.menuHidden = value; } };
  let status = null;
  const document = {
    baseURI: base,
    querySelector: (selector) => ({
      '[data-lsg-offline-save]': items ? save : null, '[data-lsg-offline-remove]': items ? remove : null,
      '#lsg-more-menu': menu, '[data-lsg-more]': more, main, '[data-lsg-offline-status]': status,
    })[selector] ?? null,
    createElement: () => {
      status = { attributes: {}, textContent: '', setAttribute(name, value) { this.attributes[name] = value; } };
      return status;
    },
  };
  const serviceWorker = worker && {
    controller,
    getRegistration: async (scope) => { state.calls.push(`get ${scope}`); return registration; },
    register: async (url, options) => {
      state.calls.push(`register ${url} ${options.scope}`);
      if (refuse) throw new Error('refused');
      registration = { scope: options.scope, unregister: async () => { state.calls.push('unregister'); return true; } };
      return registration;
    },
  };
  const caches = {
    delete: async (name) => { state.deleted.push(name); return true; },
    match: async (url, options) => { state.calls.push(`match ${url} ${options.cacheName}`); return copies[url] ?? undefined; },
    open: () => { throw new Error('a page never opens a cache'); },
    keys: () => { throw new Error('a page never lists caches'); },
  };
  const url = new URL(href);
  const context = vm.createContext({
    document, URL, setTimeout, caches,
    location: { href, protocol: url.protocol, hostname: url.hostname },
    navigator: { serviceWorker: serviceWorker || undefined, onLine: online },
    performance: { getEntriesByType: (type) => (type === 'navigation' ? [{ serverTiming: timing }] : []) },
    window: {},
  });
  vm.runInContext(script, context);
  return { state, save, remove, status: () => status, main };
}

test('before Save, the page shows Save and registers, opens and stores nothing', async () => {
  const { state } = page();
  await tick();
  assert.equal(state.save, false);
  assert.equal(state.remove, true);
  assert.deepEqual(state.calls, ['get https://example.org/gitpa/'], 'only a read of the registration');
  assert.deepEqual(state.deleted, []);
});

test('Save registers lsg-sw.js once with the folder scope, shows Remove, closes the menu and says saving is on', async () => {
  const { state, save, status } = page({ href: 'https://example.org/gitpa/Docs/Guide/', base: 'https://example.org/gitpa/' });
  await tick();
  await save.listeners.click();
  await tick();
  assert.deepEqual(state.calls.filter((call) => call.startsWith('register')),
    ['register https://example.org/gitpa/lsg-sw.js https://example.org/gitpa/']);
  assert.equal(state.save, true);
  assert.equal(state.remove, false);
  assert.equal(state.menuHidden, true);
  assert.equal(state.expanded, 'false');
  assert.equal(state.focused, 'more');
  assert.equal(status().attributes.role, 'status');
  assert.match(status().textContent, /^Saving is on\. Pages you open from now on are kept/);
});

test('a registered garden shows Remove, and Remove deletes the exact cache and unregisters', async () => {
  const { state, remove, status } = page({ registered: 'https://example.org/gitpa/' });
  await tick();
  assert.equal(state.save, true);
  assert.equal(state.remove, false);
  await remove.listeners.click();
  await tick();
  assert.ok(state.calls.includes('unregister'));
  assert.deepEqual(state.deleted, ['lsg-offline:/gitpa/']);
  assert.equal(state.save, false);
  assert.equal(state.remove, true);
  assert.equal(status().textContent, 'Offline copies removed.');
});

test('a registration of a neighbouring scope does not count as this garden\'s', async () => {
  const { state } = page({ registered: 'https://example.org/' });
  await tick();
  assert.equal(state.save, false);
  assert.equal(state.remove, true);
});

test('a refused registration says so and keeps Save', async () => {
  const { state, save, status } = page({ refuse: true });
  await tick();
  await save.listeners.click();
  await tick();
  assert.equal(state.save, false);
  assert.equal(status().textContent, 'This browser could not turn on offline reading.');
});

test('a site at an origin root, a file: page, an insecure browser and a page without the items show neither item', async () => {
  for (const options of [
    { href: 'https://example.org/Notes/', base: 'https://example.org/' },
    { href: 'file:///Users/a/site/index.html', base: 'file:///Users/a/site/' },
    { worker: false },
    { items: false },
  ]) {
    const { state } = page(options);
    await tick();
    assert.equal(state.save, true, JSON.stringify(options));
    assert.equal(state.remove, true, JSON.stringify(options));
    assert.deepEqual(state.calls, [], JSON.stringify(options));
  }
});

test('a page answered from a stored copy shows the day the copy was saved', async () => {
  const saved = Date.UTC(2026, 10, 3, 12);
  const copy = { headers: { get: (name) => (name === 'x-lsg-saved' ? String(saved) : null) } };
  const href = 'https://example.org/gitpa/Notes/';
  for (const options of [{ online: false }, { timing: [{ name: 'lsg-offline' }] }]) {
    const { state, status } = page({ href, controller: {}, copies: { [href]: copy }, ...options });
    await tick();
    assert.ok(state.calls.includes(`match ${href} lsg-offline:/gitpa/`), state.calls.join());
    const day = new Date(saved).toLocaleDateString(undefined, { day: 'numeric', month: 'long' });
    assert.equal(status().textContent, `Saved copy from ${day}`);
  }
});

test('an online page from the network, or a page without a worker, shows no notice', async () => {
  for (const options of [{ controller: {} }, { online: false }]) {
    const { state, status } = page(options);
    await tick();
    assert.equal(status(), null);
    assert.ok(!state.calls.some((call) => call.startsWith('match')));
  }
});

test('the script reads and writes no other storage', () => {
  for (const word of ['localStorage', 'sessionStorage', 'indexedDB', 'cookie', 'persist(', 'caches.open', 'caches.keys', 'fetch(']) {
    assert.equal(script.includes(word), false, word);
  }
});
