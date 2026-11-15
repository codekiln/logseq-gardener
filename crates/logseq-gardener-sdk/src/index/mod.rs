//! An in-memory index of a garden for editors: what each document writes and where.
//!
//! [`GardenIndex`] holds every Markdown file beneath `pages/` and `journals/`. For each
//! document it keeps the source, the parse and a list of [`Occurrence`]s: every page link,
//! tag, block reference, embed, property and block id with the range of source it occupies.
//! The index answers what is at an offset ([`GardenIndex::occurrence_at`]), which page or
//! block a name leads to ([`GardenIndex::resolve`]) and where a page or block is named
//! ([`GardenIndex::occurrences_of_page`], [`GardenIndex::occurrences_of_block`]).
//!
//! The index stays current one document at a time. An editor buffer's text takes the place
//! of the file on disk while the buffer is open ([`GardenIndex::set_buffer`],
//! [`GardenIndex::close_buffer`]), and [`GardenIndex::refresh_file`] reads one path again
//! after a change on disk. After any of these the index equals a fresh
//! [`GardenIndex::load`] of the same texts.
//!
//! ```
//! use logseq_gardener_sdk::index::{GardenIndex, OccurrenceKind, PageId};
//!
//! let root = std::env::temp_dir().join(format!("lsg-index-doc-{}", std::process::id()));
//! std::fs::create_dir_all(root.join("pages")).unwrap();
//! std::fs::create_dir_all(root.join("logseq")).unwrap();
//! std::fs::write(root.join("logseq/config.edn"), "{:file/name-format :triple-lowbar}").unwrap();
//! std::fs::write(root.join("pages/Fruit___Apple.md"), "alias:: Malus\n\n- Red\n").unwrap();
//! std::fs::write(root.join("pages/Notes.md"), "- See [[malus]]\n").unwrap();
//!
//! let index = GardenIndex::load(&root).unwrap();
//! let (_, occurrence) = index.occurrence_at("pages/Notes.md".as_ref(), 8).unwrap();
//! assert_eq!(occurrence.kind, OccurrenceKind::PageLink);
//! let apple = index.page_id("malus");
//! assert_eq!(apple, PageId::Document("pages/Fruit___Apple.md".into()));
//! // The page's own `alias:: Malus` and the link in Notes.
//! assert_eq!(index.occurrences_of_page(&apple).len(), 2);
//! std::fs::remove_dir_all(root).unwrap();
//! ```

mod completion;
mod definition;
mod hover;
mod lines;
mod occurrences;
mod problems;
mod references;
mod rename;

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::block_ids::read_order;
use crate::garden::{DocumentKind, GardenDocument, hidden_path, parse_document};
use crate::garden_config;
use crate::garden_settings::GardenSettings;
use crate::journals::{JournalDate, JournalFormats, journal_identity, parse_journal_reference};
use crate::namespaces::{NamespaceEntry, NamespaceIndex};
use crate::page_names::{
    Link, PageNameIndex, has_outer_space, is_empty_page, page_aliases, reference_key,
};
use crate::page_titles::page_title;

pub use completion::{
    Candidate, CandidateKind, CompletionContext, CompletionKind, Completions, completion_context,
};
pub use definition::Destination;
pub use hover::{BlockSummary, PageSummary, SUMMARY_LINES, Summary};
pub use lines::{LineIndex, LinePosition};
pub use occurrences::{BlockIdSite, Occurrence, OccurrenceKind, PropertyLine, SourceRange, Target};
pub use problems::{Problem, ProblemKind, RelatedLocation, Severity};
pub use references::{NamedBy, PageReference, Reference, Subject};
pub use rename::{
    DocumentChange, FileMove, PageRename, RenamePlan, RenameRefusal, RenameTarget, Replacement,
};

/// The folders whose Markdown files the index holds.
pub const NOTE_DIRECTORIES: [&str; 2] = ["pages", "journals"];

/// The garden's settings file, relative to its root.
pub const CONFIG_PATH: &str = "logseq/config.edn";

