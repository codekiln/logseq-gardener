//! The file Logseq writes a page or journal to: the reverse of [`crate::page_titles`].
//!
//! [`page_file_name`] is Logseq's `file-name-sanity` from
//! `src/main/frontend/util/fs.cljs`, for both values of `:file/name-format`. Reading the
//! name back with [`crate::page_titles::page_title`] gives the title again, except for the
//! titles [`title_needs_property`] reports, which Logseq stores in a `title::` property.
//!
//! ```
//! use logseq_gardener_sdk::file_names::{page_file_name, page_file_path};
//! use logseq_gardener_sdk::page_titles::FilenameFormat::{Legacy, TripleLowbar};
//!
//! assert_eq!(page_file_name("Fruit/Apple", TripleLowbar), "Fruit___Apple");
//! assert_eq!(page_file_name("Fruit/Apple", Legacy), "Fruit%2FApple");
//! assert_eq!(page_file_name("What?", TripleLowbar), "What%3F");
//! assert_eq!(
//!     page_file_path("Fruit/Apple", TripleLowbar).unwrap(),
//!     std::path::Path::new("pages/Fruit___Apple.md")
//! );
//! ```

use std::path::PathBuf;

use unicode_normalization::UnicodeNormalization;

use crate::garden::{DocumentKind, GardenDocument, parse_document};
use crate::journals::{JournalDate, JournalFormats};
use crate::page_names::remove_boundary_slashes;
use crate::page_titles::{FilenameFormat, page_title};

