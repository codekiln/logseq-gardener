//! Read Markdown sources without interpreting Logseq graph identities.

use std::error::Error;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::ast::Projection;

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
        let parsed = lsdoc::parse_to_projection(&source);
        documents.push(GardenDocument {
            relative_path,
            kind,
            source,
            parsed,
        });
    }
    Ok(Garden { documents })
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
