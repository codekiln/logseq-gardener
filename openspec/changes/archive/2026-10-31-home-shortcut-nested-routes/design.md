## Context

`lsg` writes every page, `index.html` and `lsg-shell.js` into the site root, and each page loads the shell with `<script defer src="lsg-shell.js">`. The shell's `g h` action calls `window.location.assign('index.html')`, which the browser resolves against the current page's address.

Gitpa moves each published page into a directory route, for example `/gitpa/GitP/A/Session/26/10/29-Thu/index.html` served at `/gitpa/GitP/A/Session/26/10/29-Thu/`, and rewrites each page's `href` and `src` attributes to absolute paths, so the page loads `/gitpa/lsg-shell.js`. From that page, `index.html` resolves to the page's own file, and `g h` reloads it.

The search dialog lists the shell's actions as its commands through `window.logseqGardenerShell.actions`, so its Go to home command has the same problem. `sidebar.js` and `graph.js` already resolve page routes against the directory their own script loads from.

## Goals / Non-Goals

**Goals:**

- `g h` and Go to home open the site root's `index.html` from any page address.

**Non-Goals:**

- Make Shift-click open a page in the right sidebar on rewritten page addresses ([Issue #292](https://github.com/codekiln/logseq-gardener/issues/292)), which is a separate fix in progress.

## Decisions

`g h` resolves `index.html` against the address of `lsg-shell.js`. The shell already keeps that address in `base`, from `document.currentScript.src`, to load its other scripts beside it. The browser gives `currentScript.src` as an absolute address, so `index.html` resolves to the file beside the shell. When `currentScript` is missing, `base` is the relative `lsg-shell.js`; the action first resolves it against the page's address, which gives the same result as before this change.

The address is resolved when the shortcut runs. An address the browser cannot parse then affects only `g h`, and the rest of the shell keeps working.

Alternatives considered:

- Read the `href` of the header's Home link, as `g a`, `g j` and `g g` read their sidebar links. That depends on each host rewriting the link, and `index.html` has no Home link. The shell's own address is already right on every page where the shell runs.
- Open the site's origin with `/`. A site published under a path, such as `/gitpa/` on GitHub Pages, would then open the wrong site.

## Risks / Trade-offs

[A host that copies `lsg-shell.js` somewhere other than the site root] → `g h` then opens the `index.html` beside the copy. `lsg` writes the shell only at the site root, and other shell scripts already assume they sit beside it.

## Resolved Questions

### 1 - Should `g h` open `index.html` or the site root directory?

Decided by the agent without codekiln: `index.html`, as the issue asks. A site opened from disk with `file://` shows a directory listing for a directory address, and `index.html` works on every host.

### 2 - Where should the nested-route test live?

Decided by the agent without codekiln: in `tests/shell.test.cjs`, which runs `shell.js` and records the address it opens. The Rust tests in `crates/logseq-gardener-sdk/tests/site_shell.rs` read the generated HTML and do not run the script, and `lsg` itself writes no nested routes.

## Open Questions

None.
