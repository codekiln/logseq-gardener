## Why

In Logseq, Shift-click on a breadcrumb crumb in Linked References adds a Block references card for that block, and Shift-click on a bullet adds a card for that block. Gardener does neither ([Issue #317 — Open Linked References crumbs and bullets in the right sidebar on Shift-click](https://github.com/codekiln/logseq-gardener/issues/317)):

- On a host that rewrites page addresses, such as Gitpa, a crumb opens a new browser window.
- A bullet in Linked References does nothing: the shell cancels the click and the sidebar finds no block.
- On Gardener's own sites, a crumb adds a card for the whole page.

Crumbs and bullets in Linked References and query results link to the block's anchor on its source page, such as `p-….html#source-block-58`. The sidebar does not recognize that anchor.

## What Changes

- The shell sends a Shift-click on a link whose address holds `#source-block-` to the right sidebar, at any address.
- The sidebar turns a Shift-click on such a crumb or link into a Block references card for that block of the source page, and on such a bullet into a block card.
- The sidebar finds a block in a fetched page by its `source-block-` anchor, so those cards fill, survive a reload and get their breadcrumb.
- A plain click still opens the source page at the block.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `published-right-sidebar`: Shift-click on a crumb or bullet in Linked References and query results adds a card for its block, as in Logseq, instead of a page card or nothing.

## Impact

- `crates/logseq-gardener-sdk/src/site/shell.js`: the Shift-click selector lists `[href*="#source-block-"]`.
- `crates/logseq-gardener-sdk/src/site/sidebar.js`: `target()` recognizes `#source-block-` on bullets and links, and `find()` looks blocks up by that anchor.
- Tests: `tests/shell.test.cjs` and `tests/sidebar.test.cjs` cover a crumb and a bullet at a `/gitpa/` readable route, and a restore on another page.
- Docs: `docs/local-site.md` describes the gesture and lists `#source-block-` among the addresses a host keeps working.
- Budgets: both scripts stay within the `[shell]` and `[sidebar]` budgets in `scripts/parity/budgets.toml`.
- Saved state: a card opened this way stores the anchor, such as `source-block-58`, as its block. A rebuild that moves the block loses the card's content, as a stale outline path does today.

## Citations

- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
- [My/Pref/Dev/AI/OpenSpec](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Dev___AI___OpenSpec.md)
