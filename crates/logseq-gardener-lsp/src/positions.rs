//! Positions and paths, converted in one place.
//!
//! The index speaks in paths relative to the garden root and offsets into UTF-8 text. The
//! protocol speaks in `file:` URIs and line and character positions, whose character unit
//! the client and server agree on at startup. Every handler converts through this module.

use std::path::{Component, Path, PathBuf};

use logseq_gardener_sdk::index::{IndexedDocument, LineIndex, LinePosition, SourceRange};
use lsp_types::{Position, PositionEncodingKind, Range};

/// The unit of a position's `character`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Encoding {
    /// Offsets into the line's UTF-8 text.
    Utf8,
    /// UTF-16 code units, the protocol's default.
    Utf16,
}

impl Encoding {
    /// UTF-8 when the client lists it, UTF-16 otherwise.
    pub(crate) fn negotiate(offered: Option<&[PositionEncodingKind]>) -> Self {
        if offered.is_some_and(|kinds| kinds.contains(&PositionEncodingKind::UTF8)) {
            Self::Utf8
        } else {
            Self::Utf16
        }
    }

    pub(crate) fn kind(self) -> PositionEncodingKind {
        match self {
            Self::Utf8 => PositionEncodingKind::UTF8,
            Self::Utf16 => PositionEncodingKind::UTF16,
        }
    }

    /// The UTF-8 offset of a position in `text`. A position past the end of its line gives
    /// the end of the line.
    pub(crate) fn offset(self, text: &str, lines: &LineIndex, position: Position) -> usize {
        let line = position.line as usize;
        let character = position.character as usize;
        let column = match self {
            Self::Utf8 => character,
            Self::Utf16 => {
                let (start, end) = lines.line_range(line, text);
                let mut units = 0;
                let mut column = end - start;
                for (at, c) in text[start..end].char_indices() {
                    if units >= character {
                        column = at;
                        break;
                    }
                    units += c.len_utf16();
                }
                column
            }
        };
        lines.offset(LinePosition { line, column }, text)
    }

    /// The position of a UTF-8 offset in `text`.
    pub(crate) fn position(self, text: &str, lines: &LineIndex, offset: usize) -> Position {
        let found = lines.position(offset);
        let character = match self {
            Self::Utf8 => found.column,
            Self::Utf16 => {
                let start = offset.min(text.len()) - found.column;
                text.get(start..start + found.column)
                    .map_or(found.column, |before| before.encode_utf16().count())
            }
        };
        Position::new(found.line as u32, character as u32)
    }

    /// The offset of a position in an indexed document.
    pub(crate) fn document_offset(self, document: &IndexedDocument, position: Position) -> usize {
        self.offset(document.source(), &document.lines, position)
    }

    /// The protocol range of a source range in an indexed document.
    pub(crate) fn range(self, document: &IndexedDocument, range: SourceRange) -> Range {
        let at = |offset| self.position(document.source(), &document.lines, offset);
        Range::new(at(range.start), at(range.end))
    }
}

/// The garden root, and the mapping between `file:` URIs and paths relative to it.
#[derive(Debug, Clone)]
pub(crate) struct Paths {
    /// The root as the client spelled it; URIs the server writes start here.
    root: PathBuf,
    /// The root with symlinks resolved, which file events and some clients use.
    resolved: PathBuf,
}

impl Paths {
    pub(crate) fn new(root: PathBuf) -> Self {
        let resolved = root.canonicalize().unwrap_or_else(|_| root.clone());
        Self { root, resolved }
    }

    pub(crate) fn root(&self) -> &Path {
        &self.root
    }

    /// The root with symlinks resolved.
    pub(crate) fn resolved_root(&self) -> &Path {
        &self.resolved
    }

    /// The path relative to the garden root that an absolute path names, when it is inside
    /// the garden.
    pub(crate) fn relative(&self, path: &Path) -> Option<PathBuf> {
        let relative = path
            .strip_prefix(&self.root)
            .or_else(|_| path.strip_prefix(&self.resolved))
            .ok()?;
        relative
            .components()
            .all(|part| matches!(part, Component::Normal(_)))
            .then(|| relative.to_path_buf())
    }

    /// The path relative to the garden root that a `file:` URI names.
    pub(crate) fn relative_uri(&self, uri: &str) -> Option<PathBuf> {
        self.relative(&uri_to_path(uri)?)
    }

    /// The `file:` URI of a path relative to the garden root.
    pub(crate) fn uri(&self, relative: &Path) -> String {
        path_to_uri(&self.root.join(relative))
    }
}

