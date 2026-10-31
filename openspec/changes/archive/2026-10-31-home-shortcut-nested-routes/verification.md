# Verification

## Reproduction

On main, the new test `g h and the Go to home command open the site root from a nested route` in `tests/shell.test.cjs` failed: on a page at `https://codekiln.github.io/gitpa/GitP/A/Session/26/10/29-Thu/` that loads `https://codekiln.github.io/gitpa/lsg-shell.js`, `g h` asked the browser to open `index.html`, which resolves to the page's own file. After the change it opens `https://codekiln.github.io/gitpa/index.html`, and so does the Go to home command. A site opened from disk with `file://` opens the `index.html` beside `lsg-shell.js` as well. The existing shortcut test now expects `index.html` at the site root of its page.

## Browser check

A two-page garden was published with `lsg publish --homepage Other --search` into a `gitpa/` directory and served over HTTP. Its `tmux` page was copied to `gitpa/GitP/A/Session/26/10/29-Thu/index.html` with its relative `href` and `src` values rewritten to `/gitpa/…`, as Gitpa's packaging does.

- With main's `lsg-shell.js`, `g h` on `/gitpa/GitP/A/Session/26/10/29-Thu/` went to `/gitpa/GitP/A/Session/26/10/29-Thu/index.html`, the same `tmux` page.
- With this change, `g h` on that page opened `/gitpa/index.html`, the `Other` homepage.
- Choosing Go to home in the search dialog on that page also opened `/gitpa/index.html`.
- `g h` on the unmoved page `/gitpa/p-424036ef4353ec8f.html` opened `/gitpa/index.html`, as before.

## Other scripts

No other site script opens `index.html` relative to the page. The search dialog's Go to home command calls the shell's `gh` action. `sidebar.js` and `graph.js` already resolve routes against their own script's directory. `host.mjs` names `a[href="index.html"]` only as a selector for the header's Home button when it swaps pages, and `sidebar.js` uses `index.html` as the name of the current page when its address ends in `/`; neither opens a page.

## Checks

- `node --test tests/*.test.cjs`: 122 passed.
- `lsg-shell.js` after the build's comment stripping: 6,483 characters, within the 6.9 KB budget.
- `mise run ci`: passed.
