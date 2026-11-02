# Verification: hide-empty-block-bullets

On every encode-garden page that has a block with no text, `lsg` now hides the same bullets as Logseq's build: 860 bullets on 2310 pages checked, with no page that differs. Checked on 2026-11-02 against `lsg` built from `origin/main` at `c4d7a24` (before) and from this branch (after).

## Logseq's build

The encode garden's `gh-pages` branch now holds an `lsg` build, so the reference is `gh-pages` commit `8132c95`, Logseq's last build, made from garden commit `083106d`. Both `lsg` builds publish the garden at `083106d`.

On `Logseq/Entity/Asset`, the block under "Frontmatter and body" holds only Logseq's "Invalid property names" warning. In headless Chrome at 1100 by 1800 pixels:

| Build | Pointer elsewhere | Pointer over the block |
| --- | --- | --- |
| Logseq | empty `span.bullet-container` | bullet `span.bullet-container` with its dot |
| `lsg` before | bullet visible | bullet visible |
| `lsg` after | bullet `visibility: hidden` | bullet `visibility: visible` |

## Tests

`site_collapse::a_block_with_no_text_hides_its_bullet_as_logseq_does` publishes a page with a block holding only `id::`, a bare nested `-`, a bare top-level `-` between two blocks and a `- ---` block, and expects `lsg-empty` on those four items only; the first and last blocks, a bare `-` with a child and a `- kind:: note` block keep their bullets. It fails on `origin/main`, which marks no item, and passes now. `site_collapse::an_embedded_block_with_no_text_hides_its_bullet` covers an embedded page that starts with page properties. `outline::tests::empty_blocks`, `outline::tests::hides_the_bullet_of_an_empty_block_between_the_first_and_last` and `page_properties::tests::hides_what_logseq_hides_on_a_block` cover the SDK.

## Publishing the encode garden

`lsg publish --journals` with every top-level namespace of the garden gives 8565 pages and 507 journals with both builds.

For each page or journal whose `lsg` build has a block with no visible content, 2310 in all, a script opened the page in Logseq's build in headless Chrome, scrolled until every block had loaded, and listed the blocks whose `span.bullet-container` is empty with their level. It compared that list with the `lsg-empty` items of the `lsg` build, outside embeds. Logseq renders no children of a collapsed block, so the comparison leaves out the 74 `lsg` items inside a collapsed block.

| | `lsg` before | `lsg` after | Logseq |
| --- | --- | --- | --- |
| Hidden bullets | 0 | 860 | 860 |
| Pages whose hidden bullets differ from Logseq's, by count and level | 656 | 0 | |

The first run found one page that differed, `iia/0830 Voice AI Panel`: Logseq hides the bullet of a block written `- TODO`, because it reads the word as a task marker with an empty title. The SDK now treats a first line holding only an upper-case marker word as empty, and the page matches.

`diff -rq` between the two builds reports 677 pages and journals, 41 journal feed pages, 7 flashcard deck files, 1708 Linked References files under `_refs/`, `lsg-site.css` (79 characters longer) and `lsg-site.json`, and no other file.
