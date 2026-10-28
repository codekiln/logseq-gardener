// View as slides: a deck shaped like Logseq's reveal.js deck, built from a page's published
// outline. lsg-sidebar.js loads this file with the first Slides card and calls
// window.logseqGardenerSlides(outline), which returns the card's body: the tip and the deck.
// Only the current slide and its four neighbours are in the document, so images on distant
// slides wait until the reader gets near them.
(() => {
  if (window.logseqGardenerSlides) return;
  document.head.append(Object.assign(document.createElement('style'), { textContent: '__LSG_SLIDES_CSS__' }));
  const el = (tag, attrs = {}, ...children) => {
    const node = document.createElement(tag);
    for (const [name, value] of Object.entries(attrs)) node.setAttribute(name, value);
    node.append(...children);
    return node;
  };
  const MEDIA = 'img,video,audio,iframe,object,embed,svg,canvas,hr,table';
  // Child blocks have a bullet; items of a Markdown list inside the block do not.
  const kids = li => [...li.querySelectorAll(':scope>ul>li,:scope>details>ul>li')].filter(item => item.querySelector(':scope>.bullet,:scope>details>summary>.bullet'));

  // A block's own content without bullet, collapse control, task marker, ids or children;
  // null when nothing is left to show.
  function content(li) {
    const box = document.createElement('div');
    const details = li.querySelector(':scope>details');
    const childLists = new Set(kids(li).map(item => item.parentElement));
    const nodes = [...li.childNodes].filter(node => node !== details);
    if (details) nodes.push(...(details.querySelector(':scope>summary')?.childNodes ?? []), ...[...details.children].filter(node => node.tagName !== 'SUMMARY'));
    for (const node of nodes) if (!(node.nodeName === 'UL' && childLists.has(node))) box.append(document.importNode(node, true));
    box.querySelectorAll('.bullet,.lsg-arrow,.task-checkbox,.task-marker,.page-properties,.block-properties,span[id^="block-"],span[id^="source-block-"]').forEach(node => node.remove());
    box.querySelectorAll('[id]').forEach(node => node.removeAttribute('id'));
    return box.textContent.trim() || box.querySelector(MEDIA) ? box : null;
  }

  // Columns of slides: each top-level block, then every descendant in outline order.
  // Blocks with nothing to show make no slide; empty columns are dropped.
  function columns(outline) {
    const walk = (li, out) => { out.push(li); for (const kid of kids(li)) walk(kid, out); return out; };
    return [...(outline?.children ?? [])].filter(li => li.tagName === 'LI')
      .map(li => walk(li, []).map(item => ({ li: item })).filter(slide => (slide.box = content(slide.li))))
      .filter(column => column.length);
  }

  // A slide's section, with the block's data-slide-* properties as reveal.js attributes.
  function section(slide) {
    const node = el('section');
    for (const { name, value } of slide.li.attributes) if (name.startsWith('data-slide-')) node.setAttribute('data-' + name.slice(11), value);
    const speed = { fast: '.4s', slow: '1.2s' }[node.getAttribute('data-transition-speed')];
    if (speed) node.style.setProperty('--lsg-speed', speed);
    node.append(slide.box.cloneNode(true));
    return node;
  }

  window.logseqGardenerSlides = outline => {
    const deck = el('div', { class: 'reveal', tabindex: '0', role: 'region', 'aria-label': 'Slides' });
    const body = el('div', { class: 'lsg-slides-card' }, el('p', { class: 'lsg-slides-tip' }, 'Tip: press ', el('code', {}, 'f'), ' to go fullscreen'), deck);
    const cols = columns(outline);
    const flat = cols.flatMap((column, c) => column.map((_, r) => [c, r]));
    const backgrounds = el('div', { class: 'backgrounds' });
    const slides = el('div', { class: 'slides' });
    const button = (name, label) => el('button', { type: 'button', class: 'navigate-' + name, 'aria-label': label });
    const controls = el('aside', { class: 'controls' }, button('left', 'previous slide'), button('right', 'next slide'), button('up', 'above slide'), button('down', 'below slide'));
    const progress = el('div', { class: 'progress' }, el('span'));
    deck.replaceChildren(backgrounds, slides, controls, progress);
    if (!flat.length) { deck.append(el('p', {}, 'This page has no blocks to show.')); return body; }
    let c = 0, r = 0, overview = null, blackout = null, jump = null, drawn = false;
    const shown = new Map();
    const at = (col, row) => cols[col]?.[row] && col + '/' + row;
    const index = () => flat.findIndex(([col, row]) => col === c && row === r);

    function draw() {
      const reach = { present: at(c, r), left: at(c - 1, 0), right: at(c + 1, 0), up: at(c, r - 1), down: at(c, r + 1) };
      const keys = new Map(Object.entries(reach).filter(([, key]) => key).map(([dir, key]) => [key, dir]));
      for (const [key, node] of shown) if (!keys.has(key)) { node.remove(); shown.delete(key); }
      for (const [key, dir] of keys) {
        let node = shown.get(key);
        if (!node) {
          const [col, row] = key.split('/').map(Number);
          node = section(cols[col][row]);
          node.setAttribute('data-dir', dir);
          shown.set(key, node);
          slides.append(node);
        }
      }
      // Set the new positions once the new sections have their starting ones.
      void slides.offsetWidth;
      for (const [key, dir] of keys) {
        const node = shown.get(key);
        node.classList.toggle('present', dir === 'present');
        if (dir === 'present') node.removeAttribute('data-dir');
        else {
          const [col, row] = key.split('/').map(Number);
          node.setAttribute('data-dir', col < c ? 'left' : col > c ? 'right' : row < r ? 'up' : 'down');
        }
        node.setAttribute('aria-hidden', String(dir !== 'present'));
      }
      const present = shown.get(reach.present);
      const style = backgrounds.style;
      style.backgroundImage = present.dataset.backgroundImage ? 'url(' + JSON.stringify(present.dataset.backgroundImage) + ')' : '';
      style.backgroundColor = present.dataset.backgroundColor || '';
      style.backgroundSize = present.dataset.backgroundSize || '';
      style.backgroundPosition = present.dataset.backgroundPosition || '';
      const [left, right, up, down] = controls.children;
      left.disabled = !reach.left;
      right.disabled = !reach.right;
      up.disabled = !reach.up;
      down.disabled = !reach.down;
      deck.style.setProperty('--lsg-progress', flat.length > 1 ? index() / (flat.length - 1) : 1);
      if (overview) mark();
    }
    const go = (col, row) => {
      col = Math.max(0, Math.min(cols.length - 1, col));
      row = Math.max(0, Math.min(cols[col].length - 1, row));
      if (col === c && row === r) return;
      c = col;
      r = row;
      draw();
    };
    const step = by => { const next = flat[index() + by]; if (next) go(...next); };

    // The observer follows the card's width and fullscreen, and stops after the card goes.
    function scale() {
      if (!deck.isConnected) { if (drawn) observer.disconnect(); return; }
      drawn = true;
      const { width, height } = deck.getBoundingClientRect();
      if (width && height) deck.style.setProperty('--slide-scale', Math.min(width / 960, height / 700));
    }
    const observer = new ResizeObserver(scale);
    observer.observe(deck);

    // The overview: every slide, scaled, in its column; a click or Enter chooses one.
    function mark() {
      for (const item of overview.querySelectorAll('button')) {
        const here = item.dataset.at === c + '/' + r;
        item.setAttribute('aria-current', String(here));
        if (here) item.scrollIntoView({ block: 'nearest', inline: 'nearest' });
      }
    }
    function toggleOverview() {
      if (overview) { overview.remove(); overview = null; deck.classList.remove('overview'); deck.focus(); return; }
      overview = el('div', { class: 'lsg-overview', role: 'listbox', 'aria-label': 'Slide overview' });
      cols.forEach((column, col) => overview.append(el('div', {}, ...column.map((slide, row) => el('button', { type: 'button', 'data-at': col + '/' + row, 'aria-label': 'Slide ' + (col + 1) + '/' + (row + 1) }, section(slide))))));
      deck.classList.add('overview');
      deck.append(overview);
      mark();
    }

    // g: a field that takes a slide number through the deck, or column/slide.
    function openJump() {
      jump = el('input', { class: 'lsg-jump', type: 'text', 'aria-label': 'Go to slide', placeholder: 'Go to slide' });
      jump.addEventListener('keydown', event => {
        event.stopPropagation();
        if (event.key === 'Escape' || event.key === 'Enter') {
          const value = jump.value.trim();
          const pair = value.match(/^(\d+)\s*\/\s*(\d+)$/);
          if (event.key === 'Enter' && pair) go(Number(pair[1]) - 1, Number(pair[2]) - 1);
          else if (event.key === 'Enter' && /^\d+$/.test(value) && flat[Number(value) - 1]) go(...flat[Number(value) - 1]);
          jump.remove();
          jump = null;
          deck.focus();
          event.preventDefault();
        }
      });
      deck.append(jump);
      jump.focus();
    }

    function fullscreen() {
      if (document.fullscreenElement === deck) document.exitFullscreen?.();
      else deck.requestFullscreen?.().catch(() => {});
    }

    deck.addEventListener('keydown', event => {
      if (event.target !== deck && event.target.closest('input,textarea,select')) return;
      // Keys stay with the deck: the shell's and sidebar's shortcuts are ignored here.
      event.stopPropagation();
      if (event.ctrlKey || event.metaKey || event.altKey || event.key === 'Tab') return;
      if (blackout) { blackout.remove(); blackout = null; event.preventDefault(); return; }
      const key = event.key, shift = event.shiftKey, lower = key.toLowerCase();
      const act = {
        arrowleft: () => shift ? go(0, 0) : go(c - 1, 0),
        h: () => go(c - 1, 0),
        arrowright: () => shift ? go(cols.length - 1, 0) : go(c + 1, 0),
        l: () => go(c + 1, 0),
        arrowup: () => go(c, shift ? 0 : r - 1),
        k: () => go(c, r - 1),
        arrowdown: () => go(c, shift ? cols[c].length - 1 : r + 1),
        j: () => go(c, r + 1),
        n: () => step(1),
        p: () => step(-1),
        ' ': () => step(shift ? -1 : 1),
        b: () => deck.append(blackout = el('div', { class: 'lsg-blackout' })),
        '.': () => deck.append(blackout = el('div', { class: 'lsg-blackout' })),
        g: openJump,
        escape: toggleOverview,
        o: toggleOverview,
        enter: () => { const choice = event.target.closest('.lsg-overview button'); if (choice) choice.click(); else if (overview) toggleOverview(); },
        f: fullscreen,
      }[lower];
      if (!act) return;
      event.preventDefault();
      act();
    });
    deck.addEventListener('focus', () => deck.classList.add('focused'));
    deck.addEventListener('blur', () => deck.classList.remove('focused'));
    deck.addEventListener('click', event => {
      const control = event.target.closest('.controls button');
      const choice = event.target.closest('.lsg-overview button');
      if (control) {
        const name = control.className.slice(9);
        if (name === 'left') go(c - 1, 0); else if (name === 'right') go(c + 1, 0); else go(c, r + (name === 'up' ? -1 : 1));
      } else if (choice) {
        const [col, row] = choice.dataset.at.split('/').map(Number);
        go(col, row);
        toggleOverview();
      }
      if (!event.target.closest('a,input')) deck.focus();
    });
    draw();
    setTimeout(scale);
    return body;
  };
})();
