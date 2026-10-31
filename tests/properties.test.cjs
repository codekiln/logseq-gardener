const {readFileSync} = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const test = require('node:test');
const assert = require('node:assert/strict');
const site = path.join(__dirname, '../crates/logseq-gardener-sdk/src/site');
const startup = readFileSync(path.join(site, 'properties-start.js'), 'utf8');
const controller = readFileSync(path.join(site, 'properties.js'), 'utf8');
const day = 86400000;
const key = 'lsg:/garden/';
function fixture({saved, blocked = false, now = 2000000000000} = {}) {
  const values = new Map(saved === undefined ? [] : [[key, typeof saved === 'string' ? saved : JSON.stringify(saved)]]);
  const listeners = new Map();
  const windowListeners = new Map();
  const timers = [];
  const document = {
    querySelector: () => ({href: 'https://example.test/garden/lsg-properties-revision.mjs'}),
    nodes: [],
    querySelectorAll(selector) { return selector.includes(':not') ? this.nodes.filter(node => !node.dataset.lsgP) : this.nodes.filter(node => node.dataset.lsgP); },
    getElementById() { return null; },
    addEventListener(type, callback) { if (!listeners.has(type)) listeners.set(type, []); listeners.get(type).push(callback); },
    removeEventListener(type, callback) { listeners.set(type, listeners.get(type).filter(value => value !== callback)); },
  };
  const storage = {getItem: k => values.get(k) ?? null, setItem: (k,v) => values.set(k,v), removeItem: k => values.delete(k)};
  const window = {};
  const context = {window, document, URL, Date: {now: () => now}, location: {hash: ''}, addEventListener(type, callback) { if (!windowListeners.has(type)) windowListeners.set(type, []); windowListeners.get(type).push(callback); }, requestAnimationFrame() {}, setTimeout: callback => timers.push(callback)};
  Object.defineProperty(window, 'localStorage', {get() { if (blocked) throw Error('blocked'); return storage; }});
  Object.defineProperty(context, 'localStorage', {get() { return window.localStorage; }});
  vm.createContext(context);
  vm.runInContext(startup, context);
  function root(open = true) {
    const details = {open, dataset: {}, contains: () => false};
    return {details, querySelectorAll: () => details.dataset.lsgP ? [] : [details]};
  }
  return {window, values, document, root, context,
    fire(type, event = {}) {for (const callback of windowListeners.get(type) ?? []) callback(event);},
    load() {vm.runInContext(controller, context);},
    advance(ms) {now += ms;},
    stored() {return values.has(key) ? JSON.parse(values.get(key)) : undefined;},
    click(details, {link = false, prevented = false} = {}) {
      const summary = {id: 'page-properties', parentNode: details, parentElement: details};
      const event = {target: {id: 'page-properties', parentNode: details, closest: selector => selector === 'a' ? (link ? {} : null) : summary}, defaultPrevented: prevented};
      for (const callback of [...listeners.get('click')]) callback(event);
      if (!prevented && !link) details.open = !details.open;
    },
    flush() {while (timers.length) timers.shift()();},
  };
}
test('early restoration accepts only a boolean and a current numeric timestamp', () => {
  for (const open of [false, true]) {
    const f = fixture({saved: {open, seen: 2000000000000 - day}});
    const r = f.root(!open); f.window.logseqGardenerInitProperties(r);
    assert.equal(r.details.open, open);
    f.load(); assert.equal(f.stored().seen, 2000000000000);
  }
  for (const saved of ['invalid', {open:false,seen:'2000000000000'}, {open:0,seen:2000000000000}, {open:false,seen:2000000000001}, {open:false,seen:2000000000000-30*day}]) {
    const f=fixture({saved}); const r=f.root(true); f.window.logseqGardenerInitProperties(r);
    assert.equal(r.details.open, true); f.load(); assert.equal(f.stored(), undefined);
  }
});
test('SPA visits renew the absence window, expiry restores configured defaults, repeated init preserves fragments', () => {
  const f=fixture({saved:{open:false,seen:2000000000000-29*day}}); f.load();
  f.advance(29*day); const a=f.root(true); f.window.logseqGardenerInitProperties(a);
  assert.equal(a.details.open,false); const renewed=f.stored().seen;
  a.details.open=true; f.window.logseqGardenerInitProperties(a); assert.equal(a.details.open,true);
  f.advance(31*day); const b=f.root(true); f.window.logseqGardenerInitProperties(b);
  assert.equal(b.details.open,true); assert.equal(f.stored(),undefined); assert.ok(renewed>2000000000000);
});
test('native activations retain order through module loading, and links/prevented clicks never save', () => {
  const f=fixture(); const r=f.root(true); f.window.logseqGardenerInitProperties(r);
  f.click(r.details); f.click(r.details); f.load(); f.flush(); assert.equal(f.stored().open,true);
  f.click(r.details,{link:true}); f.flush(); assert.equal(f.stored().open,true);
  f.click(r.details,{prevented:true}); f.flush(); assert.equal(f.stored().open,true);
  f.click(r.details); f.flush(); assert.equal(f.stored().open,false);
  r.details.open=true; f.flush(); assert.equal(f.stored().open,false);
});
test('blocked storage preserves both configured defaults and working native disclosure', () => {
  const f=fixture({blocked:true}); f.load();
  for (const open of [false,true]) {const r=f.root(open); f.window.logseqGardenerInitProperties(r); assert.equal(r.details.open,open); f.click(r.details); f.flush(); assert.equal(r.details.open,!open);}
});
test('pending reader choices cross replacements before and after controller adoption', () => {
  for (const open of [false, true]) {
    const f = fixture({saved:{open:!open,seen:2000000000000}});
    const first = f.root(); f.window.logseqGardenerInitProperties(first);
    f.click(first.details);
    const replacement = f.root(!open);
    f.window.logseqGardenerInitProperties(replacement);
    assert.equal(replacement.details.open, open);
    f.document.nodes = [replacement.details];
    f.load();
    assert.equal(replacement.details.open, open);
    const duringSave = f.root(!open);
    f.window.logseqGardenerInitProperties(duringSave);
    assert.equal(duringSave.details.open, open);
    f.flush(); assert.equal(f.stored().open, open);
    // A normal controller activation has the same pending-save boundary.
    f.click(duringSave.details);
    const next = f.root(open);
    f.window.logseqGardenerInitProperties(next);
    assert.equal(next.details.open, !open);
    f.flush(); assert.equal(f.stored().open, !open);
  }
});
test('early canceled activations and fragment openings do not become preferences', () => {
  const f = fixture({saved:{open:false,seen:2000000000000}});
  const first = f.root(); f.window.logseqGardenerInitProperties(first);
  f.click(first.details,{prevented:true});
  f.click(first.details,{link:true});
  first.details.open = true;
  const replacement = f.root(); f.window.logseqGardenerInitProperties(replacement);
  assert.equal(replacement.details.open, false);
  f.load(); f.flush(); assert.equal(f.stored().open, false);
});
test('early activations persist before the controller arrives', () => {
  for (const open of [false, true]) {
    const f = fixture({saved:{open:!open,seen:2000000000000}});
    const first = f.root(); f.window.logseqGardenerInitProperties(first);
    f.click(first.details);
    f.flush();
    assert.equal(f.stored().open, open);
    // A page left before the deferred write runs still keeps the choice.
    f.click(first.details);
    f.fire('pagehide');
    assert.equal(f.stored().open, !open);
    f.flush(); assert.equal(f.stored().open, !open);
    // Canceled and link activations never become preferences.
    f.click(first.details,{prevented:true}); f.click(first.details,{link:true});
    f.fire('pagehide'); f.flush(); assert.equal(f.stored().open, !open);
  }
});
test('a document restored from the Back/Forward cache applies the current choice, fallback and fragment', () => {
  for (const loaded of [false, true]) {
    for (const configured of [false, true]) {
      const f = fixture({saved:{open:false,seen:2000000000000}});
      const page = f.root(configured); f.window.logseqGardenerInitProperties(page);
      f.document.nodes = [page.details];
      if (loaded) f.load();
      assert.equal(page.details.open, false);
      // Another page stored the opposite choice before Back returned here.
      f.values.set(key, JSON.stringify({open:true,seen:2000000000000}));
      f.fire('pageshow', {persisted:false});
      assert.equal(page.details.open, false);
      f.advance(1000);
      f.fire('pageshow', {persisted:true});
      assert.equal(page.details.open, true);
      if (loaded) assert.equal(f.stored().seen, 2000000000000 + 1000);
      // Without a stored choice, the configured default returns.
      f.values.delete(key);
      f.fire('pageshow', {persisted:true});
      assert.equal(page.details.open, configured);
      // A fragment inside Properties keeps the section open.
      f.values.set(key, JSON.stringify({open:false,seen:2000000000000}));
      const target = {};
      f.context.location.hash = '#page-properties';
      f.document.getElementById = id => id === 'page-properties' ? target : null;
      page.details.contains = node => node === target;
      f.fire('pageshow', {persisted:true});
      assert.equal(page.details.open, true);
    }
  }
});
