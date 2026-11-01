//! Page relationships the SDK derives from the fixture notes, compared with the
//! relationships Logseq OG recorded for the same notes.
//!
//! `tests/fixtures/og-relationships/README.md` explains where the expected results come
//! from, what they leave out and how to regenerate them.

mod relationships;

use std::path::{Path, PathBuf};

use logseq_gardener_sdk::page_lookup::{MatchedBy, PageCatalog};
use logseq_gardener_sdk::page_titles::FilenameFormat;
use relationships::{AliasName, FilelessPage, StoredPage, derive};
use serde::Deserialize;

#[derive(Deserialize)]
struct Expected {
    configurations: Vec<Configuration>,
}

#[derive(Deserialize)]
struct Configuration {
    name: String,
    filename_format: String,
    pages: Vec<StoredPage>,
    aliases: Vec<AliasName>,
    fileless: Vec<FilelessPage>,
}

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/og-relationships")
}

fn expected() -> Expected {
    let text = std::fs::read_to_string(fixtures().join("expected.json")).unwrap();
    serde_json::from_str(&text).unwrap()
}

fn format(name: &str) -> FilenameFormat {
    match name {
        "legacy" => FilenameFormat::Legacy,
        "triple-lowbar" => FilenameFormat::TripleLowbar,
        other => panic!("unknown filename format {other}"),
    }
}

fn configuration(name: &str) -> Configuration {
    expected()
        .configurations
        .into_iter()
        .find(|configuration| configuration.name == name)
        .unwrap_or_else(|| panic!("expected.json has no {name} configuration"))
}

fn assert_matches_logseq(name: &str) {
    let expected = configuration(name);
    let format = format(&expected.filename_format);
    let root = fixtures().join("garden");
    let derived = derive(&root, format);

    // Each page's title finds that page alone, by title.
    let (catalog, _) = PageCatalog::load(&root, format).unwrap();
    for page in &derived.pages {
        let found = catalog.find(&page.title);
        assert_eq!(found.len(), 1, "{} names one page", page.title);
        assert_eq!(found[0].relative_path, Path::new(&page.file));
        assert_eq!(found[0].title, page.title);
        assert_eq!(found[0].matched_by, MatchedBy::Title);
    }
    assert_eq!(catalog.len(), derived.pages.len());

    assert_eq!(derived.pages, expected.pages, "{name}: stored pages");
    assert_eq!(derived.aliases, expected.aliases, "{name}: alias names");
    assert_eq!(
        derived.fileless, expected.fileless,
        "{name}: pages without a file"
    );
}

#[test]
fn default_filename_format_matches_logseq() {
    assert_matches_logseq("default");
}

#[test]
fn triple_lowbar_filename_format_matches_logseq() {
    assert_matches_logseq("triple-lowbar");
}

#[test]
fn expected_results_cover_both_configurations() {
    let names: Vec<_> = expected()
        .configurations
        .into_iter()
        .map(|configuration| configuration.name)
        .collect();
    assert_eq!(names, ["default", "triple-lowbar"]);
}
