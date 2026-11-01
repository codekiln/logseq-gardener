//! How a publication names page files and the links to them.
//!
//! [`PageUrls::Digest`], the default, writes each page to `p-<digest of the title>.html` beside
//! `index.html`. [`PageUrls::Readable`] writes the page titled `GitP/A/Session` to
//! `GitP/A/Session/index.html` and links to it as `GitP/A/Session/`; [`readable_url`] gives that
//! address for any title.
//!
//! ```
//! use logseq_gardener_sdk::page_urls::readable_url;
//!
//! assert_eq!(readable_url("Notes/Q&A: 2026").unwrap(), "Notes/Q%26A%3A%202026/");
//! assert!(readable_url("Notes/../Secrets").is_err());
//! ```

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::path::PathBuf;

use unicode_normalization::UnicodeNormalization;

use crate::search::digest;

/// How a publication names page files and the links to them.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum PageUrls {
    /// `p-<digest of the title>.html` at the output root.
    #[default]
    Digest,
    /// `<title path>/index.html`, linked as the title path with each segment percent-encoded
    /// and a final `/`. Home, All pages, the journals feed and the graph view stay at the
    /// output root.
    Readable,
}

/// The longest file name, in UTF-8 bytes, that common file systems accept.
const MAX_SEGMENT: usize = 255;

/// A title that cannot have a readable URL, or pages whose readable paths collide.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PageUrlError {
    /// The title cannot form a safe path.
    Unsafe {
        /// The page's title.
        title: String,
        /// What makes the title unsafe.
        problem: &'static str,
    },
    /// Two pages' paths would be one file or folder on a case-insensitive file system.
    Collision {
        /// The title sorted first.
        first: String,
        /// The other title.
        second: String,
        /// How the paths collide.
        problem: String,
    },
    /// A page's first segment is a file or folder the build writes at the output root.
    Reserved {
        /// The page's title.
        title: String,
        /// The generated file or folder.
        name: String,
    },
}

impl fmt::Display for PageUrlError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unsafe { title, problem } => {
                write!(f, "page {title:?} cannot have a readable URL: {problem}")
            }
            Self::Collision {
                first,
                second,
                problem,
            } => write!(
                f,
                "pages {first:?} and {second:?} cannot both have readable URLs: {problem}"
            ),
            Self::Reserved { title, name } => write!(
                f,
                "page {title:?} cannot have a readable URL: the build writes {name} at the output root"
            ),
        }
    }
}

impl std::error::Error for PageUrlError {}

/// The readable URL of the page titled `title`, relative to the output root: each
/// slash-separated segment percent-encoded, then a final `/`.
///
/// Each segment keeps the RFC 3986 unreserved characters (`A`–`Z`, `a`–`z`, `0`–`9`, `-`,
/// `.`, `_`, `~`) and percent-encodes every other UTF-8 byte with upper-case hex digits.
///
/// # Errors
///
/// Returns [`PageUrlError::Unsafe`] when the title is empty, or a segment is empty, `.` or
/// `..`, starts or ends with whitespace, contains a backslash or a control character, or is
/// longer than 255 bytes in UTF-8.
pub fn readable_url(title: &str) -> Result<String, PageUrlError> {
    let unsafe_title = |problem| PageUrlError::Unsafe {
        title: title.to_owned(),
        problem,
    };
    if title.is_empty() {
        return Err(unsafe_title("the title is empty"));
    }
    if title.contains('\\') {
        return Err(unsafe_title("it contains a backslash"));
    }
    if title.chars().any(char::is_control) {
        return Err(unsafe_title("it contains a control character"));
    }
    let mut url = String::with_capacity(title.len() + 1);
    for segment in title.split('/') {
        if segment.is_empty() {
            return Err(unsafe_title("it has an empty segment"));
        }
        if segment == "." || segment == ".." {
            return Err(unsafe_title("a segment is . or .."));
        }
        if segment.trim() != segment {
            return Err(unsafe_title("a segment starts or ends with whitespace"));
        }
        if segment.len() > MAX_SEGMENT {
            return Err(unsafe_title(
                "a segment is longer than 255 bytes, the longest file name most file systems accept",
            ));
        }
        for byte in segment.bytes() {
            if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
                url.push(char::from(byte));
            } else {
                url.push_str(&format!("%{byte:02X}"));
            }
        }
        url.push('/');
    }
    Ok(url)
}

