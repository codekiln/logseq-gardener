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

/// When a page lists another page that has a file as its alias, Logseq records the alias on
/// both pages: the named page gets the listing page and the listing's other names
/// (issue #453). `scripts/logseq-parser/check.py` agrees on this garden.
#[test]
fn an_alias_is_recorded_on_the_page_it_names_too() {
    let root = fixtures().with_file_name("alias-of-another-page");
    let derived = derive(&root, FilenameFormat::TripleLowbar);
    let aliases: Vec<(&str, Vec<&str>)> = derived
        .pages
        .iter()
        .map(|page| {
            (
                page.title.as_str(),
                page.aliases.iter().map(String::as_str).collect(),
            )
        })
        .collect();
    assert_eq!(
        aliases,
        [
            (
                "Homebrew",
                vec!["bottles", "brew", "formulae", "package manager"]
            ),
            ("Package Manager", vec!["bottles", "brew", "homebrew"]),
            ("brew", vec!["bottles", "homebrew", "package manager"]),
        ]
    );
    assert_eq!(
        derived.aliases,
        [
            AliasName {
                name: "bottles".into(),
                title: "Bottles".into(),
                files: vec![
                    "pages/Homebrew.md".into(),
                    "pages/Package Manager.md".into(),
                    "pages/brew.md".into(),
                ],
            },
            AliasName {
                name: "formulae".into(),
                title: "Formulae".into(),
                files: vec!["pages/Homebrew.md".into()],
            },
        ]
    );
}

#[test]
fn an_alias_in_a_namespace_names_its_namespace_parents() {
    // `AGI` lists `AI/General/Intelligence` as an alias, and `Brew` lists `CLI/Tool/Mac`,
    // whose parent `CLI` has a file. Logseq creates the other parents as pages without a file.
    let root = fixtures().with_file_name("alias-namespace-parents");
    let derived = derive(&root, FilenameFormat::TripleLowbar);
    assert_eq!(
        derived.fileless,
        [
            FilelessPage {
                name: "ai".into(),
                title: "AI".into(),
                namespace: None,
            },
            FilelessPage {
                name: "ai/general".into(),
                title: "AI/General".into(),
                namespace: Some("ai".into()),
            },
            FilelessPage {
                name: "cli/tool".into(),
                title: "CLI/Tool".into(),
                namespace: Some("cli".into()),
            },
        ]
    );
}

#[test]
fn a_boundary_slash_is_not_part_of_a_page_name() {
    // `Notes` links to `a/b/`. Logseq drops the trailing slash: the page is `a/b`, and its
    // namespace parent `a` is a page without a file too,
    // titled `A` as Logseq spells its built-in priority page.
    let root = fixtures().with_file_name("trailing-slash-name");
    let derived = derive(&root, FilenameFormat::TripleLowbar);
    assert_eq!(
        derived.fileless,
        [
            FilelessPage {
                name: "a".into(),
                title: "A".into(),
                namespace: None,
            },
            FilelessPage {
                name: "a/b".into(),
                title: "a/b".into(),
                namespace: Some("a".into()),
            },
        ]
    );
}

#[test]
fn a_date_name_takes_the_weekday_the_date_has() {
    // `Notes` links to `1967-09-03 Fri` under the journal title format `yyyy-MM-dd EEE`.
    // Logseq reads the date and spells it with its real weekday: 1967-09-03 was a Sunday, so
    // the page without a file is `1967-09-03 Sun` and no page is named `1967-09-03 Fri`. The
    // property name `date-created` is a page without a file too.
    let root = fixtures().with_file_name("wrong-weekday-date-name");
    let derived = derive(&root, FilenameFormat::TripleLowbar);
    assert_eq!(
        derived.fileless,
        [
            FilelessPage {
                name: "1967-09-03 sun".into(),
                title: "1967-09-03 Sun".into(),
                namespace: None,
            },
            FilelessPage {
                name: "date-created".into(),
                title: "date-created".into(),
                namespace: None,
            },
        ]
    );
}

#[test]
fn an_alias_written_twice_takes_its_last_spelling() {
    // `Brew` writes `alias:: [[homebrew]], [[Homebrew]]`. Logseq spells the alias name with the
    // last spelling, `Homebrew`; `scripts/logseq-parser/check.py` agrees on this garden
    // (issue #509).
    let root = fixtures().with_file_name("alias-spelling-case");
    let derived = derive(&root, FilenameFormat::TripleLowbar);
    assert_eq!(
        derived.aliases,
        [AliasName {
            name: "homebrew".into(),
            title: "Homebrew".into(),
            files: vec!["pages/Brew.md".into()],
        }]
    );
}

#[test]
fn an_alias_takes_the_spelling_of_the_first_file_that_refers_to_it() {
    // `Run` lists `Runtime` as an alias. `Notes` refers to `[[runtime]]` and `Other` to
    // `[[RUNTIME]]`; Logseq reads `Notes` first, so the alias name is spelled `runtime`. `Gear`
    // lists `Gadget` and refers to `[[gadget]]` and then `[[GADGET]]`; within a file the last
    // block wins, so it is spelled `GADGET`. `Tool` lists `Widget`, and `Meet`, read before it,
    // has the property `widget::`, which refers to the page in lower case. `scripts/logseq-parser/check.py`
    // agrees on this garden (issue #515).
    let root = fixtures().with_file_name("alias-spelling-reference");
    let derived = derive(&root, FilenameFormat::TripleLowbar);
    assert_eq!(
        derived.aliases,
        [
            AliasName {
                name: "gadget".into(),
                title: "GADGET".into(),
                files: vec!["pages/Gear.md".into()],
            },
            AliasName {
                name: "runtime".into(),
                title: "runtime".into(),
                files: vec!["pages/Run.md".into()],
            },
            AliasName {
                name: "widget".into(),
                title: "widget".into(),
                files: vec!["pages/Tool.md".into()],
            },
        ]
    );
}
