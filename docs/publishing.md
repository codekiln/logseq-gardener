# Publishing

Generate a static HTML site from selected Logseq Markdown namespaces with `lsg publish`. The command uses the Rust SDK to load the garden, resolve visible references and embeds, render pages, and copy supported referenced assets. The source garden remains unchanged.

## Try a garden

From this repository, publish part of the public encode garden into a fresh output directory:

```sh
garden_root="$(ghq list --full-path --exact github.com/codekiln/logseq-encode-garden)"
preview_root="$(mktemp -d "${TMPDIR:-/tmp}/lsg-publishing.XXXXXX")"
mise exec -- cargo run --bin lsg -- publish \
  --graph "$garden_root" --output "$preview_root/site" \
  --filename-format triple-lowbar \
  --include Logseq/Frontmatter --exclude Logseq/Frontmatter/tags
python3 -m http.server 8000 --bind 127.0.0.1 --directory "$preview_root/site"
```

Open <http://127.0.0.1:8000/> and follow the frontmatter page's selected embeds. The excluded tags explanation stays unavailable. Stop the server with Ctrl-C. Choose another port if needed. The output directory must be absent and outside the source garden; each run creates a fresh destination.

The [local-site guide](local-site.md) describes filename formats, visibility, rendering, assets, optional search and journal publication. The [garden trial](weekend-trial.md) adds workshop navigation and podcast listening checkpoints. Rust callers can use `site::publish_site` or `site::publish_site_with_options` from `logseq-gardener-sdk`; [the SDK example](../crates/logseq-gardener-sdk/examples/publish_site.rs) shows the call.

## Namespace selection

The SDK's `publishing::NamespaceSelection` selects logical page names using included and excluded namespace roots. A root includes itself and descendants separated by `/`: `My/AI` selects `My/AI/Rule`, while `My/AIM` stays outside the selection. Exclusions take precedence, and an empty inclusion list selects nothing.

Matching is exact and case-sensitive. Selectors use logical page titles derived from the configured filename format and leading title properties. Aliases help resolve references; they do not select a page into a namespace. Empty selectors, empty namespace segments, surrounding segment whitespace, and control characters produce constructor errors. Malformed candidate names are unselected. Names are not filesystem paths.

## Publication boundaries

The publisher resolves page titles, aliases, explicit outline references, and page/block embeds against selected visible content. A parsed `public:: false` withholds its entire source page. Namespace exclusions remove target pages and their contributions; they do not redact text already written on a retained page. Ambiguous or unavailable targets retain visible fallbacks and local diagnostics.

Visibility also applies to embedded content, referenced assets, reference indexes and optional search data. Mixed-visibility fixtures in [publisher tests](../crates/logseq-gardener-sdk/tests/site.rs) and [embed tests](../crates/logseq-gardener-sdk/tests/site_embeds.rs) check generated files for excluded content. Local diagnostics stay outside the site. Review the report and retained source content before publishing a sensitive garden.

## Current work

[The initial publishing outcome](https://github.com/codekiln/logseq-gardener/issues/11) and [Logseq site parity work](https://github.com/codekiln/logseq-gardener/issues/106) record delivered behavior and validation. Gitpa now [builds its podcast site with a pinned Gardener revision](https://github.com/codekiln/gitpa/blob/main/.github/workflows/gh-pages.yml); its [preparation code](https://github.com/codekiln/gitpa/blob/main/scripts/build_site.py) supplies site-specific presentation.

[Garden-wide Markdown loading](https://github.com/codekiln/logseq-gardener/pull/254) and [optional site plugin hooks](https://github.com/codekiln/logseq-gardener/pull/255) are proposals awaiting implementation. [Issue #195 — plugin ownership and consumer contract](https://github.com/codekiln/logseq-gardener/issues/195) coordinates the shared host with Gitpa. [Issue #3 — parser compatibility](https://github.com/codekiln/logseq-gardener/issues/3) retains the broader adoption evidence and remaining comparison work.

## Sources

- [Project Brief](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/Person___codekiln___GitHub___logseq-gardener___Project___Brief.md): describes per-page HTML through the shared garden engine.
- [Project Proposal](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/Person___codekiln___GitHub___logseq-gardener___Project___Proposal.md): describes Markdown export, resolved links and embeds, referenced assets, and publication filtering.
- [Astra's publishing proposal](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/Person___codekiln___GitHub___logseq-gardener___Project___Proposal___Astra.md): extends filtering to public diagnostics and excluded target names.
- [Gitpa publication workflow](https://github.com/codekiln/gitpa/blob/main/.github/workflows/gh-pages.yml): builds the podcast site with Gardener and retains RSS validation and site-specific preparation.
