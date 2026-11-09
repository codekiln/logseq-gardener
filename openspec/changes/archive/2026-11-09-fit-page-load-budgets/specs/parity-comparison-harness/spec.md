## ADDED Requirements

### Requirement: Count a keystroke that waits for a shard as cold

`mise run parity:perf` SHALL count an `lsg` keystroke as cold when it requests a search shard, or when a shard that an earlier keystroke requested is still loading at its keydown. Every other keystroke SHALL count as warm, and the warm budgets SHALL apply to those only.

#### Scenario: Key typed while its shard loads

- **WHEN** a key is typed 300 ms after the key that requested its word's shard, and the shard arrives 160 ms later
- **THEN** the second keystroke counts as cold and its latency is held to the cold keystroke budget

### Requirement: Check every script a page loads in CI

`mise run parity:budgets` SHALL fail when a fixture page of the search build, All pages included, carries more script than the `[desktop]` and `[phone]` `script_before_title_kb` budgets allow. The count SHALL include inline scripts, every script file the page's markup loads, `defer` and module scripts included, and the page-property start-up when the page lacks it, since a garden that shows page properties adds it to every page.

#### Scenario: Deferred script on All pages

- **WHEN** All pages loads a deferred script that brings its script over 16 KB with the page-property start-up counted
- **THEN** `parity:budgets` fails and names `pages.html`
