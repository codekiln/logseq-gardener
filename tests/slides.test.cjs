const { readFileSync } = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const test = require('node:test');
const assert = require('node:assert/strict');
const { Element, h } = require('./mini-dom.cjs');
const source = readFileSync(path.join(__dirname, '../crates/logseq-gardener-sdk/src/site/slides.js'), 'utf8');

let focused;
Element.onFocus = el => { focused = el; };
const bullet = () => h('a', { class: 'bullet', href: '#' });
// An outline item as the publisher writes it: anchor, bullet, content, then child blocks.
const block = (attrs, content, ...children) => h('li', attrs, h('span', { id: 'source-block-1' }), bullet(), ...content,
  ...(children.length ? [h('ul', {}, ...children)] : []));

// Shaped like the AI Engineer Summit page: a heading with nested children, a Markdown list
// inside a block, an empty block, a collapsed block and a slide background.
function outline() {
  return h('ul', { class: 'outline' },
    block({}, [h('h1', {}, 'What I learned')],
      block({}, [h('h2', {}, 'What is it?')],
        block({}, [h('h3', {}, 'How I found out')],
          block({}, [h('h4', {}, 'flash back'), h('p', {}, h('img', { src: 'a.png', loading: 'lazy' }))]))),
      block({}, [h('h3', {}, 'When'), h('ul', {}, h('li', {}, 'Thu'), h('li', {}, 'Fri'))])),
    block({}, []),
    block({ 'data-slide-background-image': 'https://example.com/b.jpg', 'data-slide-transition': 'fade' }, ['Powered by Reveal.js']),
    h('li', { class: 'outline-collapsed' }, h('details', { class: 'outline-disclosure' },
      h('summary', {}, bullet(), h('span', { class: 'task-marker' }, 'TODO'), 'Folded'),
      h('ul', {}, block({}, ['Inside the fold'])))));
}

function load() {
  focused = null;
  const head = h('head');
  const page = h('body');
  h('html', {}, head, page);
  const document = {
    head,
    createElement: tag => new Element(tag),
    importNode: node => node.cloneNode(true),
    get fullscreenElement() { return this.full ?? null; },
    exitFullscreen() { this.full = null; },
  };
  const window = {};
  vm.runInNewContext(source, {
    window, document, Object, Number, Math, String, Set, Map, JSON, setTimeout,
    ResizeObserver: class { constructor(callback) { this.callback = callback; } observe() {} disconnect() {} },
  });
  const body = window.logseqGardenerSlides(outline());
  page.append(body);
  const deck = body.querySelector('.reveal');
  deck.requestFullscreen = () => { document.full = deck; return Promise.resolve(); };
  const press = (key, extra = {}) => {
    const event = { key, target: deck, preventDefault() {}, stopPropagation() { event.stopped = true; }, ...extra };
    for (const callback of deck.listeners.keydown) callback(event);
    return event;
  };
  const present = () => deck.querySelector('.slides>section.present');
  return { document, body, deck, press, present, text: () => present().textContent };
}

test('the deck has one column per top-level block with its descendants below, skipping empty blocks', () => {
  const s = load();
  s.press('o');
  const columns = s.deck.querySelectorAll('.lsg-overview>div').map(column => column.children.map(slide => slide.textContent));
  assert.deepEqual(columns, [
    ['What I learned', 'What is it?', 'How I found out', 'flash back', 'WhenThuFri'],
    ['Powered by Reveal.js'],
    ['Folded', 'Inside the fold'],
  ]);
});

test('the card body is the tip and a focusable deck', () => {
  const s = load();
  assert.equal(s.body.querySelector('.lsg-slides-tip').textContent, 'Tip: press f to go fullscreen');
  assert.equal(s.body.querySelector('.lsg-slides-tip code').textContent, 'f');
  assert.equal(s.deck.getAttribute('tabindex'), '0');
  assert.equal(s.deck.getAttribute('aria-label'), 'Slides');
});

