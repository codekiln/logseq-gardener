//! Renaming a page or a property key across the garden: which text to replace and which
//! files to rename.
//!
//! [`GardenIndex::rename_page`] follows Logseq's `rename!` in
//! `src/main/frontend/handler/page.cljs`. [`GardenIndex::rename_property`] renames a key on
//! every `key::` line, matching keys as Logseq stores them. Each returns a [`RenamePlan`]
//! and changes nothing; the caller applies the plan, as an editor does with a workspace
//! edit.

use std::borrow::Cow;
use std::collections::BTreeSet;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use unicode_normalization::UnicodeNormalization;

use super::{
    CONFIG_PATH, GardenIndex, IndexedDocument, OccurrenceKind, PageId, SourceRange, Subject,
    Target, slashes,
};
use crate::ast::{Block, Inline};
use crate::file_names::{page_file_name, title_needs_property};
use crate::garden_config;
use crate::inline_references::tag_page_name;
use crate::journals::parse_journal_reference;
use crate::page_names::{has_outer_space, lookup_key, reference_key, remove_boundary_slashes};
use crate::page_titles::title_property;
use crate::property_names::{is_built_in_property, logseq_property_key, valid_property_name};

/// One range of a file's text and the text that takes its place.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Replacement {
    /// The range to replace. An empty range inserts.
    pub range: SourceRange,
    pub text: String,
}

/// One file rename, with both paths relative to the garden root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileMove {
    pub from: PathBuf,
    pub to: PathBuf,
}

/// What a rename does to one file: replace text, then rename the file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentChange<'a> {
    /// The file's current path, relative to the garden root.
    pub path: PathBuf,
    /// The text the ranges of `replacements` count in: an open buffer's text, else the
    /// file's.
    pub source: Cow<'a, str>,
    /// Replacements in source order, none overlapping another.
    pub replacements: Vec<Replacement>,
    /// File renames to apply in order after the replacements. A rename that changes only
    /// letter case, or only the Unicode normalization form, takes two: to a temporary name
    /// and from there to the new name, because many file systems and editors treat the old
    /// and the new name as one file.
    pub moves: Vec<FileMove>,
}

impl DocumentChange<'_> {
    /// The file's text after the replacements.
    pub fn text(&self) -> String {
        let mut text = String::with_capacity(self.source.len());
        let mut at = 0;
        for replacement in &self.replacements {
            text.push_str(&self.source[at..replacement.range.start]);
            text.push_str(&replacement.text);
            at = replacement.range.end;
        }
        text.push_str(&self.source[at..]);
        text
    }

    /// The file's path after the renames.
    pub fn new_path(&self) -> &Path {
        self.moves.last().map_or(&self.path, |last| &last.to)
    }
}

/// A page with a file that a rename gives a new title.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageRename {
    /// The file's current path, relative to the garden root.
    pub path: PathBuf,
    pub from: String,
    pub to: String,
}

/// Everything a rename changes. An empty plan changes nothing.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RenamePlan<'a> {
    /// The pages with a file that get a new title: the renamed page, its namespace
    /// children and the pages whose title links to it.
    pub pages: Vec<PageRename>,
    /// The files to change, by path, with `logseq/config.edn` among them when it names a
    /// renamed page as the home page.
    pub documents: Vec<DocumentChange<'a>>,
}

impl RenamePlan<'_> {
    pub fn is_empty(&self) -> bool {
        self.documents.is_empty()
    }

    /// How many ranges of text the plan replaces.
    pub fn replacement_count(&self) -> usize {
        self.documents
            .iter()
            .map(|document| document.replacements.len())
            .sum()
    }

    /// How many files the plan gives a new name.
    pub fn renamed_file_count(&self) -> usize {
        self.documents
            .iter()
            .filter(|document| !document.moves.is_empty())
            .count()
    }
}

/// What a rename at a place in a document is about.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenameTarget {
    /// The page or the property key to rename.
    pub subject: Subject,
    /// The name under the cursor, or an empty range at the cursor when the rename is about
    /// the document's own page.
    pub range: SourceRange,
    /// The current name: the page's title, the name as written for a page with no file, or
    /// the property key as written.
    pub name: String,
}

