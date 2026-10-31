// A small DOM for the browser-script tests: elements, text, the selectors the scripts use,
// and listeners. Element.onFocus reports focus to the test.
function compound(text) {
  const parts = { tag: null, classes: [], ids: [], attrs: [], scope: false };
  for (const [token] of text.matchAll(/:scope|\.[\w-]+|#[\w-]+|\[[^\]]+\]|[\w-]+/g)) {
    if (token === ':scope') parts.scope = true;
    else if (token[0] === '.') parts.classes.push(token.slice(1));
    else if (token[0] === '#') parts.ids.push(token.slice(1));
    else if (token[0] === '[') {
      const [, name, op, value] = token.match(/^\[([\w-]+)(?:([*^]?=)"?([^"\]]*)"?)?\]$/);
      parts.attrs.push([name, op, value]);
    } else parts.tag = token.toUpperCase();
  }
  return parts;
}
function matchesCompound(el, parts, scope) {
  if (!el || el.nodeType !== 1) return false;
  if (parts.scope && el !== scope) return false;
  if (parts.tag && el.tagName !== parts.tag) return false;
  if (!parts.classes.every(c => el.classList.contains(c))) return false;
  if (!parts.ids.every(id => el.id === id)) return false;
  return parts.attrs.every(([name, op, value]) => {
    const actual = el.getAttribute(name);
    if (actual === null) return false;
    return !op || (op === '=' ? actual === value : op === '*=' ? actual.includes(value) : actual.startsWith(value));
  });
}
function matches(el, selector, scope) {
  return selector.split(',').some(one => {
    const tokens = one.trim().replace(/\s*>\s*/g, ' > ').split(/\s+/);
    const step = (node, i) => {
      if (!matchesCompound(node, compound(tokens[i]), scope)) return false;
      if (i === 0) return true;
      if (tokens[i - 1] === '>') return step(node.parentElement, i - 2);
      for (let up = node.parentElement; up; up = up.parentElement) if (step(up, i - 1)) return true;
      return false;
    };
    return step(el, tokens.length - 1);
  });
}
class Node {
  constructor() { this.parentElement = null; this.childNodes = []; }
  remove() { if (this.parentElement) { const list = this.parentElement.childNodes; list.splice(list.indexOf(this), 1); this.parentElement = null; } }
  after(other) { const p = this.parentElement; other.remove(); p.childNodes.splice(p.childNodes.indexOf(this) + 1, 0, other); other.parentElement = p; }
}
class Text extends Node {
  constructor(text) { super(); this.nodeType = 3; this.text = text; }
  get textContent() { return this.text; }
  cloneNode() { return new Text(this.text); }
  get nodeName() { return '#text'; }
}
class Element extends Node {
  constructor(tag, attrs = {}) {
    super();
    this.nodeType = 1; this.tagName = tag.toUpperCase(); this.attrs = new Map(Object.entries(attrs)); this.listeners = {};
    const el = this;
    this.classList = {
      contains: c => (el.attrs.get('class') ?? '').split(' ').includes(c),
      add: c => { if (!el.classList.contains(c)) el.attrs.set('class', ((el.attrs.get('class') ?? '') + ' ' + c).trim()); },
      remove: c => el.attrs.set('class', (el.attrs.get('class') ?? '').split(' ').filter(x => x && x !== c).join(' ')),
      toggle: (c, force) => { const on = force ?? !el.classList.contains(c); if (on) el.classList.add(c); else el.classList.remove(c); return on; },
    };
  }
  get id() { return this.attrs.get('id') ?? ''; }
  get open() { return this.attrs.has('open'); }
  set open(value) { if (value) this.attrs.set('open', ''); else this.attrs.delete('open'); }
  get hidden() { return this.attrs.has('hidden'); }
  set hidden(value) { if (value) this.attrs.set('hidden', ''); else this.attrs.delete('hidden'); }
  get children() { return this.childNodes.filter(n => n.nodeType === 1); }
  get textContent() { return this.childNodes.map(n => n.textContent).join(''); }
  set textContent(text) { this.replaceChildren(text); }
  getAttribute(name) { return this.attrs.has(name) ? this.attrs.get(name) : null; }
  setAttribute(name, value) { this.attrs.set(name, String(value)); }
  removeAttribute(name) { this.attrs.delete(name); }
  append(...nodes) {
    for (let node of nodes) {
      if (typeof node === 'string') node = new Text(node);
      node.remove();
      node.parentElement = this;
      this.childNodes.push(node);
    }
  }
  prepend(node) { node.remove(); node.parentElement = this; this.childNodes.unshift(node); }
  insertBefore(node, before) {
    node.remove();
    node.parentElement = this;
    const at = before ? this.childNodes.indexOf(before) : -1;
    if (at < 0) this.childNodes.push(node); else this.childNodes.splice(at, 0, node);
  }
  replaceChildren(...nodes) { for (const node of [...this.childNodes]) node.remove(); this.append(...nodes); }
  descendants() { return this.children.flatMap(c => [c, ...c.descendants()]); }
  querySelectorAll(selector) { return this.descendants().filter(el => matches(el, selector, this)); }
  querySelector(selector) { return this.querySelectorAll(selector)[0] ?? null; }
  matches(selector) { return matches(this, selector, null); }
  closest(selector) { for (let el = this; el; el = el.parentElement) if (el.matches(selector)) return el; return null; }
  contains(other) { for (let el = other; el; el = el.parentElement) if (el === this) return true; return false; }
  // Always deep, as the scripts ask for.
  cloneNode() {
    const copy = new Element(this.tagName, Object.fromEntries(this.attrs));
    for (const child of this.childNodes) copy.append(child.cloneNode(true));
    return copy;
  }
  addEventListener(type, callback) { (this.listeners[type] ||= []).push(callback); }
  removeEventListener(type, callback) { this.listeners[type] = (this.listeners[type] || []).filter(c => c !== callback); }
  focus() { Element.onFocus?.(this); }
  // What the slide script reads besides the sidebar's needs.
  get nodeName() { return this.tagName; }
  get attributes() { return [...this.attrs].map(([name, value]) => ({ name, value })); }
  get dataset() { return Object.fromEntries([...this.attrs].filter(([n]) => n.startsWith('data-')).map(([n, v]) => [n.slice(5).replace(/-(\w)/g, (_, c) => c.toUpperCase()), v])); }
  get isConnected() { let el = this; while (el.parentElement) el = el.parentElement; return el.tagName === 'HTML'; }
  get offsetWidth() { return 0; }
  toggleAttribute(name, force) { const on = force ?? !this.attrs.has(name); if (on) this.attrs.set(name, ''); else this.attrs.delete(name); return on; }
  scrollIntoView() {}
  get style() { return this._style ||= { setProperty: (n, v) => { this._style[n] = String(v); }, getPropertyValue: n => this._style[n] ?? '' }; }
  set style(value) { this._style = value; }
  getBoundingClientRect() { return { width: Number(this.style?.width ?? 512) }; }
}
// h('li', {class: 'x'}, child, 'text') builds markup.
const h = (tag, attrs = {}, ...children) => { const el = new Element(tag, attrs); el.append(...children); return el; };

module.exports = { Element, Text, h, matches };
