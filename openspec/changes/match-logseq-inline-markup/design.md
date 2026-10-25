## Context

The [inline markup analysis](../../../docs/parity/inline-markup.md) compared Logseq's `gh-pages` build of the encode garden (garden commit `8bc3391`) with an `lsg` build from `581a7a4`. The text matches; the markup and styles do not. The inline renderer is `Renderer::inlines` in `crates/logseq-gardener-sdk/src/site/render.rs`: page references become bare `<a href>` or `span.unresolved`, `Inline::Tag` writes `#` and the text, outside links get no class or target, images drop the parser's `metadata` string (`{:height 265, :width 231}`), and every macro other than `embed` goes through `literal`, which writes `<code>` and a diagnostic.

Everything Logseq does here is markup and CSS, so parity costs no JavaScript and no graph data. The one size risk is the `tmux` page: 96 KB gzip before the title against the 100 KB budget, and bracket spans add about 70 bytes per reference before compression.

## Goals / Non-Goals

**Goals:**

- Readers of the encode garden see Logseq's page references, tags, outside links, emphasis, inline code, sized images and macro warnings wherever inline text appears: page outlines, page properties, the journals feed and Linked References contexts.
- Tags work as links, resolved by the same SDK lookup as page references, so the CLI and the language server can answer "what does this tag point at" the same way.
- First paint stays independent of garden size: one small inline stylesheet and a few bytes per reference.

**Non-Goals:**

