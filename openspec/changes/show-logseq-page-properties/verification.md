# Verification: show-logseq-page-properties

The page properties box now shows the keys Logseq 0.10.15 shows. Checked on 2026-11-01.

## Test: `page_properties_show_every_key_logseq_shows`

The test publishes a page whose properties are `title`, `alias:: [[Second Name]], Third Name`, `public`, `filters`, `icon`, `search-exclude`, `custom_id`, `logseq-proxy-source`, `file`, a configured hidden key, `id`, `collapsed`, `hl-page`, `logseq.order-list-type`, `card-repeats` and `tags`.

Before, on `origin/main`, the box held two rows, one of them a key Logseq hides:

```html
<dl><div class="page-property"><dt>icon</dt><dd>🎯</dd></div><div class="page-property"><dt>tags</dt><dd><a class="page-ref" href="p-1f7f34f630353ff8.html">Visible/Other</a></dd></div></dl>
```

After, the box shows `alias`, with links labelled `Second Name` and `Third Name` that open the page, and `public`, `search-exclude`, `custom_id`, `logseq-proxy-source`, `file` and `tags`. It hides `title`, `filters`, `icon`, the configured key, and Logseq's hidden built-in and card keys.

## Docs site: the `lsg` page

The source starts with `alias:: [[Logseq Gardener]]` and `tags:: [[Diataxis/Explanation]]`.

Before, at https://codekiln.github.io/logseq-gardener/:

```html
<dl><div class="page-property"><dt>tags</dt><dd><a class="page-ref" href="Diataxis/Explanation/">Diataxis/Explanation</a></dd></div></dl>
```

After, `mise run docs:site` with this branch, which reports no diagnostics:

```html
<dl><div class="page-property"><dt>alias</dt><dd><a class="page-ref" href="lsg/">Logseq Gardener</a></dd></div><div class="page-property"><dt>tags</dt><dd><a class="page-ref" href="Diataxis/Explanation/">Diataxis/Explanation</a></dd></div></dl>
```

This matches Logseq's own export of the docs graph, which shows `alias: Logseq Gardener` above `tags: Diataxis/Explanation`.