/// The file names Windows reserves. Logseq compares the whole title with these, in upper
/// case only.
const WINDOWS_RESERVED: [&str; 22] = [
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
    "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

/// The characters no file system Logseq supports allows in a name, plus `#`.
const RESERVED: [char; 9] = [':', '*', '?', '"', '<', '>', '|', '#', '\\'];

/// The characters the legacy format escapes: [`RESERVED`], `%` and the namespace `/`.
const LEGACY_ESCAPED: [char; 11] = [':', '*', '?', '"', '<', '>', '|', '#', '\\', '%', '/'];

fn push_escaped(name: &mut String, c: char) {
    let mut buffer = [0; 4];
    for unit in c.encode_utf8(&mut buffer).bytes() {
        name.push_str(&format!("%{unit:02X}"));
    }
}

/// The file name Logseq gives a page, without the folder and the `.md` extension.
///
/// Both formats drop one slash from each end of the title and apply Unicode NFC.
///
/// The triple-lowbar format then writes `%25` for the `%` of anything that reads as a
/// percent escape, percent-escapes `: * ? " < > | # \`, writes `%2E` for a leading dot,
/// ends a Windows reserved name such as `CON` and a title that ends in a dot with the
/// namespace separator, writes `%5F` for each underscore that would read as part of a
/// separator, and writes `___` for each `/`.
///
/// The legacy format percent-escapes `: * ? " < > | # \ %` and `/`.
pub fn page_file_name(title: &str, format: FilenameFormat) -> String {
    let title: String = remove_boundary_slashes(title).nfc().collect();
    match format {
        FilenameFormat::TripleLowbar => triple_lowbar(&title),
        FilenameFormat::Legacy => {
            let mut name = String::with_capacity(title.len());
            for c in title.chars() {
                if LEGACY_ESCAPED.contains(&c) {
                    push_escaped(&mut name, c);
                } else {
                    name.push(c);
                }
            }
            name
        }
    }
}

fn triple_lowbar(title: &str) -> String {
    let mut name = String::with_capacity(title.len());
    let bytes = title.as_bytes();
    for (at, c) in title.char_indices() {
        let hex = |index: usize| bytes.get(index).is_some_and(u8::is_ascii_hexdigit);
        if c == '%' && hex(at + 1) && hex(at + 2) {
            name.push_str("%25");
        } else if RESERVED.contains(&c) {
            push_escaped(&mut name, c);
        } else {
            name.push(c);
        }
    }
    if let Some(rest) = name.strip_prefix('.') {
        name = format!("%2E{rest}");
    }
    if WINDOWS_RESERVED.contains(&name.as_str()) || name.ends_with('.') {
        name.push('/');
    }
    name.replace("___", "%5F%5F%5F")
        .replace("_/", "%5F/")
        .replace("/_", "/%5F")
        .replace('/', "___")
}

/// The path, relative to the garden root, of the file Logseq creates for a page:
/// `pages/<file name>.md`. `None` for a title that gives an empty file name.
pub fn page_file_path(title: &str, format: FilenameFormat) -> Option<PathBuf> {
    let name = page_file_name(title, format);
    (!name.trim().is_empty()).then(|| PathBuf::from(format!("pages/{name}.md")))
}

/// The path, relative to the garden root, of the file Logseq creates for a journal:
/// `journals/<date in :journal/file-name-format>.md`.
pub fn journal_file_path(date: JournalDate, formats: &JournalFormats) -> PathBuf {
    PathBuf::from(format!("journals/{}.md", formats.file_name.format(date)))
}

/// Whether the file name of a title reads back as another title, so the page needs a
/// `title::` property to keep its title. Under the legacy format a title with a dot is one:
/// `Node.js` gets the file `Node.js.md`, which reads as `Node/js`.
pub fn title_needs_property(title: &str, format: FilenameFormat) -> bool {
    let Some(path) = page_file_path(title, format) else {
        return true;
    };
    let document = GardenDocument {
        relative_path: path,
        kind: DocumentKind::Page,
        source: String::new(),
        parsed: parse_document(""),
        title_spellings: Default::default(),
    };
    page_title(&document, format).ok().as_deref() != Some(title)
}

#[cfg(test)]
mod tests {
    use super::*;
    use FilenameFormat::{Legacy, TripleLowbar};

    #[test]
    fn triple_lowbar_names_follow_logseq() {
        for (title, name) in [
            ("Fruit", "Fruit"),
            ("Fruit/Apple", "Fruit___Apple"),
            ("/Fruit/Apple/", "Fruit___Apple"),
            ("multi word", "multi word"),
            // An underscore next to a separator, and three in a row, would read as part of
            // a separator.
            ("snake_case", "snake_case"),
            ("a__/bbb/ccc", "a_%5F___bbb___ccc"),
            ("a/_b", "a___%5Fb"),
            ("a___b", "a%5F%5F%5Fb"),
            ("a____b", "a%5F%5F%5F_b"),
            // A percent sign is escaped only where it would read as an escape.
            ("100% sure", "100% sure"),
            ("a%20b", "a%2520b"),
            ("aa?#/bbb/ccc", "aa%3F%23___bbb___ccc"),
            (
                "SyntaxError: Unexpected token '.'",
                "SyntaxError%3A Unexpected token '.'",
            ),
            ("a*b\"c<d>e|f\\g", "a%2Ab%22c%3Cd%3Ee%7Cf%5Cg"),
            // A leading dot would make a hidden file.
            (".NET", "%2ENET"),
            (".NET.", "%2ENET.___"),
            ("Node.js", "Node.js"),
            // Windows reserves these names and names that end in a dot.
            ("CON", "CON___"),
            ("LPT9", "LPT9___"),
            ("con", "con"),
            ("CON/sole", "CON___sole"),
            ("CON.", "CON.___"),
            // NFC: `e` and a combining acute accent become one character.
            ("Cafe\u{301}", "Caf\u{e9}"),
        ] {
            assert_eq!(page_file_name(title, TripleLowbar), name, "{title}");
        }
    }

    #[test]
    fn legacy_names_follow_logseq() {
        for (title, name) in [
            ("Fruit", "Fruit"),
            ("Fruit/Apple", "Fruit%2FApple"),
            ("/Fruit/Apple/", "Fruit%2FApple"),
            ("snake_case", "snake_case"),
            ("a___b", "a___b"),
            ("100% sure", "100%25 sure"),
            ("a%20b", "a%2520b"),
            ("aa?#/bbb", "aa%3F%23%2Fbbb"),
            ("a*b\"c<d>e|f\\g:h", "a%2Ab%22c%3Cd%3Ee%7Cf%5Cg%3Ah"),
            (".NET", ".NET"),
            ("CON", "CON"),
            ("Node.js", "Node.js"),
        ] {
            assert_eq!(page_file_name(title, Legacy), name, "{title}");
        }
    }

    /// The titles of Logseq's own round-trip test, `frontend.db.name-sanity-test`, and more.
    const TITLES: [&str; 31] = [
        "Fruit/Apple",
        "snake_case",
        "100% sure",
        "a%20b",
        "%",
        "%%41",
        ".hidden",
        "tag#one",
        "CON",
        "NUL",
        "COM1",
        "LPT9",
        "Some.Content!",
        "More _/_ Con tents",
        "More _________/________ Con tents",
        "More _________/___-_-_-_---___----__/_ Con tents",
        "Cont./__cont_ cont/ lsdksdf",
        "Cont.?/#__cont_ cont%/_ lsdksdf",
        "Cont.?__byte/#__cont_ cont%/_ lsdksdf",
        "__ont.?__byte/#__cont_ cont%/_ lsdksdf",
        "______ont.?__byte/#__cont_ cont%/_ lsdksdf",
        "__ont.?__byte/#__cont_ cont%/_ lsdksdf__",
        "+*++***+++__byte/#__cont_ cont%/_ lsdksdf__",
        "+*++_.x2A_.x2A***+++__byte/#__cont_ cont%/_ lsdksdf__",
        "__ont.?__byte/#__0xbbcont_ cont%/_ lsdksdf__",
        "__ont.?__byte&lowbar;/#_&amp;ont_ cont%/_ lsdksdf__",
        "dsa&amp&semi;l dsalfjk jkl",
        "dsa&amp&semi;l dsalfjk jkl.",
        "hls__&amp&semi;l dsalfjk jkl.",
        "CON.",
        ".NET.",
    ];

    fn read_back(title: &str, format: FilenameFormat) -> String {
        let source = "- text\n";
        let document = GardenDocument {
            relative_path: page_file_path(title, format).unwrap(),
            kind: DocumentKind::Page,
            source: source.to_owned(),
            parsed: parse_document(source),
            title_spellings: Default::default(),
        };
        page_title(&document, format).unwrap()
    }

    #[test]
    fn triple_lowbar_names_read_back_as_their_titles() {
        for title in TITLES {
            let name = page_file_name(title, TripleLowbar);
            assert_eq!(read_back(title, TripleLowbar), title, "{name}");
            assert!(!name.contains(RESERVED), "{name}");
            assert!(!name.starts_with('.') && !name.ends_with('.'), "{name}");
            assert!(!WINDOWS_RESERVED.contains(&name.as_str()), "{name}");
            assert!(!title_needs_property(title, TripleLowbar), "{title}");
        }
    }

    #[test]
    fn legacy_names_read_back_as_their_titles_unless_the_title_has_a_dot() {
        for title in TITLES {
            let name = page_file_name(title, Legacy);
            assert!(!name.contains(RESERVED) && !name.contains('/'), "{name}");
            if title.contains('.') {
                assert!(title_needs_property(title, Legacy), "{title}");
            } else {
                assert_eq!(read_back(title, Legacy), title, "{name}");
                assert!(!title_needs_property(title, Legacy), "{title}");
            }
        }
        assert_eq!(read_back("Node.js", Legacy), "Node/js");
    }

    #[test]
    fn paths_name_the_folder_and_the_extension() {
        assert_eq!(
            page_file_path("Fruit/Quince", TripleLowbar),
            Some("pages/Fruit___Quince.md".into())
        );
        assert_eq!(page_file_path("/", TripleLowbar), None);
        assert_eq!(page_file_path("  ", Legacy), None);
        let date = JournalDate::new(2026, 1, 4).unwrap();
        assert_eq!(
            journal_file_path(date, &JournalFormats::default()),
            PathBuf::from("journals/2026_01_04.md")
        );
    }
}