- Publishing pages that exist only because something references or tags them, which [Issue #115 — Match Logseq's linked references, filters, unlinked references and tagged pages](https://github.com/codekiln/logseq-gardener/issues/115) covers. Once those pages are in the page-name index, references and tags to them become links through this change's code with no further work.
- Hover previews of referenced pages, which fit [Issue #119 — Add Logseq's right sidebar](https://github.com/codekiln/logseq-gardener/issues/119).
- Expanding custom macros from `:macros` in `config.edn` or a page's `macros::` property. The encode garden defines none; a garden that does sees Logseq's warning chip for now.
- Image copy and full-screen buttons shown on hover.

## Decisions

**Where the work lives.** The SDK gains:

- `GardenSettings::show_brackets`, read by the existing `config.edn` reader. Only the boolean `false` turns brackets off.
- An `inline_references` module with `tag_page_name(&[Inline]) -> String`, the text of a tag's children as plain text. The publisher, the search index (`site/search.rs`, which already does this inline) and later the CLI and language server resolve that name with `PageNameIndex::candidates`, as they resolve page references.

The publisher renders the markup, styles and image attributes, and the macro chip.

**Page references.** `page_link` writes `<span class="page-reference">`, then `<span class="bracket">[[</span>` when brackets are on, then the existing `<a href>` or, for an unresolved or ambiguous name, a `<span>` with the label, then the closing bracket. The `unresolved` class moves to the outer span so existing styles and tests keep their hook. Links keep their `href`, unlike Logseq's click-routed anchors, so they work without JavaScript and in Gitpa. Labelled references go through the same path.

**Tags.** `Inline::Tag` resolves `tag_page_name` with `candidates`; one candidate writes `<a class="tag" href="…">#…</a>` with the children as the text, otherwise `<span class="tag">#…</span>` and the existing unresolved diagnostic. Inside a link label (`label == true`) a tag stays text, as nested links are not allowed.

**Outside links.** `link` adds `class="external-link" target="_blank" rel="noopener noreferrer"` for `http` and `https` destinations. `mailto:` links and the separate email autolinks keep their markup, as Logseq's do.

**Images.** `media` takes the link's `metadata` string and reads it with the `config.edn` form reader (no evaluation). Positive integer `:width` and `:height` become attributes. Every `img` gains `loading="lazy"` and `title` equal to its `alt`. CSS sets `max-width: 100%; height: auto` so a sized image scales down on phones; Logseq's wrapper `div` is left out because images sit inside paragraphs, where a `div` is invalid HTML.

**Macros.** `Inline::Macro` dispatches on the lower-cased name: `embed` as today; the owned names (`query`, `video`, `youtube`, `youtube-timestamp`, `cards`, `cloze`, `namespace`, `tweet`) through `literal` as today; `renderer` writes nothing and a diagnostic; any other name writes `<span class="warning" title="Unsupported macro name: <name>">{{name a, b}}</span>` and the existing diagnostic. Logseq's outer `div.macro` is left out for the same reason as the image wrapper.

**Styles.** A new `site/inline.css` is inlined into every page next to the outline styles. It styles `.page-reference`, `.bracket`, `.page-reference a`, `.tag`, `.external-link`, `strong`, `mark`, `code` outside `pre`, `img` and `.warning`, using theme tokens added to `site/theme.css`: `--lsg-ref`, `--lsg-tag`, `--lsg-external`, `--lsg-bracket`, `--lsg-code-bg`, `--lsg-mark-bg`, `--lsg-mark-fg`, `--lsg-warning-bg` and `--lsg-warning-fg`. Light values are the analysis's live measurements; dark values come from Logseq's dark theme variables in `logseq/og`'s `common.css` and are checked against the `gh-pages` build in dark mode during Apply. The font stack is `"Fira Code", monospace`; the build ships no font file, so readers without Fira Code see their monospace font.

**Highlight colour.** The build's stylesheet sets `mark` to a `#fef3ac` fill with `#262626` text, `2px 4px` padding and a 3px radius; one browser showed an amber underline instead. Apply uses `parity:screenshots` (Playwright's Chromium) on the `ICMC/…/Luminiferous Aether` fixture and matches what Logseq's build shows there, recording the result in `verification.md`.

**Budgets.** Measured by the [comparison harness](../../specs/parity-comparison-harness/spec.md):

| Measure | Budget |
| --- | --- |
| Inline styles added to every page, gzip | 1 KB |
| `tmux` page, gzip | under 100 KB (existing budget), at most 2 KB above today |
| Script added | none |
| Data and script before the title, peak heap | existing budgets |

`scripts/parity/budgets.toml` gains an `[inline]` table with `styles_gzip_kb = 1`; `parity:budgets` checks it and the largest fixture page.

## Risks / Trade-offs

- [Bracket spans push `tmux` towards its 100 KB limit] → The spans are identical bytes that compress well, and the 2 KB growth check fails CI before the page reaches the limit.
- [About 18,600 references and many tags still point at pages without files, so they stay unlinked until #115] → They now look like Logseq's references, and they become links without further change once #115 publishes those pages.
- [`target="_blank"` on every outside link changes behaviour readers of `lsg` sites are used to] → It is Logseq's behaviour, which is the goal of this work, and `rel="noopener noreferrer"` keeps the opened page from reaching back.
- [Dark theme colours are read from source, not measured] → Apply checks them against the `gh-pages` build in dark mode before merging.

## Resolved Questions

### 1 - Do page reference links keep an `href`?

Decided by the agent without codekiln: yes. Logseq's anchors have no `href` and rely on its router; a static site needs real links to work without JavaScript, in Gitpa, for crawlers and for opening in a new tab. The look is unchanged.

### 2 - How do references and tags to pages without files render in this change?

Decided by the agent without codekiln: with Logseq's brackets or tag look and no link, keeping the build diagnostic. Publishing those pages is #115's work; linking to pages that do not exist yet would produce broken links.

### 3 - Do outside links get `rel="noopener noreferrer"`?

Decided by the agent without codekiln: yes. Logseq writes `target="_blank"` with no `rel`; adding it changes nothing a reader sees and stops the opened page from controlling the published one.

### 4 - Do bold and italic switch to `<b>` and `<i>`?

Decided by the agent without codekiln: no. `<strong>` and `<em>` look the same once styled and tell screen readers the text is emphasised; only the bold weight changes, to Logseq's 600.

### 5 - Do images keep `alt`, given Logseq writes only `title`?

Decided by the agent without codekiln: keep `alt` and add `title` with the same text. Screen readers need `alt`; `title` gives the hover text Logseq shows.

### 6 - Do images and macros get Logseq's wrapper `div`s?

Decided by the agent without codekiln: no. Both appear inside paragraphs, where a `div` is invalid HTML; `max-width` and `height: auto` on the image and the chip's own class give the same look.

### 7 - What happens to `{{renderer …}}`?

Decided by the agent without codekiln: it renders nothing, with a build diagnostic, as on Logseq's published site, where renderers need plugins that are not loaded.

### 8 - Which highlight style is Logseq's?

Decided by the agent without codekiln: whichever Logseq's build shows in Playwright's Chromium on the highlight fixture, measured during Apply. The stylesheet rule (a pale yellow fill) is the expected answer; the amber underline seen once in another browser has no rule in the build's stylesheets.

### 9 - Where does inline markup styling apply?

Decided by the agent without codekiln: everywhere inline text renders, including page properties, the journals feed and Linked References contexts. Logseq uses the same inline renderer in all of them.

## Open Questions

None.
