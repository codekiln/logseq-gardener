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

// graph.js on a page: a fake clock for the loading delay, a fetch that waits until the test
// answers or the signal aborts, a worker that returns the positions it gets, and canvases
// that count the nodes they draw.
const { Element, h } = require('./mini-dom.cjs');
// Data files are stored gzip-compressed. The journals file is the sample, whose fourth page is
// a journal; the default file excludes that page, so it draws three nodes with either setting.
const SAMPLE = readFileSync(path.join(__dirname, 'fixtures/graph-sample.bin'));
const DEFAULT = (() => {
  const raw = zlib.gunzipSync(SAMPLE);
  raw[raw.indexOf(Buffer.from([2, 64, 0, 1])) + 3] = 16;
  return zlib.gzipSync(raw);
})();
const settle = async () => { for (let i = 0; i < 20; i++) await new Promise(resolve => setImmediate(resolve)); };

function browser({ pathname, body }) {
  const env = { now: 0, serial: 0, timers: [], requests: [], focused: null, held: [], holdWorker: false };
  Element.onFocus = node => { env.focused = node; };
  const canvas = node => {
    node.drawn = 0;
    node.getContext = type => type === '2d' ? new Proxy({}, {
      get: (target, key) => key in target ? target[key] : key === 'measureText' ? () => ({ width: 10 }) : key === 'arc' ? () => { node.drawn++; } : key === 'clearRect' ? () => { node.drawn = 0; } : () => {},
      set: (target, key, value) => { target[key] = value; return true; },
    }) : null;
    return node;
  };
  const html = h('html', {}, h('head'), body);
  html.querySelectorAll('canvas').forEach(canvas);
  const document = {
    documentElement: html, head: html.querySelector('head'), body,
    currentScript: { src: 'https://garden.test/lsg-graph.js' },
    querySelector: selector => html.querySelector(selector),
    createElement: tag => tag === 'canvas' ? canvas(new Element(tag)) : new Element(tag),
    get activeElement() { return env.focused; },
  };
  class Worker {
    postMessage(message) {
      if (message.type !== 'start') return;
      env.held.push(() => this.onmessage({ data: { type: 'tick', x: Float32Array.from(message.x, (_, i) => i * 50), y: new Float32Array(message.x.length) } }));
    }
  }
  const storage = new Map();
  const ResizeObserver = class { observe() {} };
  const window = { ResizeObserver };
  const context = {
    console, window, document, URL, TextEncoder, TextDecoder, Response, DecompressionStream, AbortController, Worker, ResizeObserver,
    location: { pathname, href: 'https://garden.test' + pathname, assign() {} },
    sessionStorage: { getItem: key => storage.get(key) ?? null, setItem: (key, value) => storage.set(key, String(value)) },
    performance: {},
    MutationObserver: class { observe() {} },
    requestAnimationFrame: callback => { setImmediate(callback); return 1; },
    setTimeout: (callback, ms = 0) => { env.timers.push({ id: ++env.serial, at: env.now + ms, callback }); return env.serial; },
    clearTimeout: id => { env.timers = env.timers.filter(timer => timer.id !== id); },
    fetch: (url, { signal } = {}) => new Promise((resolve, reject) => {
      env.requests.push({ path: new URL(url).pathname.slice(1), signal, resolve, reject });
      signal?.addEventListener('abort', () => reject(signal.reason));
    }),
  };
  vm.runInContext(source, vm.createContext(context));
  const pending = file => env.requests.findLast(request => request.path === file);
  const overlay = () => body.querySelector('.lsg-graph-loading');
  return {
    env, window, body,
    graph: () => window.logseqGardenerGraph,
    box: () => body.querySelector('.lsg-graph'),
    canvas: () => body.querySelector('.lsg-graph-canvas'),
    requested: () => env.requests.map(request => request.path),
    pending,
    // Answer the newest request for a file: data, a server status, or a lost connection.
    answer: (file, data) => pending(file).resolve(new Response(data)),
    refuse: (file, status) => pending(file).resolve(new Response('', { status })),
    drop: file => pending(file).reject(new TypeError('Failed to fetch')),
    tick: ms => {
      env.now += ms;
      for (let due; (due = env.timers.filter(timer => timer.at <= env.now)).length;) {
        env.timers = env.timers.filter(timer => !due.includes(timer));
        due.forEach(timer => timer.callback());
      }
    },
    layout: async () => { while (env.held.length) env.held.shift()(); await settle(); },
    // What the overlay shows: its state, the status text, the button label or null.
    shown: () => {
      const button = overlay().querySelector('button');
      return [body.querySelector('.lsg-graph').getAttribute('data-lsg-load'), overlay().querySelector('[role=status]').textContent, button.hidden ? null : button.textContent];
    },
    press: (node = overlay().querySelector('button'), detail = 1) => {
      env.focused = node;
      for (let at = node; at; at = at.parentElement) for (const callback of at.listeners?.click || []) callback({ target: node, detail, shiftKey: false });
    },
  };
}

