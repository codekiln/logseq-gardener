## Context

`site::references::References` draws the reference sections below each page. For every page, `References::render` lists the blocks that refer to it, and `References::group` copies each listed block with its children, sets the copy's outline levels to start at the top, and renders the copy with `render::render_reference_context`. A block that links to five pages is therefore copied and rendered five times in Linked References, once more if it mentions another page by name in Unlinked References, and again in each query that returns it. Its breadcrumb renders again for each listing too.

In a profile of a warm encode garden build on an Apple M2 Pro, 45% of the main thread's samples during rendering were in `References::render`, mostly in rendering copies ([Issue #290](https://github.com/codekiln/logseq-gardener/issues/290), measured for [PR #289](https://github.com/codekiln/logseq-gardener/pull/289)).

A rendered copy depends on the listing page in only two places, both at the copy's own bullets (not those of embedded pages):

- the `<li>` of each bullet that itself refers to the listing page carries `data-ref-block="N"`, its row in that page's reference data, which the filter script reads; and
- each bullet is `<a class="bullet" href="page#source-block-N">` when it refers to the listing page, and `<span class="bullet">` otherwise.

Everything else in a copy comes from the block's source, the page names, the block targets, the open depth and the bracket setting, which are the same for the whole build.

## Goals / Non-Goals

**Goals:**

- Render each block's copy and breadcrumb once per build.
- Keep every published file byte for byte the same.
- Record the effect on the publication benchmark.

**Non-Goals:**

- Render pages on several threads. Rendering runs on one thread; spreading it across processors is a larger change with its own ordering questions about assets and diagnostics.

## Decisions

### A rendered copy keeps gaps for the listing page

`render::render_reference_context` now returns a `ReferenceContext`: the copy's HTML without the page-dependent parts, plus a list of gaps. Each gap records its byte offset, the source offset of the bullet it belongs to, and whether it is the row attribute or the bullet. `ReferenceContext::fill` takes the listing page's row numbers and bullet links, both by bullet offset, and writes the finished HTML in one pass. Every caller goes through `fill`, including flashcards and unlinked references, which pass empty maps, so a single code path produces every copy.

Alternatives considered:

- Render once with a marker string at each gap and replace the markers per page. Any marker could also appear in a garden's text, and escaping does not remove arbitrary characters, so a marker could corrupt output. Byte offsets cannot collide with content.
- Render the copy once without links and patch the HTML with string searches for `<span class="bullet">`. The search would also match bullets inside embedded pages, which never link, and would depend on the exact markup.

Dialogue timestamps are linked after rendering by inserting text at recorded offsets (`dialogue::finalize`), which would move any gap after them. Only a page's own outline binds recordings to timestamps; a reference copy renders its dialogue as plain text, so a copy never has both. A debug assertion states this, and the new test lists a copy that holds a recording.

### `References` keeps each block's copy for the rest of the build

`References` holds a map from block and open depth to the rendered copy, and another from block to its breadcrumb. `group` and `card_parts` ask for a copy or breadcrumb and render it only when the map does not yet hold it. The diagnostics a copy produced are kept with it and reported again for each page that lists it, so the report is unchanged.

The maps use `RefCell` because `References` is shared by reference with queries and flashcards, all on the publishing thread. Keying by open depth keeps the cache correct if a caller passes a context with a different `ReferenceConfig`.

The copies stay in memory until the build ends. The site's documents are already held in memory until the write phase, and every copy is part of at least one of them, so the cache adds at most the size of one listing of each block. The benchmark's peak memory column shows the cost.

### What lives where

The relationship logic stays where it was: the SDK's `references::ReferenceIndex` decides which blocks refer to which pages, their order and their row numbers. The change is confined to the publisher's rendering in `site/`: `render.rs` produces copies with gaps, and `references.rs` caches and fills them. No logic moved between the SDK's index and the publisher.

### Output identity is checked on a whole garden

The encode garden is published with the binary built from `main` and with the changed binary, using the benchmark's options (`--journals`, `--search` and every top-level namespace), and the two output directories are compared file by file. Unit tests cover `fill` and a copy listed on several pages.

## Risks / Trade-offs

- [A future change adds another page-dependent part to copies] → The cached copy would show the first listing's value on every page. The `ReferenceContext` documentation names the two gaps, and a test lists one block on two pages and checks both outputs against rendering separately.
- [Peak memory grows] → The benchmark records peak memory; the rows in this change show the difference.
- [A later change binds recordings inside reference copies] → Linking timestamps would then move the gaps. The debug assertion fails in the test suite, whose new test puts a recording in a copy.

## Resolved Questions

### 1 - Which parts of a rendered copy depend on the listing page?

Decided by the agent without codekiln: only the row attribute in each copied bullet's `<li>` tag and the bullet itself. Both come from `ReferencePresentation::paths` and `entries`, the only inputs to `render_reference_context` that `group` built per page; the other inputs are fixed for the build.

### 2 - Should the copies be cached in the SDK's reference index or in the publisher?

Decided by the agent without codekiln: in the publisher's `References`. The copies are HTML, which belongs to the publisher; the index answers which blocks refer to which pages and stays free of rendering.

### 3 - Should copies be dropped once no remaining page needs them?

Decided by the agent without codekiln: no. Knowing when a copy is last used means counting its listings across Linked References, Unlinked References and queries in advance. The site's documents already stay in memory until the write phase, so the extra memory is bounded, and the benchmark records it.

### 4 - Should breadcrumbs be cached too?

Decided by the agent without codekiln: yes. A breadcrumb depends only on the block and is drawn for every listing that shows breadcrumbs, so it has the same repeated work for a few lines of code.

### 5 - How should output identity be shown?

Decided by the agent without codekiln: by publishing the whole encode garden with the `main` binary and the changed binary, with the benchmark's options, and comparing every file. The result goes in the pull request, as `parity:build-bench` does for build plugins. A permanent CI job was not added, since the benchmark job already publishes the encode garden and a second build would double its time.

## Open Questions

None.
