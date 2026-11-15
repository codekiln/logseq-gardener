use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};

use super::*;
use crate::page_titles::FilenameFormat;

const FIRST: &str = "66f1a2b0-0000-4000-8000-000000000001";
const SHARED: &str = "66f1a2b0-0000-4000-8000-000000000002";

static NEXT: AtomicUsize = AtomicUsize::new(0);

/// A garden in a temporary folder, removed when the value drops.
struct Garden(PathBuf);

impl Garden {
    fn new(files: &[(&str, &str)]) -> Self {
        let root = std::env::temp_dir().join(format!(
            "lsg-index-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, AtomicOrdering::Relaxed)
        ));
        let garden = Self(root);
        garden.write(
            CONFIG_PATH,
            "{:file/name-format :triple-lowbar\n :hidden [\"/pages/private\"]}\n",
        );
        for (path, text) in files {
            garden.write(path, text);
        }
        garden
    }

    fn write(&self, path: &str, text: &str) {
        let path = self.0.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    fn delete(&self, path: &str) {
        fs::remove_file(self.0.join(path)).unwrap();
    }

    fn load(&self) -> GardenIndex {
        GardenIndex::load(&self.0).unwrap()
    }
}

impl Drop for Garden {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn sample() -> Garden {
    Garden::new(&[
        (
            "pages/Fruit___Apple.md",
            &format!(
                "alias:: Malus\ntags:: [[Fruit]], recipes\n\n- Red and round\n  id:: {FIRST}\n- Shared\n  id:: {SHARED}\n"
            ),
        ),
        (
            "pages/Fruit.md",
            "type:: food\n\n- [[Fruit/Apple]] is one\n",
        ),
        (
            "pages/Notes.md",
            &format!(
                "type:: food\n\n- See [[malus]] and (({FIRST})) and [[Jan 2nd, 2026]]\n\t- #[[Not Yet Written]] {{{{embed (({SHARED}))}}}}\n"
            ),
        ),
        (
            "journals/2026_01_02.md",
            &format!("- Ate an [[fruit/apple]]\n  id:: {SHARED}\n  type:: meal\n"),
        ),
        ("journals/notes.md", "- no date in this file name\n"),
        ("pages/.trash/old.md", "- [[Fruit]]\n"),
        ("pages/private/secret.md", "- [[Fruit]]\n"),
        ("pages/blank.md", "\n  \n"),
        ("README.md", "- [[Fruit]]\n"),
    ])
}

fn path(text: &str) -> &Path {
    Path::new(text)
}

fn document(text: &str) -> PageId {
    PageId::Document(text.into())
}

/// The paths and written texts of occurrences.
fn written<'a>(found: &[(&'a IndexedDocument, &Occurrence)]) -> Vec<(String, &'a str)> {
    found
        .iter()
        .map(|(document, occurrence)| {
            (
                slashes(document.path()),
                occurrence.range.text(document.source()),
            )
        })
        .collect()
}

#[test]
fn loads_pages_and_journals_and_skips_hidden_paths() {
    let garden = sample();
    let mut calls = Vec::new();
    let index =
        GardenIndex::load_with_progress(&garden.0, &mut |done, total| calls.push((done, total)))
            .unwrap();
    let paths: Vec<_> = index.documents().map(|d| slashes(d.path())).collect();
    assert_eq!(
        paths,
        [
            "journals/2026_01_02.md",
            "journals/notes.md",
            "pages/Fruit.md",
            "pages/Fruit___Apple.md",
            "pages/Notes.md",
        ]
    );
    // `pages/blank.md` is read and holds no page.
    assert_eq!(calls.first(), Some(&(0, 6)));
    assert_eq!(calls.last(), Some(&(6, 6)));
    assert_eq!(
        index.settings().filename_format,
        FilenameFormat::TripleLowbar
    );
    let apple = index.document(path("pages/Fruit___Apple.md")).unwrap();
    assert_eq!(apple.title.as_deref(), Some("Fruit/Apple"));
    assert_eq!(apple.aliases, ["malus"]);
    let journal = index.document(path("journals/2026_01_02.md")).unwrap();
    assert_eq!(journal.title.as_deref(), Some("Jan 2nd, 2026"));
    assert_eq!(journal.journal, JournalDate::new(2026, 1, 2));
    assert_eq!(
        index.document(path("journals/notes.md")).unwrap().title,
        None
    );
}

