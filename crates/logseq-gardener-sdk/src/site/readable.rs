//! What a readable-URL build adds to the default site: the `<base>` and link script in each
//! document's head, and readable variants of the scripts that build page addresses.
//!
//! Every link the publisher writes is relative to the output root. A page in a folder carries
//! `<base>` so those links keep resolving there; fragment-only links and folder links opened
//! from disk need the small script below. The shared scripts stay byte for byte the same in
//! default builds: each variant replaces exact lines, and a test checks every line exists once.

use std::collections::BTreeMap;

use crate::page_urls::{base_href, is_readable};

/// `readable.js` without comments or indentation: keeps a plain click on a fragment-only link
/// in the current page while the document has a `<base>`, and, when the site is read from disk,
/// adds `index.html` to a link that ends in `/` as the reader follows it.
pub(super) fn script() -> String {
    super::references::minify(include_str!("readable.js"))
}

/// The tags a readable build adds right after `<meta charset>` of the document written at
/// `route`: `<base>` for a page in a folder, then `script`.
pub(super) fn head(route: &str, script: &str) -> String {
    let base = if is_readable(route) {
        format!("<base href=\"{}\">", base_href(route))
    } else {
        String::new()
    };
    format!("{base}<script data-lsg-readable>{script}</script>")
}

/// Replace each `(line, replacement)` pair once in `source`.
fn patch(source: &str, patches: &[(&str, &str)]) -> String {
    let mut text = source.to_owned();
    for (line, replacement) in patches {
        debug_assert_eq!(text.matches(line).count(), 1, "{line}");
        text = text.replacen(line, replacement, 1);
    }
    text
}

/// The outline script: zoom breadcrumbs link to the page's full address, and a journal's
/// bullet in the feed opens the journal's folder, adding `index.html` when read from disk.
const OUTLINE: &[(&str, &str)] = &[
    (
        "page.href = location.pathname.split('/').pop() || './';",
        "page.href = location.href.split('#')[0];",
    ),
    (
        "link.href = ZOOM + blockId(up);",
        "link.href = page.href + ZOOM + blockId(up);",
    ),
    (
        "if (page) location.href = page + ZOOM + id;",
        "if (page) { const url = new URL(page, document.baseURI); if (url.protocol === 'file:' && url.pathname.endsWith('/')) url.pathname += 'index.html'; location.href = url.href + ZOOM + id; }",
    ),
];

pub(super) fn outline(source: &str) -> String {
    patch(source, OUTLINE)
}

/// The graph script: a node opens its page's folder, and the page graph finds the current
/// page's title, and with it the neighbourhood file named by the title's digest. A page whose
/// title cannot name folders keeps its `p-<digest>.html` file in both.
const GRAPH: &[(&str, &str)] = &[
    (
        "  const routeOf = title => {
    let h = 0xcbf29ce484222325n;
    for (const byte of new TextEncoder().encode(title)) h = ((h ^ BigInt(byte)) * 0x100000001b3n) & 0xffffffffffffffffn;
    return 'p-' + h.toString(16).padStart(16, '0') + '.html';
  };",
        include_str!("readable-graph.js"),
    ),
    (
        "    let route = location.pathname.split('/').pop();
    if (!/^p-[0-9a-f]{16}\\.html$/.test(route)) route",
        "    const here = new URL(location.href);
    here.hash = '';
    here.search = '';
    let route = here.href.startsWith(base.href) ? here.href.slice(base.href.length).replace(/(^|\\/)index\\.html$/, '$1') : '';
    if (!route.endsWith('/') && !/^p-[0-9a-f]{16}\\.html$/.test(route)) route",
    ),
    (
        "'_graph/p/' + route.slice(2, 18) + '.json'",
        "'_graph/p/' + digestAt(route) + '.json'",
    ),
    (
        "for (let i = 0; i < global.n && centre < 0; i++) if (routeOf(global.title(i)) === route) centre = i;",
        "for (let i = 0; i < global.n && centre < 0; i++) if (isAt(global.title(i), route)) centre = i;",
    ),
];

/// The graph script, with `folders` from [`crate::publication::Publication::folders`] so that a
/// node under a folder that titles spell in more than one way opens that folder's one spelling.
pub(super) fn graph(source: &str, folders: &BTreeMap<String, String>) -> String {
    let pairs = serde_json::to_string(&folders.iter().collect::<Vec<_>>())
        .expect("folder spellings serialize");
    patch(source, GRAPH).replacen(
        "const FOLDERS = new Map([]);",
        &format!("const FOLDERS = new Map({pairs});"),
        1,
    )
}

/// The search engine: page routes may be readable folders, or digest files for titles that
/// cannot name folders.
const SEARCH: &[(&str, &str)] = &[(
    "/^p-[a-f0-9]{16}\\.html$/u",
    "/^(?:p-[a-f0-9]{16}\\.html|(?:(?:[\\w.~-]|%[0-9A-F]{2})+\\/)+)$/u",
)];

pub(super) fn search(source: &str) -> String {
    patch(source, SEARCH)
}

/// The plugin host: links to folders of the site are pages, the next page's `<base>` stays
/// out of the document, and the base is fixed to the absolute site root before the first
/// navigation changes the address.
const HOST: &[(&str, &str)] = &[
    ("/^([^/]+\\.html)?$/", "/^([^/]+\\.html|([^/]+\\/)+)?$/"),
    (
        "node.matches('title,meta,[data-lsg-page],[data-lsg-host]')",
        "node.matches('title,meta,base,[data-lsg-page],[data-lsg-host]')",
    ),
    (
        "  history.scrollRestoration = 'manual';",
        "  history.scrollRestoration = 'manual';
  (doc.querySelector('base') || doc.head.insertBefore(doc.createElement('base'), doc.head.firstChild)).href = base.href;",
    ),
];

pub(super) fn host(source: &str) -> String {
    patch(source, HOST)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_replaced_line_occurs_once() {
        for (name, source, patches) in [
            ("outline.js", include_str!("outline.js"), OUTLINE),
            ("graph.js", include_str!("graph.js"), GRAPH),
            ("search.mjs", include_str!("search.mjs"), SEARCH),
            ("host.mjs", include_str!("host.mjs"), HOST),
        ] {
            for (line, _) in patches {
                assert_eq!(source.matches(line).count(), 1, "{name}: {line}");
            }
        }
    }

    #[test]
    fn nested_pages_get_a_base_that_climbs_to_the_root() {
        assert!(
            head("A/B%20C/", "").starts_with("<base href=\"../../\"><script data-lsg-readable>")
        );
        assert!(head("index.html", "").starts_with("<script data-lsg-readable>"));
    }
}
