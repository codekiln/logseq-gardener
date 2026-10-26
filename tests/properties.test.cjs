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
  const timers = [];
  const document = {
    querySelector: () => ({href: 'https://example.test/garden/lsg-properties-revision.mjs'}),
    nodes: [],
    querySelectorAll() { return this.nodes.filter(node => !node.dataset.lsgP); },
    addEventListener(type, callback) { if (!listeners.has(type)) listeners.set(type, []); listeners.get(type).push(callback); },
    removeEventListener(type, callback) { listeners.set(type, listeners.get(type).filter(value => value !== callback)); },
  };
  const storage = {getItem: k => values.get(k) ?? null, setItem: (k,v) => values.set(k,v), removeItem: k => values.delete(k)};
  const window = {};
  const context = {window, document, URL, Date: {now: () => now}, addEventListener() {}, requestAnimationFrame() {}, setTimeout: callback => timers.push(callback)};
  Object.defineProperty(window, 'localStorage', {get() { if (blocked) throw Error('blocked'); return storage; }});
  Object.defineProperty(context, 'localStorage', {get() { return window.localStorage; }});
  vm.createContext(context);
  vm.runInContext(startup, context);
  function root(open = true) {
    const details = {open, dataset: {}};
    return {details, querySelectorAll: () => details.dataset.lsgP ? [] : [details]};
  }
  return {window, values, document, root,
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
