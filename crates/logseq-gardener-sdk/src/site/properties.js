/** Site-scoped disclosure preference. Early parsing restoration lives in properties-start.js. */
const early = window.logseqGardenerInitProperties;
const selector = '.page-properties > details > summary';
function visit() {
  early.o = undefined;
  const now = Date.now();
  try {
    const saved = JSON.parse(window.localStorage.getItem(early.k));
    if (saved && typeof saved.open === 'boolean' && Number.isFinite(saved.seen) &&
        saved.seen <= now && now - saved.seen < 30 * 86400000) {
      window.localStorage.setItem(early.k, JSON.stringify({ open: saved.open, seen: now }));
      early.o = saved.open;
    } else window.localStorage.removeItem(early.k);
  } catch {
    try { window.localStorage.removeItem(early.k); } catch { /* Storage is blocked. */ }
  }
}
function initialize(root = document) {
  visit();
  early(root);
}
function save(event, open) {
  // Capture the resulting state at activation time; rapid clicks retain their order.
  setTimeout(() => {
    if (event.defaultPrevented || event.target.closest('a')) return;
    early.o = open;
    try { window.localStorage.setItem(early.k, JSON.stringify({ open, seen: Date.now() })); }
    catch { /* Native disclosure remains usable. */ }
  }, 0);
}
document.removeEventListener('click', early.c);
document.addEventListener('click', event => {
  const summary = event.target.closest?.(selector);
  if (summary && !event.target.closest('a') && !event.defaultPrevented) save(event, !summary.parentElement.open);
});
initialize();
for (const [event, open] of early.q) save(event, open);
delete early.q;
delete early.c;
window.logseqGardenerInitProperties = initialize;
