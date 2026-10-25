// Shared search controller. Corpus and query engine load only after an explicit open.
const revisionPattern = /^[a-f0-9]{16}$/u;
const fail = () => { throw new Error('Search data is unavailable or from another publication. Retry, or reload this page.'); };
function staticURL(href, base, origin) {
  const url = new URL(href, base);
  if (!['http:', 'https:'].includes(url.protocol) || url.origin !== origin ||
      url.username || url.password || url.search || url.hash) fail();
  return url;
}
export function publicationFromLink(link, base, origin) {
  if (!link) return null;
  const revision = link.getAttribute('data-revision');
  if (!revisionPattern.test(revision || '')) fail();
  const manifest = staticURL(link.getAttribute('href'), base, origin);
  if (!manifest.pathname.endsWith(`/lsg-search-manifest-${revision}.json`)) fail();
  return Object.freeze({ revision, manifest: manifest.href, origin, key: manifest.href + ':' + revision });
}

export function createSearchLoader({ fetchFile = (...args) => fetch(...args), importEngine = url => import(url) } = {}) {
  let current = null, cache = null, pending = null, controller = null, generation = 0;
  function cancel() {
    generation++;
    controller?.abort(); controller = null; pending = null;
  }
  function setPublication(publication) {
    if (current?.key === publication?.key) return false;
    cancel(); current = publication; cache = null;
    return true;
  }
  async function jsonFile(url, signal, bytes) {
    const response = await fetchFile(url, { signal, redirect: 'error', credentials: 'same-origin', cache: 'no-cache' });
    if (!response.ok || (response.url && response.url !== url)) fail();
    const source = await response.text();
    if (bytes !== undefined && new TextEncoder().encode(source).length !== bytes) fail();
    return JSON.parse(source);
  }
  function load() {
    if (!current) return Promise.reject(new Error('Search is unavailable on this page.'));
    if (cache) return Promise.resolve(cache);
    if (pending) return pending;
    const publication = current, token = generation;
    controller = new AbortController();
    const signal = controller.signal;
    const active = () => { if (signal.aborted || generation !== token) throw new DOMException('Search closed or publication changed', 'AbortError'); };
    pending = (async () => {
      const manifest = await jsonFile(publication.manifest, signal);
      active();
      if (manifest.format_version !== 1 || manifest.revision !== publication.revision ||
          manifest.corpus?.href !== `lsg-search-${publication.revision}.json` ||
          manifest.engine !== `lsg-search-engine-${publication.revision}.mjs` ||
          !Number.isSafeInteger(manifest.corpus.bytes) || manifest.corpus.bytes <= 0 ||
          !Number.isSafeInteger(manifest.corpus.records) || manifest.corpus.records < 0) fail();
      const corpusURL = staticURL(manifest.corpus.href, publication.manifest, publication.origin);
      const engineURL = staticURL(manifest.engine, publication.manifest, publication.origin);
      const corpus = await jsonFile(corpusURL.href, signal, manifest.corpus.bytes);
      active();
      const engine = await importEngine(engineURL.href);
      active();
      const index = engine.prepareIndex(manifest, corpus, publication.revision);
      active();
      cache = Object.freeze({ engine, index, publication });
      return cache;
    })().finally(() => { if (token === generation) { pending = null; controller = null; } });
    return pending;
  }
  return { setPublication, load, cancel };
}

