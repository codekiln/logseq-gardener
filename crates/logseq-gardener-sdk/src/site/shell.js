/** Logseq's header, left sidebar, More menu and shell shortcuts. */
(() => {
  if (window.logseqGardenerShell) return;
  window.logseqGardenerShell = true;
  const root = document.documentElement;
  const $ = selector => document.querySelector(selector);
  const store = (key, value) => {
    try { if (value) window.localStorage.setItem(key, value); else window.localStorage.removeItem(key); }
    catch { /* The current page keeps its state. */ }
  };
  const desktop = () => { try { return window.matchMedia('(min-width: 640px)').matches; } catch { return true; } };
  const theme = () => window.logseqGardenerTheme;
  const isOpen = () => root.getAttribute('data-lsg-left-sidebar') === 'open';
  const setSidebar = (open, focus) => {
    if (open) root.setAttribute('data-lsg-left-sidebar', 'open');
    else root.removeAttribute('data-lsg-left-sidebar');
    const button = $('[data-lsg-left-toggle]');
    button?.setAttribute('aria-expanded', String(open));
    const shade = $('[data-lsg-sidebar-shade]');
    if (shade) shade.hidden = !open || desktop();
    store('logseq-gardener-left-sidebar', open && 'open');
    if (!open && focus) button?.focus();
  };
  const menu = () => $('#lsg-more-menu');
  const items = () => [...(menu()?.querySelectorAll('[role="menuitem"]') ?? [])].filter(item => !item.hidden);
  const setMenu = (open, focus = true) => {
    const list = menu(), button = $('[data-lsg-more]');
    if (!list || !button) return;
    list.hidden = !open;
    button.setAttribute('aria-expanded', String(open));
    if (open) {
      const system = $('[data-lsg-theme-system]');
      if (system) system.hidden = theme()?.choice() === 'system';
      items()[0]?.focus();
    } else if (focus) button.focus();
  };
  const toggleTheme = () => { const t = theme(); t?.set(t.appearance() === 'dark' ? 'light' : 'dark'); };
  const toggleWide = () => {
    const wide = root.getAttribute('data-lsg-wide') !== 'true';
    if (wide) root.setAttribute('data-lsg-wide', 'true'); else root.removeAttribute('data-lsg-wide');
    store('logseq-gardener-wide', wide && 'true');
  };
  const go = selector => { const href = $(selector)?.getAttribute('href'); if (href) window.location.assign(href); };
  const actions = {
    tl: () => setSidebar(!isOpen()),
    tt: toggleTheme,
    tw: toggleWide,
    gh: () => window.location.assign('index.html'),
    ga: () => go('[data-lsg-all-pages]'),
  };
  document.addEventListener('click', event => {
    const target = event.target;
    if (menu() && !menu().hidden && !target.closest?.('.lsg-more')) setMenu(false, false);
    const control = target.closest?.('[data-lsg-left-toggle],[data-lsg-more],[data-lsg-theme-toggle],[data-lsg-theme-system],[data-lsg-sidebar-shade],.lsg-left-sidebar a');
    if (!control) return;
    if (control.matches('[data-lsg-left-toggle]')) setSidebar(!isOpen());
    else if (control.matches('[data-lsg-more]')) setMenu(menu()?.hidden);
    else if (control.matches('[data-lsg-theme-toggle]')) { toggleTheme(); setMenu(false); }
    else if (control.matches('[data-lsg-theme-system]')) { theme()?.set('system'); setMenu(false); }
    else if (control.matches('[data-lsg-sidebar-shade]')) setSidebar(false, true);
    else if (!desktop()) setSidebar(false);
  });
  let pending = '', pendingAt = 0;
  document.addEventListener('keydown', event => {
    const list = menu();
    if (event.key === 'Escape') {
      if (list && !list.hidden) setMenu(false);
      else if (isOpen() && !desktop()) setSidebar(false, true);
      return;
    }
    if (list && !list.hidden && (event.key === 'ArrowDown' || event.key === 'ArrowUp')) {
      const choices = items(), index = choices.indexOf(document.activeElement);
      choices[(index + (event.key === 'ArrowDown' ? 1 : -1) + choices.length) % choices.length]?.focus();
      event.preventDefault();
      return;
    }
    const field = event.target?.closest?.('input,textarea,select,[contenteditable]:not([contenteditable="false"])');
    if (event.ctrlKey || event.altKey || event.metaKey || field || event.key.length !== 1) { pending = ''; return; }
    const key = event.key.toLowerCase(), now = Date.now(), action = actions[pending + key];
    if (action && now - pendingAt <= 1000) { pending = ''; event.preventDefault(); action(); return; }
    pending = key === 't' || key === 'g' ? key : '';
    pendingAt = now;
  });
  for (const control of document.querySelectorAll('[data-lsg-left-toggle],[data-lsg-more]')) control.hidden = false;
  $('[data-lsg-left-toggle]')?.setAttribute('aria-expanded', String(isOpen()));
})();
