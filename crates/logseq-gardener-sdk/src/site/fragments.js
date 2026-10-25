// Native fragment navigation does not run when a host swaps page content.
// Hosts can call this helper before their own fragment focus/scroll handling.
(() => {
  function reveal(fragment = location.hash, { scroll = true } = {}) {
    let id;
    try { id = decodeURIComponent(fragment.replace(/^#/, "")); }
    catch { return null; }
    const target = id && document.getElementById(id);
    if (!target) return null;
    for (let parent = target.parentElement; parent; parent = parent.parentElement) {
      if (parent.tagName === "DETAILS") parent.open = true;
      parent.classList?.remove("folded");
    }
    if (scroll) target.scrollIntoView();
    return target;
  }
  window.logseqGardenerRevealFragment = reveal;
  window.addEventListener("hashchange", () => reveal());
  reveal();
})();
