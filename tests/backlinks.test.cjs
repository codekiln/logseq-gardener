const { readFileSync } = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const test = require('node:test');
const assert = require('node:assert/strict');
const source = readFileSync(path.join(__dirname, '../crates/logseq-gardener-sdk/src/site/backlinks.js'), 'utf8');

// A small tree exercises the script's real selectors and section boundaries.
class Element {
  constructor(attributes = {}, children = []) {
    this.attributes = attributes;
    this.children = children;
    for (const child of children) child.parentElement = this;
    this.hidden = false;
    this.value = '';
    this.textContent = '';
    this.listeners = new Map();
    this.open = false;
  }
  getAttribute(name) { return this.attributes[name] ?? null; }
  matches(selector) { return Object.hasOwn(this.attributes, selector.slice(1, -1)); }
  querySelectorAll(selector) {
    return this.children.flatMap(child => [
      ...(child.matches(selector) ? [child] : []), ...child.querySelectorAll(selector),
    ]);
  }
  querySelector(selector) { return this.querySelectorAll(selector)[0] ?? null; }
  addEventListener(type, listener) {
    if (!this.listeners.has(type)) this.listeners.set(type, []);
    this.listeners.get(type).push(listener);
  }
  fire(type) { for (const listener of this.listeners.get(type) ?? []) listener({ target: this }); }
}
const el = (name, value = '', children = []) => new Element({ [name]: value }, children);
function fixture() {
  const filters = new Map();
  const options = new Map();
  for (const [route, label] of [['a.html', 'Alpha'], ['b.html', 'Beta'], ['c.html', 'Gamma'], ['one.html', 'Source One']]) {
    const filter = el('data-backlinks-filter', route);
    filters.set(route, filter);
    const option = new Element({ 'data-backlinks-option': '', 'data-backlinks-label': label }, [filter]);
    options.set(route, option);
  }
  const search = el('data-backlinks-search');
  const reset = el('data-backlinks-reset');
  const controls = el('data-backlinks-controls', '', [search, reset, ...options.values()]);
  controls.hidden = true;
  const pages = el('data-backlinks-page-count');
  const references = el('data-backlinks-reference-count');
  const status = el('data-backlinks-status');
  const entry = refs => new Element({ 'data-backlinks-entry': '', 'data-backlinks-refs': refs });
  const entries = [entry('one.html a.html b.html'), entry('one.html a.html c.html'), entry('two.html a.html b.html c.html')];
  const groups = [el('data-backlinks-group', 'one.html', entries.slice(0, 2)), el('data-backlinks-group', 'two.html', entries.slice(2))];
  const disclosure = new Element({}, [pages, references, controls, status, ...groups]);
  const section = el('data-lsg-backlinks', '', [disclosure]);
  return { section, filters, options, search, reset, controls, pages, references, status, entries, groups, disclosure };
}
function setup(...fixtures) {
  const document = new Element({}, fixtures.map(item => item.section));
  const window = {};
  vm.runInNewContext(source, { window, document });
  return { document, init: window.logseqGardenerInitBacklinks };
}
function choose(f, route, value) {
  const filter = f.filters.get(route);
  filter.value = value;
  filter.fire('change');
}
function visibleEntries(f) { return f.entries.map(entry => !entry.hidden); }

test('initial enhancement exposes controls, totals, and keeps disclosure state', () => {
  const f = fixture();
  setup(f);
  assert.equal(f.controls.hidden, false);
  assert.equal(f.pages.textContent, '2 linking pages');
  assert.equal(f.references.textContent, '3 references');
  assert.equal(f.status.textContent, '');
  assert.equal(f.disclosure.open, false);
  assert.deepEqual(visibleEntries(f), [true, true, true]);
});

