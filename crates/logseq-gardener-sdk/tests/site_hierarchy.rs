use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};

use logseq_gardener_sdk::{
    page_titles::FilenameFormat,
    publishing::NamespaceSelection,
    site::{SiteOptions, SiteReport, publish_site_with_options},
};

static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "lsg-hierarchy-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(path.join("garden/pages")).unwrap();
        Self(path)
    }
    fn write(&self, name: &str, body: &str) {
        fs::write(self.0.join("garden/pages").join(name), body).unwrap();
    }
    fn publish(&self, includes: &[&str], excludes: &[&str], homepage: Option<&str>) -> SiteReport {
        publish_site_with_options(
            self.0.join("garden"),
            self.0.join("site"),
            FilenameFormat::TripleLowbar,
            &NamespaceSelection::new(includes, excludes).unwrap(),
            &SiteOptions {
                homepage: homepage.map(str::to_owned),
                ..SiteOptions::default()
            },
        )
        .unwrap()
    }
    fn html(&self, title: &str) -> String {
        fs::read_to_string(self.0.join("site").join(route(title))).unwrap()
    }
    fn index(&self, homepage: bool) -> String {
        fs::read_to_string(self.0.join(if homepage {
            "site/pages.html"
        } else {
            "site/index.html"
        }))
        .unwrap()
    }
    fn all_html(&self) -> String {
        files(&self.0.join("site"))
            .into_values()
            .map(|bytes| String::from_utf8(bytes).unwrap())
            .collect()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
/// Every file under `root`, including the reference fragments in `_refs/`, by path.
fn files(root: &Path) -> std::collections::BTreeMap<PathBuf, Vec<u8>> {
    let mut result = std::collections::BTreeMap::new();
    let mut pending = vec![root.to_owned()];
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                pending.push(path);
            } else {
                let relative = path.strip_prefix(root).unwrap().to_owned();
                result.insert(relative, fs::read(path).unwrap());
            }
        }
    }
    result
}

fn route(title: &str) -> String {
    let hash = title
        .as_bytes()
        .iter()
        .fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
            (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
        });
    format!("p-{hash:016x}.html")
}
fn has_route(root: &Path, title: &str) -> bool {
    root.join("site").join(route(title)).is_file()
}

fn hierarchy_rows(html: &str) -> Vec<&str> {
    let section = html
        .split_once("<section class=\"page-hierarchy")
        .unwrap()
        .1;
    let list = section
        .split_once("<ul class=\"namespaces\">")
        .unwrap()
        .1
        .split_once("</ul>")
        .unwrap()
        .0;
    list.split("<li>")
        .skip(1)
        .map(|row| row.split_once("</li>").unwrap().0)
        .collect()
}

fn row_text(row: &str) -> String {
    let mut tag = false;
    row.chars()
        .filter(|character| match character {
            '<' => {
                tag = true;
                false
            }
            '>' => {
                tag = false;
                false
            }
            _ => !tag,
        })
        .collect()
}

#[test]
fn missing_parents_are_indexed_navigation_pages_without_source_lookup() {
    let f = Fixture::new();
    f.write(
        "Instruments___Synth___Preset.md",
        "- # Notes\n\t- LEAF_NOTES\n- [[Instruments/Synth]]\n- {{embed [[Instruments/Synth]]}}\n",
    );
    let report = f.publish(&["Instruments/Synth/Preset"], &[], None);
    assert_eq!(report.pages, 1);
    for title in [
        "Instruments",
        "Instruments/Synth",
        "Instruments/Synth/Preset",
    ] {
        assert!(f.index(false).contains(&format!(">{title}</a>")));
        assert!(has_route(&f.0, title));
    }
    assert!(f.html("Instruments").contains(&format!(
        "href=\"{}\">Synth</a>",
        route("Instruments/Synth")
    )));
    let parent = f.html("Instruments/Synth");
    assert!(parent.contains("aria-label=\"Hierarchy\""));
    assert!(parent.contains(&format!(
        "href=\"{}\">Preset</a>",
        route("Instruments/Synth/Preset")
    )));
    let leaf = f.html("Instruments/Synth/Preset");
    assert!(!leaf.contains("namespace-breadcrumbs"));
    assert!(leaf.contains("LEAF_NOTES"));
    assert!(leaf.contains("class=\"page-reference unresolved\"><span>Instruments/Synth"));
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("Instruments/Synth"))
    );
    assert!(leaf.contains("aria-label=\"Hierarchy\""));
    assert_eq!(
        hierarchy_rows(&leaf)
            .iter()
            .map(|row| row_text(row))
            .collect::<Vec<_>>(),
        ["Instruments/Synth"]
    );
    assert!(
        leaf.find("LEAF_NOTES").unwrap() < leaf.find("<section class=\"page-hierarchy ").unwrap()
    );
}

