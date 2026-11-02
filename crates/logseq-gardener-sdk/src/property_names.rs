//! Which property names Logseq's file parser keeps.
//!
//! The Markdown parser reads `key:: value` for any key without spaces or colons, so a
//! block that starts with the code `` `status:: [[Done]]` `` holds a property named
//! `` `status ``. Logseq's graph parser then keeps only the names that
//! `valid-property-name?` accepts (`deps/graph-parser/src/logseq/graph_parser/property.cljs`):
//! the name must read as an EDN keyword, must not contain `"`, `|`, `^`, `(`, `)`, `{` or
//! `}`, and must not start with `#`. Logseq drops every other property with its value's
//! page references, and shows the name in a warning instead of a property row.
//!
//! ```
//! use logseq_gardener_sdk::property_names::valid_property_name;
//!
//! assert!(valid_property_name("status"));
//! assert!(!valid_property_name("`status"));
//! ```

use crate::ast::Block;

/// Whether Logseq's graph parser keeps a property with this name.
///
/// Logseq reads `{:<name> nil}` with ClojureScript's EDN reader. The name's first EDN
/// token must be a keyword name that the reader accepts; text after that token counts
/// only when it is EDN whitespace, such as the comma in `done,`. Logseq also accepts a
/// few names whose remaining text reads as further EDN forms, such as `a,b,c`; this
/// function rejects them.
pub fn valid_property_name(name: &str) -> bool {
    if name.is_empty()
        || name.starts_with('#')
        || name.contains(['"', '|', '^', '(', ')', '{', '}'])
    {
        return false;
    }
    let end = name
        .find(|c: char| edn_whitespace(c) || edn_terminating(c))
        .unwrap_or(name.len());
    let (token, rest) = name.split_at(end);
    !token.is_empty()
        && !token.contains(['@', '`', '~'])
        && edn_keyword_name(token)
        && rest.chars().all(edn_whitespace)
}

/// The property names Logseq rejects in a properties block, as Logseq's warning
/// "Invalid property names" lists them.
///
/// [`crate::garden::parse_document`] drops these properties from `block`, so this reads
/// them from the block's lines in `source`, the text of the document that holds it. Logseq
/// lower-cases each name and turns spaces and underscores into hyphens; a name that
/// repeats is listed once, in the order the block first gives it. A block other than
/// [`Block::Properties`] has none.
///
/// ```
/// use logseq_gardener_sdk::{garden::parse_document, property_names::rejected_property_names};
///
/// let source = "- `Status:: [[Done]]` marks a finished task.\n";
/// let parsed = parse_document(source);
/// let names: Vec<_> = parsed
///     .blocks
///     .iter()
///     .flat_map(|block| rejected_property_names(source, block))
///     .collect();
/// assert_eq!(names, ["`status"]);
/// ```
pub fn rejected_property_names(source: &str, block: &Block) -> Vec<String> {
    let Block::Properties {
        span: Some(span), ..
    } = block
    else {
        return Vec::new();
    };
    let Some(text) = source.get(span.0..span.1.min(source.len())) else {
        return Vec::new();
    };
    let mut names: Vec<String> = Vec::new();
    for line in text.as_bytes().split(|&b| b == b'\n') {
        if let Some((key, _)) = rejected_property_line(line) {
            let name = key.to_lowercase().replace([' ', '_'], "-");
            if !names.contains(&name) {
                names.push(name);
            }
        }
    }
    names
}

/// The key and the byte where the value starts in a `key:: value` line whose key Logseq
/// rejects, read as the Markdown parser reads a property line: spaces, a key with no
/// spaces or colons, then `::`.
pub(crate) fn rejected_property_line(line: &[u8]) -> Option<(&str, usize)> {
    let indent = line
        .iter()
        .take_while(|&&b| matches!(b, b' ' | b'\t' | 0x1a | 0x0c))
        .count();
    let colons = indent + line[indent..].windows(2).position(|pair| pair == b"::")?;
    let key = &line[indent..colons];
    if key.is_empty()
        || key
            .iter()
            .any(|&b| matches!(b, b' ' | b'\t' | 0x1a | 0x0c | b'\r' | b':'))
    {
        return None;
    }
    let key = std::str::from_utf8(key).ok()?;
    (!valid_property_name(key)).then_some((key, colons + 2))
}

