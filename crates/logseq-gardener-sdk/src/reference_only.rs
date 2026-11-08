//! The pages that exist only because published blocks name them.
//!
//! Logseq makes a page for every name a block links, tags or lists in a page-valued
//! property, and for each namespace parent of such a name, whether or not a file holds it
//! (`with-page-refs` in Logseq's `deps/graph-parser/src/logseq/graph_parser/block.cljs`).
//! [`find`] returns those names for a publication: the names that published blocks write and
//! that no page with a file, alias, journal or generated namespace page answers to. A name
//! that a withheld or unselected file has as its title, alias or date is left out, so a reader
//! never sees a page for a name only a private file holds.
//!
//! [`crate::publication::select`] gives each name a route and adds it to the page-name index,
//! so the CLI, the language server and the publisher resolve `[[Deno Deploy]]` alike. The
//! reference index answers what refers to each page, as for a page with a file.
//!
//! ```
//! use logseq_gardener_sdk::garden::{DocumentKind, GardenDocument};
//! use logseq_gardener_sdk::journals::JournalFormats;
//! use logseq_gardener_sdk::reference_only::find;
//!
//! let deno = GardenDocument {
//!     relative_path: "pages/Deno.md".into(),
//!     kind: DocumentKind::Page,
//!     source: "- Deploy with [[Deno Deploy]]\n- TODO try [[Deno]]\n".to_owned(),
//!     parsed: lsdoc::parse_to_projection("- Deploy with [[Deno Deploy]]\n- TODO try [[Deno]]\n"),
//!     title_spellings: Default::default(),
//! };
//! let pages = find(
//!     &[&deno],
//!     &|name| name.eq_ignore_ascii_case("deno"),
//!     &JournalFormats::default(),
//!     &[],
//! );
//! // `Deno` has a file and the task marker `TODO` makes no page.
//! assert_eq!(pages.len(), 1);
//! assert_eq!(pages[0].title, "Deno Deploy");
//! ```

use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::PathBuf;

use crate::block_ids::parse_order_of;
use crate::garden::GardenDocument;
use crate::journals::{JournalDate, JournalFormats, parse_journal_reference};
use crate::page_names::{built_in_page_title, lookup_key, reference_key};
use crate::title_spellings::{for_each_written_name, last_spellings};

/// A page that exists only because published blocks name it, before it has a route.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceOnlyName {
    /// The [`reference_key`] of the title.
    pub key: String,
    /// The title Logseq gives the page: the spelling of the first reference it reads.
    pub title: String,
    /// The date, when the name spells one; the title is then the journal title.
    pub journal: Option<JournalDate>,
    /// The first published file, in Logseq's parse order, that writes the name.
    pub source: PathBuf,
}

/// A published page that exists only because published blocks name it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceOnlyPage {
    /// The title the page shows.
    pub title: String,
    /// The generated file, relative to the output root.
    pub route: String,
    /// The date, for a reference-only journal.
    pub journal: Option<JournalDate>,
}

