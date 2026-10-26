## 1. Stable controls

- [x] 1.1 Replace the shared hover button with per-block controls and idempotent replacement initialization, preserving zoom and native disclosure state.
- [x] 1.2 Align 24px targets with the owning bullet and expose closed, keyboard and touch controls.

## 2. Verification and handoff

- [x] 2.1 Add meaningful owner, state, repeated initialization and replacement-content fixtures; verify nested pointer paths and heading geometry in a browser.
- [x] 2.2 Document the initializer, run required checks and keep the existing performance budgets.
- [ ] 2.3 Create the native cleanup child and submit the tested implementation for independent review.