/// Whether `route` is a readable page route rather than a file name.
pub(crate) fn is_readable(route: &str) -> bool {
    route.ends_with('/')
}

/// The title a readable route was made from.
pub(crate) fn readable_title(route: &str) -> String {
    let encoded = route.strip_suffix('/').unwrap_or(route);
    let mut bytes = Vec::with_capacity(encoded.len());
    let mut rest = encoded.as_bytes();
    while let Some((&byte, tail)) = rest.split_first() {
        let hex = |digit: u8| char::from(digit).to_digit(16);
        if byte == b'%'
            && let [high, low, after @ ..] = tail
            && let (Some(high), Some(low)) = (hex(*high), hex(*low))
        {
            bytes.push(u8::try_from(high * 16 + low).expect("two hex digits fit a byte"));
            rest = after;
        } else {
            bytes.push(byte);
            rest = tail;
        }
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

/// Where a route's document is written, relative to the output root: a readable route's
/// folder `index.html`, otherwise the route itself.
pub(crate) fn output_path(route: &str) -> PathBuf {
    if is_readable(route) {
        let mut path: PathBuf = readable_title(route).split('/').collect();
        path.push("index.html");
        path
    } else {
        PathBuf::from(route)
    }
}

/// The stem that names a page's derived files, such as its reference fragments:
/// `p-<digest of the title>` in both layouts, so those files stay flat.
pub(crate) fn stem(route: &str) -> String {
    if is_readable(route) {
        format!("p-{}", digest(&readable_title(route)))
    } else {
        route.trim_end_matches(".html").to_owned()
    }
}

/// The `<base href>` that leads from a readable page's folder back to the output root.
pub(crate) fn base_href(route: &str) -> String {
    "../".repeat(route.matches('/').count())
}

/// How case-insensitive file systems compare names: NFC normalization and lower case.
fn folded(path: &str) -> String {
    path.nfc().collect::<String>().to_lowercase()
}

/// Two titles whose readable paths differ only in letter case or Unicode normalization.
pub(crate) fn same_path(first: &str, second: &str) -> PageUrlError {
    collision(
        first,
        second,
        "their paths differ only in letter case or Unicode normalization".to_owned(),
    )
}

/// A collision naming `first` and `second` in sorted order.
fn collision(first: &str, second: &str, problem: String) -> PageUrlError {
    let (first, second) = if first <= second {
        (first, second)
    } else {
        (second, first)
    };
    PageUrlError::Collision {
        first: first.to_owned(),
        second: second.to_owned(),
        problem,
    }
}

/// One spelling for each folder that `titles` spell in more than one way, by folded path. A
/// folder takes the spelling of the title that names it, when one does, and otherwise its lowest
/// spelling; each folder sits in its parent's spelling. Pass a title through [`respelled`] before
/// [`readable_url`] so that every page in a folder shares that folder's spelling.
pub(crate) fn folder_spellings<'t>(
    titles: impl IntoIterator<Item = &'t str>,
) -> BTreeMap<String, String> {
    let mut titles: Vec<&str> = titles.into_iter().collect();
    titles.sort_unstable();
    // Every folder by folded path: the name its last segment takes, and its spellings.
    let mut folders: BTreeMap<String, (&str, BTreeSet<&str>)> = BTreeMap::new();
    for &title in &titles {
        for (end, _) in title.match_indices('/').chain([(title.len(), "")]) {
            let folder = &title[..end];
            let name = folder.rsplit('/').next().unwrap_or(folder);
            let (lowest, spellings) = folders
                .entry(folded(folder))
                .or_insert((name, BTreeSet::new()));
            *lowest = (*lowest).min(name);
            spellings.insert(folder);
        }
    }
    for &title in &titles {
        if let Some((name, _)) = folders.get_mut(&folded(title)) {
            *name = title.rsplit('/').next().unwrap_or(title);
        }
    }
    let mut spellings = BTreeMap::new();
    for (key, (_, authored)) in &folders {
        let any = authored.first().expect("a folder has a spelling");
        let mut path = String::with_capacity(any.len());
        for (end, _) in any.match_indices('/').chain([(any.len(), "")]) {
            if !path.is_empty() {
                path.push('/');
            }
            path.push_str(folders[&folded(&any[..end])].0);
        }
        if authored.iter().any(|&spelling| spelling != path) {
            spellings.insert(key.clone(), path);
        }
    }
    spellings
}

/// `title` with its longest folder that [`folder_spellings`] lists in that folder's spelling.
pub(crate) fn respelled(title: &str, spellings: &BTreeMap<String, String>) -> String {
    for (end, _) in [(title.len(), "")]
        .into_iter()
        .chain(title.rmatch_indices('/'))
    {
        if let Some(spelling) = spellings.get(&folded(&title[..end])) {
            return format!("{spelling}{}", &title[end..]);
        }
    }
    title.to_owned()
}

/// Check that the readable paths of `titles` stay distinct on a case-insensitive file
/// system: no two titles share a folded path, and no title's folder is another title's
/// `index.html`. Folders that titles spell differently are one folder in the spelling
/// [`folder_spellings`] gives.
///
/// # Errors
///
/// Returns the first [`PageUrlError::Collision`], naming both titles.
pub fn check_collisions<'t>(titles: impl IntoIterator<Item = &'t str>) -> Result<(), PageUrlError> {
    let mut titles: Vec<&str> = titles.into_iter().collect();
    titles.sort_unstable();
    let mut pages: BTreeMap<String, &str> = BTreeMap::new();
    for &title in &titles {
        if let Some(other) = pages.insert(folded(title), title) {
            return Err(same_path(other, title));
        }
    }
    for &title in &titles {
        if let Some((parent, name)) = title.rsplit_once('/')
            && folded(name) == "index.html"
            && let Some(&owner) = pages.get(&folded(parent))
        {
            return Err(collision(
                owner,
                title,
                format!("the folder of {title:?} is the index.html of {owner:?}"),
            ));
        }
    }
    Ok(())
}