#[test]
fn unselected_source_parent_contributes_no_body_alias_or_assets() {
    let f = Fixture::new();
    f.write(
        "Instruments.md",
        "alias:: UNSELECTED_ALIAS\n- UNSELECTED_BODY\n- ![secret](../assets/missing.png)\n",
    );
    f.write("Instruments___Synth.md", "- SELECTED_BODY\n");
    let report = f.publish(&["Instruments/Synth"], &[], None);
    assert_eq!(report.pages, 1);
    assert_eq!(report.assets, 0);
    assert!(report.diagnostics.is_empty());
    let output = f.all_html();
    for secret in ["UNSELECTED_BODY", "UNSELECTED_ALIAS", "missing.png"] {
        assert!(!output.contains(secret));
    }
    assert!(f.html("Instruments").contains("Hierarchy"));
}

#[test]
fn private_title_override_outside_selection_blocks_synthetic_parent() {
    let f = Fixture::new();
    f.write(
        "renamed.md",
        "title:: Secret\npublic:: false\n- PRIVATE_SENTINEL\n",
    );
    f.write("Secret___PublicChild.md", "- PUBLIC_CHILD\n");
    let report = f.publish(&["Secret/PublicChild"], &[], None);
    assert_eq!(report.pages, 1);
    assert_eq!(report.withheld_pages, 0);
    assert!(!has_route(&f.0, "Secret"));
    assert!(!f.index(false).contains(">Secret</a>"));
    let leaf = f.html("Secret/PublicChild");
    assert!(leaf.contains("<span class=\"namespace-unavailable\">Secret</span>"));
    assert!(!leaf.contains(&route("Secret")));
    assert!(!f.all_html().contains("PRIVATE_SENTINEL"));
}

#[test]
fn hidden_gaps_connect_children_to_nearest_visible_ancestor() {
    let f = Fixture::new();
    f.write("Root.md", "- ROOT_BODY\n");
    f.write(
        "Root___Private.md",
        "- Parent\n\t- PRIVATE_SENTINEL\n\t  public:: false\n",
    );
    f.write("Root___Private___Visible.md", "- VISIBLE_BODY\n");
    f.write("Root___Excluded___Only.md", "- EXCLUDED_SENTINEL\n");
    f.write("Hidden___Only.md", "public:: false\n- HIDDEN_SENTINEL\n");
    let report = f.publish(&["Root", "Hidden"], &["Root/Excluded"], None);
    assert_eq!(report.pages, 2);
    assert_eq!(report.withheld_pages, 2);
    let root = f.html("Root");
    let rows = hierarchy_rows(&root);
    assert_eq!(
        rows.iter().map(|row| row_text(row)).collect::<Vec<_>>(),
        ["Root/Private/Visible"]
    );
    assert!(rows[0].contains("<span class=\"namespace-unavailable\">Private</span>"));
    assert!(rows[0].contains(&format!(
        "href=\"{}\">Visible</a>",
        route("Root/Private/Visible")
    )));
    assert!(!has_route(&f.0, "Root/Private"));
    assert!(!has_route(&f.0, "Root/Excluded"));
    assert!(!has_route(&f.0, "Hidden"));
    for secret in ["PRIVATE_SENTINEL", "EXCLUDED_SENTINEL", "HIDDEN_SENTINEL"] {
        assert!(!f.all_html().contains(secret));
    }
}

#[test]
fn selected_alias_suppresses_synthetic_prefix_without_changing_lookup() {
    let f = Fixture::new();
    f.write("Owner.md", "alias:: Branch\n- OWNER_BODY\n");
    f.write("Branch___Leaf.md", "- [[Branch]]\n- {{embed [[Branch]]}}\n");
    let report = f.publish(&["Owner", "Branch/Leaf"], &[], Some("Branch"));
    assert_eq!(report.pages, 2);
    assert!(!has_route(&f.0, "Branch"));
    assert!(
        f.html("Branch/Leaf")
            .contains(&format!("href=\"{}\">Branch</a>", route("Owner")))
    );
    assert!(f.html("Branch/Leaf").contains("OWNER_BODY"));
    assert!(
        fs::read_to_string(f.0.join("site/index.html"))
            .unwrap()
            .contains("OWNER_BODY")
    );
    assert!(!f.index(true).contains(">Branch</a>"));
}