const CONFIG = JSON.stringify({ settings: { journals: false, orphan_pages: true, builtin_pages: false, excluded_pages: false }, forces: { link_distance: 70, charge_strength: -600, charge_range: 600 }, data: '_graph/default.bin', journals: '_graph/journals.bin' });
function graphView() {
  const forces = ['link_distance', 'charge_strength', 'charge_range'].flatMap(name => [h('input', { 'data-force': name }), h('output', { 'data-for': name })]);
  const panel = h('div', { class: 'graph-filters', hidden: '' },
    h('p', { 'data-count': 'pages' }), h('p', { 'data-count': 'links' }),
    h('button', { type: 'button', role: 'switch', 'data-setting': 'journals', 'aria-checked': 'false' }),
    h('div', { 'data-hops': '', hidden: '' }), h('ul', { 'data-filters': '' }), h('button', { 'data-clear': '' }), ...forces);
  const box = h('div', { class: 'lsg-graph', id: 'global-graph', 'data-lsg-graph': CONFIG },
    h('canvas', { class: 'lsg-graph-canvas', tabindex: '0' }), h('p', { class: 'lsg-graph-status', role: 'status' }), panel);
  const b = browser({ pathname: '/graph.html', body: h('body', {}, h('main', {}, box)) });
  return { ...b, panel, pages: () => panel.querySelector('[data-count=pages]').textContent, journals: () => b.press(panel.querySelector('[data-setting=journals]')) };
}

test('the graph view shows nothing while a fast load finishes', async () => {
  const b = graphView();
  assert.deepEqual(b.requested(), ['_graph/default.bin']);
  assert.deepEqual(b.shown(), [null, '', null]);
  assert.equal(b.panel.hidden, true);
  b.tick(499);
  b.answer('_graph/default.bin', DEFAULT);
  await settle();
  b.tick(1000);
  assert.deepEqual(b.shown(), [null, '', null]);
  assert.equal(b.panel.hidden, false);
  assert.equal(b.pages(), '3 pages');
  assert.equal(b.canvas().drawn, 3);
});

test('the graph view shows a spinner, status and Cancel once a load takes 500 ms', async () => {
  const b = graphView();
  const status = b.body.querySelector('.lsg-graph-loading [role=status]');
  b.tick(499);
  assert.deepEqual(b.shown(), [null, '', null]);
  b.tick(1);
  assert.deepEqual(b.shown(), ['loading', 'Loading the graph…', 'Cancel']);
  // The live region was in the page before its message, so screen readers announce it.
  assert.equal(b.body.querySelector('.lsg-graph-loading [role=status]'), status);
  assert.ok(status.querySelector('.lsg-graph-spinner'));
  b.env.focused = b.body.querySelector('.lsg-graph-action');
  b.answer('_graph/default.bin', DEFAULT);
  await settle();
  assert.deepEqual(b.shown(), [null, '', null]);
  assert.equal(b.canvas().drawn, 3);
  // Focus on the button that went away moves to the canvas.
  assert.equal(b.env.focused, b.canvas());
});

