## Why

Readers currently repeat their Properties disclosure choice on every page. Remembering that choice lets the configured first view remain useful while respecting the reader across page navigation and return visits.

## What Changes

- Remember the reader's Properties choice for the site's base path with a sliding absence expiry.
- Restore that choice as the disclosure is parsed and provide a reusable initializer for host navigation.
- Keep configured defaults and fragment reveal working when storage is absent, invalid or blocked.

## Capabilities

### New Capabilities

### Modified Capabilities

- `published-page-properties`: Add site-scoped reader preference restoration and expiry.

## Impact

SDK property markup and page assets, a small early restoration script, browser tests and host integration documentation. No new dependency or source-graph mutation.

## Citations

- [My/Principle/Favor Readers Over Writers](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Favor%20Readers%20Over%20Writers.md)
- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md)
