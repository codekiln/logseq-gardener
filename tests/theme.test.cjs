const { readFileSync } = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const test = require('node:test');
const assert = require('node:assert/strict');
const source = readFileSync(path.join(__dirname, '../crates/logseq-gardener-sdk/src/site/theme.js'), 'utf8');
const key = 'logseq-gardener-theme';
class Events {
  constructor() { this.listeners = new Map(); }
  addEventListener(type, callback) {
    if (!this.listeners.has(type)) this.listeners.set(type, []);
    this.listeners.get(type).push(callback);
  }
  fire(type, event = {}) { for (const callback of this.listeners.get(type) || []) callback(event); }
}
function control() {
  const label = { hidden: true };
  const select = { value: 'system', matches: selector => selector === '[data-lsg-theme-select]', closest: () => label };
  return { label, select };
}
function fixture({ stored, dark = false, blocked = false, loading = false, noMedia = false } = {}) {
  const values = new Map(stored === undefined ? [] : [[key, stored]]);
  const document = new Events();
  const attributes = new Map();
  document.documentElement = { setAttribute: (name, value) => attributes.set(name, value) };
  document.readyState = loading ? 'loading' : 'complete';
  const first = control();
  document.selects = loading ? [] : [first.select];
  document.querySelectorAll = () => document.selects;
  const media = new Events(); media.matches = dark;
  const window = new Events();
  Object.defineProperty(window, 'localStorage', { get() {
    if (blocked) throw new Error('Storage unavailable');
    return { getItem: k => values.get(k) ?? null, setItem: (k, v) => values.set(k, v), removeItem: k => values.delete(k) };
  } });
  if (!noMedia) window.matchMedia = () => media;
  const context = vm.createContext({ window, document });
  vm.runInContext(source, context);
  return { window, document, first, attributes, values, media,
    choose(value) { document.selects[0].value = value; document.fire('change', { target: document.selects[0] }); },
    system(value) { media.matches = value; media.fire('change'); },
    load() { document.selects = [first.select]; document.readyState = 'complete'; document.fire('DOMContentLoaded'); },
    rerun() { vm.runInContext(source, context); },
    theme() { return attributes.get('data-lsg-theme'); },
    mode() { return attributes.get('data-lsg-theme-mode'); },
  };
}
test('system preference applies in the head and initializes controls after content arrives', () => {
  const f = fixture({ dark: true, loading: true });
  assert.equal(f.theme(), 'dark'); assert.equal(f.first.label.hidden, true);
  f.load(); assert.equal(f.first.label.hidden, false); assert.equal(f.first.select.value, 'system');
  f.system(false); assert.equal(f.theme(), 'light');
});
test('saved explicit choice wins over live system changes and reloads', () => {
  const f = fixture({ stored: 'light', dark: true });
  assert.equal(f.theme(), 'light'); assert.equal(f.first.select.value, 'light');
  f.system(false); f.system(true); assert.equal(f.theme(), 'light');
  f.choose('dark'); assert.equal(f.values.get(key), 'dark');
  assert.equal(fixture({ stored: f.values.get(key) }).theme(), 'dark');
});
test('System removes the override and immediately follows the current system', () => {
  const f = fixture({ stored: 'dark', dark: false });
  f.choose('system'); assert.equal(f.values.has(key), false); assert.equal(f.theme(), 'light');
  f.system(true); assert.equal(f.theme(), 'dark'); assert.equal(f.mode(), 'system');
});
test('unrecognized stored or control values never become appearance attributes', () => {
  const f = fixture({ stored: '<script>bad()</script>', dark: true });
  assert.equal(f.mode(), 'system'); assert.equal(f.theme(), 'dark');
  f.choose('unrecognized'); assert.equal(f.mode(), 'system'); assert.equal(f.theme(), 'dark');
});
test('unavailable storage preserves current document choice through host replacement', () => {
  const f = fixture({ blocked: true });
  f.choose('dark'); assert.equal(f.theme(), 'dark');
  const next = control(); f.document.selects = [next.select];
  f.window.logseqGardenerInitTheme(f.document);
  assert.equal(next.select.value, 'dark'); assert.equal(next.label.hidden, false);
  f.system(false); assert.equal(f.theme(), 'dark');
});
test('reinitialization and repeated script loading do not duplicate handlers or reset state', () => {
  const f = fixture(); f.choose('dark');
  const next = control(); f.document.selects = [next.select];
  for (let i = 0; i < 3; i++) f.window.logseqGardenerInitTheme(f.document);
  f.rerun();
  assert.equal(f.document.listeners.get('change').length, 1);
  assert.equal(f.media.listeners.get('change').length, 1);
  assert.equal(next.select.value, 'dark'); assert.equal(f.theme(), 'dark');
  f.choose('light'); assert.equal(f.theme(), 'light');
});
test('cross-tab updates follow validated stored appearance and ignore other keys', () => {
  const f = fixture(); f.values.set(key, 'dark');
  f.window.fire('storage', { key: 'unrelated' }); assert.equal(f.theme(), 'light');
  f.window.fire('storage', { key }); assert.equal(f.theme(), 'dark');
  f.values.clear(); f.window.fire('storage', { key: null }); assert.equal(f.mode(), 'system');
});
test('missing media support uses Light and still permits explicit Dark', () => {
  const f = fixture({ noMedia: true }); assert.equal(f.theme(), 'light');
  f.choose('dark'); assert.equal(f.theme(), 'dark');
});
test('default light and dark palette text, links and focus meet contrast thresholds', () => {
  const css = readFileSync(path.join(__dirname, '../crates/logseq-gardener-sdk/src/site/theme.css'), 'utf8');
  const luminance = hex => {
    const rgb = hex.match(/[a-f0-9]{2}/g).map(v => parseInt(v, 16) / 255)
      .map(v => v <= .04045 ? v / 12.92 : ((v + .055) / 1.055) ** 2.4);
    return rgb[0] * .2126 + rgb[1] * .7152 + rgb[2] * .0722;
  };
  const ratio = (a, b) => (Math.max(a, b) + .05) / (Math.min(a, b) + .05);
  const palettes = [...css.matchAll(/--lsg-text:#[^}]+/g)];
  assert.equal(palettes.length, 3);
  for (const [declarations] of palettes) {
    const values = Object.fromEntries([...declarations.matchAll(/--lsg-([a-z-]+):#([a-f0-9]{6})/g)].map(m => [m[1], luminance(m[2])]));
    assert.ok(ratio(values.text, values.background) >= 4.5);
    assert.ok(ratio(values.link, values.background) >= 4.5);
    assert.ok(ratio(values.focus, values.background) >= 3);
  }
});
test('the menu interface toggles explicit choices and returns to system', () => {
  const f = fixture({ dark: true });
  const theme = f.window.logseqGardenerTheme;
  assert.equal(theme.choice(), 'system'); assert.equal(theme.appearance(), 'dark');
  theme.set('light'); assert.equal(f.theme(), 'light'); assert.equal(f.values.get(key), 'light');
  f.system(true); assert.equal(f.theme(), 'light');
  theme.set('system'); assert.equal(f.theme(), 'dark'); assert.equal(f.values.has(key), false);
  theme.set('<script>'); assert.equal(theme.choice(), 'system');
});
