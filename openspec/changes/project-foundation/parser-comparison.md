# Parser comparison plan

The next parser change will compare `lsdoc` with official `mldoc`, Logseq OG's graph parser, and graph-validator before adding garden operations to the SDK.

## Questions to answer

- Do the parsers recognize the same Markdown structures and references?
- Does the SDK reproduce Logseq's interpretation of names, aliases, links, and nested notes?
- Can an unchanged note be read and written without altering its text or formatting?
- What parsing time, memory use, and repairs does each option require?

## Experiment setup

Place each comparative prototype in `openspec/changes/<change>/experiments/<experiment-name>/` with instructions for running it. Record the tested source revisions and licenses alongside the results.

Compare the public encode garden, the official Logseq documentation garden, and focused examples of ambiguous names and references. Pengx's public garden adds examples of notes with embedded media and published pages. Resolve available checkouts through ghq.

Use `lsdoc`'s comparison tooling for syntax checks. Generate expected garden relationships with Logseq's graph parser through graph-validator. Record disagreements with examples that a reviewer can inspect. If `lsdoc` needs extensive repairs, compare the work required with using official `mldoc` directly.
