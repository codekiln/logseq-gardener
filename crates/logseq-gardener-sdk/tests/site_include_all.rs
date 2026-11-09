//! A selection that includes every page publishes the whole garden, less exclusions and
//! withheld pages.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use logseq_gardener_sdk::page_titles::FilenameFormat;
use logseq_gardener_sdk::publishing::NamespaceSelection;
use logseq_gardener_sdk::site::{SiteOptions, publish_site_with_options};
use serde_json::Value;

static NEXT: AtomicUsize = AtomicUsize::new(0);

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "lsg-include-all-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).unwrap();
        let fixture = Self(root);
        fixture.write("pages/Notes.md", "- NOTES_SENTINEL\n");
        fixture.write("pages/Projects___Alpha.md", "- ALPHA_SENTINEL\n");
        fixture.write(
            "pages/Solvespace.md",
            "alias:: [[SolveSpace]]\n\n- SOLVESPACE_SENTINEL\n",
        );
        fixture.write("pages/Secret.md", "public:: false\n\n- PRIVATE_SENTINEL\n");
        fixture.write("journals/2026_11_08.md", "- JOURNAL_SENTINEL\n");
        fixture
    }

    fn write(&self, name: &str, text: &str) {
        let path = self.0.join("garden").join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    /// Publish with every page selected, less `excluded`; return the published titles.
    fn publish(&self, name: &str, excluded: &[&str]) -> (BTreeSet<String>, PathBuf) {
        let output = self.0.join(name);
        let report = publish_site_with_options(
            self.0.join("garden"),
            &output,
            FilenameFormat::TripleLowbar,
            &NamespaceSelection::new(&[], excluded)
                .unwrap()
                .including_all(),
            &SiteOptions {
                journals: true,
                ..SiteOptions::default()
            },
        )
        .unwrap();
        assert_eq!(report.withheld_pages, 1, "Secret is withheld");
        (titles(&output), output)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// Titles of the pages and journals in `lsg-site.json`.
fn titles(output: &Path) -> BTreeSet<String> {
    let site: Value =
        serde_json::from_slice(&fs::read(output.join("lsg-site.json")).unwrap()).unwrap();
    site["pages"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|page| matches!(page["kind"].as_str(), Some("page" | "journal")))
        .map(|page| page["title"].as_str().unwrap().to_owned())
        .collect()
}

/// Whether any file under `root` contains `needle`.
fn contains(root: &Path, needle: &str) -> bool {
    fs::read_dir(root).unwrap().any(|entry| {
        let path = entry.unwrap().path();
        if path.is_dir() {
            contains(&path, needle)
        } else {
            String::from_utf8_lossy(&fs::read(&path).unwrap()).contains(needle)
        }
    })
}

#[test]
fn including_all_publishes_alias_titled_pages_and_journals_but_not_withheld_pages() {
    let fixture = Fixture::new();
    let (titles, output) = fixture.publish("all", &[]);
    for title in ["Notes", "Projects/Alpha", "SolveSpace", "Nov 8th, 2026"] {
        assert!(titles.contains(title), "{title} in {titles:?}");
    }
    assert!(!titles.contains("Secret"), "{titles:?}");
    assert!(!contains(&output, "PRIVATE_SENTINEL"));
}

#[test]
fn exclusion_still_wins_when_including_all() {
    let fixture = Fixture::new();
    let (titles, output) = fixture.publish("excluded", &["Projects"]);
    assert!(!titles.contains("Projects/Alpha"), "{titles:?}");
    assert!(!contains(&output, "ALPHA_SENTINEL"));
    for title in ["Notes", "SolveSpace", "Nov 8th, 2026"] {
        assert!(titles.contains(title), "{title} in {titles:?}");
    }
}