#[test]
fn normalized_prefixes_share_deterministic_spelling_and_source_routes_win() {
    let f = Fixture::new();
    f.write("one.md", "title:: Node/One\n- ONE\n");
    f.write("two.md", "title:: node/Two\n- TWO\n");
    f.write("parent.md", "title:: node\n- REAL_PARENT_BODY\n");
    let report = f.publish(&["Node/One", "node"], &[], None);
    assert_eq!(report.pages, 3);
    assert!(!has_route(&f.0, "Node"));
    assert!(f.html("node").contains("REAL_PARENT_BODY"));
    assert_eq!(
        hierarchy_rows(&f.html("node"))
            .iter()
            .map(|row| row_text(row))
            .collect::<Vec<_>>(),
        ["Node/One", "node/Two"]
    );
    assert!(
        f.html("Node/One")
            .contains(&format!("href=\"{}\">node</a>", route("node")))
    );

    let g = Fixture::new();
    g.write("one.md", "title:: Café/One\n- ONE\n");
    g.write("two.md", "title:: Cafe\u{301}/Two\n- TWO\n");
    let report = g.publish(&["Café", "Cafe\u{301}"], &[], None);
    assert_eq!(report.pages, 2);
    assert!(has_route(&g.0, "Cafe\u{301}"));
    assert!(!has_route(&g.0, "Café"));
    assert_eq!(
        g.index(false)
            .split("<main>")
            .nth(1)
            .unwrap()
            .matches("<li><a")
            .count(),
        3
    );
    let parent = g.html("Cafe\u{301}");
    let rows = hierarchy_rows(&parent);
    assert_eq!(rows.len(), 2);
    assert!(rows[0].contains(&route("Café/One")));
    assert!(rows[1].contains(&route("Cafe\u{301}/Two")));
}

#[test]
fn escaped_hierarchy_keeps_homepage_media_and_generated_homepage_is_unavailable() {
    let f = Fixture::new();
    f.write(
        "leaf.md",
        "title:: R&D/<tag>/Leaf\n- # Notes\n- ![Recording](https://example.com/full.mp3)\n- {{embed [[Include/Details]]}}\n",
    );
    f.write("Include___Details.md", "- EMBEDDED_DETAILS\n");
    let report = f.publish(&["R&D", "Include/Details"], &[], Some("R&D/<tag>/Leaf"));
    assert_eq!(report.pages, 2);
    let output = f.all_html();
    assert!(!output.contains("<tag>"));
    assert!(output.contains("R&amp;D/&lt;tag&gt;"));
    let homepage = fs::read_to_string(f.0.join("site/index.html")).unwrap();
    assert!(homepage.contains("<audio controls"));
    assert!(homepage.contains("<h1>Notes</h1>"));
    assert!(homepage.contains("EMBEDDED_DETAILS"));
    let main = |html: &str| html[html.find("<main>").unwrap()..].to_owned();
    assert_eq!(main(&homepage), main(&f.html("R&D/<tag>/Leaf")));
    assert!(!homepage.contains("aria-label=\"Home\""));
    let g = Fixture::new();
    g.write("Root___Leaf.md", "- BODY\n");
    let result = publish_site_with_options(
        g.0.join("garden"),
        g.0.join("site"),
        FilenameFormat::TripleLowbar,
        &NamespaceSelection::new(&["Root/Leaf"], &[]).unwrap(),
        &SiteOptions {
            homepage: Some("Root".to_owned()),
            ..SiteOptions::default()
        },
    );
    assert!(result.is_err());
    assert!(!g.0.join("site").exists());
}

#[test]
fn generated_spelling_and_output_do_not_depend_on_source_filename_order() {
    let f = Fixture::new();
    let g = Fixture::new();
    for (fixture, first, second) in [
        (&f, "title:: Tree/B\n- B\n", "title:: tree/A\n- A\n"),
        (&g, "title:: tree/A\n- A\n", "title:: Tree/B\n- B\n"),
    ] {
        fixture.write("first.md", first);
        fixture.write("second.md", second);
        assert_eq!(fixture.publish(&["Tree", "tree"], &[], None).pages, 2);
        assert!(has_route(&fixture.0, "Tree"));
        assert!(!has_route(&fixture.0, "tree"));
    }
    assert_eq!(files(&f.0.join("site")), files(&g.0.join("site")));
}

