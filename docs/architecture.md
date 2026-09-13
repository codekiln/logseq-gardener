# Architecture and parser comparison

Logseq Markdown is authoritative. The foundation has one Rust binary crate with internal modules for argument handling, embedded help, and output. Additional crates need demonstrated independent consumers, dependency isolation, or build boundaries.

## Before implementing garden semantics

`lsdoc` is the Rust parser candidate. Adoption requires a recorded comparison against official `mldoc` for syntax and Logseq OG's graph parser plus graph-validator for graph meaning. A parser's syntax agreement alone does not establish graph compatibility.

The next parser change should place comparative code and results under `openspec/changes/<change>/experiments/<experiment-name>/`, with a README explaining execution. Pin all reference revisions and dependency licenses before running the comparison. Use `lsdoc`'s graph-check tooling where applicable. Record:

- Syntax differences, source files, locations, parser times, and memory use against official `mldoc`.
- Page identities, aliases, namespaces, journal names, properties, block hierarchy, UUID and page references, queries, assets, and missing targets against Logseq OG and graph-validator. Generate expected graph data through `logseq.graph-parser.cli/parse-graph`.
- Independent source-position accuracy and byte-for-byte unchanged parse/serialize behavior. Later edits must preserve surrounding bytes.

Use the public encode garden as acceptance corpus, the official Logseq documentation garden for first-party examples, focused fixtures for edge cases, and Pengx's public garden for publishing diversity. Resolve locally available repositories through ghq. Record mismatch classes and required repairs before selecting a parser. If `lsdoc` needs extensive repairs, compare the effort and runtime costs of official `mldoc`, including its outline parser.

## Future source boundary

Keep original bytes and source locations separate from a parser's AST and from graph facts derived across files. An internal parser adapter should translate the chosen parser's output only after the comparison establishes the necessary representation. The current foundation adds no parser trait, graph model, or cache. Later CLI, LSP, and publishing code can share internal graph modules once those modules exist.

## References

- [Project proposal](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/Person___codekiln___GitHub___logseq-gardener___Project___Proposal.md)
- [lsdoc candidate](https://github.com/martinkoutecky/lsdoc)
- [Official mldoc](https://github.com/logseq/mldoc)
- [Logseq OG graph parser](https://github.com/logseq/og/tree/version/file/deps/graph-parser)
- [Graph-validator](https://github.com/logseq/graph-validator)
