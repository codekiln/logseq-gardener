# Verification

## Reproduction

`cargo test --locked -p logseq-gardener-sdk --test site_public_pages` on the branch's tests with main's SDK: 3 passed, 3 failed. With the setting absent, `false`, or set to a string, `Notes` and `Notes/Unmarked` (no `public::`) were published.

After the change, all 6 pass. `garden_settings` unit tests cover both keys, conflicting keys, a commented-out key, a nested key, `nil` and a malformed value.

## Real gardens

Both builds published every top-level namespace with `--journals` and `--filename-format triple-lowbar`.

| Garden | Setting | main | This change |
| --- | --- | --- | --- |
| Encode garden at `5095bdf6` | `:publishing/all-pages-public? true` | 8,730 pages, 512 journals, 111 assets, 2 withheld | Same counts; every output file byte-identical, `lsg-site.json` revision `dab0c0af8a0147a9` in both |
| Gitpa's `gitp-garden` at `9b225f9` (no `--journals`) | `:publishing/all-pages-public? false` | 2,358 pages, 0 withheld | 2,305 pages, 53 withheld |

The garden has 2,305 pages marked `public:: true`. Gitpa's own build passes only those pages to `--include`, so its published site is unchanged.

## Checks

- `mise run ci`: format, lint, tests, release build, parity budgets, OpenSpec validation, parser baseline, skill versions, RuleSync and notices passed. `docs:check` stopped at its `mise install` step because a machine-wide mise configuration pins Python 3.12.1, which has no GitHub attestation. Its local link check passes when run alone. `clilint:check` (18 checks at score 4, with the changed `lsg publish` help) and `release:check` passed when run individually.
