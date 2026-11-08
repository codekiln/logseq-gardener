//! A query view's tree written as HTML. The SDK returns the tree; this module decides
//! what a published page may carry: a fixed list of text and link elements and
//! attributes, escaped text, and URLs that cannot run script. Anything else fails the
//! view and names the cause, so a garden author sees why.
use super::render::escape;
use crate::query_hiccup::Hiccup;

/// The most HTML one view may write, before gzip.
pub(super) const MAX_BYTES: usize = 128 * 1024;

const TAGS: &[&str] = &[
    "div",
    "span",
    "p",
    "ul",
    "ol",
    "li",
    "a",
    "b",
    "strong",
    "i",
    "em",
    "code",
    "pre",
    "br",
    "hr",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "table",
    "thead",
    "tbody",
    "tr",
    "th",
    "td",
    "blockquote",
    "small",
    "sub",
    "sup",
];
const VOID: &[&str] = &["br", "hr"];
const ATTRIBUTES: &[&str] = &[
    "href",
    "class",
    "id",
    "title",
    "rel",
    "target",
    "colspan",
    "rowspan",
    "aria-label",
];
const SCHEMES: &[&str] = &["http", "https", "mailto"];

/// Write `nodes` as HTML. `resolve` maps a page name from a `#/page/<name>` link to the
/// page's published URL, or nothing when no published page has that name.
pub(super) fn write(
    nodes: &[Hiccup],
    resolve: &dyn Fn(&str) -> Option<String>,
) -> Result<String, String> {
    let mut html = String::new();
    for node in nodes {
        node_html(node, resolve, &mut html)?;
    }
    Ok(html)
}

fn node_html(
    node: &Hiccup,
    resolve: &dyn Fn(&str) -> Option<String>,
    out: &mut String,
) -> Result<(), String> {
    if out.len() > MAX_BYTES {
        return Err(too_large());
    }
    match node {
        Hiccup::Text(text) => out.push_str(&escape(text)),
        Hiccup::Element {
            tag,
            attrs,
            children,
        } => {
            if !TAGS.contains(&tag.as_str()) {
                return Err(format!("the element <{tag}> is not allowed in a view"));
            }
            out.push('<');
            out.push_str(tag);
            for (name, value) in attrs {
                attribute(name, value, resolve, out)?;
            }
            out.push('>');
            if VOID.contains(&tag.as_str()) {
                return Ok(());
            }
            for child in children {
                node_html(child, resolve, out)?;
            }
            out.push_str("</");
            out.push_str(tag);
            out.push('>');
        }
    }
    if out.len() > MAX_BYTES {
        return Err(too_large());
    }
    Ok(())
}

fn too_large() -> String {
    format!(
        "the view wrote more than {} KB of HTML; narrow the query or use take",
        MAX_BYTES / 1024
    )
}

fn attribute(
    name: &str,
    value: &str,
    resolve: &dyn Fn(&str) -> Option<String>,
    out: &mut String,
) -> Result<(), String> {
    let allowed = ATTRIBUTES.contains(&name)
        || name.strip_prefix("data-").is_some_and(|rest| {
            !rest.is_empty()
                && rest
                    .chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_')
        });
    if !allowed {
        return Err(format!("the attribute {name} is not allowed in a view"));
    }
    let written = if name == "href" {
        href(value, resolve)?
    } else {
        value.to_owned()
    };
    out.push(' ');
    out.push_str(name);
    out.push_str("=\"");
    out.push_str(&escape(&written));
    out.push('"');
    Ok(())
}

/// A `#/page/<name>` link becomes the page's published URL when the page is published
/// and stays as written otherwise. Other links must be a fragment, a relative path, or an
/// `http`, `https` or `mailto` URL.
fn href(value: &str, resolve: &dyn Fn(&str) -> Option<String>) -> Result<String, String> {
    if let Some(name) = value.strip_prefix("#/page/") {
        return Ok(percent_decode(name)
            .and_then(|name| resolve(&name))
            .unwrap_or_else(|| value.to_owned()));
    }
    // Browsers ignore tabs, newlines and spaces inside a scheme, so read it without them.
    let compact: String = value
        .chars()
        .filter(|c| !c.is_ascii_control() && !c.is_whitespace())
        .collect();
    if let Some(colon) = compact.find(':') {
        let before = &compact[..colon];
        if !before.contains(['/', '?', '#']) {
            let scheme = before.to_ascii_lowercase();
            if !SCHEMES.contains(&scheme.as_str()) {
                return Err(format!(
                    "the link scheme {scheme}: is not allowed in a view"
                ));
            }
        }
    }
    Ok(value.to_owned())
}

