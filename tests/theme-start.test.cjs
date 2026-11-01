const { readFileSync } = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const test = require('node:test');
const assert = require('node:assert/strict');
const site = path.join(__dirname, '../crates/logseq-gardener-sdk/src/site');
// The publisher drops the comment lines of the hand-minified inline scripts.
const inline = (name) => readFileSync(path.join(site, name), 'utf8').split('\n').map((l) => l.trim())
  .filter((l) => l && !l.startsWith('//')).join('');
const start = inline('theme-start.js');
const controller = readFileSync(path.join(site, 'theme.js'), 'utf8');

// The inline theme start alone, as a page's head runs it before the first paint.
function head({ stored = {}, wide = true, dark = false, blocked = false } = {}) {
  const attributes = new Map();
  const properties = new Map();
  const root = {
    setAttribute: (name, value) => attributes.set(name, value),
    style: { setProperty: (name, value) => properties.set(name, value) },
  };
  const localStorage = { getItem: (key) => { if (blocked) throw new Error('blocked'); return stored[key] ?? null; } };
  const window = {
    matchMedia: (query) => ({ matches: query.includes('min-width') ? wide : dark }),
  };
  const context = vm.createContext({ window, localStorage, document: { documentElement: root } });
  vm.runInContext(start, context);
  return { attributes, properties, context, window, root };
}

test('a saved Dark choice paints dark from the first frame, whatever the system prefers', () => {
  const page = head({ stored: { 'logseq-gardener-theme': 'dark' } });
  assert.equal(page.attributes.get('data-lsg-theme'), 'dark');
  assert.equal(page.attributes.get('data-lsg-theme-mode'), 'dark');
});

test('without a saved choice the system appearance applies, and invalid values count as system', () => {
  assert.equal(head({ dark: true }).attributes.get('data-lsg-theme'), 'dark');
  const light = head({ stored: { 'logseq-gardener-theme': 'purple' } });
  assert.equal(light.attributes.get('data-lsg-theme'), 'light');
  assert.equal(light.attributes.get('data-lsg-theme-mode'), 'system');
});

test('the saved layout applies before paint, and the left sidebar stays closed on a phone', () => {
  const stored = {
    'logseq-gardener-left-sidebar': 'open',
    'logseq-gardener-wide': 'true',
    'logseq-gardener-right-sidebar': JSON.stringify({ open: true, width: 420 }),
  };
  const desktop = head({ stored });
  assert.equal(desktop.attributes.get('data-lsg-left-sidebar'), 'open');
  assert.equal(desktop.attributes.get('data-lsg-wide'), 'true');
  assert.equal(desktop.attributes.get('data-lsg-right-sidebar'), 'open');
  assert.equal(desktop.properties.get('--lsg-right-width'), '420px');
  const phone = head({ stored, wide: false });
  assert.equal(phone.attributes.has('data-lsg-left-sidebar'), false);
  assert.equal(phone.attributes.has('data-lsg-right-sidebar'), false);
  assert.equal(phone.attributes.get('data-lsg-wide'), 'true');
});

test('blocked storage and a broken saved sidebar keep the defaults', () => {
  const blocked = head({ blocked: true });
  assert.equal(blocked.attributes.get('data-lsg-theme'), 'light');
  assert.equal(blocked.attributes.has('data-lsg-left-sidebar'), false);
  const broken = head({ stored: { 'logseq-gardener-right-sidebar': '{', 'logseq-gardener-wide': 'true' } });
  assert.equal(broken.attributes.has('data-lsg-right-sidebar'), false);
  assert.equal(broken.attributes.get('data-lsg-wide'), 'true');
});

test('the deferred controller keeps the appearance the inline start applied', () => {
  const page = head({ stored: { 'logseq-gardener-theme': 'dark' } });
  const before = page.attributes.get('data-lsg-theme');
  Object.assign(page.window, {
    localStorage: { getItem: () => 'dark', setItem() {}, removeItem() {} },
    addEventListener() {},
  });
  page.context.document = {
    documentElement: page.root, readyState: 'interactive', addEventListener() {}, querySelectorAll: () => [],
  };
  vm.runInContext(controller, page.context);
  assert.equal(page.attributes.get('data-lsg-theme'), before);
  assert.equal(typeof page.window.logseqGardenerInitTheme, 'function');
});

test('the inline scripts every page carries total at most 1.5 KB', () => {
  const total = Buffer.byteLength(start) + Buffer.byteLength(inline('fragments.js'));
  assert.ok(total <= 1500, `${total} bytes`);
});