/// Why a rename cannot be done.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenameRefusal {
    /// The index holds no document at the path.
    NoDocument { path: PathBuf },
    /// The document has no page title, such as a journal file whose name is not a date.
    NoTitle { path: PathBuf },
    /// The cursor is on a block reference or a block id.
    Block,
    /// The property key has a built-in meaning in Logseq.
    BuiltInKey { key: String },
    /// Logseq would not read the new property key as a key.
    InvalidKey { key: String },
    /// A block writes both the old and the new property key. `line` counts from 1.
    KeyOnSameBlock {
        key: String,
        new_key: String,
        path: PathBuf,
        line: usize,
    },
    /// Several files answer to the name to rename.
    Ambiguous { name: String },
    /// The name is a journal date and the journal has no file.
    JournalWithoutFile { name: String },
    /// The new name is empty after trimming.
    EmptyName,
    /// The new name holds a line break.
    LineBreak,
    /// A new name already belongs to the page at `path`.
    NameTaken { name: String, path: PathBuf },
    /// A new name spells a journal date.
    JournalDate { name: String },
    /// The page's title is written in a form the rename cannot rewrite, such as front matter.
    TitleNotEditable { path: PathBuf },
    /// A new title gives an empty file name.
    NoFileName { name: String },
    /// A file is already at a new path, or two pages would get it.
    FileExists { path: PathBuf },
}

impl fmt::Display for RenameRefusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoDocument { path } => {
                write!(
                    f,
                    "{} is not a page or journal of the garden",
                    slashes(path)
                )
            }
            Self::NoTitle { path } => write!(f, "{} has no page title", slashes(path)),
            Self::Block => f.write_str("a block reference has no name to rename"),
            Self::BuiltInKey { key } => {
                write!(f, "the property key {key} has a built-in meaning in Logseq")
            }
            Self::InvalidKey { key } if key.is_empty() => f.write_str("the new key is empty"),
            Self::InvalidKey { key } => write!(
                f,
                "Logseq would not read {key} as a property key; use a name without spaces, colons, quotes or brackets that does not start with #"
            ),
            Self::KeyOnSameBlock {
                key,
                new_key,
                path,
                line,
            } => write!(
                f,
                "the block at line {line} of {} writes both {key} and {new_key}, and Logseq reads one value for a key on a block; remove one of the two lines first",
                slashes(path)
            ),
            Self::Ambiguous { name } => write!(
                f,
                "several files answer to the name {name}; rename one of them from inside its file after giving the others another title"
            ),
            Self::JournalWithoutFile { name } => {
                write!(f, "the journal {name} has no file to rename")
            }
            Self::EmptyName => f.write_str("the new name is empty"),
            Self::LineBreak => f.write_str("the new name holds a line break"),
            Self::NameTaken { name, path } => write!(
                f,
                "the name {name} already belongs to the page in {}; merging two pages is not available",
                slashes(path)
            ),
            Self::JournalDate { name } => {
                write!(f, "the name {name} is a journal date")
            }
            Self::TitleNotEditable { path } => write!(
                f,
                "cannot rewrite the title of {}, which is written outside a title:: property line",
                slashes(path)
            ),
            Self::NoFileName { name } => {
                write!(f, "the name {name} gives an empty file name")
            }
            Self::FileExists { path } => {
                write!(f, "a file already exists at {}", slashes(path))
            }
        }
    }
}

impl std::error::Error for RenameRefusal {}

/// Turns a written page name into its name after the rename.
struct Names<'a> {
    /// The lookup key of the page to rename.
    old: &'a str,
    /// The new name as typed, trimmed.
    new: &'a str,
}