test('every include and any exclude determine entries and distinct page counts', () => {
  const f = fixture();
  setup(f);
  choose(f, 'a.html', 'include');
  choose(f, 'b.html', 'include');
  assert.deepEqual(visibleEntries(f), [true, false, true]);
  assert.equal(f.pages.textContent, '2 of 2 linking pages');
  assert.equal(f.references.textContent, '2 of 3 references');
  choose(f, 'c.html', 'exclude');
  assert.deepEqual(visibleEntries(f), [true, false, false]);
  assert.deepEqual(f.groups.map(group => group.hidden), [false, true]);
  assert.equal(f.pages.textContent, '1 of 2 linking pages');
  assert.equal(f.references.textContent, '1 of 3 references');
  assert.match(f.status.textContent, /Showing 1 of 2 linking pages and 1 of 3 references/);
  choose(f, 'b.html', '');
  choose(f, 'c.html', '');
  assert.deepEqual(visibleEntries(f), [true, true, true]);
});

test('source identities can filter and empty results have readable status', () => {
  const f = fixture();
  setup(f);
  choose(f, 'one.html', 'include');
  assert.deepEqual(visibleEntries(f), [true, true, false]);
  choose(f, 'a.html', 'exclude');
  assert.deepEqual(visibleEntries(f), [false, false, false]);
  assert.ok(f.groups.every(group => group.hidden));
  assert.equal(f.pages.textContent, '0 of 2 linking pages');
  assert.equal(f.references.textContent, '0 of 3 references');
  assert.equal(f.status.textContent, 'No references match these filters.');
});

test('option search preserves active filters and reset restores results and controls', () => {
  const f = fixture();
  setup(f);
  choose(f, 'a.html', 'include');
  choose(f, 'c.html', 'exclude');
  f.search.value = '  bEtA  ';
  f.search.fire('input');
  assert.equal(f.options.get('a.html').hidden, false);
  assert.equal(f.options.get('b.html').hidden, false);
  assert.equal(f.options.get('c.html').hidden, false);
  assert.equal(f.options.get('one.html').hidden, true);
  assert.deepEqual(visibleEntries(f), [true, false, false]);
  assert.equal(f.references.textContent, '1 of 3 references');
  choose(f, 'c.html', '');
  assert.equal(f.options.get('c.html').hidden, true);
  f.reset.fire('click');
  assert.ok([...f.filters.values()].every(filter => filter.value === ''));
  assert.ok([...f.options.values()].every(option => !option.hidden));
  assert.equal(f.search.value, '');
  assert.deepEqual(visibleEntries(f), [true, true, true]);
  assert.ok(f.groups.every(group => !group.hidden));
  assert.equal(f.pages.textContent, '2 linking pages');
  assert.equal(f.references.textContent, '3 references');
  assert.equal(f.status.textContent, '');
});

test('repeated initialization preserves selection and installs each handler once', () => {
  const f = fixture();
  const { init } = setup(f);
  choose(f, 'a.html', 'exclude');
  init();
  init(f.section);
  init(new Element({}, [f.section]));
  assert.equal(f.filters.get('a.html').value, 'exclude');
  assert.deepEqual(visibleEntries(f), [false, false, false]);
  for (const filter of f.filters.values()) assert.equal(filter.listeners.get('change').length, 1);
  assert.equal(f.search.listeners.get('input').length, 1);
  assert.equal(f.reset.listeners.get('click').length, 1);
});

test('host replacement starts with fresh controls and isolates section state', () => {
  const old = fixture();
  const { document, init } = setup(old);
  choose(old, 'a.html', 'exclude');
  const fresh = fixture();
  // Browsers can restore native field values; a newly initialized page starts clear.
  fresh.filters.get('a.html').value = 'exclude';
  fresh.search.value = 'restored';
  document.children = [fresh.section];
  init();
  assert.equal(fresh.search.value, '');
  assert.ok([...fresh.filters.values()].every(filter => filter.value === ''));
  assert.deepEqual(visibleEntries(fresh), [true, true, true]);
  choose(fresh, 'b.html', 'include');
  assert.deepEqual(visibleEntries(fresh), [true, false, true]);
  assert.deepEqual(visibleEntries(old), [false, false, false]);
});

