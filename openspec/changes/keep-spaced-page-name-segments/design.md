## Context

`NamespaceSelection` in `crates/logseq-gardener-sdk/src/publishing.rs` used one rule, `valid_name`, for both its configured roots and the page names it is asked about. The rule rejects any slash-separated segment with surrounding whitespace. Logseq splits a title at every `/` and keeps each segment as written, so the title of `pages/Obsidian___Q___…Info %2F Warning %2F Danger blocks called%3F.md` has the segments `…colored Info `, ` Warning ` and ` Danger blocks called?`, and Logseq's database holds the generated parents `…colored info ` and `…colored info / warning `.

## Goals / Non-Goals

**Goals:**

- Every page Logseq publishes from a selected root is selectable by `lsg`, under Logseq's title.

**Non-Goals:**

- Changing how titles are derived from file names; `page_titles` already decodes `%2F` the way Logseq does.

## Decisions

- Page names get their own rule, `valid_page_name`: every segment has a non-space character and no control characters. Selectors keep the stricter `valid_name`, so a typo such as `--include "My "` is still reported.
- The relationship logic stays in the SDK; the publisher and harness only consume the result.

## Risks / Trade-offs

- A garden author who writes `[[My /Note]]` by mistake now gets a page under that name, as in Logseq. This matches Logseq and is the intended parity.

## Resolved Questions

### 1 - Is a whitespace-only segment, as in `My/ /Note`, selectable?

Decided by the agent without codekiln: no. No encode-garden page has one, Logseq's own title sanitising treats a blank name as no page, and keeping the rejection keeps `includes` from accepting names that only differ by blank segments.

### 2 - Should selectors accept surrounding segment whitespace too?

Decided by the agent without codekiln: no. Selectors are typed configuration, and a stray space there is far more likely a mistake than a real namespace; the error message points to its list and position. A root such as `Obsidian/Q` still selects the spaced descendants.
