# Verification

On October 6, 2026, `mise run ci` passed for this change, including formatting, Clippy, workspace tests, OpenSpec validation, documentation checks, and CLI conformance. The SDK namespace tests cover inclusion boundaries, nested exclusion precedence, empty inclusion, Unicode names, case differences, malformed selectors and candidates, and order/duplicate independence.

`mise exec -- cargo run -p logseq-gardener-sdk --example select_namespaces` printed:

```text
Logseq
Logseq/Frontmatter
```

The [namespace policy](../../../crates/logseq-gardener-sdk/src/publishing.rs) and [SDK example](../../../crates/logseq-gardener-sdk/examples/select_namespaces.rs) are independently usable by a Rust caller. This verification does not establish working site generation or excluded-content safety in rendered output. The [publishing guide](../../../docs/publishing.md) records the remaining end-to-end work.
