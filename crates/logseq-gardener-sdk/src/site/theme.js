/** Shared visitor appearance, independent of replaceable page content. */
(() => {
  try {
    // Saved shell layout applies before paint; the phone overlay always starts closed.
    const root = document.documentElement;
    if (window.matchMedia?.('(min-width: 640px)')?.matches && window.localStorage.getItem('logseq-gardener-left-sidebar') === 'open') root.setAttribute('data-lsg-left-sidebar', 'open');
    if (window.localStorage.getItem('logseq-gardener-wide') === 'true') root.setAttribute('data-lsg-wide', 'true');
    // The right sidebar's column takes its width before paint; its cards load after load.
    const right = JSON.parse(window.localStorage.getItem('logseq-gardener-right-sidebar') || 'null');
    if (right && window.matchMedia?.('(min-width: 640px)')?.matches) {
      root.setAttribute('data-lsg-right-sidebar', right.open ? 'open' : 'closed');
      if (right.width > 0) root.style.setProperty('--lsg-right-width', right.width + 'px');
    }
  } catch { /* Layout keeps its defaults. */ }
  if (typeof window.logseqGardenerInitTheme === 'function') return;
  const key = 'logseq-gardener-theme';
  const valid = value => value === 'light' || value === 'dark';
  const read = () => {
    try { const value = window.localStorage.getItem(key); return valid(value) ? value : 'system'; }
    catch { return 'system'; }
  };
  let choice = read();
  let media;
  try { media = window.matchMedia?.('(prefers-color-scheme: dark)'); } catch { /* Light fallback. */ }
  const appearance = () => choice === 'system' ? (media?.matches ? 'dark' : 'light') : choice;
  const initialize = (root = document) => {
    for (const select of root.querySelectorAll('[data-lsg-theme-select]')) {
      select.value = choice;
      const control = select.closest('[data-lsg-theme-control]');
      if (control) control.hidden = false;
    }
  };
  const apply = () => {
    document.documentElement.setAttribute('data-lsg-theme', appearance());
    document.documentElement.setAttribute('data-lsg-theme-mode', choice);
    initialize();
  };
  const set = value => {
    if (!valid(value) && value !== 'system') return;
    choice = value;
    try {
      if (choice === 'system') window.localStorage.removeItem(key);
      else window.localStorage.setItem(key, choice);
    } catch { /* Keep this document's choice when storage is unavailable. */ }
    apply();
  };
  window.logseqGardenerInitTheme = initialize;
  window.logseqGardenerTheme = { choice: () => choice, appearance, set };
  apply();
  if (media?.addEventListener) media.addEventListener('change', apply);
  else media?.addListener?.(apply);
  document.addEventListener('change', event => {
    const select = event.target;
    if (select?.matches?.('[data-lsg-theme-select]')) set(select.value);
  });
  window.addEventListener('storage', event => {
    if (event.key === key || event.key === null) { choice = read(); apply(); }
  });
  if (document.readyState === 'loading') document.addEventListener('DOMContentLoaded', () => initialize(), { once: true });
})();