test('slides show the block without bullet, marker, anchors or child blocks', () => {
  const s = load();
  assert.equal(s.text(), 'What I learned');
  assert.equal(s.present().querySelectorAll('.bullet,.task-marker,[id]').length, 0);
  assert.equal(s.present().querySelectorAll('h1').length, 1);
  s.press('ArrowDown'); s.press('ArrowDown'); s.press('ArrowDown');
  assert.equal(s.text(), 'flash back');
  assert.equal(s.present().querySelectorAll('img').length, 1);
  s.press('ArrowDown');
  // A Markdown list inside the block stays on its slide.
  assert.equal(s.present().querySelectorAll('ul>li').length, 2);
  s.press('ArrowRight', { shiftKey: true });
  assert.equal(s.text(), 'Folded');
  assert.equal(s.present().querySelectorAll('.task-marker').length, 0);
});

test('keys move as reveal.js does and only nearby slides are in the document', () => {
  const s = load();
  const progress = () => Number(s.deck.style.getPropertyValue('--lsg-progress'));
  s.press('ArrowDown'); s.press('ArrowDown'); s.press('ArrowDown');
  assert.equal(s.text(), 'flash back');
  assert.equal(progress(), 3 / 7);
  // The current slide and its neighbours above, below and to the right.
  assert.equal(s.deck.querySelectorAll('.slides>section').length, 4);
  s.press('ArrowRight');
  assert.equal(s.text(), 'Powered by Reveal.js');
  s.press('h'); assert.equal(s.text(), 'What I learned');
  s.press('n'); s.press(' '); assert.equal(s.text(), 'How I found out');
  s.press('p'); s.press(' ', { shiftKey: true }); assert.equal(s.text(), 'What I learned');
  s.press('ArrowDown', { shiftKey: true }); assert.equal(s.text(), 'WhenThuFri');
  s.press('ArrowUp', { shiftKey: true }); assert.equal(s.text(), 'What I learned');
  s.press('l'); s.press('l'); s.press('j'); assert.equal(s.text(), 'Inside the fold');
  s.press('k'); assert.equal(s.text(), 'Folded');
  const [left, right, up, down] = s.deck.querySelectorAll('.controls button');
  assert.deepEqual([left.disabled, right.disabled, up.disabled, down.disabled], [false, true, true, false]);
  assert.deepEqual([left, right, up, down].map(b => b.getAttribute('aria-label')),
    ['previous slide', 'next slide', 'above slide', 'below slide']);
});

test('keys stay with the deck, and b and g work', () => {
  const s = load();
  assert.equal(s.press('t').stopped, true);
  s.press('b');
  assert.ok(s.deck.querySelector('.lsg-blackout'));
  s.press('ArrowRight');
  assert.equal(s.deck.querySelector('.lsg-blackout'), null);
  assert.equal(s.text(), 'What I learned');
  s.press('g');
  const field = s.deck.querySelector('input');
  assert.equal(focused, field);
  field.value = '3/2';
  for (const callback of field.listeners.keydown) callback({ key: 'Enter', preventDefault() {}, stopPropagation() {} });
  assert.equal(s.text(), 'Inside the fold');
  assert.equal(focused, s.deck);
  s.press('g');
  s.deck.querySelector('input').value = '6';
  for (const callback of s.deck.querySelector('input').listeners.keydown) callback({ key: 'Enter', preventDefault() {}, stopPropagation() {} });
  assert.equal(s.text(), 'Powered by Reveal.js');
});

test('slide properties reach the section and draw the background', () => {
  const s = load();
  s.press('ArrowRight');
  const section = s.present();
  assert.equal(section.getAttribute('data-background-image'), 'https://example.com/b.jpg');
  assert.equal(section.getAttribute('data-transition'), 'fade');
  assert.equal(s.deck.querySelector('.backgrounds').style.backgroundImage, 'url("https://example.com/b.jpg")');
  s.press('ArrowLeft');
  assert.equal(s.deck.querySelector('.backgrounds').style.backgroundImage, '');
});

test('f toggles fullscreen and the overview chooses a slide', () => {
  const s = load();
  s.press('f');
  assert.equal(s.document.fullscreenElement, s.deck);
  s.press('f');
  assert.equal(s.document.fullscreenElement, null);
  s.press('Escape');
  assert.ok(s.deck.classList.contains('overview'));
  const choice = s.deck.querySelectorAll('.lsg-overview button')[5];
  for (const callback of s.deck.listeners.click) callback({ target: choice });
  assert.equal(s.text(), 'Powered by Reveal.js');
  assert.equal(s.deck.classList.contains('overview'), false);
});
