# Verification

## Reproduction

On main, the new tests failed:

- `journals::tests::writes_and_reads_weekday_tokens_as_logseq_does`: `DateFormat::parse("E, MM/dd/yyyy")` returned `UnsupportedToken("E")`.
- `garden_settings::tests::reads_journal_formats_with_fallbacks`: `:journal/page-title-format "E, MM/dd/yyyy"` produced the diagnostic "unsupported date token "E"; using "MMM do, yyyy"".
- `site_journals::weekday_title_format_names_and_links_journals`: the site had that diagnostic and "Unresolved or excluded page reference: Fri, 10/23/2026".

After the change, all three pass. The unit test writes and reads back `E`, `EEE` and `EEEE` titles for seven consecutive days, reads `friday` and `FRI` for the short and long patterns, and rejects a contradicting weekday. `rejects_unsupported_tokens` covers `EE` and `EEEEE`.

## Real garden

[Pengx's knowledge garden](https://github.com/pengx17/knowledge-garden) at `cdec71d` sets `:journal/page-title-format "E, yyyy/MM/dd"`. Both builds ran `lsg publish --journals --filename-format triple-lowbar` with every root namespace of `pages/` included.

| Build | Journal title for `journals/2022-07-12.md` | Diagnostics | Published |
| --- | --- | --- | --- |
| main | `Jul 12th, 2022` | 355, including the `E` token diagnostic and an unresolved `[[Tue, 2022/07/12]]` | 170 pages, 144 journals, 40 assets |
| This change | `Tue, 2022/07/12` | 353; those two are gone, the rest are unchanged | 170 pages, 144 journals, 40 assets |

With the slash in each title, the site also gains 72 namespace parent pages such as `Tue, 2022` and `Tue, 2022/07`. Logseq creates the same parent pages for a journal title that contains `/` (`extract.cljc`, `namespace-pages`).

## Checks

- `mise run ci`: format, lint, tests, release build, parity budgets, OpenSpec validation, parser baseline, skill versions, RuleSync and notices passed. `docs:check` stopped at its `mise install` step because a machine-wide mise configuration pins Python 3.12.1, which has no GitHub attestation. Its local link check passes when run alone. `clilint:check` (18 checks at score 4) and `release:check` passed when run individually.
