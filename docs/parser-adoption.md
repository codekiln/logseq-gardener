# Parser adoption

lsdoc remains the Rust candidate for read-only Markdown parsing. The tested [lsdoc 0.5.8 revision](https://github.com/martinkoutecky/lsdoc/tree/32e63ef095c711d6d9947257bf5fd07d540fa59d) declares AGPL-3.0-only and supplies the complete license text, matching Gardener's declared license. The [license audit](../openspec/changes/parser-license-evidence/experiments/lsdoc-licenses/README.md) records its locked package graph and actual license and attribution files.

## License findings

The candidate's dependency declarations comprise MIT, MIT/Apache alternatives, Unlicense/MIT alternatives, and additional Unicode data terms. The planned permissive-license choice is MIT wherever offered; unicode-ident also requires preservation of its Unicode-3.0 terms. [The inventory](../openspec/changes/parser-license-evidence/experiments/lsdoc-licenses/inventory.json) identifies each package, version, source checksum, declaration, and license-file hash. [The supplied texts](../openspec/changes/parser-license-evidence/experiments/lsdoc-licenses/license-texts.txt) preserve copyright attribution and every supplied license alternative.

Treat lsdoc as an unmodified dependency at its tested revision. Preserve its AGPL text and identify its upstream repository in notices. If adoption changes upstream code, record the modification date, changes, and attribution under the existing [licensing process](licensing.md). The project's corresponding-source materials must include the exact dependency sources and build scripts for distributed builds; AGPL sections 4–6 govern source and binary conveyance, and section 13 covers a modified program's remote network interaction. See [the AGPL text](https://www.gnu.org/licenses/agpl.en.html) for those requirements.

The full MIT and Unicode notices must accompany relevant distributed code or data. Preserve the supplied copyright files, including rand's separate attribution files. Keep all license alternatives in the generated collection so reviewers can inspect the original terms.

## Add the parser to the SDK

1. Finish the remaining compatibility review and record the supported parser scope under [Issue #3 — Parser compatibility decision](https://github.com/codekiln/logseq-gardener/issues/3).
2. Pin the selected dependency revision in the SDK manifest, explain the dependency in the dependency guide, and commit Gardener's resulting Cargo.lock.
3. Run `mise run notices:update` and inspect the resulting package inventory and license collection. Gardener's resolved versions can differ from the candidate's lockfile.
4. Record the parser's upstream revision, attribution, and modification status in THIRD_PARTY_NOTICES.md; verify that the actual dependency's license and attribution files are preserved.
5. Run `mise run notices:check` and the applicable SDK compatibility checks. Follow the existing source-material process whenever distributing builds.

## Compatibility still needed

The merged [parser findings](../openspec/changes/parser-comparison-evidence/experiments/parser-comparison/findings.md) support the candidate for read-only Markdown parsing with Logseq relationships implemented separately in the SDK. [PR #8 — Correct Org references](https://github.com/codekiln/logseq-gardener/pull/8) repairs the comparison runner. [PR #9 — Compare current Logseq OG relationships](https://github.com/codekiln/logseq-gardener/pull/9) records agreement between current and historical relationships on the shared fixtures. Both await maintainer merge.

Pengx's publishing corpus still needs clone approval and comparison. The final adoption decision remains open until that evidence is reviewed. The existing math-rendering difference can be documented for a read-only lookup scope; formatting-preserving editing requires separate evidence. mldoc and the Logseq graph implementations remain research references with their own licenses documented in the experiments. Any future incorporation of their code needs its own source and license review.
