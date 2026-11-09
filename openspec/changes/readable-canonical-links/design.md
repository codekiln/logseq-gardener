## Context

`lsg` writes every link relative to the output root. A readable build gives each page in a folder a `<base>` that climbs to that root, so the site works from disk and under any server path, such as `https://codekiln.github.io/gitpa/`. `lsg` is never told the site's public URL or its path prefix. The digest file redirects of `--redirect-old-urls` already carry `<link rel="canonical" href="<route>">`, relative to the output root where they sit. Gitpa's removed code wrote `/gitpa/<route>`, built from a site URL that only gitpa's build script knew.

The plugin host keeps one document while the reader moves between pages: it swaps `<main>`, the title and the page metadata, and appends any head element of the next page that the document lacks.

## Goals / Non-Goals

**Goals:**

- Each readable page names one address that resolves to its folder wherever the site is served.

**Non-Goals:**

None.

## Decisions

### The link names the route, relative to `<base>`

The `href` is the route the page's links already use, such as `GitP/A/Session/`, placed after `<base>`. A browser or crawler resolves it against `<base>`, which is the site root, so it gives the absolute folder URL under any path prefix. This is the form the redirect files already use, so a page and its redirect name the same address. An absolute URL would need a new option for the site's public URL; no build has one today, and the relative form gives the same resolved address.

### The home page names the site root

`index.html` and the home page's folder show the same page. The Home button and a reader typing the site's address both reach the root, so the root is the address to keep. `./` resolves to the root from `index.html`, which has no `<base>`, and from the folder copy through its `<base>`.

### Pages that keep their digest file name that file

Such a page's route is `p-<digest>.html`, and every link opens it there, so that file is its address.

### Lists carry no link

All pages, the journals feed and the graph view have one address each and no redirects lead to them under another, so a canonical link adds nothing.

### The readable host swaps the link

The host appends head elements the document lacks, so without a change a reader who visits three pages would leave three canonical links in the document. The readable host patch skips `link[rel=canonical]` when it copies head elements, and after adopting a page it removes the current link and imports the next page's. Gitpa's player read the canonical link to identify the current page, so it must name the page on screen. The default host stays unchanged.

### The route knowledge stays where it is

`page_urls` and the publication already give each page's route. The site module writes the link where it writes `<base>`, from the same route, so no new route logic is needed.

## Risks / Trade-offs

- [Some crawlers may resolve a relative canonical URL against the document address instead of `<base>`] → The HTML standard resolves every `href` against the document base URL, which `<base>` sets; Google documents support for relative canonical URLs. A site that wants absolute URLs can ask for an option for its public URL in a later issue.

## Resolved Questions

### 1 - Should the canonical URL be absolute, root-relative with the base path, or relative to the output root?

Decided by the agent without codekiln: relative to the output root, resolved through `<base>`. `lsg` knows neither the public URL nor the path prefix such as `/gitpa/`, and the relative form resolves to the same absolute address under any prefix. It matches the canonical link the redirect files already write.

### 2 - Which address should the home page name?

Decided by the agent without codekiln: the site root, `./`, in both copies. The root is the address readers and the Home button use, and naming it from both copies leaves the page with one address.

### 3 - Should All pages, the journals feed and the graph view get a canonical link?

Decided by the agent without codekiln: no. Each has one address, so the link adds nothing, and the issue asks for pages.

### 4 - Do the digest file redirects need a change?

Decided by the agent without codekiln: no. They already name the page's route as canonical, as `old-page-url-redirects` requires.

### 5 - Should the link also appear without `--readable-urls`?

Decided by the agent without codekiln: no. Logseq's published site has no canonical links, and default output must match what earlier releases wrote.
