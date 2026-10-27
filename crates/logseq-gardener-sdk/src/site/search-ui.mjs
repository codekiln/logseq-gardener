// Shared search controller. The engine and search data load only after an explicit open.
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

// Loads the manifest, engine and title index on first use, then the shards each query
// needs and the chunks of the rows it draws. Fetched files stay for the publication.
export function createSearchLoader({ fetchFile = (...args) => fetch(...args), importEngine = url => import(url) } = {}) {
  let current = null, cache = null, pending = null, controller = null, generation = 0, parts = new Map();
  function cancel() {
    generation++;
    controller?.abort(); controller = null; pending = null;
    // Unfinished shard and chunk fetches restart on the next query.
    for (const [key, part] of parts) if (!part.done) parts.delete(key);
  }
  function setPublication(publication) {
    if (current?.key === publication?.key) return false;
    cancel(); current = publication; cache = null; parts = new Map();
    return true;
  }
  async function textFile(url, signal) {
    const response = await fetchFile(url, { signal, redirect: 'error', credentials: 'same-origin', cache: 'no-cache' });
    if (!response.ok || (response.url && response.url !== url)) fail();
    return response.text();
  }
  function signal() {
    controller ||= new AbortController();
    const token = generation, active = controller.signal;
    return { signal: active, check() { if (active.aborted || generation !== token) throw new DOMException('Search closed or publication changed', 'AbortError'); } };
  }
  function load() {
    if (!current) return Promise.reject(new Error('Search is unavailable on this page.'));
    if (cache) return Promise.resolve(cache);
    if (pending) return pending;
    const publication = current, token = generation, run = signal();
    pending = (async () => {
      const manifest = JSON.parse(await textFile(publication.manifest, run.signal));
      run.check();
      if (manifest?.format_version !== 2 || manifest.revision !== publication.revision ||
          !/^lsg-search-engine-[a-f0-9]{16}\.mjs$/u.test(manifest.engine || '')) fail();
      const engineURL = staticURL(manifest.engine, publication.manifest, publication.origin);
      const engine = await importEngine(engineURL.href);
      run.check();
      const prepared = engine.prepareManifest(manifest, publication.revision);
      const titles = await textFile(staticURL(engine.titlesFile(prepared), publication.manifest, publication.origin).href, run.signal);
      run.check();
      const index = engine.createIndex(prepared, titles);
      cache = Object.freeze({ engine, index, publication });
      return cache;
    })().finally(() => { if (token === generation) pending = null; });
    return pending;
  }
  // Fetch each named file once; concurrent queries share the request.
  function fetchParts(names, add) {
    const { publication } = cache, run = signal();
    return Promise.all(names.map(([key, name, number]) => {
      if (!parts.has(key)) {
        const part = { done: false };
        part.promise = textFile(staticURL(name, publication.manifest, publication.origin).href, run.signal)
          .then(source => { run.check(); add(number, source); part.done = true; })
          .catch(problem => { if (parts.get(key) === part) parts.delete(key); throw problem; });
        parts.set(key, part);
      }
      return parts.get(key).promise;
    })).then(() => run.check());
  }
  /** Load the shards `input` needs into the cached index. */
  function shards(input) {
    const { engine, index } = cache;
    return fetchParts(engine.missingShards(index, input).map(key => ['shard:' + key, engine.shardFile(index.manifest, key), key]),
      (key, source) => engine.addShard(index, key, source));
  }
  /** Load the chunks that hold these block numbers into the cached index. */
  function chunks(blocks) {
    const { engine, index } = cache;
    return fetchParts(engine.missingChunks(index, blocks).map(n => ['chunk:' + n, engine.chunkFile(index.manifest, n), n]),
      (n, source) => engine.addChunk(index, n, source));
  }
  return { setPublication, load, shards, chunks, cancel };
}

