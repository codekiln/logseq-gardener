# Verification

The implementation is tested with the repaired outline disclosure fix from [PR #181 — reachable outline disclosure controls](https://github.com/codekiln/logseq-gardener/pull/181) present. The Properties product diff changes no outline source or styling. The October 30 restack retains main's media, query, flashcard and sidebar behavior.

- `mise run ci`: passed, including optimized release, unchanged parity budgets, strict OpenSpec validation, RuleSync and release checks.
- `cargo test --locked -p logseq-gardener-sdk --test site_properties`: passed. Fixtures retain configured defaults, privacy filtering and source immutability, and check early API availability on pages with no visible rows.
- `node --test tests/properties.test.cjs`: passed. Covers both choices, malformed/future/expired storage, renewal, blocked storage, ordered queued activation and programmatic reveal.
- `mise run properties:browser-test`: passed on generated publications. Covers no-JS defaults, both base paths, readable nested routes, early parsing restoration with the controller held, keyboard activation, no-properties-to-properties SPA transitions, fragment precedence, theme/search and outline disclosure/zoom compatibility.

Every ordinary reading page in the combined Properties and Search fixture remains below the unchanged initial-script limit. The maximum is 11,763 against 16,000, measured with the existing conservative parity resource parser. The check covers all six Notes pages across the configured-open and configured-closed gardens. Generated Graph and All pages utilities retain their separate controller budgets in `parity:budgets`; full CI passes those checks. The early Properties reader contributes 793, and the preference controller loads after paint. Disabled Properties publishes neither its reader nor its module. This measures static resources; it does not establish browser timing.

The held-controller regression now checks both opening and closing before the host replaces the page. The replacement uses the accepted choice before the module arrives and retains it after adoption. VM fixtures also check replacement during a normal deferred save, canceled activations and programmatic fragment openings. Pending activations retain their order until their writes complete.

The persisted value contains only `open` and `seen`; the latter is the last-seen timestamp. The key uses the directory of the output-relative Properties asset, so it does not contain a page title or route.

Follow-up: [Issue #182 — sync and archive Properties preferences](https://github.com/codekiln/logseq-gardener/issues/182) is the native cleanup child for `remember-page-properties` after the implementation reaches main.
