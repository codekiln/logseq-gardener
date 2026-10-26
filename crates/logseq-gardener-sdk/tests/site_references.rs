use logseq_gardener_sdk::journals::JournalDate;
use logseq_gardener_sdk::page_titles::FilenameFormat;
use logseq_gardener_sdk::publishing::NamespaceSelection;
use logseq_gardener_sdk::site::{SiteOptions, SiteReport, publish_site, publish_site_with_options};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let p = std::env::temp_dir().join(format!(
            "lsg-references-{}-{}",
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
    fn publish_journals(&self) -> SiteReport {
        publish_site_with_options(
            self.0.join("garden"),
            self.0.join("site"),
            FilenameFormat::TripleLowbar,
            &NamespaceSelection::new(&["Notes"], &[]).unwrap(),
            &SiteOptions {
                journals: true,
                build_date: JournalDate::new(2026, 10, 25),
                ..SiteOptions::default()
            },
        )
        .unwrap()
    }
    fn page(&self, title: &str) -> String {
        fs::read_to_string(self.0.join("site").join(route(title))).unwrap()
    }
    /// The fragment files of one kind (`linked` or `unlinked`) for a page, in order.
    fn fragments(&self, title: &str, kind: &str) -> Vec<String> {
        let stem = route(title).trim_end_matches(".html").to_owned();
        (1..)
            .map(|n| {
                self.0
                    .join("site/_refs")
                    .join(format!("{stem}-{kind}-{n}.html"))
            })
            .take_while(|path| path.exists())
            .map(|path| fs::read_to_string(path).unwrap())
            .collect()
    }
    /// The Linked References header and every linked fragment.
    fn linked(&self, title: &str) -> String {
        section(&self.page(title), "linked").to_owned() + &self.fragments(title, "linked").concat()
    }
    /// The data the first linked fragment carries for the filter dialog.
    fn data(&self, title: &str) -> serde_json::Value {
        data(&self.fragments(title, "linked")[0])
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
/// The page's section for `kind`, or an empty string when the page has none.
fn section<'a>(page: &'a str, kind: &str) -> &'a str {
    page.split_once(&format!("data-refs=\"{kind}\""))
        .map(|(_, rest)| rest.split_once("</section>").unwrap().0)
        .unwrap_or("")
}
fn data(fragment: &str) -> serde_json::Value {
    let json = fragment
        .split_once("class=\"references-data\">")
        .unwrap()
        .1
        .split_once("</script>")
        .unwrap()
        .0;
    serde_json::from_str(json).unwrap()
}
fn titles(data: &serde_json::Value) -> Vec<String> {
    data["pages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|page| page[0].as_str().unwrap().to_owned())
        .collect()
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
fn aliases_repeats_and_tags_count_referring_blocks() {
    let f = Fixture::new();
    f.write("pages/Notes___Target.md","alias:: [[Étiquette]]\n- Target self [[Notes/Target]]\n  id:: 12345678-1234-1234-1234-123456789abc\n");
    f.write("pages/Notes___A.md","- Parent [[Notes/Ancestor]]\n  - First [[Étiquette]] [[Notes/Target]] #Notes/Topic\n    context:: [[Notes/Other]]\n    - Child [[Notes/ChildTopic]]\n  - Second [[ÉTIQUETTE]]\n- Literal `[[Notes/Target]]`\n- ~~~text\n  [[Notes/Target]]\n  ~~~\n- ((12345678-1234-1234-1234-123456789abc))\n");
    f.write("pages/Notes___B.md", "- {{embed [[Notes/Target]]}}\n");
    for name in ["Ancestor", "Topic", "Other", "ChildTopic"] {
        f.write(&format!("pages/Notes___{name}.md"), "- Context page\n");
    }
    let report = f.publish();
    assert_eq!(report.pages, 7);
    let target = f.page("Notes/Target");
    assert!(target.contains("<h2>3 Linked References</h2>"), "{target}");
    let refs = f.linked("Notes/Target");
    assert_eq!(refs.matches(" data-ref-block=").count(), 3);
    let data = f.data("Notes/Target");
    assert_eq!(data["count"], 3);
    let titles = titles(&data);
    for name in ["A", "B", "Ancestor", "Topic", "Other", "ChildTopic", ""] {
        let title = if name.is_empty() {
            "Notes".to_owned()
        } else {
            format!("Notes/{name}")
        };
        assert!(titles.contains(&title), "missing {title}: {titles:?}");
    }
    assert!(!titles.contains(&"Notes/Target".to_owned()));
    // A referring block below the top level shows its parent as a crumb.
    assert!(refs.contains("<div class=\"breadcrumb block-parents\"><a href="));
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
    assert!(
        f.page("Notes/Target")
            .contains("<h2>1 Linked Reference</h2>")
    );
    let refs = f.linked("Notes/Target");
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
    assert!(section(&target, "linked").contains("<details class=\"references-section\"><summary>"));
    let refs = f.linked("Notes/Target");
    assert_eq!(
        refs.matches("<details class=\"outline-disclosure\">")
            .count(),
        2,
        "{refs}"
    );
    let original = f.page("Notes/Source");
    let body = original.split("data-refs=").next().unwrap();
    assert_eq!(
        body.matches("<details class=\"outline-disclosure\">")
            .count(),
        1
    );
    assert_eq!(
        fs::read_to_string(f.0.join("garden/pages/Notes___Source.md")).unwrap(),
        source
    );
    // References to Notes/Target also count for its namespace parent.
    let parent = f.page("Notes");
    assert!(parent.contains("<h2>2 Linked References</h2>"), "{parent}");
}

#[test]
fn pages_without_references_have_no_linked_section_and_ambiguous_aliases_stay_closed() {
    let f = Fixture::new();
    f.write("pages/Notes___Namespace___Leaf.md", "- Leaf\n");
    f.write("pages/Notes___Source.md", "- [[Notes/Namespace]]\n");
    f.write("pages/Notes___One.md", "alias:: [[Ambiguous]]\n- One\n");
    f.write("pages/Notes___Two.md", "alias:: [[Ambiguous]]\n- Two\n");
    f.write("pages/Notes___AmbiguousSource.md", "- [[Ambiguous]]\n");
    f.publish();
    assert!(
        f.page("Notes/Namespace")
            .contains("<h2>1 Linked Reference</h2>")
    );
    for title in ["Notes/Namespace/Leaf", "Notes/One", "Notes/Two"] {
        let page = f.page(title);
        assert!(section(&page, "linked").is_empty(), "{title}");
        assert!(page.contains("data-refs=\"unlinked\""), "{title}");
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
    assert!(f.linked("Notes/Namespace").contains("<img src=\"assets/"));
    // The fragment's base resolves asset and page routes from the site root.
    assert!(f.fragments("Notes/Namespace", "linked")[0].contains("<base href=\"../\">"));
}

#[test]
fn escaped_titles_and_nested_asset_contexts_do_not_create_duplicate_ids() {
    let f = Fixture::new();
    f.write("pages/Target.md", "title:: Notes/Target\n- Target\n");
    f.write(
        "pages/Related.md",
        "title:: Notes/Related & \"quoted\" <b>\n- Related\n",
    );
    f.write("pages/nested/Source.md", "title:: Notes/Source\n- Parent [[Notes/Related & \"quoted\" <b>]]\n  - Reference [[Notes/Target]] ![art](../../assets/art.png)\n    id:: 12345678-1234-1234-1234-123456789abc\n    - Descendant\n");
    f.write("assets/art.png", "IMAGE_CONTENT");
    f.publish();
    let refs = f.linked("Notes/Target");
    assert!(titles(&f.data("Notes/Target")).contains(&"Notes/Related & \"quoted\" <b>".to_owned()));
    // The data never closes its script element early.
    assert!(!refs.contains("<b>"));
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
    let target = f.page("Notes/Target");
    assert_eq!(target.matches("<script").count(), 3);
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
    assert!(
        f.page("Notes/Target")
            .contains("<h2>2 Linked References</h2>")
    );
    let titles = titles(&f.data("Notes/Target"));
    for name in ["Missing sentinel", "Shared", "Notes/One", "Notes/Two"] {
        assert!(!titles.contains(&name.to_owned()), "{name}");
    }
}

#[test]
fn threshold_counts_referring_blocks() {
    let f = Fixture::new();
    f.write(
        "logseq/config.edn",
        "{:ref/linked-references-collapsed-threshold 2}",
    );
    f.write("pages/Notes___Target.md", "- Target\n");
    f.write("pages/Notes___Single.md", "- Single\n");
    f.write(
        "pages/Notes___Source.md",
        "- [[Notes/Target]]\n- [[Notes/Target]] [[Notes/Single]]\n",
    );
    f.publish();
    let target = f.page("Notes/Target");
    assert!(section(&target, "linked").contains("<details class=\"references-section\"><summary>"));
    let single = f.page("Notes/Single");
    assert!(section(&single, "linked").contains("<details class=\"references-section\" open>"));
}

#[test]
fn overlapping_contexts_share_one_outline_and_keep_sources_and_counts() {
    let f = Fixture::new();
    f.write("pages/Notes___Target.md", "- Target\n");
    f.write("pages/Notes___Related.md", "- Related\n");
    let source = "- Ancestor heading\n  - Parent unique [[Notes/Target]]\n    collapsed:: true\n    - Child unique [[Notes/Target]] [[Notes/Related]]\n      - Grandchild unique\n  - Sibling unique [[Notes/Target]]\n";
    f.write("pages/Notes___Source.md", source);
    f.publish();
    assert!(
        f.page("Notes/Target")
            .contains("<h2>3 Linked References</h2>")
    );
    let refs = f.linked("Notes/Target");
    assert_eq!(refs.matches("Child unique").count(), 1, "{refs}");
    assert_eq!(refs.matches("Grandchild unique").count(), 1);
    assert_eq!(refs.matches(" data-ref-block=").count(), 3);
    assert!(refs.contains("<details class=\"outline-disclosure\"><summary>"));
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
    let refs = f.linked("Notes/Target");
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

#[test]
fn groups_put_journals_first_and_breadcrumbs_keep_three_ancestors() {
    let f = Fixture::new();
    f.write("pages/Notes___Target.md", "- Target\n");
    f.write("pages/Notes___zeta.md", "- [[Notes/Target]] zeta\n");
    f.write("pages/Notes___Alpha.md", "- [[Notes/Target]] alpha\n");
    f.write("journals/2026_10_04.md", "- [[Notes/Target]] older\n");
    f.write("journals/2026_10_19.md", "- [[Notes/Target]] newer\n");
    f.write(
        "pages/Notes___Deep.md",
        "- One [[Notes/Alpha]]\n  - Two\n    - Three\n      - Four\n        - Deep [[Notes/Target]]\n",
    );
    f.publish_journals();
    let refs = f.linked("Notes/Target");
    let position = |text: &str| refs.find(text).unwrap_or_else(|| panic!("{text}: {refs}"));
    assert!(position(">Oct 19th, 2026</a>") < position(">Oct 4th, 2026</a>"));
    assert!(position(">Oct 4th, 2026</a>") < position(">Notes/Alpha</a></summary>"));
    assert!(position(">Notes/Alpha</a></summary>") < position(">Notes/Deep</a></summary>"));
    assert!(position(">Notes/Deep</a></summary>") < position(">Notes/zeta</a></summary>"));
    let crumbs = refs
        .split("<div class=\"breadcrumb block-parents\">")
        .nth(1)
        .unwrap();
    let crumbs = crumbs.split("</div>").next().unwrap();
    assert!(
        crumbs.starts_with("<span class=\"crumb-more\">⋯</span>"),
        "{crumbs}"
    );
    assert!(!crumbs.contains("One"));
    assert_eq!(crumbs.matches("<span class=\"crumb-sep\"").count(), 3);
    assert!(crumbs.contains(">Two</a>") && crumbs.contains(">Four</a>"));
}

#[test]
fn starting_filters_set_the_first_count_and_funnel() {
    let f = Fixture::new();
    f.write(
        "pages/Notes___Target.md",
        "filters:: {\"notes/howto\" true}\n\n- Target\n",
    );
    f.write("pages/Notes___HowTo.md", "- How to\n");
    f.write(
        "pages/Notes___Source.md",
        "- [[Notes/HowTo]]\n  - [[Notes/Target]] kept\n- [[Notes/Target]] dropped\n",
    );
    f.write("pages/Notes___Plain.md", "filters:: {}\n\n- Plain\n");
    f.write("pages/Notes___Other.md", "- [[Notes/Plain]]\n");
    f.publish();
    let target = f.page("Notes/Target");
    let header = section(&target, "linked");
    assert!(
        header.contains("<h2>1 of 2 Linked References</h2>"),
        "{header}"
    );
    assert!(header.contains("data-state=\"include\""));
    let data = f.data("Notes/Target");
    let howto = titles(&data)
        .iter()
        .position(|title| title == "Notes/HowTo")
        .unwrap();
    assert_eq!(data["filter"], serde_json::json!([[howto], []]));
    assert!(!target.contains("<dt>filters</dt>"));
    let plain = section(&f.page("Notes/Plain"), "linked").to_owned();
    assert!(plain.contains("<h2>1 Linked Reference</h2>") && plain.contains("data-state=\"none\""));
}

#[test]
fn tagged_pages_list_lower_cased_heading_and_sorted_pages() {
    let f = Fixture::new();
    f.write("pages/Notes___Term.md", "- Term\n");
    f.write("pages/Notes___zebra.md", "tags:: Notes/Term\n\n- z\n");
    f.write(
        "pages/Notes___Apple.md",
        "tags:: [[Notes/Term]], other\n\n- a\n",
    );
    f.write("pages/Notes___Mango.md", "- tags:: Notes/Term\n");
    f.publish();
    let html = f.page("Notes/Term");
    let page = html.split_once("<main>").unwrap().1;
    let tagged = page
        .split_once("<section class=\"references tagged-pages\">")
        .unwrap()
        .1;
    assert!(tagged.contains("<h2>Pages tagged with \"notes/term\"</h2>"));
    let apple = tagged.find(">Notes/Apple</a>").unwrap();
    assert!(apple < tagged.find(">Notes/zebra</a>").unwrap());
    assert!(
        !tagged
            .split("</section>")
            .next()
            .unwrap()
            .contains("Notes/Mango")
    );
    // Logseq's order: Pages tagged with, Linked References, Hierarchy, Unlinked References.
    let order: Vec<usize> = [
        "tagged-pages",
        "data-refs=\"linked\"",
        "class=\"namespace-hierarchy\"",
        "data-refs=\"unlinked\"",
    ]
    .iter()
    .map(|marker| page.find(marker).unwrap_or(usize::MAX))
    .collect();
    assert!(order.windows(2).all(|pair| pair[0] < pair[1]), "{order:?}");
}

#[test]
fn unlinked_references_load_from_fragments_with_logseqs_match_rule() {
    let f = Fixture::new();
    f.write(
        "pages/Notes___Gadget.md",
        "- Notes/Gadget mentions itself\n",
    );
    f.write(
        "pages/Notes___Source.md",
        "- I use a NOTES/GADGET daily\n- #notes/gadget tag\n- notes/gadgets plural\n- :LOGBOOK:\n  notes/gadget\n  :END:\n- [[Notes/Gadget]] link\n",
    );
    f.write("pages/Notes___Quiet.md", "- Nothing here\n");
    f.publish();
    let page = f.page("Notes/Gadget");
    let header = section(&page, "unlinked");
    assert!(header.contains("<h2>Unlinked References</h2>"));
    assert!(!header.contains(" open>"));
    let fragments = f.fragments("Notes/Gadget", "unlinked");
    assert_eq!(fragments.len(), 1);
    assert_eq!(data(&fragments[0])["count"], 1);
    assert!(fragments[0].contains("I use a NOTES/GADGET daily"));
    assert!(!fragments[0].contains("plural"));
    assert!(!fragments[0].contains("mentions itself"));
    assert!(header.contains(&format!(
        "<noscript><a href=\"_refs/{}-unlinked-1.html\"",
        route("Notes/Gadget").trim_end_matches(".html")
    )));
    // No mentions: no file, and the script shows the zero count.
    let quiet = f.page("Notes/Quiet");
    assert!(section(&quiet, "unlinked").contains("No unlinked references."));
    assert!(!section(&quiet, "unlinked").contains("data-src"));
    assert!(f.fragments("Notes/Quiet", "unlinked").is_empty());
}

#[test]
fn bodies_stay_out_of_the_page_and_fragments_page_with_next_links() {
    let f = Fixture::new();
    f.write("pages/Notes___Target.md", "- Target\n");
    for n in 0..120 {
        f.write(
            &format!("pages/Notes___Source{n:03}.md"),
            &format!("- [[Notes/Target]] BODY_{n:03}\n"),
        );
    }
    f.publish();
    let page = f.page("Notes/Target");
    assert!(!page.contains("BODY_"));
    assert!(page.contains("<noscript><a href=\"_refs/"));
    let fragments = f.fragments("Notes/Target", "linked");
    assert_eq!(fragments.len(), 3);
    assert!(fragments[0].contains("BODY_000") && !fragments[0].contains("BODY_050"));
    assert!(fragments[0].contains("-linked-2.html\" rel=\"next\">More linked references</a>"));
    assert!(!fragments[2].contains("rel=\"next\""));
    let data = data(&fragments[0]);
    assert_eq!(data["count"], 120);
    assert_eq!(data["files"].as_array().unwrap().len(), 3);
    assert_eq!(data["blocks"][119][0], 2);
    assert!(!fragments[1].contains("references-data"));
}
