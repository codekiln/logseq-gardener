// Inline in the head of every document of a --readable-urls build. A page in a folder carries
// <base href> that points at the site root, so its links resolve as the publisher wrote them.
// The build trims each line and drops comment lines, so end every statement with a semicolon.
(() => {
  const doc = document;
  const follow = (event) => {
    const link = event.target.closest?.('a[href]');
    if (!link) return;
    const href = link.getAttribute('href');
    // Under <base>, a fragment-only link names the site root; keep a plain click in this page.
    if (href[0] === '#') {
      if (event.type === 'click' && !event.button && !event.metaKey && !event.ctrlKey && !event.shiftKey && !event.altKey && doc.querySelector('base[href]')) {
        event.preventDefault();
        if (href.length > 1) location.hash = href;
      }
      return;
    }
    // From disk, a URL that ends in / shows a folder listing; open the folder's index.html.
    if (location.protocol !== 'file:') return;
    const url = new URL(link.href);
    if (url.protocol === 'file:' && url.pathname.endsWith('/')) {
      url.pathname += 'index.html';
      link.href = url.href;
    }
  };
  for (const type of ['click', 'auxclick', 'contextmenu']) doc.addEventListener(type, follow, true);
})();