#[test]
fn finds_the_occurrence_at_an_offset() {
    let garden = sample();
    let index = garden.load();
    let notes = index.document(path("pages/Notes.md")).unwrap();
    let at = |text: &str| notes.source().find(text).unwrap();
    let (_, link) = index
        .occurrence_at(path("pages/Notes.md"), at("malus"))
        .unwrap();
    assert_eq!(link.kind, OccurrenceKind::PageLink);
    assert_eq!(link.target, Target::Page("malus".into()));
    let (_, reference) = index
        .occurrence_at(path("pages/Notes.md"), at(FIRST) + 3)
        .unwrap();
    assert_eq!(reference.kind, OccurrenceKind::BlockReference);
    assert_eq!(reference.target, Target::Block(FIRST.into()));
    // Just after the closing brackets still finds the link; plain words find nothing.
    assert!(notes.occurrence_at(at("[[malus]]") + 9).is_some());
    assert!(notes.occurrence_at(at("See") + 1).is_none());
    assert!(index.occurrence_at(path("pages/Missing.md"), 0).is_none());
}

#[test]
fn resolves_titles_aliases_journal_dates_and_missing_pages() {
    let garden = sample();
    let index = garden.load();
    let apple = document("pages/Fruit___Apple.md");
    assert_eq!(index.page_id("FRUIT/apple"), apple);
    assert_eq!(index.page_id("malus"), apple);
    assert_eq!(
        index.page_id("Jan 2nd, 2026"),
        document("journals/2026_01_02.md")
    );
    assert_eq!(
        index.page_id("2026-01-02"),
        document("journals/2026_01_02.md")
    );
    assert_eq!(
        index.page_id("2026-01-03"),
        PageId::Journal(JournalDate::new(2026, 1, 3).unwrap())
    );
    assert_eq!(
        index.page_id("Not Yet Written"),
        PageId::Name("not yet written".into())
    );
    assert_eq!(
        written(&index.occurrences_of_page(&apple)),
        [
            ("journals/2026_01_02.md".to_owned(), "[[fruit/apple]]"),
            ("pages/Fruit.md".to_owned(), "[[Fruit/Apple]]"),
            // The alias the page gives itself.
            ("pages/Fruit___Apple.md".to_owned(), "Malus"),
            ("pages/Notes.md".to_owned(), "[[malus]]"),
        ]
    );
    assert_eq!(
        written(&index.occurrences_of_page(&index.page_id("not yet written"))),
        [("pages/Notes.md".to_owned(), "#[[Not Yet Written]]")]
    );
    assert_eq!(
        written(&index.occurrences_of_page(&document("journals/2026_01_02.md"))),
        [("pages/Notes.md".to_owned(), "[[Jan 2nd, 2026]]")]
    );
    match index.resolve(&Target::Property("type".into())) {
        Resolved::Property { count, .. } => assert_eq!(count, 3),
        other => panic!("{other:?}"),
    }
}

#[test]
fn several_documents_with_one_alias_are_ambiguous() {
    let garden = Garden::new(&[
        ("pages/A.md", "alias:: shared\n\n- a\n"),
        ("pages/B.md", "alias:: shared\n\n- b\n"),
    ]);
    let index = garden.load();
    assert_eq!(index.page_id("Shared"), PageId::Ambiguous("shared".into()));
    assert_eq!(index.page_documents("shared").len(), 2);
}

#[test]
fn a_duplicated_block_id_names_the_block_logseq_reads_first() {
    let garden = sample();
    let index = garden.load();
    let sites: Vec<_> = index
        .block_id_sites(SHARED)
        .iter()
        .map(|location| slashes(location.document.path()))
        .collect();
    assert_eq!(sites, ["journals/2026_01_02.md", "pages/Fruit___Apple.md"]);
    let kept = index.block(&SHARED.to_uppercase()).unwrap();
    assert_eq!(kept.document.path(), path("journals/2026_01_02.md"));
    assert_eq!(kept.site.block_start, 0);
    assert_eq!(index.duplicated_block_ids().collect::<Vec<_>>(), [SHARED]);
    assert!(
        index
            .block("66f1a2b0-0000-4000-8000-00000000ffff")
            .is_none()
    );
    assert_eq!(
        written(&index.occurrences_of_block(SHARED)),
        [(
            "pages/Notes.md".to_owned(),
            &*format!("{{{{embed (({SHARED}))}}}}")
        )]
    );
}

#[test]
fn counts_property_keys_and_values() {
    let garden = sample();
    let index = garden.load();
    let keys: Vec<_> = index.property_keys().collect();
    assert_eq!(keys, [("alias", 1), ("id", 3), ("tags", 1), ("type", 3)]);
    let values: Vec<_> = index.property_values("Type").collect();
    assert_eq!(values, [("food", 2), ("meal", 1)]);
}

