const { readFileSync } = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const test = require('node:test');
const assert = require('node:assert/strict');
const source = readFileSync(path.join(__dirname, '../crates/logseq-gardener-sdk/src/site/fragments.js'), 'utf8');

function element(tagName, parentElement = null) {
  return { tagName, parentElement, open: false, scrolls: 0,
    scrollIntoView() { this.scrolls++; } };
}
function page(hash, targets) {
  const listeners = new Map();
  const location = { hash };
  const window = { addEventListener(name, callback) { listeners.set(name, callback); } };
  const document = { getElementById(id) { return targets.get(id) ?? null; } };
  vm.runInNewContext(source, { window, location, document });
  return { window, location, listeners };
}

test('initial encoded fragment reveals nested ancestors and keeps another branch closed', () => {
  const outer = element('DETAILS');
  const inner = element('DETAILS', outer);
  const target = element('SPAN', inner);
  const other = element('DETAILS');
  page('#deep%20target', new Map([['deep target', target], ['other', element('SPAN', other)]]));
  assert.equal(outer.open, true);
  assert.equal(inner.open, true);
  assert.equal(other.open, false);
  assert.equal(target.scrolls, 1);
});

test('later hash navigation opens its branch without changing unrelated states', () => {
  const first = element('DETAILS');
  const later = element('DETAILS');
  const unrelated = element('DETAILS');
  const target = element('SPAN', later);
  const state = page('#first', new Map([['first', element('SPAN', first)], ['later', target], ['unrelated', element('SPAN', unrelated)]]));
  state.location.hash = '#later';
  state.listeners.get('hashchange')();
  assert.equal(first.open, true);
  assert.equal(later.open, true);
  assert.equal(unrelated.open, false);
  assert.equal(target.scrolls, 1);
});

test('host API finds replacement content and can leave scrolling to the host', () => {
  const targets = new Map();
  const state = page('', targets);
  const outer = element('DETAILS');
  const target = element('SPAN', element('LI', outer));
  targets.set('replacement', target);
  assert.equal(state.window.logseqGardenerRevealFragment('#replacement', { scroll: false }), target);
  assert.equal(outer.open, true);
  assert.equal(target.scrolls, 0);
});

test('missing and malformed fragments leave closed content unchanged', () => {
  const closed = element('DETAILS');
  const target = element('SPAN', closed);
  const state = page('#absent', new Map([['target', target]]));
  for (const hash of ['', '#absent', '#%invalid']) {
    assert.equal(state.window.logseqGardenerRevealFragment(hash), null);
  }
  assert.equal(closed.open, false);
  assert.equal(target.scrolls, 0);
});


test('property summary fragment reveals its own disclosure without opening another branch', () => {
  const properties = element('DETAILS');
  const summary = element('SUMMARY', properties);
  const outline = element('DETAILS');
  const state = page('', new Map([['page-properties', summary], ['other', element('SPAN', outline)]]));
  assert.equal(state.window.logseqGardenerRevealFragment('#page-properties'), summary);
  assert.equal(properties.open, true);
  assert.equal(outline.open, false);
  assert.equal(summary.scrolls, 1);
});