export function installSearch(window, document) {
  if (window.logseqGardenerInitSearch) { window.logseqGardenerInitSearch(); return; }
  const loader = createSearchLoader();
  let publication = null, resolvePage = null, dialog = null, input, status, list, retry, reload;
  let invoker = null, restore = true, prepared = null, task = 0, timer = null;
  const triggerSelector = '[data-lsg-search]';
  const supported = () => typeof window.HTMLDialogElement?.prototype.showModal === 'function';
  const controls = () => [...document.querySelectorAll(triggerSelector)];
  const anchors = () => [...list.querySelectorAll('a')];
  function announce(message) { status.textContent = message; }
  function clearResults() { list.replaceChildren(); }
  function error() {
    clearResults(); prepared = null;
    announce('Search data is unavailable or from another publication. Retry, or reload this page.');
    retry.hidden = false; reload.hidden = false;
  }
  function close(focus = true) { if (dialog?.open) { restore = focus; dialog.close(); } }
  function make(tag, attributes = {}, text = '') {
    const node = document.createElement(tag);
    for (const [key, value] of Object.entries(attributes)) node.setAttribute(key, value);
    node.textContent = text;
    return node;
  }
  function createDialog() {
    if (dialog) return;
    dialog = make('dialog', { id: 'lsg-search-dialog', 'aria-labelledby': 'lsg-search-title' });
    const heading = make('div', { class: 'lsg-search-heading' });
    const title = make('h2', { id: 'lsg-search-title' }, 'Search garden');
    const dismiss = make('button', { type: 'button', 'aria-label': 'Close search' }, 'Close');
    dismiss.addEventListener('click', () => close());
    heading.append(title, dismiss);
    const label = make('label', { for: 'lsg-search-input' }, 'Search published pages and notes');
    input = make('input', { id: 'lsg-search-input', type: 'search', autocomplete: 'off', maxlength: '512', 'aria-controls': 'lsg-search-results', 'aria-describedby': 'lsg-search-status' });
    status = make('p', { id: 'lsg-search-status', role: 'status', 'aria-live': 'polite', 'aria-atomic': 'true' });
    list = make('ul', { id: 'lsg-search-results', 'aria-label': 'Search results' });
    retry = make('button', { type: 'button', hidden: '' }, 'Retry search');
    reload = make('button', { type: 'button', hidden: '' }, 'Reload page');
    retry.addEventListener('click', () => { loader.cancel(); prepared = null; void load(); });
    reload.addEventListener('click', () => window.location.reload());
    input.addEventListener('input', () => { clearTimeout(timer); clearResults(); if (prepared) announce('Searching…'); timer = setTimeout(evaluate, 60); });
    dialog.addEventListener('close', () => {
      task++; clearTimeout(timer); loader.cancel();
      if (restore) (invoker?.isConnected ? invoker : controls()[0])?.focus();
    });
    dialog.addEventListener('click', event => {
      const link = event.target.closest?.('#lsg-search-results a');
      if (link && event.button === 0 && !event.metaKey && !event.ctrlKey && !event.shiftKey && !event.altKey) close(false);
    });
    dialog.addEventListener('keydown', event => {
      event.stopPropagation();
      if (event.key === 'Escape') { event.preventDefault(); close(); return; }
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'k' && !event.altKey && !event.shiftKey) { event.preventDefault(); input.focus(); input.select(); return; }
      if (event.key === 'Enter' && document.activeElement === input) { clearTimeout(timer); evaluate(); }
      const choices = anchors();
      if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
        if (!choices.length || ![input, ...choices].includes(document.activeElement)) return;
        const current = choices.indexOf(document.activeElement);
        const next = current < 0 ? (event.key === 'ArrowDown' ? 0 : choices.length - 1)
          : (current + (event.key === 'ArrowDown' ? 1 : -1) + choices.length) % choices.length;
        choices[next].focus(); event.preventDefault();
      } else if (event.key === 'Enter' && document.activeElement === input && !event.isComposing && !event.metaKey && !event.ctrlKey && !event.altKey && !event.shiftKey) {
        if (choices.length) { event.preventDefault(); choices[0].click(); }
      }
    });
    dialog.append(heading, label, input, status, retry, reload, list);
    document.body.append(dialog);
  }
  function evaluate() {
    if (!dialog?.open || !prepared) return;
    clearResults();
    const outcome = prepared.engine.query(prepared.index, input.value);
    if (outcome.error) { announce('Use up to 256 characters and 16 distinct words.'); return; }
    for (const result of outcome.results) {
      const item = make('li');
      const link = make('a', { href: prepared.engine.resolveResult(result,
        resolvePage || ((_title, href) => new URL(href, publication.manifest).href)) });
      link.append(make('span', { class: 'lsg-search-result-title' }, result.title));
      if (result.ancestors.length) link.append(make('span', { class: 'lsg-search-context' }, result.ancestors.join(' › ')));
      if (result.text) link.append(make('span', { class: 'lsg-search-snippet' }, result.text));
      item.append(link); list.append(item);
    }
    announce(!input.value.trim() ? 'Type to search published pages and notes.'
      : outcome.results.length ? `${outcome.results.length} results shown.` : 'No results.');
  }
  async function load() {
    const token = ++task;
    retry.hidden = true; reload.hidden = true; clearResults();
    announce('Loading search…'); dialog.setAttribute('aria-busy', 'true');
    try {
      const data = await loader.load();
      if (token !== task || !dialog.open) return;
      prepared = data; evaluate();
    } catch (problem) {
      if (token === task && dialog.open && problem.name !== 'AbortError') error();
    } finally { if (token === task) dialog.removeAttribute('aria-busy'); }
  }
  function initialize(_root = document, options = {}) {
    if (Object.hasOwn(options, 'resolvePage')) resolvePage = options.resolvePage;
    let next = null;
    try { next = publicationFromLink(document.head.querySelector('link[rel="lsg-search"]'), document.baseURI, window.location.origin); }
    catch { /* Invalid metadata keeps ordinary navigation usable. */ }
    const changed = loader.setPublication(next);
    publication = next;
    if (changed) { task++; prepared = null; clearTimeout(timer); if (dialog) { clearResults(); close(); } }
    for (const control of controls()) control.hidden = !publication || !supported();
  }
  function open(control) {
    initialize();
    if (!publication || !supported()) return;
    createDialog();
    if (!dialog.open) { invoker = control || document.activeElement; restore = true; dialog.showModal(); }
    input.focus(); input.select();
    if (prepared) evaluate(); else void load();
  }
  document.addEventListener('click', event => {
    const control = event.target.closest?.(triggerSelector);
    if (control) { event.preventDefault(); open(control); }
  });
  document.addEventListener('keydown', event => {
    if ((event.metaKey || event.ctrlKey) && !event.altKey && !event.shiftKey && event.key.toLowerCase() === 'k' && !event.isComposing && publication && supported()) {
      event.preventDefault(); open(document.activeElement);
    }
  });
  window.logseqGardenerInitSearch = initialize;
  initialize();
}
if (typeof window !== 'undefined' && typeof document !== 'undefined') installSearch(window, document);