export function installSearch(window, document) {
  if (window.logseqGardenerInitSearch) { window.logseqGardenerInitSearch(); return; }
  const loader = createSearchLoader();
  let publication = null, resolvePage = null, dialog = null, input, status, list, retry, reload;
  let invoker = null, restore = true, prepared = null, task = 0, timer = null, completion = 0, evaluation = 0;
  const triggerSelector = '[data-lsg-search]';
  const supported = () => typeof window.HTMLDialogElement?.prototype.showModal === 'function';
  const controls = () => [...document.querySelectorAll(triggerSelector)];
  const anchors = () => [...list.querySelectorAll('a')];
  function announce(message) { status.textContent = message; }
  function clearResults() { list.replaceChildren(); }
  function error() {
    clearResults(); prepared = null; dialog.setAttribute('data-lsg-search-state', 'error');
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
    input.addEventListener('input', () => { window.cancelAnimationFrame(timer); clearResults(); if (prepared) { announce('Searching…'); dialog.setAttribute('data-lsg-search-state', 'searching'); } timer = window.requestAnimationFrame(evaluate); });
    dialog.addEventListener('close', () => {
      task++; window.cancelAnimationFrame(timer); loader.cancel(); dialog.setAttribute('data-lsg-search-state', 'closed');
      if (restore) (invoker?.isConnected ? invoker : controls()[0])?.focus();
    });
    dialog.addEventListener('click', event => {
      const link = event.target.closest?.('#lsg-search-results a');
      if (link && event.button === 0 && !event.metaKey && !event.ctrlKey && !event.shiftKey && !event.altKey) close(false);
    });
    dialog.addEventListener('keydown', event => {
      event.stopPropagation();
      if (event.isComposing || event.keyCode === 229) return;
      if (event.key === 'Escape') { event.preventDefault(); close(); return; }
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === 'k' && !event.altKey && !event.shiftKey) { event.preventDefault(); input.focus(); input.select(); return; }
      if (event.key === 'Enter' && document.activeElement === input) { window.cancelAnimationFrame(timer); evaluate(); }
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
  function complete() {
    dialog.setAttribute('data-lsg-search-state', 'ready');
    list.setAttribute('data-lsg-search-complete', String(++completion));
  }
  // Rows drawn per query until the grouped dialog replaces this list.
  const ROWS = 20;
  // Queries whose shards and chunks are loaded draw at once; otherwise the missing files
  // load first and the query runs again, unless a newer one replaced it.
  function evaluate() {
    window.cancelAnimationFrame(timer); timer = null;
    if (!dialog?.open || !prepared) return;
    const token = ++evaluation, data = prepared, value = input.value, { engine, index } = data;
    const again = () => { if (token === evaluation && prepared === data) evaluate(); };
    const failed = problem => { if (token === evaluation && prepared === data && dialog.open && problem.name !== 'AbortError') error(); };
    if (engine.missingShards(index, value).length) { loader.shards(value).then(again, failed); return; }
    const outcome = engine.query(index, value);
    if (outcome.error) { clearResults(); announce('Use up to 256 characters and 16 distinct words.'); complete(); return; }
    const rows = [...outcome.pages.results, ...outcome.blocks.results].slice(0, ROWS);
    const blocks = rows.filter(row => row.kind === 'block').map(row => row.block);
    if (engine.missingChunks(index, blocks).length) { loader.chunks(blocks).then(again, failed); return; }
    clearResults();
    const route = resolvePage || ((_title, href) => new URL(href, publication.manifest).href);
    for (const result of rows) {
      const row = result.kind === 'block' ? engine.blockRow(index, result.block) : result;
      const item = make('li');
      const link = make('a', { href: engine.resolveResult(row, route) });
      link.append(make('span', { class: 'lsg-search-result-title' }, row.alias || row.title));
      if (row.alias) link.append(make('span', { class: 'lsg-search-context' }, `alias of ${row.title}`));
      if (row.parents?.length) link.append(make('span', { class: 'lsg-search-context' }, row.parents.join(' › ')));
      if (row.text) link.append(make('span', { class: 'lsg-search-snippet' }, engine.snippet(row.text, value)));
      item.append(link); list.append(item);
    }
    announce(!value.trim() ? 'Type to search published pages and notes.'
      : rows.length ? `${rows.length} results shown.` : 'No results.');
    complete();
  }
  async function load() {
    const token = ++task;
    retry.hidden = true; reload.hidden = true; clearResults();
    announce('Loading search…'); dialog.setAttribute('data-lsg-search-state', 'loading'); dialog.setAttribute('aria-busy', 'true');
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
    if (changed) { task++; prepared = null; window.cancelAnimationFrame(timer); if (dialog) { clearResults(); close(); } }
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
