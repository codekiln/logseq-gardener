# Verification

The implementation is tested with the neighboring outline disclosure fix from [PR181 — reachable outline disclosure controls](https://github.com/codekiln/logseq-gardener/pull/181) (`815d740f12cc3262472e2f1131bdcc372c0497a2`) present. The Properties product diff changes no outline source or styling.

- `mise run ci`: passed, including optimized release, unchanged parity budgets, strict OpenSpec validation, RuleSync and release checks.
- `cargo test --locked -p logseq-gardener-sdk --test site_properties`: passed. Fixtures retain configured defaults, privacy filtering and source immutability, and check early API availability on pages with no visible rows.
- `node --test tests/properties.test.cjs`: passed. Covers both choices, malformed/future/expired storage, renewal, blocked storage, ordered queued activation and programmatic reveal.
- `mise run properties:browser-test`: passed on generated publications. Covers no-JS defaults, both base paths, readable nested routes, early parsing restoration with the controller held, keyboard activation, no-properties-to-properties SPA transitions, fragment precedence, theme/search and outline disclosure/zoom compatibility.

The combined enabled Properties and Search fixture carries 15,990 bytes of initially referenced script against the unchanged 16,000 limit, measured with the existing conservative parity resource parser. The small early Properties reader contributes 678; the preference controller loads after paint. Disabled Properties publishes neither its reader nor its module. This is static resource accounting, not a new browser timing profile.

The persisted value contains only `open` and `seen`; the latter is the last-seen timestamp. The key uses the directory of the output-relative Properties asset, so it does not contain a page title or route.

Follow-up: [native cleanup issue182 — sync and archive Properties preferences](https://github.com/codekiln/logseq-gardener/issues/182) tracks exact requirement synchronization and archival of `remember-page-properties` after the implementation reaches main. Independent root review is required before merge.
