// Hosts can initialize replaced page content through the same section-local API.
(() => {
  const initialized = new WeakSet();

  function enhance(section) {
    if (initialized.has(section)) return;
    const controls = section.querySelector("[data-backlinks-controls]");
    const pageCount = section.querySelector("[data-backlinks-page-count]");
    const referenceCount = section.querySelector("[data-backlinks-reference-count]");
    const status = section.querySelector("[data-backlinks-status]");
    const explanation = section.querySelector("[data-backlinks-count-explanation]");
    if (!controls || !pageCount || !referenceCount || !status) return;
    const filters = Array.from(controls.querySelectorAll("[data-backlinks-filter]"));
    const options = Array.from(controls.querySelectorAll("[data-backlinks-option]"));
    const search = controls.querySelector("[data-backlinks-search]");
    const reset = controls.querySelector("[data-backlinks-reset]");
    const refsFor = element => new Set((element.getAttribute("data-backlinks-refs") || "").split(/\s+/).filter(Boolean));
    const groups = Array.from(section.querySelectorAll("[data-backlinks-group]"), group => ({
      element: group,
      entries: Array.from(group.querySelectorAll("[data-backlinks-entry]"), element => ({
        element,
        refs: refsFor(element),
        nodes: Array.from(element.querySelectorAll("[data-backlinks-node]"), element => ({
          element,
          refs: refsFor(element),
        })),
      })),
    }));
    const totalPages = groups.length;
    const totalReferences = groups.reduce((sum, group) => sum + group.entries.length, 0);

    function searchOptions() {
      const query = (search ? search.value : "").trim().toLocaleLowerCase();
      for (const option of options) {
        const filter = option.querySelector("[data-backlinks-filter]");
        const label = (option.getAttribute("data-backlinks-label") || "").toLocaleLowerCase();
        option.hidden = !!query && !label.includes(query) && !(filter && filter.value);
      }
    }

    function update() {
      const includes = [];
      const excludes = [];
      for (const filter of filters) {
        const route = filter.getAttribute("data-backlinks-filter");
        if (filter.value === "include") includes.push(route);
        if (filter.value === "exclude") excludes.push(route);
      }
      const active = includes.length > 0 || excludes.length > 0;
      const matchesPath = refs => includes.every(route => refs.has(route)) &&
        !excludes.some(route => refs.has(route));
      let pages = 0;
      let references = 0;
      for (const group of groups) {
        let visible = 0;
        for (const entry of group.entries) {
          let matches;
          if (entry.nodes.length) {
            const nodes = new Set(entry.nodes.map(node => node.element));
            const retained = new Set(entry.nodes
              .filter(node => !active || matchesPath(node.refs)).map(node => node.element));
            // DOM order puts descendants after ancestors. Restore each matching
            // branch's parents while leaving native disclosure state unchanged.
            for (let index = entry.nodes.length - 1; index >= 0; index--) {
              const node = entry.nodes[index].element;
              if (!retained.has(node)) continue;
              for (let parent = node.parentElement; parent && parent !== entry.element; parent = parent.parentElement) {
                if (nodes.has(parent)) {
                  retained.add(parent);
                  break;
                }
              }
            }
            for (const node of entry.nodes) node.element.hidden = !retained.has(node.element);
            matches = retained.size > 0;
          } else {
            matches = matchesPath(entry.refs);
          }
          entry.element.hidden = !matches;
          if (matches) visible++;
        }
        group.element.hidden = visible === 0;
        if (visible) pages++;
        references += visible;
      }
      const pageLabel = (active ? totalPages : pages) === 1 ? "page" : "pages";
      const showContexts = pages !== references || (active && totalPages !== totalReferences);
      pageCount.textContent = `${pages}${active ? ` of ${totalPages}` : ""} ${pageLabel}`;
      referenceCount.textContent = ` · ${references}${active ? ` of ${totalReferences}` : ""} referring contexts`;
      referenceCount.hidden = !showContexts;
      if (explanation) explanation.hidden = !showContexts;
      status.textContent = active
        ? references === 0
          ? "No pages match these filters."
          : `Showing ${pages} of ${totalPages} ${pageLabel}${showContexts ? ` and ${references} of ${totalReferences} referring contexts` : ""}.`
        : "";
      searchOptions();
    }

    for (const filter of filters) {
      filter.value = "";
      filter.addEventListener("change", update);
    }
    if (search) {
      search.value = "";
      search.addEventListener("input", searchOptions);
    }
    if (reset) reset.addEventListener("click", () => {
      for (const filter of filters) filter.value = "";
      if (search) search.value = "";
      update();
    });
    initialized.add(section);
    update();
    controls.hidden = false;
  }

  function init(root = document) {
    if (root.matches && root.matches("[data-lsg-backlinks]")) enhance(root);
    for (const section of root.querySelectorAll("[data-lsg-backlinks]")) enhance(section);
  }
  window.logseqGardenerInitBacklinks = init;
  init();
})();
