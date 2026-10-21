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
        fs::read_dir(self.0.join("site"))
            .unwrap()
            .map(|entry| fs::read_to_string(entry.unwrap().path()).unwrap())
            .collect()
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
        .fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
            (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
        });
    format!("p-{hash:016x}.html")
}
fn has_route(root: &Path, title: &str) -> bool {
    root.join("site").join(route(title)).is_file()
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
    assert!(leaf.contains("<main><nav class=\"namespace-breadcrumbs\""));
    assert!(leaf.contains("aria-current=\"page\">Preset"));
    assert!(leaf.contains("LEAF_NOTES"));
    assert!(leaf.contains("class=\"unresolved\">Instruments/Synth"));
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.message.contains("Instruments/Synth"))
    );
    assert!(!leaf.contains("aria-label=\"Hierarchy\""));
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
    assert!(leaf.contains("<li><span>Secret</span></li>"));
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
    assert!(f.html("Root").contains(&format!(
        "href=\"{}\">Private/Visible</a>",
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
    assert_eq!(g.index(false).matches("<li><a href=").count(), 3);
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
    assert_eq!(homepage, f.html("R&D/<tag>/Leaf"));
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
    let contents = |fixture: &Fixture| {
        fs::read_dir(fixture.0.join("site"))
            .unwrap()
            .map(|entry| {
                let entry = entry.unwrap();
                (entry.file_name(), fs::read(entry.path()).unwrap())
            })
            .collect::<std::collections::BTreeMap<_, _>>()
    };
    assert_eq!(contents(&f), contents(&g));
}