test('Cancel stops the graph view download, and Load the graph downloads it again', async () => {
  const b = graphView();
  b.tick(500);
  const first = b.pending('_graph/default.bin');
  b.press();
  await settle();
  assert.equal(first.signal.aborted, true);
  assert.deepEqual(b.shown(), ['cancelled', 'The graph was not loaded.', 'Load the graph']);
  assert.equal(b.panel.hidden, true);
  // The second click of a double-click on Cancel does not start the load.
  b.press(undefined, 2);
  assert.deepEqual(b.requested(), ['_graph/default.bin']);
  const button = b.body.querySelector('.lsg-graph-action');
  b.press();
  assert.deepEqual(b.shown(), ['loading', 'Loading the graph…', 'Cancel']);
  assert.equal(b.env.focused, button);
  assert.deepEqual(b.requested(), ['_graph/default.bin', '_graph/default.bin']);
  b.answer('_graph/default.bin', DEFAULT);
  await settle();
  assert.deepEqual(b.shown(), [null, '', null]);
  assert.equal(b.panel.hidden, false);
  assert.equal(b.canvas().drawn, 3);
});

test('a failed graph view download offers Retry, which downloads it again', async () => {
  for (const fail of [b => b.drop('_graph/default.bin'), b => b.refuse('_graph/default.bin', 500)]) {
    const b = graphView();
    fail(b);
    await settle();
    assert.deepEqual(b.shown(), ['failed', 'The graph could not be loaded.', 'Retry']);
    b.press();
    assert.deepEqual(b.shown(), ['loading', 'Loading the graph…', 'Cancel']);
    assert.deepEqual(b.requested(), ['_graph/default.bin', '_graph/default.bin']);
    b.answer('_graph/default.bin', DEFAULT);
    await settle();
    assert.deepEqual(b.shown(), [null, '', null]);
    assert.equal(b.pages(), '3 pages');
  }
});

test('the Journals switch loads through the same states and the newest load wins', async () => {
  const b = graphView();
  b.answer('_graph/default.bin', DEFAULT);
  await settle();
  // Cancel the journals graph: the default graph stays drawn, dimmed, behind the message.
  b.journals();
  assert.equal(b.panel.querySelector('[data-setting=journals]').getAttribute('aria-checked'), 'true');
  b.tick(500);
  assert.deepEqual(b.shown(), ['loading', 'Loading the graph…', 'Cancel']);
  b.press();
  await settle();
  assert.deepEqual(b.shown(), ['cancelled', 'The graph was not loaded.', 'Load the graph']);
  assert.equal(b.canvas().drawn, 3);
  // Turning Journals off draws the kept default graph without a download.
  b.journals();
  await settle();
  assert.deepEqual(b.shown(), [null, '', null]);
  assert.deepEqual(b.requested(), ['_graph/default.bin', '_graph/journals.bin']);
  // A cancelled file is fetched again; a load replaced by a newer one draws nothing.
  b.journals();
  const replaced = b.pending('_graph/journals.bin');
  assert.deepEqual(b.requested(), ['_graph/default.bin', '_graph/journals.bin', '_graph/journals.bin']);
  b.journals();
  assert.equal(replaced.signal.aborted, true);
  await settle();
  b.tick(1000);
  assert.deepEqual(b.shown(), [null, '', null]);
  assert.equal(b.pages(), '3 pages');
  b.journals();
  b.answer('_graph/journals.bin', SAMPLE);
  await settle();
  assert.equal(b.pages(), '4 pages');
  assert.equal(b.canvas().drawn, 4);
});

// The Page graph card on a page, as the right sidebar fills it.
const NEIGHBOURHOOD = JSON.stringify({ t: ['Foam', 'Obsidian', 'Tool'], f: [0, 0, 2], e: [0, 1, 0, 2] });
async function pageCard(title = 'Foam') {
  const graph = graphModule();
  const route = graph.routeOf(title);
  const card = h('div', { class: 'lsg-page-graph' });
  const b = browser({ pathname: '/' + route, body: h('body', {}, h('main'), card) });
  b.graph().page(card);
  await settle();
  return { ...b, card, file: `_graph/p/${route.slice(2, 18)}.json`, toggle: () => b.press(card.querySelector('.lsg-switch')) };
}

