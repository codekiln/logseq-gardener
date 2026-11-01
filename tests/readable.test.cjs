const { readFileSync } = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const test = require('node:test');
const assert = require('node:assert/strict');
const site = path.join(__dirname, '../crates/logseq-gardener-sdk/src/site');
const script = readFileSync(path.join(site, 'readable.js'), 'utf8');
const graphRoutes = readFileSync(path.join(site, 'readable-graph.js'), 'utf8');

// A document with one link and the listeners readable.js adds, at `address`.
function page({ address, href, base = true }) {
  const listeners = {};
  const link = {
    getAttribute: name => name === 'href' ? href : null,
    // A link's href property is its attribute resolved against the base, as in a browser.
    get href() { return new URL(href, base ? new URL('../../', address) : address).href; },
    set href(value) { href = value; },
    closest: selector => selector === 'a[href]' ? link : null,
  };
  const location = new URL(address);
  const document = {
    addEventListener: (type, listener, capture) => { assert.equal(capture, true); listeners[type] = listener; },
    querySelector: selector => selector === 'base[href]' && base ? {} : null,
  };
  vm.runInNewContext(script, { document, location, URL });
  const fire = (type, options = {}) => {
    const event = { type, button: 0, target: link, prevented: false, preventDefault() { this.prevented = true; }, ...options };
    listeners[type](event);
    return event;
  };
  return { fire, location, link, get href() { return href; } };
}

test('a plain click on a fragment-only link stays in the page under a base', () => {
  const bullet = page({ address: 'https://host/site/A/B/', href: '#' });
  assert.equal(bullet.fire('click').prevented, true);
  assert.equal(bullet.location.hash, '');
  const block = page({ address: 'https://host/site/A/B/', href: '#block-1' });
  assert.equal(block.fire('click').prevented, true);
  assert.equal(block.location.hash, '#block-1');
});

test('modified clicks and documents without a base keep the browser default', () => {
  const bullet = page({ address: 'https://host/site/A/B/', href: '#' });
  assert.equal(bullet.fire('click', { shiftKey: true }).prevented, false);
  assert.equal(bullet.fire('click', { button: 1 }).prevented, false);
  assert.equal(bullet.fire('auxclick', { button: 1 }).prevented, false);
  const root = page({ address: 'https://host/site/index.html', href: '#x', base: false });
  assert.equal(root.fire('click').prevented, false);
  assert.equal(root.location.hash, '');
});

test('from disk, a link to a folder opens its index.html', () => {
  for (const type of ['click', 'auxclick', 'contextmenu']) {
    const folder = page({ address: 'file:///site/A/B/index.html', href: 'Tools/Q%26A/' });
    assert.equal(folder.fire(type).prevented, false);
    assert.equal(folder.href, 'file:///site/Tools/Q%26A/index.html');
  }
  const block = page({ address: 'file:///site/A/B/index.html', href: 'Tools/#block-1' });
  block.fire('click');
  assert.equal(block.href, 'file:///site/Tools/index.html#block-1');
  const file = page({ address: 'file:///site/A/B/index.html', href: 'pages.html' });
  file.fire('click');
  assert.equal(file.href, 'pages.html');
});

test('a served site keeps folder links as written', () => {
  const folder = page({ address: 'https://host/site/A/B/', href: 'Tools/' });
  folder.fire('click');
  assert.equal(folder.href, 'Tools/');
});

test('readable graph routes match the publisher', () => {
  const { digestOf, routeOf, titleAt } = vm.runInNewContext(`${graphRoutes};({ digestOf, routeOf, titleAt })`, { TextEncoder });
  // The same titles and routes as the SDK's page_urls tests.
  for (const [title, route] of [
    ['GitP/A/Session/26/10/20-Tue', 'GitP/A/Session/26/10/20-Tue/'],
    ['Notes/Q&A: 2026', 'Notes/Q%26A%3A%202026/'],
    ['a~b_c.d-e', 'a~b_c.d-e/'],
    ["!*'()#?%+", '%21%2A%27%28%29%23%3F%25%2B/'],
    ['Café/日本', 'Caf%C3%A9/%E6%97%A5%E6%9C%AC/'],
  ]) {
    assert.equal(routeOf(title), route);
    assert.equal(titleAt(route), title);
  }
  // The digest that names p-<digest>.html in the default layout.
  assert.equal(digestOf('tmux'), '424036ef4353ec8f');
});
