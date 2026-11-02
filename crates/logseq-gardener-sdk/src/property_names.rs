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
}
