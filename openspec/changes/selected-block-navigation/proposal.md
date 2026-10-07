## Why

Selected workshop pages contain UUID references to nested notes, but the published site currently leaves those references as literal text. Readers should be able to follow a reference to a note that the namespace policy publishes.

## What Changes

- Add anchors for explicit UUID IDs on rendered Markdown outline items.
- Link unique selected targets using bounded plain target text as the default label.
- Preserve source labels and diagnose references whose targets cannot be published safely.
- Demonstrate workshop navigation and verify that excluded content remains absent.

## Capabilities

### New Capabilities

- `selected-block-navigation`: Local links to explicit IDs on published outline items, with safe fallbacks for unavailable targets.

### Modified Capabilities

- `local-static-publishing`: Supported outline UUID references become navigable links; unavailable references retain fallbacks.

## Impact

The SDK site planner and renderer gain a selected-target index. CLI publishing inherits the behavior without new arguments. The local-site guide documents reference behavior. Integration depends on selected garden media PR #36; cleanup follows that publisher's base specifications.

## Citations

- [My/Principle/Simplify](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Simplify.md) shapes the bounded navigation scope.
- [My/Principle/Favor Readers Over Writers](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Favor%20Readers%20Over%20Writers.md) supports readable link labels.
- [AI/ES/25/ws/What I Learned at the AI Engineer Summit Workshops 2025](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/AI___ES___25___ws___What%20I%20Learned%20at%20the%20AI%20Engineer%20Summit%20Workshops%202025.md) supplies real references to the [MCP workshop](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/AI___ES___25___ws___1___Building%20Agents%20with%20Model%20Context%20Protocol.md).
