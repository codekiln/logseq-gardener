//! Page relationships the SDK derives from the fixture notes, compared with the
//! relationships Logseq OG recorded for the same notes.
//!
//! `tests/fixtures/og-relationships/README.md` explains where the expected results come
//! from, what they leave out and how to regenerate them.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use logseq_gardener_sdk::garden::{DocumentKind, GardenDocument, load_garden};
use logseq_gardener_sdk::graph::{GraphIndex, GraphSettings, GraphSource};
use logseq_gardener_sdk::journals::{JournalFormats, journal_identity, parse_journal_reference};
use logseq_gardener_sdk::page_lookup::{MatchedBy, PageCatalog};
use logseq_gardener_sdk::page_names::{lookup_key, page_alias_spellings, page_aliases};
use logseq_gardener_sdk::page_titles::{FilenameFormat, page_title};
use logseq_gardener_sdk::references::{Contribution, PageEntry, ReferenceIndex};
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

#[derive(Debug, Deserialize, PartialEq)]
struct StoredPage {
    name: String,
    title: String,
    file: String,
    aliases: Vec<String>,
    namespace: Option<String>,
    journal_day: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq)]
struct AliasName {
    name: String,
    title: String,
    files: Vec<String>,
}

#[derive(Debug, Deserialize, PartialEq)]
struct FilelessPage {
    name: String,
    title: String,
    namespace: Option<String>,
}

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/og-relationships")
}

fn expected() -> Expected {
    let text = std::fs::read_to_string(fixtures().join("expected.json")).unwrap();
    serde_json::from_str(&text).unwrap()
}

fn path(document: &GardenDocument) -> String {
    document.relative_path.to_string_lossy().replace('\\', "/")
}

/// What the SDK derives for one filename format, in the shape of the expected results.
struct Derived {
    pages: Vec<StoredPage>,
    aliases: Vec<AliasName>,
    fileless: Vec<FilelessPage>,
}

fn derive(format: FilenameFormat, expected: &Configuration) -> Derived {
    let root = fixtures().join("garden");
    let garden = load_garden(&root).unwrap();
    let (catalog, diagnostics) = PageCatalog::load(&root, format).unwrap();
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    let formats = JournalFormats::default();

    let mut titles = BTreeMap::new();
    let mut journals = BTreeSet::new();
    let mut days = BTreeMap::new();
    for document in &garden.documents {
        let title = if document.kind == DocumentKind::Journal {
            let identity = journal_identity(document, &formats).unwrap();
            journals.insert(path(document));
            days.insert(
                path(document),
                format!(
                    "{:04}-{:02}-{:02}",
                    identity.date.year(),
                    identity.date.month(),
                    identity.date.day()
                ),
            );
            identity.title
        } else {
            page_title(document, format).unwrap()
        };
        titles.insert(path(document), title);
    }

    // A written name names a page when exactly one page answers to it, as published links do.
    let resolve = |name: &str| match catalog.find(name).as_slice() {
        [page] => Some(page.relative_path.to_string_lossy().replace('\\', "/")),
        _ => None,
    };
    let index = ReferenceIndex::build(
        garden.documents.iter().map(|document| PageEntry {
            id: path(document),
            title: titles[&path(document)].clone(),
            aliases: page_alias_spellings(document, &titles[&path(document)]),
        }),
        garden
            .documents
            .iter()
            .map(|document| Contribution::from_document(path(document), document, &resolve, None))
            .collect(),
    );
    let graph_index = GraphIndex::new(&GraphSource {
        index: &index,
        journals: &journals,
        hidden: &BTreeSet::new(),
        is_journal_name: &|name| parse_journal_reference(name, &formats).is_some(),
    });
    let namespace = |title: &str| graph_index.namespace_parent(title).map(lookup_key);

    let mut pages = Vec::new();
    for document in &garden.documents {
        let file = path(document);
        let title = titles[&file].clone();
        let found = catalog.find(&title);
        assert_eq!(found.len(), 1, "{title} names one page");
        assert_eq!(found[0].relative_path, document.relative_path);
        assert_eq!(found[0].title, title);
        assert_eq!(found[0].matched_by, MatchedBy::Title);
        pages.push(StoredPage {
            name: lookup_key(&title),
            aliases: page_aliases(document, &title),
            namespace: namespace(&title),
            journal_day: days.get(&file).cloned(),
            title,
            file,
        });
    }
    assert_eq!(catalog.len(), pages.len());

    // Each alias name Logseq keeps as a separate page finds the files that declare it.
    let aliases = expected
        .aliases
        .iter()
        .map(|alias| {
            let found = catalog.find(&alias.title);
            assert!(
                found.iter().all(|page| page.matched_by == MatchedBy::Alias),
                "{} matches only aliases",
                alias.title
            );
            AliasName {
                name: lookup_key(&alias.title),
                title: alias.title.clone(),
                files: found
                    .iter()
                    .map(|page| page.relative_path.to_string_lossy().replace('\\', "/"))
                    .collect(),
            }
        })
        .collect();

    // Names without a file: graph nodes without a page that no catalog entry answers to.
    // An ambiguous alias is such a node too; the alias comparison covers it.
    let graph = graph_index.graph(GraphSettings::everything(true));
    let mut fileless: Vec<FilelessPage> = graph
        .nodes
        .iter()
        .filter(|node| node.page.is_none() && catalog.find(&node.title).is_empty())
        .map(|node| FilelessPage {
            name: lookup_key(&node.title),
            title: node.title.clone(),
            namespace: namespace(&node.title),
        })
        .collect();
    fileless.sort_by(|a, b| a.name.cmp(&b.name));

    pages.sort_by(|a, b| a.file.cmp(&b.file));
    Derived {
        pages,
        aliases,
        fileless,
    }
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
    let derived = derive(format(&expected.filename_format), &expected);
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
