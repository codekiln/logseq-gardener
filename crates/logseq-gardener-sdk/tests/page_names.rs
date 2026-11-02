use logseq_gardener_sdk::garden::{DocumentKind, GardenDocument};
use logseq_gardener_sdk::page_names::{
    AliasEntry, AliasGroups, PageNameIndex, lookup_key, page_aliases,
};

fn document(source: &str) -> GardenDocument {
    GardenDocument {
        relative_path: "pages/Example.md".into(),
        kind: DocumentKind::Page,
        source: source.to_owned(),
        parsed: lsdoc::parse_to_projection(source),
    }
}

#[test]
fn syntax_is_leading_last_property_and_literal_safe() {
    let page = document(
        "alias:: Old\nALIAS:: Plain, [[Comma, Name]]， Other, `[[Code]]`, **Formatted**\n\n- Body\n  alias:: Nested\n",
    );
    assert_eq!(page_aliases(&page, "Plain"), ["comma, name", "other"]);
    assert!(page_aliases(&document("aliases:: [[Ignored]]\n\n- Body\n"), "Page").is_empty());
    assert!(page_aliases(&document("- Body\n  alias:: Nested\n"), "Page").is_empty());
    assert_eq!(
        page_aliases(&document("alias:: \"[[Literal]], Whole\"\n"), "Page"),
        ["\"[[literal]], whole\""]
    );
    assert_eq!(
        page_aliases(&document("alias:: `[[Literal]]`\n"), "Page"),
        ["`[[literal]]`"]
    );
    assert!(page_aliases(&document("alias:: \n"), "Page").is_empty());
}

#[test]
fn candidates_reproduce_shared_og_alias_ambiguity() {
    let alias_source = document(include_str!(
        "fixtures/og-relationships/garden/pages/Alias Source.md"
    ));
    let chosen = document(include_str!(
        "fixtures/og-relationships/garden/pages/Title Override.md"
    ));
    let mut index = PageNameIndex::default();
    index.insert("source", "Alias Source", &alias_source);
    index.insert("chosen", "Chosen Name", &chosen);
    index.insert("source", "Alias Source", &alias_source);
    assert_eq!(index.candidates("Other Name"), ["chosen", "source"]);
    assert_eq!(index.candidates("Second Name"), ["source"]);
    assert_eq!(index.candidates("Chosen Name"), ["chosen"]);
    assert!(index.candidates("Missing Target").is_empty());
}

#[test]
fn normalized_titles_and_aliases_share_candidates() {
    let mut index = PageNameIndex::default();
    index.insert(2, "Café", &document("alias:: /SELF/, Café\n"));
    index.insert(1, "Other", &document("alias:: [[CAFÉ]], café, /Other/\n"));
    assert_eq!(lookup_key(" /CAFÉ/ "), "café");
    assert_eq!(index.candidates(" /café/ "), [1, 2]);
    assert_eq!(index.candidates("self"), [2]);
    assert_eq!(index.candidates("other"), [1]);
}

#[test]
fn tags_and_nested_references_supply_aliases_as_in_logseq() {
    // Logseq reads `#[[Name]]` and `#name` as the page they tag.
    assert_eq!(
        page_aliases(&document("alias:: #[[Changelog]]\n"), "Page"),
        ["changelog"]
    );
    assert_eq!(
        page_aliases(&document("alias:: #[[Tag Alias]], #short, plain\n"), "Page"),
        ["plain", "short", "tag alias"]
    );
    // A nested reference names its outer page only.
    assert_eq!(
        page_aliases(&document("alias:: [[Outer [[Inner]] Name]]\n"), "Page"),
        ["outer [[inner]] name"]
    );
}

/// Logseq 0.10.15 lists, in a page's `alias` row, the page's own aliases and then every other
/// name one or two alias links away, and opens a name that is a page's title on that page.
#[test]
fn alias_groups_list_names_as_logseq_does() {
    let mut groups = AliasGroups::default();
    groups.insert(
        "garden",
        "Knowledge/Garden",
        &document("alias:: [[Digital Garden]], [[Zettelkasten]]\n"),
    );
    groups.insert("digital", "Digital Garden", &document(""));
    groups.insert("source", "Source", &document("alias:: [[Target]]\n"));
    groups.insert("target", "Target", &document("alias:: [[Other Name]]\n"));
    groups.insert("a", "Claim A", &document("alias:: [[Collision]]\n"));
    groups.insert("b", "Claim B", &document("alias:: [[Collision]]\n"));
    groups.insert(
        "brew",
        "brew",
        &document("alias:: [[brew]], [[homebrew]], [[Homebrew]]\n"),
    );
    let entry = |name: &str, page, authored| AliasEntry {
        name: name.to_owned(),
        page,
        authored,
    };
    assert_eq!(
        groups.entries(&"garden"),
        [
            entry("Digital Garden", "digital", true),
            entry("Zettelkasten", "garden", true)
        ]
    );
    assert_eq!(
        groups.entries(&"digital"),
        [
            entry("Knowledge/Garden", "garden", false),
            entry("Zettelkasten", "garden", false)
        ]
    );
    assert_eq!(
        groups.entries(&"source"),
        [
            entry("Target", "target", true),
            entry("Other Name", "target", false)
        ]
    );
    assert_eq!(
        groups.entries(&"target"),
        [
            entry("Other Name", "target", true),
            entry("Source", "source", false)
        ]
    );
    assert_eq!(
        groups.entries(&"a"),
        [entry("Collision", "a", true), entry("Claim B", "b", false)]
    );
    assert_eq!(
        groups.entries(&"brew"),
        [entry("brew", "brew", true), entry("homebrew", "brew", true)]
    );
    assert!(groups.entries(&"missing").is_empty());
}
