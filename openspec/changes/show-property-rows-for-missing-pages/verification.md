# Verification: show-property-rows-for-missing-pages

The page properties box now shows rows whose references name pages with no file. Checked on 2026-11-01.

## Test: `page_property_rows_show_names_of_pages_with_no_file`

The test publishes the issue's example page, `tags:: [[No File]]`, `see-also:: [[Other Page]]` and `foo:: bar`, with three more rows: `mixed:: [[Visible/Target]], [[No File]]`, a reference to a private page by its title and by its alias, and a reference to a page outside the selected namespaces.

Before, on `origin/main`, the box held one row:

```html
<dl><div class="page-property"><dt>foo</dt><dd>bar</dd></div></dl>
```

After, the box shows `tags`, `see-also`, `foo` and `mixed`, in authored order:

```html
<dt>tags</dt><dd><span class="page-reference unresolved no-brackets"><span>No File</span></span></dd>
<dt>see-also</dt><dd><span class="page-reference unresolved no-brackets"><span>Other Page</span></span></dd>
<dt>foo</dt><dd>bar</dd>
<dt>mixed</dt><dd><a class="page-ref" href="…">Visible/Target</a>, <span class="page-reference unresolved no-brackets"><span>No File</span></span></dd>
```

The rows that name the private page and the excluded page stay out, and their names do not appear on the page.

## The encode garden

A copy of the encode garden at its current `main`, with `:publishing/page-properties? true` added to `logseq/config.edn`, published with every namespace and `--journals`, by `origin/main` and by this branch:

| | origin/main | this branch |
| --- | --- | --- |
| Property rows on the site | 17,318 | 18,164 |
| Pages that gain rows | | 775 |
| Pages that lose rows | | 0 |
| Diagnostics | 12,499 | 12,499, the same lines |

The gained rows are mostly `tags` (458), `date-created` (232) and `created-by` (82).

[Knowledge/Garden](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/Knowledge___Garden.md) starts with `tags:: [[AI/Generated]]`, and no file holds `AI/Generated`. Before:

```html
<dl><div class="page-property"><dt>see-also</dt><dd><a class="page-ref" href="Knowledge/Garden/er/">Knowledge/Garden/er</a>, <a class="page-ref" href="Knowledge/Concept/Knowledge%20Base%20vs%20Knowledge%20Garden/">Knowledge/Concept/Knowledge Base vs Knowledge Garden</a></dd></div></dl>
```

After:

```html
<dl><div class="page-property"><dt>tags</dt><dd><span class="page-reference unresolved no-brackets"><span>AI/Generated</span></span></dd></div><div class="page-property"><dt>see-also</dt><dd><a class="page-ref" href="Knowledge/Garden/er/">Knowledge/Garden/er</a>, <a class="page-ref" href="Knowledge/Concept/Knowledge%20Base%20vs%20Knowledge%20Garden/">Knowledge/Concept/Knowledge Base vs Knowledge Garden</a></dd></div></dl>
```

Logseq shows both rows, `tags` first, as the garden's own Logseq graph does.
