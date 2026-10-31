//! Read Markdown sources without interpreting Logseq graph identities.

use std::collections::HashSet;
use std::error::Error;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::ast::{Block, Projection};
use crate::garden_config;

/// A read-only collection of Markdown documents, sorted by relative path.
#[derive(Debug, Clone, PartialEq)]
pub struct Garden {
    /// Visible Markdown documents from every folder of the garden that Logseq reads.
    pub documents: Vec<GardenDocument>,
}

/// Original source and syntax for a single garden file.
#[derive(Debug, Clone, PartialEq)]
pub struct GardenDocument {
    /// Location relative to the root supplied to [`load_garden`].
    pub relative_path: PathBuf,
    /// Journal beneath `journals/`, page anywhere else.
    pub kind: DocumentKind,
    /// Exact UTF-8 text read from disk, including whitespace and line endings.
    pub source: String,
    /// Parsed Markdown blocks and parser-extracted page and block references.
    /// References are unresolved syntax, not logical graph identities.
    pub parsed: Projection,
}

/// Classification by source directory, without filename or date interpretation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentKind {
    /// Source outside `journals/`, usually beneath `pages/`.
    Page,
    /// Source beneath `journals/`.
    Journal,
}

/// A failed load; no partial garden is returned.
#[derive(Debug)]
pub enum GardenLoadError {
    /// A filesystem or UTF-8 read failed.
    Io {
        /// Path of the failed operation.
        path: PathBuf,
        /// Underlying filesystem or decoding error.
        source: io::Error,
    },
    /// The input does not satisfy the supported garden layout.
    InvalidInput {
        /// Path that failed validation.
        path: PathBuf,
        /// Explanation of the unsupported input.
        reason: &'static str,
    },
}

impl GardenLoadError {
    /// Path of the input or operation that stopped loading.
    pub fn path(&self) -> &Path {
        match self {
            Self::Io { path, .. } | Self::InvalidInput { path, .. } => path,
        }
    }
}

impl fmt::Display for GardenLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io { path, source } => write!(f, "{}: {source}", path.display()),
            Self::InvalidInput { path, reason } => write!(f, "{}: {reason}", path.display()),
        }
    }
}

impl Error for GardenLoadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            Self::InvalidInput { .. } => None,
        }
    }
}

fn io_error(path: &Path, source: io::Error) -> GardenLoadError {
    GardenLoadError::Io {
        path: path.to_path_buf(),
        source,
    }
}

fn invalid(path: &Path, reason: &'static str) -> GardenLoadError {
    GardenLoadError::InvalidInput {
        path: path.to_path_buf(),
        reason,
    }
}

fn require_directory(path: &Path, metadata: &fs::Metadata) -> Result<(), GardenLoadError> {
    if metadata.file_type().is_symlink() {
        Err(invalid(path, "symlinks are unsupported"))
    } else if !metadata.is_dir() {
        Err(invalid(path, "expected a directory"))
    } else {
        Ok(())
    }
}

/// Load visible lowercase `.md` files from every folder of the garden, as Logseq does.
///
/// The root must be a directory containing `pages/` or `journals/`; an absent
/// counterpart and empty note directories are allowed. A file beneath `journals/`
/// has the journal kind and every other file the page kind. The walk skips the
/// paths Logseq skips: any segment starting with `.`, any `node_modules` segment,
/// and `logseq/bak/`, `logseq/.recycle/` and `logseq/version-files/`. It then skips
/// files that an entry of `:hidden` in `logseq/config.edn` hides, with Logseq's
/// prefix rule, in every folder. Outside `pages/` and `journals/` it also skips files
/// git ignores when the root is in a git work tree, so a local build loads what a
/// fresh checkout holds, and it skips symlinks and special files without following
/// them. Beneath the note directories, visible symlinks and special files stop
/// loading with a path-bearing error. Unreadable inputs and invalid UTF-8 stop
/// loading too. Ordinary non-Markdown files are ignored. No source files are written.
///
/// The result preserves raw parser output without deriving names from paths,
/// resolving links or aliases, or determining publication visibility; see
/// [`Garden::remove_shadowed_documents`] for title collisions across folders.
/// Markdown parsing is permissive, not strict input validation. Filesystem reads
/// form a local snapshot; concurrent filesystem changes and symlink ancestors of the
/// supplied root are outside this reader's guarantees.
pub fn load_garden(root: impl AsRef<Path>) -> Result<Garden, GardenLoadError> {
    let root = root.as_ref();
    // Remove trailing separators so metadata cannot follow a final symlink as a directory.
    let metadata_path: PathBuf = root.components().collect();
    let metadata = fs::symlink_metadata(&metadata_path).map_err(|error| io_error(root, error))?;
    require_directory(root, &metadata)?;
    let mut found_directory = false;
    for name in NOTE_DIRECTORIES {
        let path = root.join(name);
        let metadata = match fs::symlink_metadata(&path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) => return Err(io_error(&path, error)),
        };
        require_directory(&path, &metadata)?;
        found_directory = true;
    }
    if !found_directory {
        return Err(invalid(root, "expected a pages or journals directory"));
    }
    let walk = Walk {
        root,
        hidden: garden_config::hidden_patterns(root),
        checkout: Checkout::read(root),
    };
    let mut files = Vec::new();
    walk.directory(root, "", &mut files)?;
    files.sort_by(|a, b| a.0.cmp(&b.0));
    let mut documents = Vec::with_capacity(files.len());
    for (relative_path, kind) in files {
        let path = root.join(&relative_path);
        let source = fs::read_to_string(&path).map_err(|error| io_error(&path, error))?;
        let parsed = parse_document(&source);
        documents.push(GardenDocument {
            relative_path,
            kind,
            source,
            parsed,
        });
    }
    Ok(Garden { documents })
}

