//! Which page properties Logseq 0.10.15 shows in a page's properties box.
//!
//! Logseq hides a page property when its key is in `hidden-built-in-properties`
//! (`deps/graph-parser/src/logseq/graph_parser/property.cljs`), including the card
//! scheduling keys that its flashcards register there, or in
//! `hidden-editable-page-properties` (`src/main/frontend/util/property.cljs`). The graph's
//! `:block-hidden-properties` hides more keys; callers apply that setting themselves.

/// Keys Logseq hides from every block's properties.
const HIDDEN_BUILT_IN: [&str; 30] = [
    "id",
    "custom-id",
    "background-color",
    "background_color",
    "heading",
    "collapsed",
    "created-at",
    "updated-at",
    "last-modified-at",
    "created_at",
    "last_modified_at",
    "query-table",
    "query-properties",
    "query-sort-by",
    "query-sort-desc",
    "ls-type",
    "hl-type",
    "hl-page",
    "hl-stamp",
    "hl-color",
    "logseq.macro-name",
    "logseq.macro-arguments",
    "logseq.order-list-type",
    "logseq.tldraw.page",
    "logseq.tldraw.shape",
    "todo",
    "doing",
    "now",
    "later",
    "done",
];

/// Keys Logseq hides from page properties only; a block property with one of these
/// keys stays visible.
const HIDDEN_ON_PAGES: [&str; 3] = ["title", "filters", "icon"];

/// Whether Logseq 0.10.15 leaves a page property out of the page's properties box,
/// before the graph's `:block-hidden-properties`. `key` is compared without case.
pub fn is_hidden_page_property(key: &str) -> bool {
    let key = key.trim().to_lowercase();
    HIDDEN_BUILT_IN.contains(&key.as_str())
        || HIDDEN_ON_PAGES.contains(&key.as_str())
        || crate::cards::is_hidden_property(&key)
}

#[cfg(test)]
mod tests {
    use super::is_hidden_page_property;

    #[test]
    fn hides_what_logseq_hides_on_a_page() {
        for key in [
            "id",
            "Title",
            "filters",
            "icon",
            "hl-page",
            "DONE",
            "card-repeats",
        ] {
            assert!(is_hidden_page_property(key), "{key}");
        }
        for key in [
            "alias",
            "aliases",
            "public",
            "tags",
            "search-exclude",
            "custom_id",
            "updated_at",
            "logseq.color",
            "file-path",
        ] {
            assert!(!is_hidden_page_property(key), "{key}");
        }
    }
}