impl Names<'_> {
    /// The new name of the page itself and of a namespace child: `old` gives `new` and
    /// `old/x` gives `new/x`, keeping the rest as written.
    fn direct(&self, name: &str) -> Option<String> {
        if reference_key(name) == self.old {
            return Some(self.new.to_owned());
        }
        name.match_indices('/').find_map(|(slash, _)| {
            let parent = &name[..slash];
            (!parent.is_empty() && reference_key(parent) == self.old)
                .then(|| format!("{}{}", self.new, &name[slash..]))
        })
    }

    /// [`Self::direct`], or the name with each `[[old]]` and `[[old/x]]` inside it
    /// renamed, as Logseq renames a page titled `[[old]] is a tool`.
    fn renamed(&self, name: &str) -> Option<String> {
        if let Some(direct) = self.direct(name) {
            return Some(direct);
        }
        let mut result = String::new();
        let mut rest = name;
        let mut changed = false;
        while let Some(open) = rest.find("[[") {
            let inner_start = open + 2;
            let Some(length) = rest[inner_start..].find("]]") else {
                break;
            };
            let inner = &rest[inner_start..inner_start + length];
            result.push_str(&rest[..inner_start]);
            match self.direct(inner) {
                Some(renamed) => {
                    changed = true;
                    result.push_str(&renamed);
                }
                None => result.push_str(inner),
            }
            rest = &rest[inner_start + length..];
        }
        result.push_str(rest);
        changed.then_some(result)
    }

    /// Whether a lookup key can be a name [`Self::renamed`] changes, without the cost of
    /// trying.
    fn may_rename(&self, key: &str) -> bool {
        key.contains("[[")
            || key
                .strip_prefix(self.old)
                .is_some_and(|rest| rest.is_empty() || rest.starts_with('/'))
            // One boundary slash, which a lookup key of a written name can keep.
            || key.starts_with('/')
    }
}

/// Whether `#name` reads back as a tag of that name, so the name needs no brackets.
fn can_be_bare_tag(name: &str) -> bool {
    if name.is_empty() || name.contains(char::is_whitespace) {
        return false;
    }
    let text = format!("#{name}");
    matches!(
        lsdoc::inline(&text, "markdown").as_slice(),
        [Inline::Tag { children, span: Some(span) }]
            if span.0 == 0 && span.1 == text.len() && tag_page_name(children) == name
    )
}

/// A path as a file system that ignores letter case and Unicode normalization compares it.
fn folded(path: &Path) -> String {
    slashes(path).to_lowercase().nfc().collect()
}

/// An EDN string literal.
fn edn_string(text: &str) -> String {
    let mut literal = String::with_capacity(text.len() + 2);
    literal.push('"');
    for c in text.chars() {
        if matches!(c, '"' | '\\') {
            literal.push('\\');
        }
        literal.push(c);
    }
    literal.push('"');
    literal
}

/// The `title::` line of a document's leading properties, when it has one.
fn title_line(document: &IndexedDocument) -> Option<SourceRange> {
    let Some(Block::Properties {
        span: Some(span), ..
    }) = document.document.parsed.blocks.first()
    else {
        return None;
    };
    document
        .properties
        .iter()
        .rev()
        .find(|line| line.key == "title" && line.value_range.end <= span.1)
        .map(|line| line.value_range)
}

/// Whether a document starts with `key:: value` lines.
fn has_leading_properties(document: &IndexedDocument) -> bool {
    matches!(
        document.document.parsed.blocks.first(),
        Some(Block::Properties { span: Some(span), .. })
            if document.properties.iter().any(|line| line.value_range.end <= span.1)
    )
}

impl GardenIndex {
    /// What a rename at an offset of a document is about: the page that the link, tag, embed
    /// or page name in a property value there leads to, following an alias to its page, the
    /// property key of a `key::` line, and the document's own page anywhere else.
    pub fn rename_target_at(
        &self,
        path: &Path,
        offset: usize,
    ) -> Result<RenameTarget, RenameRefusal> {
        let document = self
            .documents
            .get(path)
            .ok_or_else(|| RenameRefusal::NoDocument {
                path: path.to_path_buf(),
            })?;
        let Some(occurrence) = document.occurrence_at(offset) else {
            let name = document
                .title
                .clone()
                .ok_or_else(|| RenameRefusal::NoTitle {
                    path: path.to_path_buf(),
                })?;
            let range = title_line(document)
                .filter(|range| range.start <= offset && offset <= range.end)
                .unwrap_or(SourceRange::new(offset, offset));
            return Ok(RenameTarget {
                subject: Subject::Page(PageId::Document(path.to_path_buf())),
                range,
                name,
            });
        };
        let written = match &occurrence.target {
            Target::Page(name) => name,
            Target::Block(_) => return Err(RenameRefusal::Block),
            Target::Property(key) => {
                let written = occurrence.name_range.text(document.source());
                if is_built_in_property(key) {
                    return Err(RenameRefusal::BuiltInKey {
                        key: written.to_owned(),
                    });
                }
                return Ok(RenameTarget {
                    subject: Subject::Property(key.clone()),
                    range: occurrence.name_range,
                    name: written.to_owned(),
                });
            }
        };
        let page = self.page_id(written);
        let shown = || remove_boundary_slashes(written.trim()).to_owned();
        let name = match &page {
            PageId::Document(target) => self
                .documents
                .get(target)
                .and_then(|document| document.title.clone())
                .ok_or_else(|| RenameRefusal::NoTitle {
                    path: target.clone(),
                })?,
            PageId::Name(_) => shown(),
            PageId::Ambiguous(_) => return Err(RenameRefusal::Ambiguous { name: shown() }),
            PageId::Journal(_) => return Err(RenameRefusal::JournalWithoutFile { name: shown() }),
        };
        Ok(RenameTarget {
            subject: Subject::Page(page),
            range: occurrence.name_range,
            name,
        })
    }

