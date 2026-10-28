## Context

The parser, lsdoc, reports a heading line written with a bullet as a `Bullet` node and one written without a bullet as a `Heading` node. Every SDK view finds blocks by looking for `Bullet` nodes, so a `Heading` node becomes part of the text of the bullet before it, or part of the page's leading text when it comes first. Logseq's graph parser starts a block at every heading node, bulleted or not. In the encode garden all 1,082 unbulleted heading lines start at column 0, and lsdoc gives each of them level 1.

## Goals / Non-Goals

**Goals:**

- Blocks, block counts and block references match Logseq for pages with unbulleted heading lines.

**Non-Goals:**

None.

## Decisions

- Turn top-level `Heading` nodes into `Bullet` nodes with the same fields once, in `garden::parse_document`, which `load_garden` calls. One change at the parse step fixes the outline, linked references, queries, transclusion and search together, where changing each reader would leave them free to disagree. The alternative, teaching each reader that a `Heading` node also starts a block, touches about a dozen match sites.
- Keep the level lsdoc reports. A column-0 heading is level 1, a top-level block; an indented `\t- child` below it becomes its child, as in Logseq.

## Risks / Trade-offs

- Pages that open with `# Title` used to show the title without a bullet, above the outline. They now show it as the first bulleted block. → Logseq shows it the same way.
- Properties written directly under an unbulleted heading are now that heading's block properties instead of joining the page's leading text. → Logseq reads them the same way.

## Resolved Questions

### 1 - Where should a heading line start a block: in the parser output or in each reader?

Decided by the agent without codekiln: in the parser output, at load time. The readers already share one block list, so changing that list keeps them consistent, and lsdoc is an outside dependency that this repository does not change.

### 2 - Should an indented heading line inside a bullet's text also start a block?

Decided by the agent without codekiln: yes, for any top-level `Heading` node lsdoc reports, at the level lsdoc gives it, because Logseq starts a block at every heading node. The encode garden has no such line outside code blocks today, so no page changes because of this choice.