#[test]
fn stays_equal_to_a_fresh_load_after_files_change() {
    let garden = sample();
    let mut index = garden.load();
    let change = |index: &mut GardenIndex, path_text: &str, text: Option<&str>| {
        match text {
            Some(text) => garden.write(path_text, text),
            None => garden.delete(path_text),
        }
        index.refresh_file(path(path_text)).unwrap();
        assert!(
            *index == garden.load(),
            "differs from a fresh load after changing {path_text}"
        );
    };
    // Add a page that takes over an alias and writes a namespace child.
    change(
        &mut index,
        "pages/Fruit___Pear.md",
        Some(&format!(
            "alias:: Pyrus\n\n- Like an [[Fruit/Apple]]\n  id:: {SHARED}\n"
        )),
    );
    assert_eq!(index.page_id("pyrus"), document("pages/Fruit___Pear.md"));
    // Edit: change the alias, the links and the properties.
    change(
        &mut index,
        "pages/Fruit___Apple.md",
        Some("alias:: Pomme\ncolour:: red\n\n- Now links [[Notes]] and #recipes\n"),
    );
    assert_eq!(index.page_id("malus"), PageId::Name("malus".into()));
    assert_eq!(index.page_id("pomme"), document("pages/Fruit___Apple.md"));
    // Edit without changing the page's names.
    change(
        &mut index,
        "pages/Notes.md",
        Some("type:: note\n\n- Only [[pomme]] now\n"),
    );
    // Rename: the old path goes and the new one appears.
    garden.write("pages/Fruit___Quince.md", "alias:: Pyrus\n\n- Renamed\n");
    garden.delete("pages/Fruit___Pear.md");
    index.refresh_file(path("pages/Fruit___Quince.md")).unwrap();
    index.refresh_file(path("pages/Fruit___Pear.md")).unwrap();
    assert!(index == garden.load(), "differs from a fresh load");
    assert_eq!(index.page_id("pyrus"), document("pages/Fruit___Quince.md"));
    // Delete, empty a file, and add a journal.
    change(&mut index, "pages/Fruit.md", None);
    change(&mut index, "pages/Notes.md", Some("  \n"));
    change(&mut index, "journals/2026_01_03.md", Some("- [[Fruit]]\n"));
    // Paths the garden leaves out change nothing.
    change(&mut index, "pages/private/more.md", Some("- [[Fruit]]\n"));
    change(&mut index, "notes/elsewhere.md", Some("- [[Fruit]]\n"));
    assert_eq!(index.len(), 5);
}

#[test]
fn a_folder_refresh_follows_a_renamed_folder() {
    let garden = sample();
    let mut index = garden.load();
    fs::create_dir_all(garden.0.join("pages/old")).unwrap();
    garden.write("pages/old/Inner.md", "- [[Fruit]]\n");
    index.refresh_folder(path("pages/old")).unwrap();
    assert!(index == garden.load(), "differs from a fresh load");
    fs::rename(garden.0.join("pages/old"), garden.0.join("pages/new")).unwrap();
    index.refresh_folder(path("pages/old")).unwrap();
    index.refresh_folder(path("pages/new")).unwrap();
    assert!(index == garden.load(), "differs from a fresh load");
    assert!(index.document(path("pages/new/Inner.md")).is_some());
}

#[test]
fn an_open_buffer_takes_the_place_of_the_file() {
    let garden = sample();
    let mut index = garden.load();
    let notes = path("pages/Notes.md");
    assert!(index.set_buffer(notes, "alias:: Jottings\n\n- [[Fruit]] only\n".into()));
    assert_eq!(index.page_id("jottings"), document("pages/Notes.md"));
    assert_eq!(index.buffers().collect::<Vec<_>>(), [notes]);
    // A change on disk, and a reload of the settings, keep the buffer's text.
    garden.write("pages/Notes.md", "- from disk [[Fruit/Apple]]\n");
    index.refresh_file(notes).unwrap();
    assert!(index.document(notes).unwrap().source().contains("only"));
    garden.write(CONFIG_PATH, "{}");
    index.reload(&mut |_, _| {}).unwrap();
    assert_eq!(index.settings().filename_format, FilenameFormat::Legacy);
    assert!(index.document(notes).unwrap().source().contains("only"));
    assert_eq!(
        index
            .document(path("pages/Fruit___Apple.md"))
            .unwrap()
            .title
            .as_deref(),
        Some("Fruit___Apple")
    );
    // Closing reads the file again.
    index.close_buffer(notes).unwrap();
    assert!(index == garden.load(), "differs from a fresh load");
    // A buffer for a page with no file is a document until it closes.
    let new = path("pages/Brand New.md");
    assert!(index.set_buffer(new, "- draft\n".into()));
    assert!(index.document(new).is_some());
    index.close_buffer(new).unwrap();
    assert!(index.document(new).is_none());
    assert!(!index.set_buffer(path("README.md"), "- text\n".into()));
    assert!(index == garden.load(), "differs from a fresh load");
}