    /// Whether a lookup key names a page by its title, or names a page with no file. A key
    /// that reaches a page only through an alias or a journal date does neither.
    fn names_by_title(&self, key: &str) -> bool {
        let candidates = if has_outer_space(key) {
            &[]
        } else {
            self.names.candidates(key)
        };
        candidates.is_empty()
            || candidates.iter().any(|path| {
                self.documents
                    .get(path)
                    .and_then(|document| document.title.as_deref())
                    .is_some_and(|title| lookup_key(title) == key)
            })
    }

    /// Refuse a new name that another page answers to or that spells a journal date. `own`
    /// is the file that gets the name, when one does.
    fn check_new_name(&self, name: &str, own: Option<&Path>) -> Result<(), RenameRefusal> {
        let own_date = own
            .and_then(|path| self.documents.get(path))
            .and_then(|document| document.journal);
        let date = parse_journal_reference(name, &self.journal_formats);
        if date.is_some() && date != own_date {
            return Err(RenameRefusal::JournalDate {
                name: name.to_owned(),
            });
        }
        match self
            .names
            .candidates(name)
            .iter()
            .find(|path| Some(path.as_path()) != own)
        {
            Some(path) => Err(RenameRefusal::NameTaken {
                name: name.to_owned(),
                path: path.clone(),
            }),
            None => Ok(()),
        }
    }

