## Context

Gardener publishes every page as `p-<hash>.html` beside `lsg-shell.js` and `lsg-sidebar.js` at the site root. Gitpa copies each page to a readable route, `/gitpa/<Title>/index.html`, and rewrites every relative `href` and `src` to an absolute path under `/gitpa/`. The scripts still load from the site root, so `lsg-sidebar.js` resolves card routes against `/gitpa/`.

Two checks decide whether Shift-click adds a card:

- `shell.js` intercepts Shift-click only on a bullet, a `#zoom/` link or a link whose `href` starts with `p-`. Any other click falls through to the browser, which opens a new window.
- `target()` in `sidebar.js` builds a page card only for an `href` matching `p-*.html`, and takes the current page's route from the last path segment, which is empty for `/gitpa/GitP/A/Session/26/10/29-Thu/` and becomes `index.html`.

Card headers also write the card's route as a relative `href`. From a page at a nested address, that `href` resolves below the current page instead of below the site root.

## Goals / Non-Goals

**Goals:**

- Shift-click on an inline page reference, a query result, a tag, a block reference or a bullet adds the right card on a host that serves pages at readable routes.
- Cards for those routes load, survive a reload from any page, and their Open as page, crumb and zoom links reach the page.
- Gardener's own sites keep their current saved routes.

**Non-Goals:**

- Recognize every page link Gardener emits at a rewritten address: breadcrumb crumbs, reference group titles, journal titles, embed headers, Pages tagged with and inline links in property values carry no page-link class, so on a rewriting host they still need a `p-*.html` address. A build-time class on every page link would cover them.

## Decisions

- Recognize page links by Gardener's markup: `.page-reference a`, `a.page-ref` and `a.tag`, plus `a.block-ref-link` for block references. These are the classes Logseq uses and the classes Gitpa keeps when it rewrites addresses. The alternative, treating any same-site link that ends in `/` or `.html` as a page, would also catch All pages, the graph view and fragment links that have no page to show.
- Keep the `p-*.html` address check beside the markup check, so links without a page-link class keep working on Gardener's own sites.
- Store a card's route as the link's resolved address relative to the site root, the folder that serves `lsg-sidebar.js`. On Gardener's own sites this is the same `p-*.html` name as before, and the site root itself stays `index.html`. An address outside the site root is stored whole. A route relative to the site root resolves the same from every page, which a path relative to the current page would not.
- Resolve every address the sidebar writes or opens (fetch, breadcrumb crumbs, the Slides title, Open as page, zoom) from the site root through one helper. Those addresses are now absolute, so the shell matches `#zoom/` anywhere in a link's address, and the page link at the start of a card's breadcrumb and the Slides title carry the `page-ref` class. Shift-click on them keeps adding cards, as it did when they were relative `p-*.html` links.
- Find the sidebar's own cards with `section.lsg-card`. Flashcard blocks are `li.lsg-card`, and the bare `.lsg-card` lookup in `target()` threw on them before this change.
- Split the shell's check into two `closest()` calls: the target list, then `main,.lsg-right-sidebar`. The tests' small DOM can evaluate both lists, where it cannot evaluate a complex selector inside `:is()`.

## Risks / Trade-offs

- A link inside `.page-reference` that is not a page link would now open a card. → Gardener puts only the page's own link inside `.page-reference`.
- Gitpa writes its homepage list of recent sessions itself, as plain `<a>` rows without `page-ref`. → Shift-click on those rows still opens a new window until Gitpa adds the `page-ref` class Gardener uses for query results.

## Resolved Questions

### 1 - Which links count as page links?

Decided by the agent without codekiln, following the issue: links inside `.page-reference`, `a.page-ref` and `a.tag`, plus `p-*.html` addresses. Block references count by `a.block-ref-link`, because they fail the same way on a rewriting host and the sidebar already recognizes the class.

### 2 - Should a block card for a bullet on the current page also use the resolved address?

Decided by the agent without codekiln: yes. On a page at a readable route the old last-segment rule stored `index.html`, so the card showed a block of the home page instead of the current one.

### 3 - Should the fix also cover the flashcard `li.lsg-card` collision?

Decided by the agent without codekiln: yes. Shift-click on a page reference inside a flashcard block reaches `target()`, and the bare `.lsg-card` lookup threw there; the fix is one selector in the function this change rewrites.