/// The names that the `published` documents write as page references, tags or page-valued
/// property values, with their namespace parents, for which `answered` is false, sorted by
/// key. `answered` says whether a page with a file, an alias, a journal or a generated
/// namespace page answers to a name, published or not. Task markers and priorities add no
/// name, and neither do block UUIDs and asset paths, which Logseq makes no page for. A name a
/// file writes only in the values of `hidden` properties (`:block-hidden-properties`, lower-cased)
/// adds no name either, so no page shows what readers cannot see.
///
/// A name takes the spelling that the first file Logseq reads writes it in, and within that
/// file the spelling Logseq's parser keeps (see [`crate::title_spellings`]); one of the pages
/// Logseq creates in every graph keeps Logseq's spelling. A name that spells a date takes the
/// journal title format's title for that date.
pub fn find(
    published: &[&GardenDocument],
    answered: &dyn Fn(&str) -> bool,
    formats: &JournalFormats,
    hidden: &[String],
) -> Vec<ReferenceOnlyName> {
    // Per key: the first file that writes it, its first spelling there, the date it spells,
    // and whether that file writes it in another spelling too.
    let mut found: HashMap<String, (usize, String, Option<JournalDate>, bool)> = HashMap::new();
    let mut checked: HashMap<String, bool> = HashMap::new();
    for index in parse_order_of(published.iter().copied()) {
        let only_hidden = names_only_in_hidden_values(published[index], hidden);
        for_each_written_name(published[index], false, &mut |name| {
            if only_hidden.contains(&reference_key(name)) {
                return;
            }
            let missing = *checked
                .entry(name.to_owned())
                .or_insert_with(|| !answered(name) && !crate::graph::uuid_or_asset(name));
            if !missing {
                return;
            }
            let journal = parse_journal_reference(name, formats);
            let key = match journal {
                Some(date) => lookup_key(&formats.title.format(date)),
                None => reference_key(name),
            };
            if key.is_empty() {
                return;
            }
            match found.get_mut(&key) {
                Some((first, spelling, _, several)) => {
                    if *first == index && spelling != name {
                        *several = true;
                    }
                }
                None => {
                    found.insert(key, (index, name.to_owned(), journal, false));
                }
            }
        });
    }
    // A file that writes a name in several spellings: Logseq's parser keeps one of them.
    let mut respell: BTreeMap<usize, HashSet<String>> = BTreeMap::new();
    for (key, (index, _, journal, several)) in &found {
        if *several && journal.is_none() && built_in_page_title(key).is_none() {
            respell.entry(*index).or_default().insert(key.clone());
        }
    }
    let mut respelled: HashMap<String, String> = HashMap::new();
    for (index, keys) in respell {
        let wanted: HashSet<&str> = keys.iter().map(String::as_str).collect();
        respelled.extend(last_spellings(published[index], &wanted, false));
    }
    let mut names: Vec<ReferenceOnlyName> = found
        .into_iter()
        .map(|(key, (index, spelling, journal, _))| {
            let title = match journal {
                Some(date) => formats.title.format(date),
                None => built_in_page_title(&key)
                    .map(str::to_owned)
                    .or_else(|| respelled.get(&key).cloned())
                    .unwrap_or(spelling),
            };
            ReferenceOnlyName {
                key,
                title,
                journal,
                source: published[index].relative_path.clone(),
            }
        })
        .collect();
    names.sort_by(|left, right| left.key.cmp(&right.key));
    names
}