impl GardenDocument {
    /// Whether the source sits beneath `pages/` or `journals/`, the folders that hold
    /// the pages the author maintains.
    pub fn in_note_directory(&self) -> bool {
        self.relative_path.components().next().is_some_and(|first| {
            NOTE_DIRECTORIES
                .iter()
                .any(|name| first.as_os_str() == *name)
        })
    }
}

impl Garden {
    /// Remove each document outside `pages/` and `journals/` whose page name matches a
    /// document beneath them, and return the removed documents' relative paths.
    ///
    /// `name` gives a document's lookup key, such as [`crate::page_names::lookup_key`]
    /// of its title, or `None` when it has none. The note-directory document wins
    /// because it is the page the author maintains; callers report the returned paths.
    pub fn remove_shadowed_documents(
        &mut self,
        mut name: impl FnMut(&GardenDocument) -> Option<String>,
    ) -> Vec<PathBuf> {
        let names: Vec<_> = self.documents.iter().map(&mut name).collect();
        let notes: HashSet<&String> = self
            .documents
            .iter()
            .zip(&names)
            .filter(|(document, _)| document.in_note_directory())
            .filter_map(|(_, name)| name.as_ref())
            .collect();
        let shadowed: Vec<bool> = self
            .documents
            .iter()
            .zip(&names)
            .map(|(document, name)| {
                !document.in_note_directory() && name.as_ref().is_some_and(|n| notes.contains(n))
            })
            .collect();
        let mut removed = Vec::new();
        let mut flags = shadowed.into_iter();
        self.documents.retain(|document| {
            let skip = flags.next().unwrap_or(false);
            if skip {
                removed.push(document.relative_path.clone());
            }
            !skip
        });
        removed
    }
}

const NOTE_DIRECTORIES: [&str; 2] = ["pages", "journals"];

/// Folders Logseq keeps its own backups and history in.
const LOGSEQ_INTERNAL: [&str; 3] = ["logseq/bak", "logseq/.recycle", "logseq/version-files"];

/// Files a fresh checkout of the garden's repository would hold, outside the note folders.
struct Checkout {
    files: HashSet<String>,
    /// Every folder that holds a listed file, so the walk can skip the rest unread.
    folders: HashSet<String>,
}

impl Checkout {
    /// List tracked and untracked, not ignored, Markdown files outside `pages/` and
    /// `journals/` with one `git ls-files` call. `None` when the root is not in a git work
    /// tree or git cannot run: then no git filter applies.
    fn read(root: &Path) -> Option<Self> {
        let output = Command::new("git")
            .arg("-C")
            .arg(root)
            .args([
                "ls-files",
                "-z",
                "--cached",
                "--others",
                "--exclude-standard",
                "--",
                "*.md",
                ":(exclude)pages/",
                ":(exclude)journals/",
            ])
            .stderr(Stdio::null())
            .output()
            .ok()?;
        if !output.status.success() {
            return None;
        }
        let mut files = HashSet::new();
        let mut folders = HashSet::new();
        for path in output.stdout.split(|byte| *byte == 0) {
            let Ok(path) = std::str::from_utf8(path) else {
                continue;
            };
            if path.is_empty() {
                continue;
            }
            let mut folder = path;
            while let Some((parent, _)) = folder.rsplit_once('/') {
                if !folders.insert(parent.to_owned()) {
                    break;
                }
                folder = parent;
            }
            files.insert(path.to_owned());
        }
        Some(Self { files, folders })
    }
}

struct Walk<'a> {
    root: &'a Path,
    hidden: Vec<String>,
    checkout: Option<Checkout>,
}