test('the Page graph card draws a fast load with no loading state', async () => {
  const b = await pageCard();
  assert.deepEqual(b.requested(), [b.file]);
  b.answer(b.file, NEIGHBOURHOOD);
  await settle();
  await b.layout();
  b.tick(1000);
  assert.deepEqual(b.shown(), [null, '', null]);
  assert.equal(b.canvas().drawn, 3);
});

test('the Page graph card shows its loading state, and Cancel leaves Load the page graph', async () => {
  const b = await pageCard();
  b.tick(500);
  assert.deepEqual(b.shown(), ['loading', 'Loading the page graph…', 'Cancel']);
  const first = b.pending(b.file);
  b.press();
  await settle();
  assert.equal(first.signal.aborted, true);
  assert.deepEqual(b.shown(), ['cancelled', 'The page graph was not loaded.', 'Load the page graph']);
  // A cancelled neighbourhood file never falls back to the global file.
  assert.deepEqual(b.requested(), [b.file]);
  b.press();
  assert.deepEqual(b.shown(), ['loading', 'Loading the page graph…', 'Cancel']);
  b.answer(b.file, NEIGHBOURHOOD);
  await settle();
  await b.layout();
  assert.deepEqual(b.shown(), [null, '', null]);
  assert.deepEqual(b.requested(), [b.file, b.file]);
  assert.equal(b.canvas().drawn, 3);
  assert.equal(b.env.focused, b.canvas());
});

test('a lost connection in the Page graph card offers Retry instead of the global file', async () => {
  const b = await pageCard();
  b.drop(b.file);
  await settle();
  assert.deepEqual(b.shown(), ['failed', 'The page graph could not be loaded.', 'Retry']);
  assert.deepEqual(b.requested(), [b.file]);
  b.press();
  b.answer(b.file, NEIGHBOURHOOD);
  await settle();
  await b.layout();
  assert.deepEqual(b.shown(), [null, '', null]);
  assert.equal(b.canvas().drawn, 3);
});

test('Cancel during the Page graph layout draws nothing when the layout arrives', async () => {
  const b = await pageCard();
  b.answer(b.file, NEIGHBOURHOOD);
  await settle();
  b.tick(500);
  assert.deepEqual(b.shown(), ['loading', 'Loading the page graph…', 'Cancel']);
  b.press();
  await settle();
  await b.layout();
  assert.deepEqual(b.shown(), ['cancelled', 'The page graph was not loaded.', 'Load the page graph']);
  assert.equal(b.canvas().drawn, 0);
  // The neighbourhood file arrived, so loading again only lays it out.
  b.press();
  await b.layout();
  assert.deepEqual(b.shown(), [null, '', null]);
  assert.deepEqual(b.requested(), [b.file]);
  assert.equal(b.canvas().drawn, 3);
});

test('a hub without a neighbourhood file loads the global file through the same states', async () => {
  const b = await pageCard('tmux');
  b.refuse(b.file, 404);
  await settle();
  assert.deepEqual(b.requested(), [b.file, '_graph/default.bin']);
  b.tick(500);
  assert.deepEqual(b.shown(), ['loading', 'Loading the page graph…', 'Cancel']);
  // Show Journals replaces the global download in progress.
  const replaced = b.pending('_graph/default.bin');
  b.toggle();
  assert.equal(replaced.signal.aborted, true);
  assert.deepEqual(b.requested(), [b.file, '_graph/default.bin', '_graph/journals.bin']);
  b.drop('_graph/journals.bin');
  await settle();
  assert.deepEqual(b.shown(), ['failed', 'The page graph could not be loaded.', 'Retry']);
  b.press();
  b.answer('_graph/journals.bin', SAMPLE);
  await settle();
  assert.deepEqual(b.shown(), [null, '', null]);
  // tmux with its two neighbours from the sample file.
  assert.equal(b.canvas().drawn, 3);
});
