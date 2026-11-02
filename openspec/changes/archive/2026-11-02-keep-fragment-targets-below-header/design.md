## Context

The header is `position:sticky; top:0; height:48px` at every viewport width; the phone layout changes the left sidebar, not the header. A fragment target reaches the viewport in two ways: the browser scrolls to it on page load and on a hash change, and `fragments.js` calls `target.scrollIntoView()` after it opens collapsed parents, which hosts also call after swapping page content. Both honor `scroll-padding-top` on the scrolling element, which for a published page is `html`.

## Goals / Non-Goals

**Goals:**

- A fragment target's first line stays visible below the header after any fragment scroll, at desktop and phone widths, with no added script or request.

**Non-Goals:**

None.

## Decisions

- Set `html{scroll-padding-top:56px}` in the shell's inline styles. One rule covers native fragment scrolling and every `scrollIntoView()` call, so `fragments.js` stays unchanged.
- Use a fixed pixel value. The header height is in pixels, so a `rem` value would drift from it when a reader changes the browser's default font size.

## Risks / Trade-offs

- [The header height changes later and the padding falls behind it] → `tests/site_shell.rs` asserts the rule next to the header's height, so a change to one shows the other.

## Resolved Questions

### 1 - How much padding should sit above a fragment target?

Decided by the agent without codekiln: 56px, the 48px header plus 8px. With exactly 48px, the target's identifier lands at the header's bottom edge, but the block's `li` starts 5px above that identifier, so the bullet row still sits partly under the header. The extra 8px clears it and leaves a small gap, and it equals the `3.5rem` Gitpa chose in `site/player.css` at the default font size, so Gitpa can drop its rule without changing where targets land.

## Open Questions

None.