/// One page or journal and what it writes.
#[derive(Debug, Clone, PartialEq)]
pub struct IndexedDocument {
    /// Path relative to the garden root, kind, source text and parse. The parse keeps every
    /// `id::` the source writes; [`GardenIndex::block`] says which block Logseq gives an id
    /// that several blocks write.
    pub document: GardenDocument,
    /// Line starts of the source.
    pub lines: LineIndex,
    /// The page title or journal title. `None` when the file name gives none, such as a
    /// journal file whose name is not a date.
    pub title: Option<String>,
    /// The date of a journal.
    pub journal: Option<JournalDate>,
    /// The names in the leading `alias::` value, as lookup keys.
    pub aliases: Vec<String>,
    /// Every link, tag, block reference, embed, property key, property value name and block
    /// id, sorted by position.
    pub occurrences: Vec<Occurrence>,
    /// Every `key:: value` line, in source order.
    pub properties: Vec<PropertyLine>,
    /// Every block that writes a valid id, in source order.
    pub block_ids: Vec<BlockIdSite>,
    /// Whether the page holds no text, which changes how an alias of its title resolves.
    empty: bool,
}

impl IndexedDocument {
    /// The path relative to the garden root.
    pub fn path(&self) -> &Path {
        &self.document.relative_path
    }

    /// The source text.
    pub fn source(&self) -> &str {
        &self.document.source
    }

    /// The occurrence whose whole range holds `offset`. An offset just after an occurrence
    /// counts when no other occurrence starts there, so a cursor at the end of a link finds it.
    pub fn occurrence_at(&self, offset: usize) -> Option<&Occurrence> {
        let after = self
            .occurrences
            .partition_point(|occurrence| occurrence.range.start <= offset);
        let before = self.occurrences[..after].iter().rev();
        before
            .clone()
            .find(|occurrence| occurrence.range.contains(offset))
            .or_else(|| before.clone().find(|o| o.range.end == offset))
    }
}

/// What a page name leads to.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PageId {
    /// The one page or journal with a file that answers to the name, by its path relative to
    /// the garden root.
    Document(PathBuf),
    /// A journal that has no file, by its date.
    Journal(JournalDate),
    /// A page that has no file, by [`reference_key`] of its name.
    Name(String),
    /// A name that several files answer to, by its lookup key.
    Ambiguous(String),
}

/// A block and the document that holds it.
#[derive(Debug, Clone, Copy)]
pub struct BlockLocation<'a> {
    pub document: &'a IndexedDocument,
    pub site: &'a BlockIdSite,
}

/// What a [`Target`] leads to.
#[derive(Debug, Clone)]
pub enum Resolved<'a> {
    Page(PageId),
    /// The block Logseq gives the id, or `None` when no block writes it.
    Block(Option<BlockLocation<'a>>),
    /// A property key and how many lines in the garden write it.
    Property {
        key: String,
        count: usize,
    },
}

/// The index could not read the garden.
#[derive(Debug)]
pub struct IndexError {
    /// The path of the failed read.
    pub path: PathBuf,
    pub source: io::Error,
}

impl fmt::Display for IndexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.path.display(), self.source)
    }
}

impl std::error::Error for IndexError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

/// Where a block id is written, ordered as Logseq reads files and then by position.
#[derive(Debug, Clone, PartialEq, Eq)]
struct SiteKey {
    path: PathBuf,
    /// The path with `/` separators, which Logseq's read order compares.
    spelling: String,
    block_start: usize,
}

impl Ord for SiteKey {
    fn cmp(&self, other: &Self) -> Ordering {
        read_order(&self.spelling, &other.spelling)
            .then_with(|| self.spelling.cmp(&other.spelling))
            .then_with(|| self.block_start.cmp(&other.block_start))
    }
}

