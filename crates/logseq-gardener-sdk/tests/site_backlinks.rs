use logseq_gardener_sdk::page_titles::FilenameFormat;
use logseq_gardener_sdk::publishing::NamespaceSelection;
use logseq_gardener_sdk::site::{SiteReport, publish_site};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "lsg-backlinks-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&p).unwrap();
        Self(p)
    }
    fn write(&self, path: &str, text: &str) {
        let p = self.0.join("garden").join(path);
        fs::create_dir_all(p.parent().unwrap()).unwrap();
        fs::write(p, text).unwrap();
    }
    fn publish(&self) -> SiteReport {
        publish_site(
            self.0.join("garden"),
            self.0.join("site"),
            FilenameFormat::TripleLowbar,
            &NamespaceSelection::new(&["Notes"], &["Notes/Excluded"]).unwrap(),
        )
        .unwrap()
    }
    fn page(&self, title: &str) -> String {
        fs::read_to_string(self.0.join("site").join(route(title))).unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
fn route(title: &str) -> String {
    let hash = title
        .as_bytes()
        .iter()
        .fold(0xcbf2_9ce4_8422_2325u64, |h, b| {
            (h ^ u64::from(*b)).wrapping_mul(0x0000_0100_0000_01b3)
        });
    format!("p-{hash:016x}.html")
}
fn section(page: &str) -> &str {
    page.split("<section class=\"linked-references\"")
        .nth(1)
        .unwrap()
        .split("</main>")
        .next()
        .unwrap()
}
fn all_files(path: &Path) -> Vec<String> {
    let mut result = Vec::new();
    for entry in fs::read_dir(path).unwrap() {
        let p = entry.unwrap().path();
        if p.is_dir() {
            result.extend(all_files(&p));
        } else {
            result.push(String::from_utf8_lossy(&fs::read(p).unwrap()).into_owned());
        }
    }
    result
}

#[test]
fn aliases_repeats_tags_and_source_groups_have_distinct_counts() {
    let f = Fixture::new();
    f.write("pages/Notes___Target.md","alias:: [[Étiquette]]\n- Target self [[Notes/Target]]\n  id:: 12345678-1234-1234-1234-123456789abc\n");
    f.write("pages/Notes___A.md","- Parent [[Notes/Ancestor]]\n  - First [[Étiquette]] [[Notes/Target]] #Notes/Topic\n    context:: [[Notes/Other]]\n    - Child [[Notes/ChildTopic]]\n  - Second [[ÉTIQUETTE]]\n- Literal `[[Notes/Target]]`\n- ~~~text\n  [[Notes/Target]]\n  ~~~\n- ((12345678-1234-1234-1234-123456789abc))\n");
    f.write("pages/Notes___B.md", "- {{embed [[Notes/Target]]}}\n");
    for name in ["Ancestor", "Topic", "Other", "ChildTopic"] {
        f.write(&format!("pages/Notes___{name}.md"), "- Context page\n");
    }
    let report = f.publish();
    assert_eq!(report.pages, 7);
    let target = f.page("Notes/Target");
    let refs = section(&target);
    assert!(refs.contains(">2 pages</span>"), "{refs}");
    assert!(refs.contains("> · 3 referring contexts</span>"), "{refs}");
    assert!(refs.contains("Include or exclude selected related pages."));
    assert!(!refs.contains("Include every selected page"));
    assert_eq!(refs.matches("data-backlinks-entry ").count(), 3);
    for name in ["A", "B", "Ancestor", "Topic", "Other", "ChildTopic"] {
        assert!(
            refs.contains(&format!(
                "data-backlinks-filter=\"{}\"",
                route(&format!("Notes/{name}"))
            )),
            "missing {name}: {refs}"
        );
    }
    assert!(!refs.contains(&format!(
        "data-backlinks-filter=\"{}\"",
        route("Notes/Target")
    )));
    assert!(refs.contains("data-backlinks-node"));
    assert!(!refs.contains("id=\"block-12345678-1234-1234-1234-123456789abc\""));
    let source = f.page("Notes/A");
    for part in refs.split("#source-block-").skip(1) {
        let start = part.split('"').next().unwrap();
        assert!(
            source.contains(&format!("id=\"source-block-{start}\""))
                || f.page("Notes/B")
                    .contains(&format!("id=\"source-block-{start}\""))
        );
    }
}

#[test]
fn properties_are_minimal_and_private_sources_never_supply_metadata() {
    let f = Fixture::new();
    f.write("pages/Notes___Target.md", "- Target\n");
    f.write(
        "pages/Notes___Properties.md",
        "prev:: [[Notes/Target]]\nunrelated:: UNRELATED_PROPERTY_SENTINEL\n- Ordinary body\n",
    );
    f.write(
        "pages/Notes___Private.md",
        "public:: false\n- [[Notes/Target]] PRIVATE_BLOCK_SENTINEL [[PRIVATE_NAME_SENTINEL]]\n",
    );
    f.write(
        "pages/Notes___Excluded___Hidden.md",
        "- [[Notes/Target]] EXCLUDED_SENTINEL\n",
    );
    let report = f.publish();
    assert_eq!(report.withheld_pages, 1);
    let page = f.page("Notes/Target");
    let refs = section(&page);
    assert!(refs.contains(">1 page</span>"));
    assert!(refs.contains("data-backlinks-reference-count hidden> · 1 referring contexts</span>"));
    assert!(refs.contains("data-backlinks-count-explanation hidden>"));
    assert!(refs.contains("Page reference</a>"));
    assert!(!refs.contains("Ordinary body"));
    for file in all_files(&f.0.join("site")) {
        for secret in [
            "PRIVATE_BLOCK_SENTINEL",
            "PRIVATE_NAME_SENTINEL",
            "EXCLUDED_SENTINEL",
            "UNRELATED_PROPERTY_SENTINEL",
        ] {
            assert!(!file.contains(secret), "leaked {secret}");
        }
    }
}

#[test]
fn threshold_zero_and_depth_control_copies_without_changing_authored_source() {
    let f = Fixture::new();
    f.write(
        "logseq/config.edn",
        "{:ref/linked-references-collapsed-threshold 0 :ref/default-open-blocks-level 2}",
    );
    f.write("pages/Notes___Target.md", "- Target\n");
    let source = "- [[Notes/Target]]\n  - Depth two\n    - Depth three\n- Authored [[Notes/Target]]\n  collapsed:: true\n  - Child\n";
    f.write("pages/Notes___Source.md", source);
    f.publish();
    let target = f.page("Notes/Target");
    let refs = section(&target);
    assert!(refs.contains("<details class=\"backlinks-disclosure\">"));
    assert_eq!(
        refs.matches("<details class=\"outline-disclosure\">")
            .count(),
        2,
        "{refs}"
    );
    let original = f.page("Notes/Source");
    let body = original
        .split("<section class=\"namespace-hierarchy\"")
        .next()
        .unwrap();
    assert_eq!(
        body.matches("<details class=\"outline-disclosure\">")
            .count(),
        1
    );
    assert_eq!(
        fs::read_to_string(f.0.join("garden/pages/Notes___Source.md")).unwrap(),
        source
    );
    let empty = f.page("Notes");
    assert!(section(&empty).contains("<details class=\"backlinks-disclosure\">"));
    assert!(section(&empty).contains(">0 pages</span>"));
}

#[test]
fn generated_namespace_targets_accept_only_authored_references_and_alias_ambiguity_stays_closed() {
    let f = Fixture::new();
    f.write("pages/Notes___Namespace___Leaf.md", "- Leaf\n");
    f.write("pages/Notes___Source.md", "- [[Notes/Namespace]]\n");
    f.write("pages/Notes___One.md", "alias:: [[Ambiguous]]\n- One\n");
    f.write("pages/Notes___Two.md", "alias:: [[Ambiguous]]\n- Two\n");
    f.write("pages/Notes___AmbiguousSource.md", "- [[Ambiguous]]\n");
    f.publish();
    let generated = f.page("Notes/Namespace");
    assert!(section(&generated).contains(">1 page</span>"));
    assert!(section(&f.page("Notes/Namespace/Leaf")).contains(">0 pages</span>"));
    for name in ["One", "Two"] {
        assert!(section(&f.page(&format!("Notes/{name}"))).contains(">0 pages</span>"));
    }
}

#[test]
fn original_source_relative_assets_are_retained_in_namespace_reference_copies() {
    let f = Fixture::new();
    f.write("pages/Notes___Namespace___Leaf.md", "- Leaf\n");
    f.write(
        "pages/Notes___Source.md",
        "- [[Notes/Namespace]] ![art](../assets/art.png)\n",
    );
    f.write("assets/art.png", "TEST_IMAGE");
    let report = f.publish();
    assert_eq!(report.assets, 1);
    assert!(section(&f.page("Notes/Namespace")).contains("<img src=\"assets/"));
}

#[test]
fn escaped_labels_and_nested_asset_contexts_do_not_create_duplicate_ids() {
    let f = Fixture::new();
    f.write("pages/Target.md", "title:: Notes/Target\n- Target\n");
    f.write(
        "pages/Related.md",
        "title:: Notes/Related & \"quoted\"\n- Related\n",
    );
    f.write("pages/nested/Source.md", "title:: Notes/Source\n- Parent [[Notes/Related & \"quoted\"]]\n  - Reference [[Notes/Target]] ![art](../../assets/art.png)\n    id:: 12345678-1234-1234-1234-123456789abc\n    - Descendant\n");
    f.write("assets/art.png", "IMAGE_CONTENT");
    f.publish();
    let target = f.page("Notes/Target");
    let refs = section(&target);
    assert!(refs.contains("data-backlinks-label=\"Notes/Related &amp; &quot;quoted&quot;\""));
    assert!(refs.contains("<img src=\"assets/"));
    assert!(!refs.contains("id=\"block-"));
    assert!(!refs.contains("id=\"source-block-"));
    let source = f.page("Notes/Source");
    assert_eq!(
        source
            .matches("id=\"block-12345678-1234-1234-1234-123456789abc\"")
            .count(),
        1
    );
    assert_eq!(target.matches("<script").count(), 3);
    assert!(target.contains(&format!(
        "<script data-lsg-theme-script>{}</script>",
        include_str!("../src/site/theme.js")
    )));
}

#[test]
fn leading_bare_tags_match_and_missing_or_ambiguous_targets_do_not_supply_options() {
    let f = Fixture::new();
    f.write("pages/Notes___Target.md", "- Target\n");
    f.write(
        "pages/Notes___Source.md",
        "tags:: Notes/Target\n- [[Notes/Target]] [[Missing sentinel]] [[Shared]]\n",
    );
    f.write("pages/Notes___One.md", "alias:: Shared\n- One\n");
    f.write("pages/Notes___Two.md", "alias:: Shared\n- Two\n");
    f.publish();
    let target = f.page("Notes/Target");
    let refs = section(&target);
    assert!(refs.contains(">1 page</span>"));
    assert!(refs.contains("data-backlinks-reference-count> · 2 referring contexts</span>"));
    assert!(refs.contains("A context is a referring block or a page’s properties."));
    assert!(!refs.contains("data-backlinks-label=\"Missing"));
    assert!(!refs.contains("data-backlinks-label=\"Shared"));
    for name in ["One", "Two"] {
        assert!(!refs.contains(&format!(
            "data-backlinks-filter=\"{}\"",
            route(&format!("Notes/{name}"))
        )));
    }
}

#[test]
fn defaults_open_zero_reference_sections_and_threshold_uses_reference_not_page_count() {
    let f = Fixture::new();
    f.write(
        "logseq/config.edn",
        "{:ref/linked-references-collapsed-threshold 2}",
    );
    f.write("pages/Notes___Target.md", "- Target\n");
    f.write(
        "pages/Notes___Source.md",
        "- [[Notes/Target]]\n- [[Notes/Target]]\n",
    );
    f.publish();
    let target = f.page("Notes/Target");
    assert!(section(&target).contains("<details class=\"backlinks-disclosure\">"));
    let source = f.page("Notes/Source");
    assert!(section(&source).contains("<details class=\"backlinks-disclosure\" open>"));
    assert!(section(&target).contains(">1 page</span>"));
    assert!(section(&target).contains("> · 2 referring contexts</span>"));
}

#[test]
fn overlapping_contexts_share_one_outline_and_keep_sources_and_counts() {
    let f = Fixture::new();
    f.write("pages/Notes___Target.md", "- Target\n");
    f.write("pages/Notes___Related.md", "- Related\n");
    let source = "- Ancestor heading\n  - Parent unique [[Notes/Target]]\n    collapsed:: true\n    - Child unique [[Notes/Target]] [[Notes/Related]]\n      - Grandchild unique\n  - Sibling unique [[Notes/Target]]\n";
    f.write("pages/Notes___Source.md", source);
    f.publish();
    let html = f.page("Notes/Target");
    let refs = section(&html);
    assert_eq!(refs.matches("Child unique").count(), 1, "{refs}");
    assert_eq!(refs.matches("Grandchild unique").count(), 1);
    assert_eq!(refs.matches("data-backlinks-entry").count(), 3);
    assert!(refs.contains("3 referring contexts"));
    assert!(refs.contains("<details class=\"outline-disclosure\"><summary>"));
    assert!(!refs.contains("Open source block"));
    let original = f.page("Notes/Source");
    for text in [
        "Ancestor heading",
        "Parent unique",
        "Child unique",
        "Sibling unique",
    ] {
        let start = source.find(text).unwrap()
            - if text == "Ancestor heading" {
                2
            } else if text == "Child unique" {
                6
            } else {
                4
            };
        assert!(
            refs.contains(&format!("#source-block-{start}")),
            "{text}: {refs}"
        );
        assert!(
            original.contains(&format!("id=\"source-block-{start}\"")),
            "{text}: {original}"
        );
    }
    assert_eq!(
        fs::read_to_string(f.0.join("garden/pages/Notes___Source.md")).unwrap(),
        source
    );
}

#[test]
fn property_cards_reuse_only_safe_relevant_displayed_rows() {
    let f = Fixture::new();
    f.write(
        "logseq/config.edn",
        "{:publishing/page-properties? true :block-hidden-properties #{:secret}}",
    );
    f.write(
        "pages/Notes___Target.md",
        "alias:: [[Target alias]]\n- Target\n",
    );
    f.write("pages/Notes___Other.md", "- Other\n");
    f.write("pages/Notes___Private.md", "public:: false\n- Private\n");
    f.write("pages/Notes___Source.md", "instrument:: [[Target alias]]\nother:: [[Notes/Other]]\nsecret:: [[Notes/Target]]\nmixed:: [[Notes/Target]], [[Notes/Private]]\nlocal:: /Users/private/audio.wav\n- Body\n");
    f.publish();
    let html = f.page("Notes/Target");
    let refs = section(&html);
    assert!(refs.contains("<dt>instrument</dt>"));
    assert!(refs.contains(&format!(
        "href=\"{}#page-properties\">Page properties",
        route("Notes/Source")
    )));
    assert!(!refs.contains("<dt>other</dt>"));
    assert!(!refs.contains("<dt>secret</dt>"));
    assert!(!refs.contains("<dt>mixed</dt>"));
    assert!(!refs.contains("/Users/private"));
    assert_eq!(
        refs.matches("<dl class=\"backlink-properties\">").count(),
        1
    );
}
