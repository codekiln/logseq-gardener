## Context

The `ableton-set` build step copies a track's settings from the block properties of its page's heading into the Set page's entry as strings, and the browser module shows each string. Live shows a routing in two boxes: a target, such as `Ext. In` or the name of another track, and a detail, such as `3`, `Track In` or `Post Mixer`. The garden's import task will write a routing to another track as a reference to that track's page followed by the detail.

## Goals / Non-Goals

**Goals:**

- A reader sees which track a routing points at and can open it.
- Pages written before the import task changes keep their view.

**Non-Goals:**

- Render every Logseq inline form inside a setting, such as several references, tags or emphasis. Routing needs one leading reference.

## Decisions

**A leading page reference only.** `leading_reference` reads `[[name]]` at the start of a routing value and the text after it. A reference later in the value stays text. This matches what the import task writes and keeps the entry's shape small.

**The entry says whether the target is a page.** A resolved target is an object `{title, url, detail}`; anything else is a string. The module needs no rule of its own to tell a link from text, and a link without a URL cannot be written. `title` is the text a column shows for that page, so a routing and the column it points at carry the same name.

**Any one published page resolves, not only a track of this Set.** The build step resolves the name as embeds do. A Set may route to a track page that it does not embed; the link is still correct.

**An unresolved reference becomes text.** The entry holds the last segment of the written name, then the detail. The written name is the author's own text on a published page; the withheld page's heading and URL stay out.

## Risks / Trade-offs

[A module from an older `lsg` paired with a newer entry would skip a linked routing] → First-party modules ship inside the `lsg` that writes the entry, so a site never pairs the two.

## Resolved Questions

### 1 - Is a linked routing an object in the entry, or a string the module parses?

Decided by the agent without codekiln: an object. The build step already knows which pages are published, and the module would otherwise repeat the garden's name resolution in the browser.

### 2 - Does the change need its own OpenSpec change?

Decided by the agent without codekiln: yes. It changes what `ableton-set-plugin` requires of the entry and of the view.

### 3 - Can the change be proposed, implemented and synced in one pull request?

Decided by the agent without codekiln: proposed and implemented in one pull request; synced and archived in a follow-up tracked by [Sync and archive link-ableton-set-routing · #634](https://github.com/codekiln/logseq-gardener/issues/634).

## Open Questions

None.
