// Inline in index.html of a --redirect-old-urls build. Opens the page that a Logseq
// #/page/<name> link names, and All pages or the graph view for #/all-pages and #/graph.
// The name map is fetched only when a link needs it.
// The build trims each line and drops comment lines, so end every statement with a semicolon.
(() => {
  let routes = null;
  const open = async () => {
    const hash = location.hash;
    const path = hash.split('?')[0];
    if (path === '#/all-pages') return location.replace(__ALL_PAGES__);
    if (path === '#/graph') return location.replace(__GRAPH__);
    if (!path.startsWith('#/page/')) return;
    let name;
    try { name = decodeURIComponent(path.slice(7)); } catch { return; }
    // The page name comparison of the publisher: trim, one slash off each end, lower case, NFC.
    name = name.trim().replace(/^\//, '').replace(/\/$/, '').toLowerCase().normalize('NFC');
    try { routes = routes || await (await fetch(__MAP__)).json(); } catch { return; }
    if (location.hash === hash && Object.hasOwn(routes, name)) location.replace(routes[name]);
  };
  addEventListener('hashchange', open);
  open();
})();