/// The absolute path a `file:` URI names, or `None` for another scheme or host.
pub(crate) fn uri_to_path(uri: &str) -> Option<PathBuf> {
    let rest = uri.strip_prefix("file://")?;
    // An empty host or `localhost`, then the path.
    let path = match rest.find('/') {
        Some(0) => rest,
        Some(at) if rest[..at].eq_ignore_ascii_case("localhost") => &rest[at..],
        _ => return None,
    };
    let path = path.split(['?', '#']).next().unwrap_or(path);
    let mut decoded = Vec::with_capacity(path.len());
    let raw = path.as_bytes();
    let mut at = 0;
    while at < raw.len() {
        let hex = |index: usize| raw.get(index).and_then(|c| (*c as char).to_digit(16));
        match (raw[at], hex(at + 1), hex(at + 2)) {
            (b'%', Some(high), Some(low)) => {
                decoded.push((high * 16 + low) as u8);
                at += 3;
            }
            (other, _, _) => {
                decoded.push(other);
                at += 1;
            }
        }
    }
    let path = String::from_utf8(decoded).ok()?;
    // `file:///C:/garden` names `C:/garden` on Windows.
    let windows = path.as_bytes().get(2) == Some(&b':') && cfg!(windows);
    Some(PathBuf::from(if windows { &path[1..] } else { &path }))
}

/// The `file:` URI of an absolute path, with every character outside the unreserved set
/// percent-encoded in each segment.
pub(crate) fn path_to_uri(path: &Path) -> String {
    let spelling = path.to_string_lossy().replace('\\', "/");
    let mut uri = String::from("file://");
    if !spelling.starts_with('/') {
        uri.push('/');
    }
    for c in spelling.chars() {
        if c.is_ascii_alphanumeric() || matches!(c, '/' | '-' | '.' | '_' | '~' | ':') {
            uri.push(c);
        } else {
            let mut buffer = [0; 4];
            for unit in c.encode_utf8(&mut buffer).bytes() {
                uri.push_str(&format!("%{unit:02X}"));
            }
        }
    }
    uri
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEXT: &str = "- plain [[A]]\n\t- é 🍎 [[B]] end\n";

    fn both(needle: &str) -> (usize, Position, Position) {
        let lines = LineIndex::new(TEXT);
        let offset = TEXT.find(needle).unwrap();
        (
            offset,
            Encoding::Utf8.position(TEXT, &lines, offset),
            Encoding::Utf16.position(TEXT, &lines, offset),
        )
    }

    #[test]
    fn ascii_lines_agree_in_both_encodings() {
        let (_, utf8, utf16) = both("[[A]]");
        assert_eq!(utf8, Position::new(0, 8));
        assert_eq!(utf16, Position::new(0, 8));
    }

    #[test]
    fn utf16_counts_an_emoji_as_two_units() {
        let (offset, utf8, utf16) = both("[[B]]");
        // Tab, dash, space, `é` (two UTF-8 units), space, the apple (four), space.
        assert_eq!(utf8, Position::new(1, 11));
        assert_eq!(utf16, Position::new(1, 8));
        let lines = LineIndex::new(TEXT);
        assert_eq!(Encoding::Utf8.offset(TEXT, &lines, utf8), offset);
        assert_eq!(Encoding::Utf16.offset(TEXT, &lines, utf16), offset);
    }

    #[test]
    fn positions_outside_the_text_stay_inside_it() {
        let lines = LineIndex::new(TEXT);
        let line_end = TEXT.find('\n').unwrap();
        for encoding in [Encoding::Utf8, Encoding::Utf16] {
            assert_eq!(
                encoding.offset(TEXT, &lines, Position::new(0, 400)),
                line_end
            );
            assert_eq!(
                encoding.offset(TEXT, &lines, Position::new(9, 0)),
                TEXT.len()
            );
        }
        // A UTF-16 position inside the apple's pair gives the character after it.
        let apple = TEXT.find('🍎').unwrap();
        assert_eq!(
            Encoding::Utf16.offset(TEXT, &lines, Position::new(1, 6)),
            apple + 4
        );
    }

    #[test]
    fn negotiates_utf8_only_when_offered() {
        let offered = [PositionEncodingKind::UTF16, PositionEncodingKind::UTF8];
        assert_eq!(Encoding::negotiate(Some(&offered)), Encoding::Utf8);
        assert_eq!(Encoding::negotiate(Some(&offered[..1])), Encoding::Utf16);
        assert_eq!(Encoding::negotiate(None), Encoding::Utf16);
    }

    #[cfg(unix)]
    #[test]
    fn maps_uris_to_paths_and_back() {
        let paths = Paths::new("/tmp/my garden".into());
        let uri = paths.uri(Path::new("pages/Fruit___Ap ple é.md"));
        assert_eq!(
            uri,
            "file:///tmp/my%20garden/pages/Fruit___Ap%20ple%20%C3%A9.md"
        );
        assert_eq!(
            paths.relative_uri(&uri),
            Some("pages/Fruit___Ap ple é.md".into())
        );
        assert_eq!(
            paths.relative_uri("file://localhost/tmp/my%20garden/journals/2026_01_02.md"),
            Some("journals/2026_01_02.md".into())
        );
        assert_eq!(paths.relative_uri("file:///tmp/elsewhere/a.md"), None);
        assert_eq!(paths.relative_uri("untitled:Untitled-1"), None);
        assert_eq!(
            paths.relative_uri("file:///tmp/my%20garden/pages/../x.md"),
            None
        );
    }
}