/// Check that no readable page's first segment is a file or folder the build writes at the
/// output root, comparing folded names.
///
/// # Errors
///
/// Returns [`PageUrlError::Reserved`] naming the first such page and the generated name.
pub(crate) fn check_reserved<'t>(
    titles: impl IntoIterator<Item = &'t str>,
    generated: impl IntoIterator<Item = &'t str>,
) -> Result<(), PageUrlError> {
    let generated: BTreeMap<String, &str> = generated
        .into_iter()
        .map(|name| (folded(name), name))
        .collect();
    let mut titles: Vec<&str> = titles.into_iter().collect();
    titles.sort_unstable();
    for title in titles {
        let first = title.split('/').next().unwrap_or(title);
        if let Some(name) = generated.get(&folded(first)) {
            return Err(PageUrlError::Reserved {
                title: title.to_owned(),
                name: (*name).to_owned(),
            });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn readable_urls_encode_each_segment() {
        assert_eq!(
            readable_url("GitP/A/Session/26/10/20-Tue").unwrap(),
            "GitP/A/Session/26/10/20-Tue/"
        );
        assert_eq!(
            readable_url("Notes/Q&A: 2026").unwrap(),
            "Notes/Q%26A%3A%202026/"
        );
        assert_eq!(readable_url("a~b_c.d-e").unwrap(), "a~b_c.d-e/");
        assert_eq!(
            readable_url("!*'()#?%+").unwrap(),
            "%21%2A%27%28%29%23%3F%25%2B/"
        );
        assert_eq!(
            readable_url("Café/日本").unwrap(),
            "Caf%C3%A9/%E6%97%A5%E6%9C%AC/"
        );
    }

    #[test]
    fn unsafe_titles_are_refused() {
        for (title, problem) in [
            ("", "the title is empty"),
            ("A//B", "it has an empty segment"),
            ("/A", "it has an empty segment"),
            ("A/", "it has an empty segment"),
            ("A/./B", "a segment is . or .."),
            ("A/..", "a segment is . or .."),
            ("A/ B", "a segment starts or ends with whitespace"),
            ("A /B", "a segment starts or ends with whitespace"),
            ("A\\B", "it contains a backslash"),
            ("A\u{7}B", "it contains a control character"),
            ("A\u{85}B", "it contains a control character"),
        ] {
            assert_eq!(
                readable_url(title),
                Err(PageUrlError::Unsafe {
                    title: title.to_owned(),
                    problem
                }),
                "{title:?}"
            );
        }
        assert!(readable_url(&"x".repeat(255)).is_ok());
        assert!(matches!(
            readable_url(&format!("A/{}", "é".repeat(128))),
            Err(PageUrlError::Unsafe { .. })
        ));
        assert!(readable_url("...").is_ok());
    }

    #[test]
    fn readable_routes_map_back_to_titles_files_and_stems() {
        for title in ["GitP/A/Session", "Notes/Q&A: 2026", "Café/100%"] {
            let route = readable_url(title).unwrap();
            assert_eq!(readable_title(&route), title);
            assert_eq!(stem(&route), format!("p-{}", digest(title)));
        }
        let route = readable_url("Notes/Q&A: 2026").unwrap();
        assert_eq!(
            output_path(&route),
            PathBuf::from("Notes").join("Q&A: 2026").join("index.html")
        );
        assert_eq!(base_href(&route), "../../");
        assert_eq!(output_path("p-1.html"), PathBuf::from("p-1.html"));
        assert_eq!(stem("p-1.html"), "p-1");
    }

    #[test]
    fn collisions_name_both_pages() {
        let error = check_collisions(["Tools/Vim", "tools/vim"]).unwrap_err();
        assert_eq!(
            error.to_string(),
            "pages \"Tools/Vim\" and \"tools/vim\" cannot both have readable URLs: their paths differ only in letter case or Unicode normalization"
        );
        assert!(check_collisions(["Caf\u{e9}", "Cafe\u{301}"]).is_err());
        assert!(check_collisions(["Tools/A", "tools/B"]).is_ok());
        let error = check_collisions(["A", "A/index.html"]).unwrap_err();
        assert!(error.to_string().contains("index.html of \"A\""), "{error}");
        assert!(check_collisions(["A", "A/B", "A/B/C", "B"]).is_ok());
        assert!(check_collisions(["A/index.html"]).is_ok());
    }

    #[test]
    fn folders_spelled_two_ways_take_one_spelling() {
        let titles = [
            "CLAUDE",
            "Claude/Artifact",
            "tmux/Session/Picker",
            "tmux/session/Window",
            "tmux/session/Window/Pane",
            "Tools/A",
        ];
        let spellings = folder_spellings(titles);
        assert_eq!(
            spellings,
            BTreeMap::from(
                [
                    ("claude", "CLAUDE"),
                    ("claude/artifact", "CLAUDE/Artifact"),
                    ("tmux/session", "tmux/Session"),
                    ("tmux/session/window", "tmux/Session/Window"),
                    ("tmux/session/window/pane", "tmux/Session/Window/Pane"),
                ]
                .map(|(key, spelling)| (key.to_owned(), spelling.to_owned()))
            )
        );
        assert_eq!(respelled("Claude/Artifact", &spellings), "CLAUDE/Artifact");
        assert_eq!(respelled("claude/X/Y", &spellings), "CLAUDE/X/Y");
        assert_eq!(
            respelled("tmux/session/Window/Pane", &spellings),
            "tmux/Session/Window/Pane"
        );
        assert_eq!(respelled("Tools/A", &spellings), "Tools/A");
        assert!(folder_spellings(["A", "A/B", "B/C"]).is_empty());
    }

    #[test]
    fn reserved_root_names_are_refused() {
        let error = check_reserved(["Assets/Logo", "Tools"], ["assets", "index.html"]).unwrap_err();
        assert_eq!(
            error,
            PageUrlError::Reserved {
                title: "Assets/Logo".to_owned(),
                name: "assets".to_owned()
            }
        );
        assert!(check_reserved(["Tools"], ["assets", "index.html"]).is_ok());
    }
}
