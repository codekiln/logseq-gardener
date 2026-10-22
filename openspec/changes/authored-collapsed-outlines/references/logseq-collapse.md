# Logseq collapse reference

Inspected the local Logseq repository at tag `0.10.6`, commit `cd2211a042332792ac3f797e34c08bc77c4e4dd4`.

- [graph_parser/property.cljs](https://github.com/logseq/logseq/blob/0.10.6/deps/graph-parser/src/logseq/graph_parser/property.cljs): declares collapsed as a boolean built-in property.
- [graph_parser/block.cljs](https://github.com/logseq/logseq/blob/0.10.6/deps/graph-parser/src/logseq/graph_parser/block.cljs): converts a true parsed collapsed property into the block's collapsed state.
- [frontend/util.cljc](https://github.com/logseq/logseq/blob/0.10.6/src/main/frontend/util.cljc): reads the stored block collapsed flag.
- [components/block.cljs](https://github.com/logseq/logseq/blob/0.10.6/src/main/frontend/components/block.cljs): the normal outline uses the stored state; block-children hides descendants when closed and block-control provides expansion and collapse.

Direct Gardener reproduction with a Dialogue bullet, attached collapsed true property, descendant sentinel, and following sibling emitted an ordinary expanded nested list. The live October 20 episode contained eleven Dialogue labels and only the open bottom Hierarchy disclosure. Gitpa's selected_body removed authored collapsed metadata before rendering. Both layers need their scoped changes for the episode behavior to match.

## Integrated browser verification

The Gitpa preview retained eleven closed Dialogue disclosures and the expanded bottom Hierarchy. Native click and Enter toggled a disclosure. At 390px, the disclosure and transcript wrapped within the viewport. A generic static fixture opened nested ancestors for an initial block fragment while leaving an unrelated branch closed. Gitpa internal navigation revealed a target inside a collapsed Dialogue before scrolling. One recording continued advancing through outline interaction and internal navigation; playback was paused after testing.

Full local CI, the release build, and independent renderer and fragment-helper review passed. Cleanup is tracked separately in #65.
