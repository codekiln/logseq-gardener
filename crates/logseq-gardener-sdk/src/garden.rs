//! Read Markdown sources without interpreting Logseq graph identities.

use std::error::Error;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::ast::{Block, Projection};

/// A read-only collection of Markdown documents, sorted by relative path.
#[derive(Debug, Clone, PartialEq)]
pub struct Garden {
    /// Visible Markdown documents beneath `pages/` and `journals/`.
    pub documents: Vec<GardenDocument>,
}

/// Original source and syntax for a single garden file.
#[derive(Debug, Clone, PartialEq)]
pub struct GardenDocument {
    /// Location relative to the root supplied to [`load_garden`].
    pub relative_path: PathBuf,
    /// The note directory containing the source.
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
    /// Source beneath `pages/`.
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

/// Load visible lowercase `.md` files recursively from `pages/` and `journals/`.
///
/// The root must be a directory containing at least one note directory. An
/// absent counterpart and empty note directories are allowed. Dot-prefixed
/// entries are skipped; ordinary non-Markdown files are ignored. Visible
/// symlinks and special files, unreadable inputs, and invalid UTF-8 stop loading
/// with a path-bearing error. No source files are written.
///
/// The result preserves raw parser output without deriving names from paths,
/// resolving links or aliases, or determining publication visibility. Markdown
/// parsing is permissive, not strict input validation. Filesystem reads form a
/// local snapshot; concurrent filesystem changes and symlink ancestors of the
/// supplied root are outside this reader's guarantees.
pub fn load_garden(root: impl AsRef<Path>) -> Result<Garden, GardenLoadError> {
    let root = root.as_ref();
    // Remove trailing separators so metadata cannot follow a final symlink as a directory.
    let metadata_path: PathBuf = root.components().collect();
    let metadata = fs::symlink_metadata(&metadata_path).map_err(|error| io_error(root, error))?;
    require_directory(root, &metadata)?;
    let mut files = Vec::new();
    let mut found_directory = false;
    for (name, kind) in [
        ("pages", DocumentKind::Page),
        ("journals", DocumentKind::Journal),
    ] {
        let path = root.join(name);
        let metadata = match fs::symlink_metadata(&path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
            Err(error) => return Err(io_error(&path, error)),
        };
        require_directory(&path, &metadata)?;
        found_directory = true;
        collect_files(root, &path, kind, &mut files)?;
    }
    if !found_directory {
        return Err(invalid(root, "expected a pages or journals directory"));
    }
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

fn collect_files(
    root: &Path,
    directory: &Path,
    kind: DocumentKind,
    files: &mut Vec<(PathBuf, DocumentKind)>,
) -> Result<(), GardenLoadError> {
    let entries = fs::read_dir(directory).map_err(|error| io_error(directory, error))?;
    let mut paths = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| io_error(directory, error))?;
        if !entry.file_name().as_encoded_bytes().starts_with(b".") {
            paths.push(entry.path());
        }
    }
    paths.sort();
    for path in paths {
        let metadata = fs::symlink_metadata(&path).map_err(|error| io_error(&path, error))?;
        let file_type = metadata.file_type();
        if file_type.is_symlink() {
            return Err(invalid(&path, "symlinks are unsupported"));
        }
        if file_type.is_dir() {
            collect_files(root, &path, kind, files)?;
        } else if file_type.is_file() {
            if path.extension().is_some_and(|extension| extension == "md") {
                // All entries descend from root; stripping is lexical, not filesystem resolution.
                files.push((
                    path.strip_prefix(root)
                        .expect("entry beneath root")
                        .to_path_buf(),
                    kind,
                ));
            }
        } else {
            return Err(invalid(&path, "special files are unsupported"));
        }
    }
    Ok(())
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