#[test]
fn bottom_hierarchy_contains_all_flat_full_paths_and_independent_segment_links() {
    let f = Fixture::new();
    for title in ["A/B/C", "A/B/C/D", "A/B/C/E", "A/B/C/E/F", "A/B/Cousin"] {
        f.write(
            &format!("{}.md", title.replace('/', "___")),
            "- SOURCE_BODY\n",
        );
    }
    let report = f.publish(&["A"], &[], None);
    assert_eq!(report.pages, 5);
    let page = f.html("A/B/C");
    assert!(!page.contains("namespace-breadcrumbs"));
    assert!(page.contains("<details class=\"references-section\" open><summary><span class=\"ref-chevron\"></span><h2>Hierarchy</h2></summary><ul class=\"namespaces\">"));
    assert!(
        page.find("SOURCE_BODY").unwrap() < page.find("<section class=\"page-hierarchy ").unwrap()
    );
    let rows = hierarchy_rows(&page);
    assert_eq!(
        rows.iter().map(|row| row_text(row)).collect::<Vec<_>>(),
        ["A/B/C/D", "A/B/C/E", "A/B/C/E/F"]
    );
    for (row, title) in rows.iter().zip(["A/B/C/D", "A/B/C/E", "A/B/C/E/F"]) {
        let parts = title.split('/').collect::<Vec<_>>();
        assert_eq!(row.matches("<a ").count(), parts.len());
        for (position, label) in parts.iter().enumerate() {
            assert!(row.contains(&format!(
                "href=\"{}\">{label}</a>",
                route(&parts[..=position].join("/"))
            )));
        }
        assert!(!row.contains("<ul>"));
    }
    assert!(rows.iter().all(|row| !row.contains("aria-current")));
    assert!(
        rows.iter().all(
            |row| row.matches("class=\"page-reference\"").count() == row.matches("<a ").count()
        )
    );
    assert!(!page.contains(&route("A/B/Cousin")));
    let leaf = f.html("A/B/C/E/F");
    let rows = hierarchy_rows(&leaf);
    assert_eq!(rows.len(), 1);
    assert_eq!(row_text(rows[0]), "A/B/C/E");
    assert_eq!(rows[0].matches("<a ").count(), 4);
    assert!(leaf.contains("overflow-wrap:anywhere"));
    assert!(leaf.contains("summary:focus-visible"));
}

#[test]
fn alias_and_private_gaps_stay_plain_across_all_descendant_levels() {
    let f = Fixture::new();
    f.write("Root.md", "- ROOT\n");
    f.write("Root___Private.md", "public:: false\n- PRIVATE_SENTINEL\n");
    f.write("Owner.md", "alias:: Root/Private/Alias\n- OWNER\n");
    f.write(
        "deep.md",
        "title:: Root/Private/Alias/Leaf/Deeper\n- DEEP_BODY\n",
    );
    f.publish(&["Root", "Owner"], &[], None);
    let root = f.html("Root");
    let rows = hierarchy_rows(&root);
    assert_eq!(
        rows.iter().map(|row| row_text(row)).collect::<Vec<_>>(),
        ["Root/Private/Alias/Leaf/Deeper"]
    );
    for row in &rows {
        assert!(row.contains("namespace-unavailable\">Private</span>"));
        assert!(row.contains("namespace-unavailable\">Alias</span>"));
        assert!(!row.contains(&route("Owner")));
    }
    assert!(!has_route(&f.0, "Root/Private"));
    assert!(!has_route(&f.0, "Root/Private/Alias"));
    assert!(has_route(&f.0, "Root/Private/Alias/Leaf"));
    assert!(rows[0].contains(&format!(
        "href=\"{}\">Leaf</a>",
        route("Root/Private/Alias/Leaf")
    )));
    assert!(!f.all_html().contains("PRIVATE_SENTINEL"));
}

