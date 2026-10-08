use logseq_gardener_sdk::garden::{DocumentKind, GardenDocument};
use logseq_gardener_sdk::page_names::{PageNameIndex, lookup_key, page_aliases};

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
        "../../../openspec/changes/parser-comparison-baseline/experiments/mldoc-baseline/fixtures/garden/pages/Alias Source.md"
    ));
    let chosen = document(include_str!(
        "../../../openspec/changes/parser-comparison-evidence/experiments/parser-comparison/fixtures/pages/Title Override.md"
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