impl Walk<'_> {
    /// Logseq's `hidden?`: with a leading `/` on both, the path starts with the entry.
    fn hidden(&self, relative: &str) -> bool {
        let path = format!("/{relative}");
        self.hidden.iter().any(|pattern| {
            if pattern.starts_with('/') {
                path.starts_with(pattern.as_str())
            } else {
                path.strip_prefix('/')
                    .is_some_and(|rest| rest.starts_with(pattern.as_str()))
            }
        })
    }

    fn directory(
        &self,
        directory: &Path,
        relative: &str,
        files: &mut Vec<(PathBuf, DocumentKind)>,
    ) -> Result<(), GardenLoadError> {
        let entries = fs::read_dir(directory).map_err(|error| io_error(directory, error))?;
        let mut names = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|error| io_error(directory, error))?;
            let name = entry.file_name();
            if !name.as_encoded_bytes().starts_with(b".") && name != "node_modules" {
                names.push(name);
            }
        }
        names.sort();
        for name in names {
            let path = directory.join(&name);
            let child = match name.to_str() {
                Some(name) if relative.is_empty() => name.to_owned(),
                Some(name) => format!("{relative}/{name}"),
                // Git and `:hidden` name paths as UTF-8; such a path matches neither.
                None => format!("{relative}/\u{fffd}"),
            };
            let top = child.split('/').next().unwrap_or_default();
            let in_notes = NOTE_DIRECTORIES.contains(&top);
            let metadata = fs::symlink_metadata(&path).map_err(|error| io_error(&path, error))?;
            let file_type = metadata.file_type();
            if file_type.is_dir() {
                if LOGSEQ_INTERNAL.contains(&child.as_str())
                    // Every file beneath the folder starts with `/<folder>/`.
                    || self.hidden(&format!("{child}/"))
                    || (!in_notes
                        && self
                            .checkout
                            .as_ref()
                            .is_some_and(|checkout| !checkout.folders.contains(&child)))
                {
                    continue;
                }
                self.directory(&path, &child, files)?;
            } else if file_type.is_file() {
                if path.extension().is_none_or(|extension| extension != "md")
                    || self.hidden(&child)
                    || (!in_notes
                        && self
                            .checkout
                            .as_ref()
                            .is_some_and(|checkout| !checkout.files.contains(&child)))
                {
                    continue;
                }
                let kind = if top == "journals" {
                    DocumentKind::Journal
                } else {
                    DocumentKind::Page
                };
                // All entries descend from root; stripping is lexical, not filesystem resolution.
                let relative_path = path
                    .strip_prefix(self.root)
                    .expect("entry beneath root")
                    .to_path_buf();
                files.push((relative_path, kind));
            } else if in_notes {
                let reason = if file_type.is_symlink() {
                    "symlinks are unsupported"
                } else {
                    "special files are unsupported"
                };
                return Err(invalid(&path, reason));
            }
            // Symlinks and special files elsewhere belong to tooling; they are skipped unread.
        }
        Ok(())
    }
}

/// Parse a page's Markdown into Logseq's blocks.
///
/// Logseq starts a block at every heading, so a heading line written without a bullet,
/// such as `# [[Filed]]` after a bulleted block, is a block of its own rather than more
/// text of the bullet above. The parser reports such a line as a heading; this turns it
/// into a bullet at the same level.
pub fn parse_document(source: &str) -> Projection {
    let mut parsed = lsdoc::parse_to_projection(source);
    for block in &mut parsed.blocks {
        if let Block::Heading {
            level,
            size,
            inline,
            marker,
            priority,
            htags,
            span,
        } = block
        {
            *block = Block::Bullet {
                level: *level,
                size: *size,
                inline: std::mem::take(inline),
                marker: marker.take(),
                priority: priority.take(),
                htags: std::mem::take(htags),
                span: span.take(),
            };
        }
    }
    parsed
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::outline::Outline;

    #[test]
    fn unbulleted_heading_starts_a_block() {
        let parsed = parse_document("- bulleted [[A]]\n# [[Filed]]\n\t- child\n- next\n");
        let outline = Outline::from_blocks(&parsed.blocks);
        let shape: Vec<_> = outline
            .blocks
            .iter()
            .map(|b| (b.depth, b.heading, b.id.to_string()))
            .collect();
        assert_eq!(
            shape,
            [
                (1, None, "1".to_owned()),
                (1, Some(1), "2".to_owned()),
                (2, None, "2.1".to_owned()),
                (1, None, "3".to_owned()),
            ]
        );
    }

    #[test]
    fn headings_inside_quotes_stay_text() {
        let parsed = parse_document("- a\n  > # quoted\n");
        let bullets = parsed
            .blocks
            .iter()
            .filter(|b| matches!(b, Block::Bullet { .. }))
            .count();
        assert_eq!(bullets, 1);
    }
}