/// Whether ClojureScript's `parse-symbol` accepts a keyword's token: an optional
/// namespace before the first `/`, then a name that does not start with a digit and holds
/// no further `/`.
fn edn_keyword_name(token: &str) -> bool {
    match token.split_once('/') {
        None => true,
        Some(("", _)) => token == "/",
        Some((_, name)) => {
            !name.is_empty()
                && !name.starts_with(|c: char| c.is_ascii_digit())
                && (name == "/" || !name.contains('/'))
        }
    }
}

/// The EDN reader's whitespace: JavaScript's `\s` and the comma.
fn edn_whitespace(c: char) -> bool {
    matches!(
        c,
        ',' | '\t' | '\n' | '\u{0B}' | '\u{0C}' | '\r' | ' ' | '\u{A0}' | '\u{1680}' | '\u{2000}'
            ..='\u{200A}'
                | '\u{2028}'
                | '\u{2029}'
                | '\u{202F}'
                | '\u{205F}'
                | '\u{3000}'
                | '\u{FEFF}'
    )
}

/// Characters that end an EDN token: the reader's macro characters other than `#`, `'`
/// and `:`.
fn edn_terminating(c: char) -> bool {
    matches!(
        c,
        '"' | ';' | '^' | '(' | ')' | '[' | ']' | '{' | '}' | '\\'
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Names and whether Logseq keeps them, from `valid-property-name?` run with
    /// nbb-logseq on Logseq's graph parser at the revision in
    /// `scripts/logseq-parser/source.json`.
    const LOGSEQ: &[(&str, bool)] = &[
        ("status", true),
        ("`status", false),
        ("status`", false),
        ("a`b", false),
        ("~x", false),
        ("x~", false),
        ("@x", false),
        ("x@", false),
        ("a,b", false),
        (",x", false),
        ("x,", true),
        ("x[a]", false),
        ("x]", false),
        ("x;", false),
        ("x\\a", false),
        ("a/b", true),
        ("a/b/c", false),
        ("a/1", false),
        ("/a", false),
        ("a/", false),
        ("/", true),
        ("a//b", false),
        ("#tag", false),
        ("a#b", true),
        ("x'", true),
        ("'x", true),
        ("1x", true),
        ("-1x", true),
        ("+1", true),
        (".5", true),
        ("a.b", true),
        ("*x*", true),
        ("x<y", true),
        ("x=y", true),
        ("x&y", true),
        ("x%", true),
        ("x!", true),
        ("x?", true),
        ("x$", true),
        ("Ü", true),
        ("日本", true),
        ("x\"", false),
        ("x|y", false),
        ("x^", false),
        ("x(", false),
        ("x)", false),
        ("x{", false),
        ("x}", false),
        ("x[]", false),
        ("a_b", true),
        ("A-B", true),
    ];

    #[test]
    fn keeps_the_names_logseq_keeps() {
        for &(name, keeps) in LOGSEQ {
            assert_eq!(valid_property_name(name), keeps, "{name}");
        }
    }

    #[test]
    fn rejects_names_whose_rest_reads_as_more_edn_forms() {
        // Logseq keeps these three, because `{:a,b,c nil}` reads as a map of two entries.
        for name in ["a,b,c", "x[a]b", "x\\a\\b"] {
            assert!(!valid_property_name(name), "{name}");
        }
    }

    #[test]
    fn lists_the_rejected_names_as_logseq_shows_them() {
        let source = "- `Logseq_Entity:: [[Asset]]` names the type.\n  kind:: note\n  `logseq entity:: x\n  `owner:: [[AWS]]`\n  `OWNER:: y\n";
        let parsed = crate::garden::parse_document(source);
        let names: Vec<_> = parsed
            .blocks
            .iter()
            .flat_map(|block| rejected_property_names(source, block))
            .collect();
        assert_eq!(names, ["`logseq-entity", "`owner"]);
    }

    #[test]
    fn a_block_of_accepted_names_lists_none() {
        let source = "- note\n  kind:: note\n  status:: done\n";
        let parsed = crate::garden::parse_document(source);
        assert!(
            parsed
                .blocks
                .iter()
                .all(|block| rejected_property_names(source, block).is_empty())
        );
    }
}