#[test]
fn stored_rows_skip_generated_prefixes_but_keep_every_segment_route_and_stored_parent() {
    let f = Fixture::new();
    for title in ["Root", "Root/Stored", "Root/Stored/Auto/Deep/Leaf"] {
        f.write(
            &format!("{}.md", title.replace('/', "___")),
            "- PUBLIC_BODY\n",
        );
    }
    let source = fs::read(f.0.join("garden/pages/Root___Stored___Auto___Deep___Leaf.md")).unwrap();
    assert_eq!(f.publish(&["Root"], &[], None).pages, 3);
    let root = f.html("Root");
    let rows = hierarchy_rows(&root);
    assert_eq!(
        rows.iter().map(|row| row_text(row)).collect::<Vec<_>>(),
        ["Root/Stored", "Root/Stored/Auto/Deep/Leaf"]
    );
    let current = f.html("Root/Stored/Auto");
    let generated_rows = hierarchy_rows(&current);
    assert_eq!(
        generated_rows
            .iter()
            .map(|row| row_text(row))
            .collect::<Vec<_>>(),
        ["Root/Stored/Auto/Deep/Leaf"]
    );
    assert!(!current.contains("<span class=\"page-kind\">Namespace</span></li>"));
    assert!(generated_rows[0].contains("class=\"namespace-link\""));
    for title in ["Root/Stored/Auto", "Root/Stored/Auto/Deep"] {
        assert!(has_route(&f.0, title));
        assert!(f.index(false).contains(&format!(">{title}</a>")));
        assert!(rows[1].contains(&format!("href=\"{}\"", route(title))));
    }
    assert_eq!(
        fs::read(f.0.join("garden/pages/Root___Stored___Auto___Deep___Leaf.md")).unwrap(),
        source
    );
}

#[test]
fn unselected_stored_prefixes_and_absent_prefixes_have_identical_hierarchy_rows() {
    let with_source = Fixture::new();
    let absent = Fixture::new();
    with_source.write("Root.md", "- UNSELECTED_SENTINEL\n");
    with_source.write("Root___Generated.md", "- ALSO_UNSELECTED\n");
    for f in [&with_source, &absent] {
        f.write("Root___Generated___Stored.md", "- SELECTED\n");
        f.publish(&["Root/Generated/Stored"], &[], None);
    }
    for title in ["Root", "Root/Generated", "Root/Generated/Stored"] {
        let left = with_source.html(title);
        let right = absent.html(title);
        assert_eq!(hierarchy_rows(&left), hierarchy_rows(&right));
        assert!(!left.contains("UNSELECTED_SENTINEL"));
        assert!(!left.contains("ALSO_UNSELECTED"));
    }
}

#[test]
fn plain_pages_have_no_section_and_alias_pages_take_their_namespace() {
    let f = Fixture::new();
    f.write("Abbreviation.md", "- PLAIN\n");
    f.write(
        "AGI.md",
        "alias:: General AI, AI/General/Intelligence\n- AGI_BODY\n",
    );
    f.write("AI___General.md", "- GENERAL\n");
    f.publish(&["Abbreviation", "AGI", "AI"], &[], None);
    assert!(!f.html("Abbreviation").contains("aria-label=\"Hierarchy\""));
    let agi = f.html("AGI");
    let rows = hierarchy_rows(&agi);
    assert_eq!(
        rows.iter().map(|row| row_text(row)).collect::<Vec<_>>(),
        ["AI/General"]
    );
    assert!(rows[0].contains(&format!("href=\"{}\">General</a>", route("AI/General"))));
}

#[test]
fn long_hierarchies_continue_in_fragment_files() {
    let f = Fixture::new();
    f.write("Big.md", "- ROOT\n");
    for index in 0..620 {
        f.write(&format!("Big___{index:04}.md"), "- CHILD\n");
    }
    f.publish(&["Big"], &[], None);
    let page = f.html("Big");
    assert_eq!(hierarchy_rows(&page).len(), 100);
    let first = format!(
        "_refs/{}-hierarchy-1.html",
        route("Big").trim_end_matches(".html")
    );
    assert!(page.contains(&format!("data-hierarchy-src=\"{first}\"")));
    assert!(page.contains(&format!(
        "<noscript><a href=\"{first}\">Show more of the hierarchy</a></noscript>"
    )));
    let fragment = fs::read_to_string(f.0.join("site").join(&first)).unwrap();
    assert_eq!(fragment.matches("<li>").count(), 500);
    assert!(fragment.contains(&format!("href=\"{}\">0100</a>", route("Big/0100"))));
    assert!(fragment.contains("rel=\"next\""));
    let second = first.replace("-1.html", "-2.html");
    let fragment = fs::read_to_string(f.0.join("site").join(second)).unwrap();
    assert_eq!(fragment.matches("<li>").count(), 20);
    assert!(fragment.contains(">0619</a>"));
}
