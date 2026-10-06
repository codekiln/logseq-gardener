//! Derive display titles for Markdown pages with an explicit filename format.

use std::error::Error;
use std::fmt;
use std::path::{Component, Path, PathBuf};

use crate::ast::Block;
use crate::garden::{DocumentKind, GardenDocument};

/// Filename conventions selected by the garden's `:file/name-format` setting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FilenameFormat {
    /// Legacy filenames: dots separate namespaces; percent escapes decode only if every escape and the resulting UTF-8 are valid.
    Legacy,
    /// Triple underscores delimit namespaces; ASCII percent escapes decode individually.
    TripleLowbar,
}

/// An input whose page title cannot be supplied by this API.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PageTitleError {
    /// Journal identity requires date-format configuration.
    UnsupportedJournal { path: PathBuf },
    /// The relative path is not a normal Markdown page path.
    InvalidPath { path: PathBuf },
    /// The final filename stem cannot be interpreted as UTF-8.
    NonUtf8Stem { path: PathBuf },
    /// The derived title is empty or consists only of whitespace.
    EmptyTitle { path: PathBuf },
}

impl PageTitleError {
    /// Relative source path associated with the failure.
    pub fn path(&self) -> &Path {
        match self {
            Self::UnsupportedJournal { path }
            | Self::InvalidPath { path }
            | Self::NonUtf8Stem { path }
            | Self::EmptyTitle { path } => path,
        }
    }
}

impl fmt::Display for PageTitleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let reason = match self {
            Self::UnsupportedJournal { .. } => "journal titles require date-format configuration",
            Self::InvalidPath { .. } => "expected a normal relative pages/ path ending in .md",
            Self::NonUtf8Stem { .. } => "page filename stem is not UTF-8",
            Self::EmptyTitle { .. } => "page title is empty or whitespace-only",
        };
        write!(f, "{}: {reason}", self.path().display())
    }
}

impl Error for PageTitleError {}

/// Derive a display title without changing the document or reading the filesystem.
///
/// The caller must supply the graph's filename convention. A leading parsed
/// properties block supplies the last case-insensitive `title` property; later
/// blocks do not override the filename. Paths starting with `pages/contents.`
/// yield `Contents` before considering properties. Capitalization and property
/// values are preserved. This does not create canonical lookup keys, resolve
/// aliases, read graph configuration, or derive journal dates.
pub fn page_title(
    document: &GardenDocument,
    format: FilenameFormat,
) -> Result<String, PageTitleError> {
    let path = &document.relative_path;
    if document.kind == DocumentKind::Journal {
        return Err(PageTitleError::UnsupportedJournal { path: path.clone() });
    }
    let mut components = path.components();
    // Components alone normalize interior `.` and repeated separators. Require
    // the original spelling to contain only nonempty, normal lexical segments.
    let bytes = path.as_os_str().as_encoded_bytes();
    let normal_spelling = bytes
        .split(|byte| is_separator(*byte))
        .all(|segment| !segment.is_empty() && segment != b"." && segment != b"..");
    if !normal_spelling
        || components.next() != Some(Component::Normal("pages".as_ref()))
        || components.clone().next().is_none()
        || !components.all(|component| matches!(component, Component::Normal(_)))
        || path.extension().is_none_or(|extension| extension != "md")
    {
        return Err(PageTitleError::InvalidPath { path: path.clone() });
    }
    let stem = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .ok_or_else(|| PageTitleError::NonUtf8Stem { path: path.clone() })?;
    let is_contents = path.components().count() == 2
        && path
            .file_name()
            .is_some_and(|name| name.as_encoded_bytes().starts_with(b"contents."));
    let title = if is_contents {
        "Contents".to_owned()
    } else if let Some(Block::Properties { props, .. }) = document.parsed.blocks.first()
        && let Some(property) = props
            .iter()
            .rev()
            .find(|property| property.0.eq_ignore_ascii_case("title"))
    {
        property.1.clone()
    } else {
        match format {
            FilenameFormat::Legacy => {
                let replaced = stem.replace('.', "/");
                decode_legacy(&replaced).unwrap_or(replaced)
            }
            FilenameFormat::TripleLowbar => decode_triple_lowbar(&stem.replace("___", "/")),
        }
    };
    if title.trim().is_empty() {
        Err(PageTitleError::EmptyTitle { path: path.clone() })
    } else {
        Ok(title)
    }
}

fn is_separator(byte: u8) -> bool {
    byte == b'/' || (cfg!(windows) && byte == b'\\')
}

fn hex(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'a'..=b'f' => Some(byte - b'a' + 10),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn escape(bytes: &[u8], index: usize) -> Option<u8> {
    Some(hex(*bytes.get(index + 1)?)? * 16 + hex(*bytes.get(index + 2)?)?)
}

fn decode_legacy(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%' {
            decoded.push(escape(bytes, index)?);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }
    String::from_utf8(decoded).ok()
}

fn decode_triple_lowbar(value: &str) -> String {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'%'
            && let Some(byte) = escape(bytes, index).filter(u8::is_ascii)
        {
            decoded.push(byte);
            index += 3;
        } else {
            decoded.push(bytes[index]);
            index += 1;
        }
    }
    // Replacing ASCII escapes preserves valid UTF-8 from the input.
    String::from_utf8(decoded)
        .expect("ASCII escape decoding preserves UTF-8")
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join("/")
}
