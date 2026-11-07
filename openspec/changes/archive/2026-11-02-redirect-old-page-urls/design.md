## Context

`lsg publish` already knows every published title, journal title, alias and route, and in the readable layout it computes each title's digest only for titles that keep their digest file. Gitpa and logseq-encode-garden rebuild the same facts from `lsg-site.json` and the page files after the build, and each writes its own name map, `index.html` script and digest file redirects.

## Goals / Non-Goals

**Goals:**

- One option gives both sites the redirects they write today, so they can delete that code.

**Non-Goals:**

None.

## Decisions

### A built-in option, not a build plugin

Build plugins return rows and metadata for pages. They cannot write files or add a script to `index.html`, so a plugin would first need a new contribution type for arbitrary files. The redirects follow from routes the publisher already has, so the publisher writes them.

### A separate flag from `--readable-urls`

`--readable-urls` promises nothing beyond the folder layout, and Gitpa never published digest files, so it gains nothing from them. A separate flag keeps the readable layout's output unchanged and lets a site choose redirects without readable URLs, where only the `#/page/` links apply.

### The name map is a file that `index.html` fetches on demand

logseq-encode-garden has about 10,000 titles and aliases. Putting the map in `index.html` would add hundreds of kilobytes to every home page load. The script in `index.html` is a few hundred bytes and fetches `lsg-old-urls.json` only when the fragment starts with `#/`. This is what logseq-encode-garden does today.

### Digest names come from the titles the digest layout used

The digest layout names a page `p-<digest of its title>.html`, using the title before any folder respelling. The redirect uses that same title, so a page whose folder takes another spelling still gets the file its old link names. Since v0.0.25, a page that refers to itself in another spelling, such as `pages/Solvespace.md` with `alias:: [[SolveSpace]]`, takes Logseq's spelling as its title. Releases before that named its file after the file-name title, which logseq-encode-garden's live site still serves, so such a page gets a second redirect at that name.

### An alias that names two pages gets no entry

The map holds titles, journal titles and namespace pages first, then each alias that names exactly one published page and is not itself a title. Logseq would open either page for an ambiguous alias, so neither choice is right.

### Redirect files stay out of `lsg-site.json`

`lsg-site.json` lists pages and assets. Redirect files are neither, so they stay out of its `pages` list and each page keeps its one `url`.

## Risks / Trade-offs

- A garden that later renames a page loses the old digest link, because the digest follows the new title. → Same as the current site scripts; renames already break old links.
- Doubling the number of root files adds one small file per page. → About 300 bytes each, 3 MB for 10,000 pages.

## Migration Plan

Gitpa and logseq-encode-garden pass `--redirect-old-urls` once they build with a release that has it, compare the redirect files and the name map with their current output, and delete their own redirect code.

## Resolved Questions

### 1 - Should the redirects be a standard build plugin or a built-in publish option?

A built-in publish option. Build plugins cannot write files, and the publisher already has every route the redirects need.