    /// Plan the rename of a page to `new_name`, reading `logseq/config.edn` from disk.
    pub fn rename_page(
        &self,
        page: &PageId,
        new_name: &str,
    ) -> Result<RenamePlan<'_>, RenameRefusal> {
        let config = garden_config::read_text(&self.root).ok().flatten();
        self.rename_page_with_config(page, new_name, config)
    }

    /// Plan the rename of a page to `new_name`, as Logseq's `rename!` does it.
    ///
    /// The plan replaces every page link, tag, page embed and page name in a property value
    /// that names the page by its title, in any letter case, with the new name as typed. A
    /// name written with one of the page's aliases, or as a journal date in another format,
    /// stays. Namespace children follow: `old/x` becomes `new/x` in titles, references and
    /// file names, and so does a page whose title holds `[[old]]`. Each page file gets the
    /// name [`page_file_name`] gives its new title, in the folder it is in; a journal keeps
    /// its file name. A page keeps an existing `title::` property with the new title, and
    /// gets one when its file name cannot carry the title. `config` is the text of
    /// `logseq/config.edn`; its `:default-home` page follows the rename.
    ///
    /// A new name equal to the page's title gives an empty plan.
    pub fn rename_page_with_config(
        &self,
        page: &PageId,
        new_name: &str,
        config: Option<String>,
    ) -> Result<RenamePlan<'_>, RenameRefusal> {
        let new = remove_boundary_slashes(new_name.trim()).trim();
        if new.is_empty() {
            return Err(RenameRefusal::EmptyName);
        }
        if new.contains(['\n', '\r']) {
            return Err(RenameRefusal::LineBreak);
        }
        let old = match page {
            PageId::Document(path) => {
                let document = self
                    .documents
                    .get(path)
                    .ok_or_else(|| RenameRefusal::NoDocument { path: path.clone() })?;
                let title = document
                    .title
                    .as_deref()
                    .ok_or_else(|| RenameRefusal::NoTitle { path: path.clone() })?;
                if self.names.candidates(title).len() > 1 {
                    return Err(RenameRefusal::Ambiguous {
                        name: title.to_owned(),
                    });
                }
                if title == new {
                    return Ok(RenamePlan::default());
                }
                lookup_key(title)
            }
            PageId::Name(key) => key.clone(),
            PageId::Ambiguous(key) => return Err(RenameRefusal::Ambiguous { name: key.clone() }),
            PageId::Journal(_) => {
                return Err(RenameRefusal::JournalWithoutFile {
                    name: new_name.trim().to_owned(),
                });
            }
        };
        let names = Names { old: &old, new };

        // The pages with a file that get a new title.
        let mut pages = Vec::new();
        for (path, document) in &self.documents {
            let Some(title) = document.title.as_deref() else {
                continue;
            };
            if !names.may_rename(&lookup_key(title)) {
                continue;
            }
            if let Some(to) = names.renamed(title).filter(|to| to != title) {
                self.check_new_name(&to, Some(path))?;
                pages.push(PageRename {
                    path: path.clone(),
                    from: title.to_owned(),
                    to,
                });
            }
        }

        // The written names to replace, and the files that write them.
        let mut keys = BTreeSet::new();
        let mut writers = BTreeSet::new();
        for (key, paths) in &self.page_links {
            if !names.may_rename(key) || !self.names_by_title(key) {
                continue;
            }
            let Some(to) = names.renamed(key) else {
                continue;
            };
            // A name that only references carry merges into a page that has the new name.
            if self.names.candidates(key).is_empty() {
                self.check_new_name(&to, None)?;
            }
            keys.insert(key.as_str());
            writers.extend(paths);
        }
        writers.extend(pages.iter().map(|page| &page.path));

        let format = self.settings.filename_format;
        let mut targets = BTreeSet::new();
        let mut temporary = 0;
        let mut documents = Vec::new();
        for path in writers {
            let Some(document) = self.documents.get(path) else {
                continue;
            };
            let source = document.source();
            let mut replacements = Vec::new();
            for occurrence in &document.occurrences {
                let Target::Page(written) = &occurrence.target else {
                    continue;
                };
                if !keys.contains(reference_key(written).as_str()) {
                    continue;
                }
                let Some(to) = names.renamed(written) else {
                    continue;
                };
                let bare_tag = occurrence.kind == OccurrenceKind::Tag
                    && !occurrence.range.text(source).starts_with("#[[");
                let replacement = if bare_tag {
                    let text = if can_be_bare_tag(&to) {
                        format!("#{to}")
                    } else {
                        format!("#[[{to}]]")
                    };
                    Replacement {
                        range: occurrence.range,
                        text,
                    }
                } else if occurrence.kind == OccurrenceKind::PropertyValue
                    && to.contains([',', '，'])
                {
                    // A bare name with a comma would read as two names.
                    Replacement {
                        range: occurrence.name_range,
                        text: format!("[[{to}]]"),
                    }
                } else {
                    Replacement {
                        range: occurrence.name_range,
                        text: to,
                    }
                };
                replacements.push(replacement);
            }

            let mut moves = Vec::new();
            if let Some(renamed) = pages.iter().find(|page| &page.path == path) {
                let to = &renamed.to;
                if let Some(range) = title_line(document) {
                    replacements.push(Replacement {
                        range,
                        text: to.clone(),
                    });
                } else if title_property(&document.document).is_some() {
                    return Err(RenameRefusal::TitleNotEditable { path: path.clone() });
                } else if document.journal.is_some() || title_needs_property(to, format) {
                    let gap = if has_leading_properties(document) {
                        "\n"
                    } else {
                        "\n\n"
                    };
                    replacements.push(Replacement {
                        range: SourceRange::new(0, 0),
                        text: format!("title:: {to}{gap}"),
                    });
                }
                if document.journal.is_none() {
                    let file_name = page_file_name(to, format);
                    if file_name.trim().is_empty() {
                        return Err(RenameRefusal::NoFileName { name: to.clone() });
                    }
                    let folder = path.parent().unwrap_or(Path::new(""));
                    let new_path = folder.join(format!("{file_name}.md"));
                    if &new_path != path {
                        let same_file = folded(&new_path) == folded(path);
                        let taken = self.documents.contains_key(&new_path)
                            || fs::symlink_metadata(self.root.join(&new_path)).is_ok();
                        if !targets.insert(folded(&new_path)) || (taken && !same_file) {
                            return Err(RenameRefusal::FileExists { path: new_path });
                        }
                        if same_file {
                            let through = loop {
                                temporary += 1;
                                let name = format!(".lsg-rename-{temporary}.tmp");
                                let candidate = folder.join(name);
                                if fs::symlink_metadata(self.root.join(&candidate)).is_err() {
                                    break candidate;
                                }
                            };
                            moves.push(FileMove {
                                from: path.clone(),
                                to: through.clone(),
                            });
                            moves.push(FileMove {
                                from: through,
                                to: new_path,
                            });
                        } else {
                            moves.push(FileMove {
                                from: path.clone(),
                                to: new_path,
                            });
                        }
                    }
                }
            }

            replacements
                .sort_by_key(|replacement| (replacement.range.start, replacement.range.end));
            let mut end = 0;
            replacements.retain(|replacement| {
                let keep = replacement.range.start >= end
                    && replacement.range.text(source) != replacement.text;
                if keep {
                    end = replacement.range.end;
                }
                keep
            });
            if !replacements.is_empty() || !moves.is_empty() {
                documents.push(DocumentChange {
                    path: path.clone(),
                    source: Cow::Borrowed(source),
                    replacements,
                    moves,
                });
            }
        }

        // `:default-home {:page "old"}` follows the rename, as in Logseq.
        if let Some(text) = config
            && let Some((home, start, end)) = garden_config::default_home_page(&text)
            && let Some(to) = names.renamed(&home)
        {
            let literal = edn_string(&to);
            if text[start..end] != literal {
                documents.push(DocumentChange {
                    path: PathBuf::from(CONFIG_PATH),
                    replacements: vec![Replacement {
                        range: SourceRange::new(start, end),
                        text: literal,
                    }],
                    source: Cow::Owned(text),
                    moves: Vec::new(),
                });
            }
        }
        documents.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(RenamePlan { pages, documents })
    }

    /// Plan the rename of a property key to `new_key` on every `key::` line of the garden.
    ///
    /// A line is renamed when its key equals `key` as Logseq stores keys
    /// ([`logseq_property_key`]): in any letter case, with `_` counting as `-`. The new key
    /// is written as typed, without the spaces around it and without one `::` at its end.
    /// Values and all other text stay, and so does the page of the same name with the links
    /// to it. Blocks that already write the new key keep their line, so the two keys become
    /// one.
    ///
    /// The rename is refused when either key has a built-in meaning in Logseq, when Logseq
    /// would not read the new key as a key, and when a block writes both keys, since Logseq
    /// reads one value for a key on a block. A key no line writes gives an empty plan.
    pub fn rename_property(
        &self,
        key: &str,
        new_key: &str,
    ) -> Result<RenamePlan<'_>, RenameRefusal> {
        let new = new_key.trim();
        let new = new.strip_suffix("::").unwrap_or(new).trim_end();
        if new.is_empty()
            || new.contains(char::is_whitespace)
            || new.contains(':')
            || !valid_property_name(new)
        {
            return Err(RenameRefusal::InvalidKey {
                key: new.to_owned(),
            });
        }
        for name in [key.trim(), new] {
            if is_built_in_property(name) {
                return Err(RenameRefusal::BuiltInKey {
                    key: name.to_owned(),
                });
            }
        }
        let old_stored = logseq_property_key(key);
        let new_stored = logseq_property_key(new);
        let mut documents = Vec::new();
        for (path, document) in &self.documents {
            let source = document.source();
            let lines = || {
                document
                    .properties
                    .iter()
                    .filter(|line| logseq_property_key(&line.key) == old_stored)
            };
            if new_stored != old_stored
                && let Some(line) = lines().find(|line| {
                    document.properties.iter().any(|other| {
                        other.block_start == line.block_start
                            && logseq_property_key(&other.key) == new_stored
                    })
                })
            {
                let before = source.get(..line.block_start).unwrap_or_default();
                return Err(RenameRefusal::KeyOnSameBlock {
                    key: line.key_range.text(source).to_owned(),
                    new_key: new.to_owned(),
                    path: path.clone(),
                    line: before.matches('\n').count() + 1,
                });
            }
            let replacements: Vec<Replacement> = lines()
                .filter(|line| line.key_range.text(source) != new)
                .map(|line| Replacement {
                    range: line.key_range,
                    text: new.to_owned(),
                })
                .collect();
            if !replacements.is_empty() {
                documents.push(DocumentChange {
                    path: path.clone(),
                    source: Cow::Borrowed(source),
                    replacements,
                    moves: Vec::new(),
                });
            }
        }
        Ok(RenamePlan {
            pages: Vec::new(),
            documents,
        })
    }
}

#[cfg(test)]
mod tests;
