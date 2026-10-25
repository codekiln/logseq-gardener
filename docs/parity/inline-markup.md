# Inline markup: links, tags, emphasis, images and macros

Logseq draws page references as links between faint `[[` `]]` brackets, tags as small accent-coloured `#tag` links, external links with a 1px underline that open a new tab, inline code as a grey chip, and images at their authored size. Unknown macros show as a warning chip. `lsg` publishes the same text, but tags and unknown macros come out as plain text or code, page references lose their brackets, external links look like page links and open in the same tab, and images ignore their `{:height …, :width …}` size.

Most of the gap is markup and CSS in the publisher's inline renderer. Two items need graph data from the SDK: links for tags, and links for references to pages that exist only because something references them. The second depends on the reference-only pages that #111 and #115 track; until those pages are published, such references stay unlinked.

Measured on 2026-10-25 against the `gh-pages` build of garden commit `8bc3391` (served locally, light theme) and an `lsg` build of the whole garden from `581a7a4` (`mise run parity:build`).

## What Logseq's published site shows

Source links point at the local `logseq/og` checkout, `6e7afa8eb`, the file-graph continuation of 0.10 (`frontend` = `src/main/frontend`). The published site renders inline markup exactly as the app does: publishing never sets `:html-export?`, so the app-only branches for brackets and inline HTML apply. The inline renderer is `inline` in [`components/block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L1564-L1663). Measured live values are given first and win where the checkout's CSS differs.

### Page references

```html
<span class="page-reference" data-ref="tmux/Config">
  <span class="text-gray-500 bracket">[[</span>
  <a class="page-ref" data-ref="tmux/config">tmux/Config</a>
  <span class="text-gray-500 bracket">]]</span>
</span>
```

- Built by `page-reference`, `page-cp` and `page-inner` ([`block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L533-L780)). The link has no `href`; a click handler routes it. On desktop the link is wrapped in a hover-preview trigger.
- **Brackets** show unless `:ui/show-brackets?` is `false` ([`state.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/state.cljs#L652-L654)). The encode garden leaves the key commented out, so brackets show. Live: grey `rgb(107,114,128)` at opacity 0.3, weight 400.
- **Link:** live colour `rgb(16,107,163)`, weight 400, no underline, same size as the text. `.page-reference` has a 3px radius and a faint background on hover ([`common.css`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/common.css#L580-L645)).
- **Text:** the text as typed, so `[[Foo BAR]]` shows `Foo BAR`; an all-lower-case reference shows the page's original name. Namespaced references show the whole path, `tmux/Config`.
- **Pages without files** render the same link. Every referenced page exists in Logseq's graph, so no reference is ever unlinked.
- **Journal references** such as `[[2026-07-04 Sat]]` are ordinary page references; the text is not reformatted.
- **Labelled references** `[label]([[Page]])` show the label inside the same brackets. Live, `LangSmith/26/09/29 Tue - Deep Agents/1045 Engine Workshop/Take-Aways` shows `[[LangSmith roadshow, Boston]]` linking to `LangSmith/26/09/29 Tue - Deep Agents`. The garden has 49.

### Tags

```html
<a class="tag" data-ref="keyshort">#Keyshort</a>
```

- `#tag` and `#[[multi word]]` render as one link with no brackets; `#[[multi word]]` shows `#multi word` ([`block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L1575-L1578)).
- Live on `1Password/Quick Access`: colour `rgb(52,81,178)`, 0.96em (15.36px in 16px text), opacity 0.8, padding `0 4px`, radius 4px. Hover sets opacity 1 and underlines ([`common.css`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/common.css#L484-L499)).
- A tag targets its page like a page reference. The garden has about 2,170 tags outside code blocks.

### External links and email

- `[text](https://…)` and bare URLs render `<a class="external-link" href="…" target="_blank">` with no `rel` ([`block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L1226-L1234)).
- Live: colour `rgb(0,65,116)`, weight 300 in an `h1`, no text decoration, `border-bottom: 1px solid` in the link colour ([`common.css`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/common.css#L202-L209)).
- Email autolinks render `<a href="mailto:…">` with no class ([`block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L1595-L1598)). The `email-autolinks` spec already covers them.

### Emphasis and code

- Bold `<b>` (live weight 600), italic `<i>`, strikethrough `<del>`, highlight `<mark>`, superscript and subscript `<sup>`/`<sub>` (`emphasis-cp`, [`block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L1544-L1573)). `==text==` produces `<mark>`; the garden has 155, and no `^^text^^`.
- **Highlight:** the build's stylesheet sets `mark { background: #fef3ac; color: #262626; padding: 2px 4px; border-radius: 3px }`. In the in-app browser the computed style was instead a 4px amber (`rgb(255,198,0)`) underline with no fill, and no rule in the build's stylesheets explains it. Treat the stylesheet rule as Logseq's behaviour and confirm with `parity:screenshots` (Playwright's Chromium) during Apply.
- **Inline code:** `<code>` with no class. Live: `"Fira Code", monospace`, 0.8em (12.8px in 16px text), background `rgb(241,243,245)`, text colour unchanged, padding `3px 5px`, radius 4px ([`common.css`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/common.css#L597-L629)).

### Images

```html
<div class="resize image-resize" style="width: 231px">
  <div class="asset-container">
    <img loading="lazy" referrerpolicy="no-referrer" src="…" title="1Password connect diagram" height="265" width="231" class="rounded-sm relative">
    …copy and maximize buttons, shown on hover…
  </div>
</div>
```

- `resizable-image` ([`block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L265-L459)) copies the `{:height 265, :width 231}` map onto the `img` as attributes and the width onto the wrapper. Live on `1Password/Dev/Connect Server`.
- The alt text goes into `title`; there is no `alt` attribute. `lsg` should keep `alt` (it is better for screen readers) and may add the same `title`.
- `max-width: 100%`, radius 3px, lazy loading. Relative `../assets/x.png` resolves to `assets/x.png` in the build. The garden has 1,191 images, 102 with a size map.

### Macros

- **Unknown macro:** `<div class="macro" data-macro-name="foo"><span class="warning" title="Unsupported macro name: foo">{{foo a, b}}</span></div>`, arguments joined by `, ` (`render-macro`, [`block.cljs`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/components/block.cljs#L855-L969)). The chip uses `--ls-warning-background-color` and `--ls-warning-text-color`, padding `.1em .4em` ([`common.css`](https://github.com/logseq/og/blob/6e7afa8eb/src/main/frontend/common.css#L434-L442)). `{{pdf}}` has no handler and falls into this case.
- **Custom macros** from `:macros` in `config.edn` or a page's `macros::` property expand in place. The encode garden's `:macros` is `{}`.
- **`{{renderer …}}`** renders nothing on the published site (it needs plugins).
- Macros owned by other issues: `{{embed}}` (#114), `{{query}}` (#124), `{{video}}`, `{{youtube}}` and `{{youtube-timestamp}}` (#122), `{{cards}}` and `{{cloze}}` (#123). `{{namespace}}` and `{{tweet}}` are covered under [Out of scope](#out-of-scope).

## What lsg does today

From `crates/logseq-gardener-sdk/src/site/render.rs` and the whole-garden build:

- **Page references:** `<a href="p-….html">tmux/Config</a>` with no brackets and no class, in `--lsg-link` colour. The text is as typed. A reference to a page with no file renders `<span class="unresolved">1Password/Desktop</span>`; the build reports 18,628 such references, mostly reference-only pages.
- **Labelled references:** `<a href="…">LangSmith roadshow, Boston</a>`, no brackets.
- **Tags:** plain text. `Inline::Tag` writes `#` and the tag's text with no link (`render.rs`, around line 807); `1Password/Quick Access` shows `#Keyshort #card` as text inside its heading.
- **External links:** `<a href="https://…">` styled like page links, no class, no new tab, no underline.
- **Emphasis:** `<strong>`, `<em>`, `<del>` and `<mark>` with browser defaults (bold 700, browser yellow highlight).
- **Inline code:** `<code>` at 0.9em in the browser's monospace font, no background or padding.
- **Images:** `<img src="…" alt="…">`; the size map is dropped, so `1Password/Dev/Connect Server`'s diagram shows at its natural size.
- **Unknown and unsupported macros:** printed as `<code>{{name args}}</code>` with a diagnostic (`literal`, `render.rs` line 389). The build reports 4,220 macro diagnostics, almost all `youtube-timestamp` and `embed`, which belong to other issues. Footnote references, inline math and timestamps use the same fallback.

## Gaps

Each gap names a check the Apply step can run against the fixture pages.

- 1. **Brackets on page references.** Wrap each reference in `span.page-reference` with grey `[[` and `]]` at opacity 0.3, around plain and labelled references, unless `:ui/show-brackets? false`. Check: on `tmux`, `tmux/Config` shows `[[tmux/Config]]` with the bracket colour and opacity above; with `:ui/show-brackets? false` in a test garden, no brackets render.
- 2. **Page reference style.** Link colour and hover background as Logseq's, no underline, normal weight, in both themes. Check: computed colour and weight of `tmux/Config` on `tmux` match Logseq's.
- 3. **Tags as links.** `#tag` and `#[[multi word]]` link to the tag's page and render `a.tag` with `#` plus the page text, Logseq's size, colour, opacity and padding. Check: on `1Password/Quick Access`, `#Keyshort` links to the `Keyshort` page with the computed style above.
- 4. **References to pages without files.** Once reference-only pages are published (#111, #115), references and tags to them render as links instead of `span.unresolved`. Check: on `1Password/Quick Access`, `1Password/Desktop` and `#card` are links.
- 5. **External links.** `a.external-link`, opening a new tab with `rel="noopener noreferrer"`, styled with the 1px bottom border and Logseq's colour. Check: on `tmux`, the heading link to `https://github.com/tmux/tmux` has `target="_blank"` and a 1px solid bottom border.
- 6. **Emphasis.** Bold at weight 600; `<mark>` with Logseq's highlight; `<del>` for strikethrough. Check: on `ICMC/25/d2/1100 Concert n06/05 Luminiferous Aether`, the highlighted phrase's computed style matches Logseq's in the same browser.
- 7. **Inline code.** Logseq's mono font stack, 0.8em, grey background, `3px 5px` padding and 4px radius. Check: on `1Password/Quick Access`, `Cmd+\` matches the computed values above.
- 8. **Image size.** Apply `{:height …, :width …}` as `height` and `width` attributes, and the width to a wrapper, keeping `max-width: 100%`, a 3px radius and `loading="lazy"`. Check: on `1Password/Dev/Connect Server`, the diagram renders at 231 by 265.
- 9. **Unknown macros.** Render a warning chip `span.warning` with title `Unsupported macro name: <name>` and text `{{name a, b}}`, styled with Logseq's warning colours. Keep the build diagnostic. Macros other issues own keep their current fallback until those issues land. Check: a test page with `{{pdf x}}` shows the chip and title.
- 10. **Performance.** All of the above is markup and CSS; no inline feature needs JavaScript or graph data in the browser. Brackets add about 70 bytes of markup per reference before compression. Check: `parity:budgets` passes and `tmux` stays under the 100 KB gzip first-paint budget.

## Where the work belongs

- **SDK:** resolving a tag to its page the same way as a page reference (names, aliases, case), and knowing which referenced pages exist without files, so the CLI and language server can use the same answers. The garden's `:ui/show-brackets?` setting is read with the existing config reader.
- **Publisher:** the markup, the CSS for both themes, the image size attributes and the macro warning chip.

## Fixture pages

| Page | Why |
| --- | --- |
| `tmux` | Page references, a journal reference, an external heading link; the size budget |
| `1Password/Quick Access` | Tags `#Keyshort` and `#card`, inline code, bold, images, a reference to a page without a file |
| `1Password/Dev/Connect Server` | Image with `{:height 265, :width 231}`, external links |
| `ICMC/25/d2/1100 Concert n06/05 Luminiferous Aether` | `==highlight==`, tags, inline code, images |
| `LangSmith/26/09/29 Tue - Deep Agents/1045 Engine Workshop/Take-Aways` | Labelled page reference |

## Out of scope

- Hover previews of a referenced page. They need the target page's content in the browser; #119 (right sidebar) is the closer fit, and a static build would load them per page on demand.
- Macros other issues own: `embed` (#114), `query` (#124), `video`, `youtube` and `youtube-timestamp` (#122), `cards` and `cloze` (#123).
- `{{namespace}}` (8 uses) belongs with the hierarchy section (#116); `{{tweet}}` (8 uses) loads Twitter's script and is left for #122's media work.
- Footnotes and inline math, which `lsg` prints as literal code today. They are not listed in #112's goal; record them under #106 if the comparison shows them on fixture pages.
- Editing behaviour such as clicking a reference to a missing page to create it.
