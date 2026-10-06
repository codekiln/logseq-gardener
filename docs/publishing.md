# Publishing

The current priority is a Rust workflow that publishes selected Logseq garden namespaces as a locally browsable static site. The user set October 10–11, 2026 as the first weekend to try it. [Issue #11 — Rust static site publishing](https://github.com/codekiln/logseq-gardener/issues/11) tracks that outcome. The CLI currently provides help and version commands; it does not yet generate a site.

## Namespace selection

The SDK's `publishing::NamespaceSelection` selects logical page names using included and excluded namespace roots. A root includes itself and descendants separated by `/`: `My/AI` selects `My/AI/Rule`, while `My/AIM` stays outside the selection. Exclusions take precedence, and an empty inclusion list selects nothing.

Run the SDK example from the repository root:

```sh
mise exec -- cargo run -p logseq-gardener-sdk --example select_namespaces
```

The example includes `Logseq`, excludes `Logseq/Draft`, and prints `Logseq` and `Logseq/Frontmatter`. It uses supplied logical names; it does not read a garden.

Matching is exact and case-sensitive. The future loader must resolve filenames, title overrides, aliases, and graph naming rules consistently for page identities and selectors. Empty selectors, empty namespace segments, surrounding segment whitespace, and control characters produce constructor errors. Malformed candidate names are unselected. Names are not filesystem paths.

## Build the publishing workflow

The next steps are garden loading and page identity, publication-property evaluation, rendering links and embeds, referenced asset collection, and a real-garden demonstration. [Issue #4 — SDK page lookup](https://github.com/codekiln/logseq-gardener/issues/4) covers loading and identity; [Issue #3 — Parser comparison](https://github.com/codekiln/logseq-gardener/issues/3) holds the adoption evidence. [Issue #12 — Namespace selection](https://github.com/codekiln/logseq-gardener/issues/12) provides the policy that a publisher can reuse.

Publication filtering must cover every source of emitted content, including embedded text, backlinks, assets, search data, and diagnostics. A selected page can reference an excluded page; selection alone does not establish that the rendered page is safe to publish. Add mixed-visibility fixtures with distinctive excluded text and target names, then inspect all generated output for leaks. The [Logseq/Frontmatter page in codekiln/logseq-encode-garden](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/Logseq___Frontmatter.md) embeds its child pages and can test whether the publisher includes only visible embedded content.

The end-to-end demonstration should build a site from the public encode garden with explicit inclusion and exclusion, produce readable per-page HTML, and report unsupported constructs locally. The shared SDK should support both the CLI and a Rust application consuming the garden. Ordinary Markdown export and direct HTML rendering remain implementation alternatives to evaluate against that outcome. Functional usefulness and product-market fit take priority over distribution.

## Sources

- [Project Brief](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/Person___codekiln___GitHub___logseq-gardener___Project___Brief.md): describes per-page HTML through the shared garden engine.
- [Project Proposal](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/Person___codekiln___GitHub___logseq-gardener___Project___Proposal.md): describes Markdown export, resolved links and embeds, referenced assets, and publication filtering.
- [Astra's publishing proposal](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/Person___codekiln___GitHub___logseq-gardener___Project___Proposal___Astra.md): extends filtering to public diagnostics and excluded target names.
- [Gitpa publication workflow](https://github.com/codekiln/gitpa/blob/main/.github/workflows/gh-pages.yml): delegates garden publication to Logseq publish-spa.