test('zero references and absent sections initialize without losing native disclosure', () => {
  const f = fixture();
  f.disclosure.children = f.disclosure.children.filter(child => !f.groups.includes(child));
  const { init } = setup(f);
  assert.equal(f.pages.textContent, '0 linking pages');
  assert.equal(f.references.textContent, '0 references');
  assert.equal(f.disclosure.open, false);
  init(new Element());
});

function outlinedFixture() {
  const f = fixture();
  const node = (refs, children = []) => new Element({
    'data-backlinks-node': '', 'data-backlinks-refs': refs,
  }, children);
  const matching = node('one.html a.html b.html');
  const unrelated = node('one.html a.html c.html');
  const intermediate = node('one.html a.html', [new Element({}, [matching])]);
  const parent = node('one.html a.html', [new Element({}, [intermediate, unrelated])]);
  // The intermediate wrappers model UL/details elements in rendered outlines.
  f.entries[0].children = [parent];
  parent.parentElement = f.entries[0];
  return { ...f, parent, intermediate, matching, unrelated };
}

test('descendant inclusion retains its ancestor chain and hides unrelated branches', () => {
  const f = outlinedFixture();
  setup(f);
  choose(f, 'b.html', 'include');
  assert.equal(f.parent.hidden, false);
  assert.equal(f.intermediate.hidden, false);
  assert.equal(f.matching.hidden, false);
  assert.equal(f.unrelated.hidden, true);
  assert.equal(f.entries[0].hidden, false);
  assert.equal(f.references.textContent, '2 of 3 references');
  assert.equal(f.pages.textContent, '2 of 2 linking pages');
  assert.equal(f.intermediate.open, false);
});

test('matching parent remains while excluded descendant hides; reset restores every branch', () => {
  const f = outlinedFixture();
  setup(f);
  f.intermediate.open = true;
  choose(f, 'a.html', 'include');
  choose(f, 'c.html', 'exclude');
  assert.equal(f.parent.hidden, false);
  assert.equal(f.intermediate.hidden, false);
  assert.equal(f.matching.hidden, false);
  assert.equal(f.unrelated.hidden, true);
  assert.equal(f.references.textContent, '1 of 3 references');
  choose(f, 'b.html', 'exclude');
  assert.equal(f.parent.hidden, false);
  assert.equal(f.matching.hidden, true);
  assert.equal(f.intermediate.hidden, false);
  assert.equal(f.intermediate.open, true);
  f.reset.fire('click');
  for (const node of [f.parent, f.intermediate, f.matching, f.unrelated]) assert.equal(node.hidden, false);
  assert.deepEqual(visibleEntries(f), [true, true, true]);
  assert.equal(f.references.textContent, '3 references');
  assert.equal(f.intermediate.open, true);
});

test('node paths determine entry matching and all hidden branches hide their source group', () => {
  const f = outlinedFixture();
  setup(f);
  choose(f, 'one.html', 'exclude');
  assert.equal(f.entries[0].hidden, true);
  assert.equal(f.parent.hidden, true);
  assert.equal(f.matching.hidden, true);
  assert.equal(f.groups[0].hidden, true);
  assert.equal(f.entries[2].hidden, false);
  assert.equal(f.pages.textContent, '1 of 2 linking pages');
  assert.equal(f.references.textContent, '1 of 3 references');
});

test('includes must match one node path rather than a union of sibling paths or entry fallback', () => {
  const f = outlinedFixture();
  f.entries[0].attributes['data-backlinks-refs'] = 'one.html a.html b.html c.html';
  setup(f);
  choose(f, 'b.html', 'include');
  choose(f, 'c.html', 'include');
  assert.equal(f.entries[0].hidden, true);
  assert.equal(f.groups[0].hidden, true);
  assert.equal(f.matching.hidden, true);
  assert.equal(f.unrelated.hidden, true);
  assert.equal(f.entries[2].hidden, false);
  assert.equal(f.references.textContent, '1 of 3 references');
});
