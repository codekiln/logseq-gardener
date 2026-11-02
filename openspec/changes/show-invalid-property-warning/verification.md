# Verification: show-invalid-property-warning

`lsg` now shows Logseq's "Invalid property names" warning on the same 13 blocks of the same 11 encode-garden pages as Logseq's build, with the same names and byte-identical markup. Checked on 2026-11-02 against `lsg` built from `origin/main` at `5687461` (before) and from this branch (after).

## Logseq's build

The encode garden's `gh-pages` branch now holds an `lsg` build, so the reference is `gh-pages` commit `8132c95`, Logseq's last build, made from garden commit `083106d`. Served locally and opened page by page, it shows 13 `div.invalid-properties` elements on 11 pages:

| Page | Names |
| --- | --- |
| `Logseq/Entity/Asset` | `` `logseq-entity `` |
| `Logseq/Entity/Asset/B2` | `` `logseq-entity `` |
| `Logseq/Entity/Field of Study` | `` `logseq-entity `` |
| `Logseq/Entity/Hardware/Audio/Speaker` | `` `logseq-entity `` |
| `Logseq/Entity/Hardware/Connector/Audio` | `` `logseq-entity `` |
| `Logseq/Entity/Music/Scale` | `` `logseq-entity `` |
| `Logseq/Entity/Person/Maker` | `` `logseq-entity `` on two blocks |
| `Logseq/Entity/Podcast/Episode/Frontmatter/podcast-guid` | `` `podcast-guid `` |
| `Logseq/Entity/Podcast/Episode/Frontmatter/podcast-published-at` | `` `podcast-published-at `` |
| `Logseq/Entity/Podcast/Episode/GitP/A/Frontmatter/public` | `` `public `` on two blocks |
| `Logseq/Entity/Trade-Off/Component` | `` `logseq-entity `` |

Every one has this markup, with the name in the button:

```html
<div class="invalid-properties mb-2"><div title="Invalid properties" class="warning">Invalid property names: <button class="p-1 mr-2">`logseq-entity</button></div><code>Property name begins with a non-numeric character and can contain alphanumeric characters and . * + ! - _ ? $ % &amp; = &lt; &gt;. If -, + or . are the first character, the second character (if any) must be non-numeric.</code></div>
```

## Tests

`site_properties::a_block_whose_property_name_logseq_rejects_shows_logseq_warning` publishes a block that starts with `` `Logseq_Entity:: [[Visible/Asset]]` `` and expects that markup once, naming `` `logseq-entity ``. It fails on `origin/main`, whose page shows an empty block, and passes now. `site_properties::a_page_property_name_logseq_rejects_shows_logseq_warning_once` covers leading page properties and a name given twice. `property_names::tests` and the doc test of `rejected_property_names` cover the SDK call.

## Publishing the encode garden

`lsg publish --include Logseq --include Podcast --include Ableton` on the encode garden at `e317ae9` gives 1253 pages with both builds.

| | Before | After | Logseq |
| --- | --- | --- | --- |
| Pages with the warning | 0 | 11 | 11 |
| Blocks with the warning | 0 | 13 | 13 |
| Names per page and block | none | same as Logseq | |
| Markup | none | identical to Logseq's | |

`diff -rq` between the two builds reports the 11 pages, `lsg-site.css` and `lsg-site.json`, and no other file.

## Styles

Computed styles of the warning on `Logseq/Entity/Asset`, dark theme:

| Element | Property | Logseq | `lsg` |
| --- | --- | --- | --- |
| `div.invalid-properties` | margin-bottom | 7px | 7px |
| `div.warning` | background, colour | `rgb(148, 104, 0)`, `rgb(249, 230, 140)` | same |
| `div.warning` | padding, radius, font size | 1.6px 6.4px, 4px, 16px | same |
| `button` | padding, margin-right | 3.5px, 7px | same |
| `button` | background, border, colour, cursor | transparent, 0, inherited, pointer | same |
| `code` | font, padding, radius | 12.8px Fira Code, 3px 5px, 4px | same |
| `code` | white-space | pre-wrap | pre-wrap |

The block's bullet differs: Logseq hides the bullet of a block with no text, and `lsg` draws it. [Issue #445 — Hide the bullet of a block with no text, as Logseq does](https://github.com/codekiln/logseq-gardener/issues/445) tracks that rule, which covers every empty block.