impl PartialOrd for SiteKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Every page and journal of a garden, and what each one writes.
#[derive(Debug, PartialEq)]
pub struct GardenIndex {
    root: PathBuf,
    settings: GardenSettings,
    journal_formats: JournalFormats,
    /// The entries of `:hidden`.
    hidden: Vec<String>,
    documents: BTreeMap<PathBuf, IndexedDocument>,
    /// The paths whose text comes from an editor buffer.
    buffers: BTreeSet<PathBuf>,
    names: PageNameIndex<PathBuf>,
    namespaces: NamespaceIndex,
    /// The documents that write each page name, by [`reference_key`].
    page_links: BTreeMap<String, BTreeSet<PathBuf>>,
    /// The documents that refer to or embed each block id.
    block_links: BTreeMap<String, BTreeSet<PathBuf>>,
    /// The blocks that write each id, in the order Logseq reads them.
    block_ids: BTreeMap<String, BTreeSet<SiteKey>>,
    property_keys: BTreeMap<String, usize>,
    property_values: BTreeMap<String, BTreeMap<String, usize>>,
}

fn slashes(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

impl GardenIndex {
    /// Read the settings and every page and journal of the garden at `root`.
    pub fn load(root: impl AsRef<Path>) -> Result<Self, IndexError> {
        Self::load_with_progress(root, &mut |_, _| {})
    }

    /// Load like [`Self::load`], calling `progress` with the number of files read so far and
    /// the total, before the first file and after each one.
    pub fn load_with_progress(
        root: impl AsRef<Path>,
        progress: &mut dyn FnMut(usize, usize),
    ) -> Result<Self, IndexError> {
        let mut index = Self::empty(root.as_ref());
        index.read_everything(progress)?;
        Ok(index)
    }

    fn empty(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
            settings: GardenSettings::default(),
            journal_formats: JournalFormats::default(),
            hidden: Vec::new(),
            documents: BTreeMap::new(),
            buffers: BTreeSet::new(),
            names: PageNameIndex::default(),
            namespaces: NamespaceIndex::default(),
            page_links: BTreeMap::new(),
            block_links: BTreeMap::new(),
            block_ids: BTreeMap::new(),
            property_keys: BTreeMap::new(),
            property_values: BTreeMap::new(),
        }
    }

    /// Read the settings and every file again, keeping the text of open buffers. Call this
    /// when `logseq/config.edn` changes.
    pub fn reload(&mut self, progress: &mut dyn FnMut(usize, usize)) -> Result<(), IndexError> {
        let buffers: Vec<(PathBuf, String)> = self
            .buffers
            .iter()
            .filter_map(|path| {
                let document = self.documents.get(path)?;
                Some((path.clone(), document.source().to_owned()))
            })
            .collect();
        let open = std::mem::take(&mut self.buffers);
        let mut fresh = Self::empty(&self.root);
        fresh.read_everything(progress)?;
        *self = fresh;
        self.buffers = open;
        for (path, text) in buffers {
            if self.holds(&path) {
                self.replace(&path, Some(text));
            }
        }
        Ok(())
    }

    fn read_everything(
        &mut self,
        progress: &mut dyn FnMut(usize, usize),
    ) -> Result<(), IndexError> {
        self.settings = GardenSettings::read(&self.root).0;
        self.journal_formats = JournalFormats::from_settings(&self.settings);
        self.hidden = garden_config::hidden_patterns(&self.root);
        let mut files = Vec::new();
        for folder in NOTE_DIRECTORIES {
            self.list(Path::new(folder), &mut files)?;
        }
        files.sort();
        progress(0, files.len());
        for (done, path) in files.iter().enumerate() {
            // Logseq reads no page from a file that holds only whitespace.
            if let Some(text) = self.read(path)?.filter(|text| !text.trim().is_empty()) {
                self.insert(path, text);
            }
            progress(done + 1, files.len());
        }
        self.rebuild_names();
        Ok(())
    }

    /// List the Markdown files beneath `folder`, a path relative to the root.
    fn list(&self, folder: &Path, files: &mut Vec<PathBuf>) -> Result<(), IndexError> {
        let directory = self.root.join(folder);
        let entries = match fs::read_dir(&directory) {
            Ok(entries) => entries,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
            Err(source) => {
                return Err(IndexError {
                    path: directory,
                    source,
                });
            }
        };
        for entry in entries {
            let entry = entry.map_err(|source| IndexError {
                path: directory.clone(),
                source,
            })?;
            let path = folder.join(entry.file_name());
            // `file_type` does not follow a symlink, so a symlink is neither.
            let Ok(file_type) = entry.file_type() else {
                continue;
            };
            if file_type.is_dir() {
                if self.visible(&path, true) {
                    self.list(&path, files)?;
                }
            } else if file_type.is_file() && self.holds(&path) {
                files.push(path);
            }
        }
        Ok(())
    }

    /// Whether no segment of a relative path starts with a dot or is `node_modules`, and no
    /// entry of `:hidden` hides the path.
    fn visible(&self, path: &Path, folder: bool) -> bool {
        let mut spelling = slashes(path);
        if spelling
            .split('/')
            .any(|segment| segment.starts_with('.') || segment == "node_modules")
        {
            return false;
        }
        if folder {
            spelling.push('/');
        }
        !hidden_path(&self.hidden, &spelling)
    }

    /// Whether the index holds a document for a path relative to the root when the file
    /// exists: a visible `.md` file beneath `pages/` or `journals/`.
    pub fn holds(&self, path: &Path) -> bool {
        let mut components = path.components();
        let in_notes = components.next().is_some_and(|first| {
            NOTE_DIRECTORIES
                .iter()
                .any(|name| first.as_os_str() == *name)
        });
        in_notes
            && components.all(|part| matches!(part, std::path::Component::Normal(_)))
            && path.extension().is_some_and(|extension| extension == "md")
            && self.visible(path, false)
    }

    /// The text of a file, or `None` when there is no regular file at the path or it is not
    /// UTF-8.
    fn read(&self, path: &Path) -> Result<Option<String>, IndexError> {
        let full = self.root.join(path);
        match fs::symlink_metadata(&full) {
            Ok(metadata) if metadata.is_file() => {}
            Ok(_) => return Ok(None),
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(source) => return Err(IndexError { path: full, source }),
        }
        match fs::read_to_string(&full) {
            Ok(text) => Ok(Some(text)),
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::NotFound | io::ErrorKind::InvalidData
                ) =>
            {
                Ok(None)
            }
            Err(source) => Err(IndexError { path: full, source }),
        }
    }

    /// Use an editor buffer's text for a path until [`Self::close_buffer`]. Returns whether
    /// the index holds documents for this path; a path outside `pages/` and `journals/`
    /// changes nothing.
    pub fn set_buffer(&mut self, path: &Path, text: String) -> bool {
        if !self.holds(path) {
            return false;
        }
        self.buffers.insert(path.to_path_buf());
        self.replace(path, Some(text));
        true
    }

    /// Forget a path's editor buffer and read its file again.
    pub fn close_buffer(&mut self, path: &Path) -> Result<(), IndexError> {
        if self.buffers.remove(path) {
            self.refresh_file(path)?;
        }
        Ok(())
    }

    /// Read one path again after its file was created, changed or deleted. A path with an
    /// open buffer keeps the buffer's text. A failed read leaves the document as it was.
    pub fn refresh_file(&mut self, path: &Path) -> Result<(), IndexError> {
        if self.buffers.contains(path) || !self.holds(path) {
            return Ok(());
        }
        let text = self.read(path)?;
        self.replace(path, text);
        Ok(())
    }

    /// Read again every file at or beneath a path relative to the root, and drop the
    /// documents beneath it whose files are gone. Use it when a folder was created, renamed
    /// or deleted.
    pub fn refresh_folder(&mut self, folder: &Path) -> Result<(), IndexError> {
        let mut paths: BTreeSet<PathBuf> = self
            .documents
            .keys()
            .filter(|path| path.starts_with(folder))
            .cloned()
            .collect();
        let in_notes = NOTE_DIRECTORIES
            .iter()
            .any(|name| folder.starts_with(name) || Path::new(name).starts_with(folder));
        if in_notes {
            let mut files = Vec::new();
            for name in NOTE_DIRECTORIES {
                let name = Path::new(name);
                if name.starts_with(folder) {
                    self.list(name, &mut files)?;
                } else if folder.starts_with(name) && self.visible(folder, true) {
                    self.list(folder, &mut files)?;
                }
            }
            paths.extend(files);
        }
        for path in paths {
            self.refresh_file(&path)?;
        }
        Ok(())
    }

    /// Put `text` in place of a path's document, or remove the document for `None` and for
    /// a text of only whitespace, which Logseq reads as no page.
    fn replace(&mut self, path: &Path, text: Option<String>) {
        let old = self.remove(path);
        let identity = |document: &IndexedDocument| {
            (
                document.title.clone(),
                document.journal,
                document.aliases.clone(),
                document.empty,
            )
        };
        let before = old.as_ref().map(identity);
        let after = text
            .filter(|text| !text.trim().is_empty())
            .map(|text| identity(self.insert(path, text)));
        if before != after {
            self.rebuild_names();
        }
    }

    /// Parse a text and add its document to every map but the name index.
    fn insert(&mut self, path: &Path, source: String) -> &IndexedDocument {
        let kind = if path.starts_with("journals") {
            DocumentKind::Journal
        } else {
            DocumentKind::Page
        };
        let parsed = parse_document(&source);
        let found = occurrences::extract(&source, &parsed.blocks);
        let document = GardenDocument {
            relative_path: path.to_path_buf(),
            kind,
            source,
            parsed,
            title_spellings: BTreeMap::new(),
        };
        let (title, journal) = match kind {
            DocumentKind::Journal => match journal_identity(&document, &self.journal_formats) {
                Ok(identity) => (Some(identity.title), Some(identity.date)),
                Err(_) => (None, None),
            },
            DocumentKind::Page => (
                page_title(&document, self.settings.filename_format).ok(),
                None,
            ),
        };
        let aliases = title
            .as_deref()
            .map(|title| page_aliases(&document, title))
            .unwrap_or_default();
        let indexed = IndexedDocument {
            lines: LineIndex::new(&document.source),
            empty: is_empty_page(&document),
            title,
            journal,
            aliases,
            occurrences: found.occurrences,
            properties: found.properties,
            block_ids: found.block_ids,
            document,
        };
        self.count(&indexed, true);
        self.documents.entry(path.to_path_buf()).or_insert(indexed)
    }

    /// Take a path's document out of every map but the name index.
    fn remove(&mut self, path: &Path) -> Option<IndexedDocument> {
        let document = self.documents.remove(path)?;
        self.count(&document, false);
        Some(document)
    }

    /// Add a document's entries to the garden-wide maps, or take them out.
    fn count(&mut self, document: &IndexedDocument, add: bool) {
        fn mark(
            map: &mut BTreeMap<String, BTreeSet<PathBuf>>,
            key: String,
            path: &Path,
            add: bool,
        ) {
            if add {
                map.entry(key).or_default().insert(path.to_path_buf());
            } else if let Some(paths) = map.get_mut(&key) {
                paths.remove(path);
                if paths.is_empty() {
                    map.remove(&key);
                }
            }
        }
        let path = document.path();
        for occurrence in &document.occurrences {
            match (&occurrence.target, occurrence.kind) {
                (Target::Page(name), _) => {
                    mark(&mut self.page_links, reference_key(name), path, add);
                }
                // An `id::` line is where a block gets its id; `block_ids` holds those.
                (Target::Block(_), OccurrenceKind::BlockId) | (Target::Property(_), _) => {}
                (Target::Block(id), _) => mark(&mut self.block_links, id.clone(), path, add),
            }
        }
        for site in &document.block_ids {
            let key = SiteKey {
                path: path.to_path_buf(),
                spelling: slashes(path),
                block_start: site.block_start,
            };
            if add {
                self.block_ids
                    .entry(site.id.clone())
                    .or_default()
                    .insert(key);
            } else if let Some(sites) = self.block_ids.get_mut(&site.id) {
                sites.remove(&key);
                if sites.is_empty() {
                    self.block_ids.remove(&site.id);
                }
            }
        }
        for line in &document.properties {
            if add {
                *self.property_keys.entry(line.key.clone()).or_default() += 1;
                *self
                    .property_values
                    .entry(line.key.clone())
                    .or_default()
                    .entry(line.value.clone())
                    .or_default() += 1;
                continue;
            }
            if let Some(count) = self.property_keys.get_mut(&line.key) {
                *count -= 1;
                if *count == 0 {
                    self.property_keys.remove(&line.key);
                }
            }
            if let Some(values) = self.property_values.get_mut(&line.key) {
                if let Some(count) = values.get_mut(&line.value) {
                    *count -= 1;
                    if *count == 0 {
                        values.remove(&line.value);
                    }
                }
                if values.is_empty() {
                    self.property_values.remove(&line.key);
                }
            }
        }
    }

    /// Build name resolution and namespaces from the titles and aliases the documents hold.
    /// Nothing is parsed again.
    fn rebuild_names(&mut self) {
        let mut names = PageNameIndex::with_journal_formats(self.journal_formats.clone());
        let mut entries = Vec::new();
        for (path, document) in &self.documents {
            let Some(title) = &document.title else {
                continue;
            };
            names.insert(path.clone(), title, &document.document);
            if let Some(date) = document.journal {
                names.insert_journal(path.clone(), date);
            }
            entries.push(NamespaceEntry::new(
                slashes(path),
                title.clone(),
                document.aliases.clone(),
            ));
        }
        self.names = names;
        self.namespaces = NamespaceIndex::build(entries, []);
    }

    /// The garden root this index was loaded from.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// The settings read from `logseq/config.edn`.
    pub fn settings(&self) -> &GardenSettings {
        &self.settings
    }

    /// The journal title and file name formats in use.
    pub fn journal_formats(&self) -> &JournalFormats {
        &self.journal_formats
    }

    /// The document at a path relative to the garden root.
    pub fn document(&self, path: &Path) -> Option<&IndexedDocument> {
        self.documents.get(path)
    }

    /// Every document, by path.
    pub fn documents(&self) -> impl Iterator<Item = &IndexedDocument> {
        self.documents.values()
    }

    /// How many documents the index holds.
    pub fn len(&self) -> usize {
        self.documents.len()
    }

    pub fn is_empty(&self) -> bool {
        self.documents.is_empty()
    }

    /// The paths whose text comes from an editor buffer.
    pub fn buffers(&self) -> impl Iterator<Item = &Path> {
        self.buffers.iter().map(PathBuf::as_path)
    }

    /// Titles, aliases and journal dates of the documents, for lookups this index has no
    /// method for.
    pub fn names(&self) -> &PageNameIndex<PathBuf> {
        &self.names
    }

    /// Namespace parents and children of the documents' titles. An entry's key is its
    /// document's path with `/` separators.
    pub fn namespaces(&self) -> &NamespaceIndex {
        &self.namespaces
    }

    /// The document at `path` and the occurrence at a UTF-8 offset into its source.
    pub fn occurrence_at(
        &self,
        path: &Path,
        offset: usize,
    ) -> Option<(&IndexedDocument, &Occurrence)> {
        let document = self.documents.get(path)?;
        Some((document, document.occurrence_at(offset)?))
    }

    /// What a page name leads to, as Logseq resolves a page reference: a title, then an
    /// alias, then the journal with the date the name spells, ignoring letter case.
    pub fn page_id(&self, name: &str) -> PageId {
        match self.names.link(name) {
            Link::Page(path) | Link::Generated(path, _, _) => PageId::Document(path.clone()),
            Link::Ambiguous => PageId::Ambiguous(reference_key(name)),
            Link::Missing => {
                let date = (!has_outer_space(name))
                    .then(|| parse_journal_reference(name, &self.journal_formats))
                    .flatten();
                match date {
                    Some(date) => PageId::Journal(date),
                    None => PageId::Name(reference_key(name)),
                }
            }
        }
    }

    /// The documents a page name leads to: one, none, or several for an ambiguous name.
    pub fn page_documents(&self, name: &str) -> Vec<&IndexedDocument> {
        if has_outer_space(name) {
            return Vec::new();
        }
        self.names
            .candidates(name)
            .iter()
            .filter_map(|path| self.documents.get(path))
            .collect()
    }

    /// Every occurrence in the garden that leads to `page`, with its document, in path and
    /// then source order.
    pub fn occurrences_of_page(&self, page: &PageId) -> Vec<(&IndexedDocument, &Occurrence)> {
        // The written names that lead to the page, and the documents that write them.
        let mut keys = BTreeSet::new();
        let mut paths = BTreeSet::new();
        for (key, writers) in &self.page_links {
            if &self.page_id(key) == page {
                keys.insert(key.as_str());
                paths.extend(writers);
            }
        }
        let mut found = Vec::new();
        for path in paths {
            let Some(document) = self.documents.get(path) else {
                continue;
            };
            found.extend(
                document
                    .occurrences
                    .iter()
                    .filter(|occurrence| match &occurrence.target {
                        Target::Page(name) => keys.contains(reference_key(name).as_str()),
                        _ => false,
                    })
                    .map(|occurrence| (document, occurrence)),
            );
        }
        found
    }

    /// Every block that writes an id, in the order Logseq reads them. Logseq gives the id to
    /// the first and a new id to each of the others.
    pub fn block_id_sites(&self, id: &str) -> Vec<BlockLocation<'_>> {
        let Some(sites) = self.block_ids.get(&id.trim().to_ascii_lowercase()) else {
            return Vec::new();
        };
        sites
            .iter()
            .filter_map(|key| {
                let document = self.documents.get(&key.path)?;
                let site = document
                    .block_ids
                    .iter()
                    .find(|site| site.block_start == key.block_start)?;
                Some(BlockLocation { document, site })
            })
            .collect()
    }

    /// The block a block id names: the first of [`Self::block_id_sites`].
    pub fn block(&self, id: &str) -> Option<BlockLocation<'_>> {
        self.block_id_sites(id).into_iter().next()
    }

    /// Every block id that more than one block writes.
    pub fn duplicated_block_ids(&self) -> impl Iterator<Item = &str> {
        self.block_ids
            .iter()
            .filter(|(_, sites)| sites.len() > 1)
            .map(|(id, _)| id.as_str())
    }

    /// Every block reference and block embed of an id, with its document, in path and then
    /// source order.
    pub fn occurrences_of_block(&self, id: &str) -> Vec<(&IndexedDocument, &Occurrence)> {
        let id = id.trim().to_ascii_lowercase();
        let Some(paths) = self.block_links.get(&id) else {
            return Vec::new();
        };
        let mut found = Vec::new();
        for path in paths {
            let Some(document) = self.documents.get(path) else {
                continue;
            };
            found.extend(
                document
                    .occurrences
                    .iter()
                    .filter(|occurrence| {
                        occurrence.kind != OccurrenceKind::BlockId
                            && occurrence.target == Target::Block(id.clone())
                    })
                    .map(|occurrence| (document, occurrence)),
            );
        }
        found
    }

    /// What an occurrence's target leads to.
    pub fn resolve(&self, target: &Target) -> Resolved<'_> {
        match target {
            Target::Page(name) => Resolved::Page(self.page_id(name)),
            Target::Block(id) => Resolved::Block(self.block(id)),
            Target::Property(key) => Resolved::Property {
                key: key.clone(),
                count: self.property_keys.get(key).copied().unwrap_or(0),
            },
        }
    }

    /// Every property key, in lower case, with the number of lines that write it.
    pub fn property_keys(&self) -> impl Iterator<Item = (&str, usize)> {
        self.property_keys
            .iter()
            .map(|(key, count)| (key.as_str(), *count))
    }

    /// Every value written for a property key, with the number of lines that write it.
    pub fn property_values(&self, key: &str) -> impl Iterator<Item = (&str, usize)> {
        self.property_values
            .get(&key.trim().to_lowercase())
            .into_iter()
            .flatten()
            .map(|(value, count)| (value.as_str(), *count))
    }
}

#[cfg(test)]
mod tests;
