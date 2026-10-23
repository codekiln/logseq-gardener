## 1. Shared appearance controller

- [ ] 1.1 Apply the validated initial appearance in the document head, follow live system changes, and persist explicit overrides with safe storage fallbacks.
- [ ] 1.2 Add the labelled native Appearance selector and idempotent host initializer, preserving active document state through replaced controls.

## 2. Palettes and integration guidance

- [ ] 2.1 Replace fixed shared colors with light/dark palette variables, native color schemes, visible focus and a CSS system-preference fallback without JavaScript.
- [ ] 2.2 Document appearance choices, the host initializer, storage behavior and site palette overrides.

## 3. Verification

- [ ] 3.1 Add generic JavaScript and publication regressions for system changes, overrides, System reset, invalid/unavailable storage, repeated initialization and replaced host controls.
- [ ] 3.2 Verify desktop/mobile keyboard controls, readable hierarchy/disclosures/backlinks/media, contrast and preserved content with a rendered fixture.
- [ ] 3.3 Run full CI and independent implementation/codekiln reviews, submit the implementation PR, and create its native specification-cleanup child under issue81.