/// The [`reference_key`]s of the names a file writes in the values of `hidden` properties and
/// nowhere else.
fn names_only_in_hidden_values(document: &GardenDocument, hidden: &[String]) -> HashSet<String> {
    if hidden.is_empty() {
        return HashSet::new();
    }
    let mut in_hidden = HashSet::new();
    let mut visible = String::with_capacity(document.source.len());
    for line in document.source.split_inclusive('\n') {
        let property = line
            .trim_start()
            .trim_start_matches("- ")
            .split_once("::")
            .filter(|(key, _)| hidden.contains(&key.trim().to_lowercase()));
        match property {
            Some((_, value)) => {
                in_hidden.extend(lsdoc::refs(value, "markdown").page);
                visible.push('\n');
            }
            None => visible.push_str(line),
        }
    }
    if in_hidden.is_empty() {
        return HashSet::new();
    }
    let shown: HashSet<String> = lsdoc::refs(&visible, "markdown")
        .page
        .iter()
        .map(|name| reference_key(name))
        .collect();
    in_hidden
        .iter()
        .map(|name| reference_key(name))
        .filter(|key| !shown.contains(key))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::garden::DocumentKind;

    fn document(path: &str, source: &str) -> GardenDocument {
        GardenDocument {
            relative_path: path.into(),
            kind: if path.starts_with("journals/") {
                DocumentKind::Journal
            } else {
                DocumentKind::Page
            },
            source: source.to_owned(),
            parsed: lsdoc::parse_to_projection(source),
            title_spellings: BTreeMap::new(),
        }
    }

    fn titles(documents: &[GardenDocument], known: &[&str]) -> Vec<String> {
        let known: Vec<String> = known.iter().map(|name| lookup_key(name)).collect();
        let published: Vec<&GardenDocument> = documents.iter().collect();
        find(
            &published,
            &|name| known.contains(&reference_key(name)),
            &JournalFormats::default(),
            &[],
        )
        .into_iter()
        .map(|name| name.title)
        .collect()
    }

    #[test]
    fn links_tags_and_page_valued_properties_make_pages() {
        let documents = [
            document(
                "pages/Deno.md",
                "- Deploy with [[Deno Deploy]]\n- (pytest issue #10977)\n",
            ),
            document(
                "pages/Sourcegraph.md",
                "tags:: company, [[Search]]\n\n- Code search\n",
            ),
        ];
        assert_eq!(
            titles(&documents, &["Deno", "Sourcegraph", "Search"]),
            ["10977)", "company", "Deno Deploy"]
        );
    }

    #[test]
    fn namespace_parents_make_pages_unless_a_page_answers() {
        let documents = [document(
            "pages/1Password___Dev.md",
            "- See [[1Password/Desktop/Settings/Developer/SSH Agent/Advanced]]\n",
        )];
        assert_eq!(
            titles(
                &documents,
                &["1Password/Dev", "1Password", "1Password/Desktop"]
            ),
            [
                "1Password/Desktop/Settings",
                "1Password/Desktop/Settings/Developer",
                "1Password/Desktop/Settings/Developer/SSH Agent",
                "1Password/Desktop/Settings/Developer/SSH Agent/Advanced",
            ]
        );
    }

    #[test]
    fn task_markers_priorities_uuids_and_assets_make_no_page() {
        let documents = [document(
            "pages/Work.md",
            "- TODO [#A] ship\n- [[6add3ffb-e52b-4e8f-908a-9958186e4dfb]] [[../assets/x.png]]\n",
        )];
        assert!(titles(&documents, &["Work"]).is_empty());
    }

    #[test]
    fn a_written_task_marker_name_makes_a_page_in_logseqs_spelling() {
        let documents = [document("pages/Work.md", "- see [[todo]] and #card\n")];
        assert_eq!(titles(&documents, &["Work"]), ["card", "TODO"]);
    }

    #[test]
    fn the_first_file_logseq_reads_spells_the_title() {
        // Journals come first in Logseq's parse order, newest first, then pages by path.
        let documents = [
            document("pages/a.md", "- [[TypeScript]] [[macOS]]\n"),
            document("journals/2024_01_01.md", "- [[Typescript]]\n"),
            document("journals/2024_01_02.md", "- [[MacOS]]\n"),
        ];
        assert_eq!(
            titles(&documents, &["a", "2024-01-01", "2024-01-02"]),
            ["MacOS", "Typescript"]
        );
    }

    #[test]
    fn several_spellings_in_one_file_follow_logseqs_parser() {
        // Within a file the last block wins; within the leading properties the last name.
        let documents = [document("pages/a.md", "- [[langgraph]]\n- [[LangGraph]]\n")];
        assert_eq!(titles(&documents, &["a"]), ["LangGraph"]);
    }

    #[test]
    fn boundary_slashes_spaces_and_dates() {
        let documents = [document(
            "pages/Mac.md",
            "- (C#/D♭) [[MacOS /Applications Folder]] [[Nov 18th, 2024]] [[2024-11-18]]\n",
        )];
        let published: Vec<&GardenDocument> = documents.iter().collect();
        let names = find(
            &published,
            &|name| lookup_key(name) == "mac",
            &JournalFormats::default(),
            &[],
        );
        let titles: Vec<&str> = names.iter().map(|name| name.title.as_str()).collect();
        assert_eq!(
            titles,
            [
                "D♭)",
                "MacOS ",
                "MacOS /Applications Folder",
                "Nov 18th, 2024"
            ]
        );
        let journal = names.iter().find(|name| name.journal.is_some()).unwrap();
        assert_eq!(journal.title, "Nov 18th, 2024");
        assert_eq!(journal.source, PathBuf::from("pages/Mac.md"));
    }

    #[test]
    fn names_only_hidden_properties_write_make_no_page() {
        let documents = [document(
            "pages/a.md",
            "entity:: [[Hidden Type]], [[Shown Type]]\n\n- [[Shown Type]]\n",
        )];
        let published: Vec<&GardenDocument> = documents.iter().collect();
        let names = find(
            &published,
            &|name| lookup_key(name) == "a",
            &JournalFormats::default(),
            &["entity".to_owned()],
        );
        let titles: Vec<&str> = names.iter().map(|name| name.title.as_str()).collect();
        assert_eq!(titles, ["Shown Type"]);
    }

    #[test]
    fn answered_names_make_no_page() {
        let documents = [document(
            "pages/a.md",
            "- [[Secret]] [[Alias Name]] [[Nov 18th, 2024]]\n",
        )];
        assert!(titles(&documents, &["a", "Secret", "Alias Name", "Nov 18th, 2024"]).is_empty());
    }
}
