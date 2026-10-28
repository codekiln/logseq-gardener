const { readFileSync } = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const zlib = require('node:zlib');
const test = require('node:test');
const assert = require('node:assert/strict');
const site = path.join(__dirname, '../crates/logseq-gardener-sdk/src/site');
const source = readFileSync(path.join(site, 'graph.js'), 'utf8');
const workerSource = readFileSync(path.join(site, 'graph-worker.js'), 'utf8');

// graph.js on a page without a graph: it only defines its helpers.
function graphModule() {
  const window = {};
  const document = { currentScript: null, documentElement: { getAttribute: () => null }, querySelector: () => null };
  vm.runInContext(source, vm.createContext({ window, document, location: { href: 'http://localhost/tmux.html' }, URL, TextEncoder, TextDecoder, BigInt, DataView, Uint8Array, Uint32Array, Float32Array, Int32Array, Math, String, Array, Object, Number, JSON, Promise, Map, Set, Error }));
  return window.logseqGardenerGraph;
}

test('the reader decodes the data file the publisher writes', () => {
  const graph = graphModule();
  // Written by the Rust test javascript_fixture_matches_the_encoder.
  const bytes = zlib.gunzipSync(readFileSync(path.join(__dirname, 'fixtures/graph-sample.bin')));
  const g = graph.decode(bytes.buffer.slice(bytes.byteOffset, bytes.byteOffset + bytes.length));
  assert.equal(g.n, 4);
  assert.deepEqual([0, 1, 2, 3].map(g.title), ['tmux', 'Ünïcode/名前', 'c', 'd']);
  assert.deepEqual([...g.flags], [2, 64, 0, 1]);
  assert.deepEqual([...g.degree], [300, 300, 300, 300]);
  assert.deepEqual([...g.a], [0, 0, 1, 2]);
  assert.deepEqual([...g.b], [1, 3, 2, 3]);
  assert.equal(g.links, 9);
  assert.deepEqual([g.x[0], g.y[0]], [-100, 5]);
  assert.deepEqual([g.x[1], g.y[1]], [100, -5]);
  assert.ok(Math.abs(g.x[3] - 50) < 0.01);
});

test('the reader refuses an unknown format', () => {
  const graph = graphModule();
  assert.throws(() => graph.decode(new Uint8Array(40).buffer), /unknown graph format/);
});

test('routes match the publisher and colours match ClojureScript hashes', () => {
  const graph = graphModule();
  // lsg's route digest for these titles.
  assert.equal(graph.routeOf('tmux'), 'p-424036ef4353ec8f.html');
  assert.equal(graph.routeOf('Foam'), 'p-077e10850d78a1dc.html');
  // Clojure's (hash s), computed independently as Murmur3 of Java's String.hashCode with mmh3.
  for (const [text, hash] of [['a', 1455541201], ['Logseq/Entity', 1475162357], ['ls-selected-nodes', -1691645926], ['tmux', -900948401], ['Person', -546207508], ['Ünïcode/名前', 1408383083]]) {
    assert.equal(graph.cljsHash(text), hash, text);
  }
  assert.equal(graph.parentColor('Logseq/Entity'), '#7f7f7f');
  assert.equal(graph.cljsHash(''), 0);
});

test('node sizes follow Logseq', () => {
  const graph = graphModule();
  assert.equal(graph.sizeOf(1025), 80);
  assert.equal(graph.sizeOf(1), 8);
  assert.equal(graph.sizeOf(0), 8);
  assert.equal(graph.sizeOf(27), 24);
});

test('a neighbourhood from the global file keeps the page, its neighbours and their edges', () => {
  const graph = graphModule();
  const g = { n: 5, flags: Uint8Array.from([0, 1, 2, 3, 4]), x: Float32Array.from([0, 1, 2, 3, 4]), y: new Float32Array(5), a: Uint32Array.from([0, 0, 1, 3]), b: Uint32Array.from([1, 2, 2, 4]), title: i => 'n' + i };
  const near = graph.around(g, 0);
  assert.equal(near.n, 3);
  assert.deepEqual([0, 1, 2].map(near.title), ['n0', 'n1', 'n2']);
  assert.equal(near.a.length, 3);
});

// The worker with a fake `self`; resolves with the messages it posts until it settles.
function runWorker(message) {
  return new Promise(resolve => {
    const posted = [];
    const self = {
      postMessage: data => { posted.push(data); if (data.done) resolve(posted); },
    };
    vm.runInContext(workerSource, vm.createContext({ self, Math, Date, setTimeout, clearTimeout, Float64Array, Float32Array, Uint32Array, Int32Array, Map }));
    self.onmessage({ data: message });
  });
}

test('the worker lays out a neighbourhood with Logseq forces until it settles', async () => {
  const n = 30, a = [], b = [];
  for (let i = 1; i < n; i++) { a.push(0); b.push(i); }
  const posted = await runWorker({ type: 'start', x: new Float32Array(n), y: new Float32Array(n), a: Uint32Array.from(a), b: Uint32Array.from(b), forces: { link_distance: 70, charge_strength: -600, charge_range: 600 }, phyllotaxis: true });
  const { x, y } = posted.at(-1);
  assert.ok([...x, ...y].every(Number.isFinite));
  let closest = Infinity;
  for (let i = 0; i < n; i++) for (let j = i + 1; j < n; j++) closest = Math.min(closest, Math.hypot(x[i] - x[j], y[i] - y[j]));
  assert.ok(closest > 20, `closest pair ${closest}`);
  for (let i = 1; i < n; i++) {
    const length = Math.hypot(x[i] - x[0], y[i] - y[0]);
    assert.ok(length > 40 && length < 400, `spoke ${i} is ${length}`);
  }
});