/// The text a router reads from a percent-encoded name, or nothing for an invalid escape.
fn percent_decode(text: &str) -> Option<String> {
    let mut bytes = Vec::with_capacity(text.len());
    let mut source = text.bytes();
    while let Some(byte) = source.next() {
        if byte == b'%' {
            let high = (source.next()? as char).to_digit(16)?;
            let low = (source.next()? as char).to_digit(16)?;
            bytes.push((high * 16 + low) as u8);
        } else {
            bytes.push(byte);
        }
    }
    String::from_utf8(bytes).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn element(tag: &str, attrs: &[(&str, &str)], children: Vec<Hiccup>) -> Hiccup {
        Hiccup::Element {
            tag: tag.into(),
            attrs: attrs
                .iter()
                .map(|(k, v)| ((*k).into(), (*v).into()))
                .collect(),
            children,
        }
    }

    fn text(s: &str) -> Hiccup {
        Hiccup::Text(s.into())
    }

    fn pages(name: &str) -> Option<String> {
        (name == "tools/queries/typed/one").then(|| "p-one".to_owned())
    }

    #[test]
    fn writes_escaped_text_and_resolves_page_links() {
        let tree = [element(
            "ul",
            &[],
            vec![element(
                "li",
                &[("class", "a\"b")],
                vec![
                    element(
                        "a",
                        &[("href", "#/page/tools%2Fqueries%2Ftyped%2Fone")],
                        vec![text("A <b>&</b>")],
                    ),
                    element("br", &[], vec![]),
                ],
            )],
        )];
        assert_eq!(
            write(&tree, &pages).unwrap(),
            "<ul><li class=\"a&quot;b\"><a href=\"p-one\">A &lt;b&gt;&amp;&lt;/b&gt;</a><br></li></ul>"
        );
    }

    #[test]
    fn unmatched_page_links_stay_as_written() {
        let tree = [element(
            "a",
            &[("href", "#/page/Withheld")],
            vec![text("x")],
        )];
        assert_eq!(
            write(&tree, &pages).unwrap(),
            "<a href=\"#/page/Withheld\">x</a>"
        );
        let tree = [element("a", &[("href", "#/page/%zz")], vec![])];
        assert_eq!(write(&tree, &pages).unwrap(), "<a href=\"#/page/%zz\"></a>");
    }

    #[test]
    fn rejects_other_elements_attributes_and_schemes() {
        for (tree, message) in [
            (element("script", &[], vec![text("x")]), "<script>"),
            (element("style", &[], vec![]), "<style>"),
            (element("img", &[("src", "x")], vec![]), "<img>"),
            (element("p", &[("onclick", "x()")], vec![]), "onclick"),
            (element("p", &[("style", "color:red")], vec![]), "style"),
            (element("p", &[("data-", "x")], vec![]), "data-"),
            (element("p", &[("data-X", "x")], vec![]), "data-X"),
            (
                element("a", &[("href", "javascript:alert(1)")], vec![]),
                "javascript:",
            ),
            (
                element("a", &[("href", "JaVa\tScRiPt:alert(1)")], vec![]),
                "javascript:",
            ),
            (
                element("a", &[("href", "data:text/html,x")], vec![]),
                "data:",
            ),
        ] {
            let error = write(&[tree], &pages).unwrap_err();
            assert!(error.contains(message), "{error}");
        }
    }

    #[test]
    fn allows_fragments_relative_paths_data_attributes_and_allowed_schemes() {
        for href in [
            "#top",
            "page.html",
            "../x/y?q=a:b",
            "/abs",
            "https://example.com/a:b",
            "http://example.com",
            "mailto:a@example.com",
        ] {
            let tree = [element("a", &[("href", href), ("data-x-1", "v")], vec![])];
            assert!(write(&tree, &pages).is_ok(), "{href}");
        }
    }

    #[test]
    fn stops_at_the_size_limit() {
        let row = element("li", &[], vec![text(&"x".repeat(1000))]);
        let tree = [element("ul", &[], vec![row; 200])];
        let error = write(&tree, &pages).unwrap_err();
        assert!(error.contains("128 KB"), "{error}");
    }
}
