//! What `--redirect-old-urls` adds to a site: a file at each page's old `p-<digest>.html` name
//! that opens its readable URL, and a script in `index.html` that opens the page a Logseq
//! `#/page/<name>` link names.

use std::collections::BTreeMap;

use crate::page_names::lookup_key;
use crate::search::digest;

use super::SiteError;
use super::render::escape;

/// The name map that the `index.html` script fetches: page name to route.
pub(super) const MAP_ROUTE: &str = "lsg-old-urls.json";

/// The redirects of a build: generated files by route, and the script for `index.html`.
pub(super) struct Redirects {
    pub(super) files: BTreeMap<String, String>,
    pub(super) script: String,
}

/// Plan the redirects for `pages`, the `(title, route)` of every published page, journal and
/// namespace page, and `aliases`, alias names with the routes they name. Digest files are
/// written only for pages that do not already live at theirs, and only with readable URLs.
///
/// # Errors
///
/// Returns [`SiteError::Collision`] when a digest file would share its name with a generated
/// file in `generated`, or two pages would share one.
pub(super) fn plan<'a>(
    pages: impl IntoIterator<Item = (&'a str, &'a str)>,
    aliases: impl IntoIterator<Item = (String, &'a [String])>,
    readable: bool,
    all_pages: &str,
    graph: &str,
    generated: impl Fn(&str) -> bool,
) -> Result<Redirects, SiteError> {
    let mut files = BTreeMap::new();
    let mut names = BTreeMap::new();
    for (title, route) in pages {
        names.insert(lookup_key(title), route.to_owned());
        let old = format!("p-{}.html", digest(title));
        if !readable || old == route {
            continue;
        }
        if generated(&old) || files.insert(old.clone(), page(title, route)).is_some() {
            return Err(SiteError::Collision {
                title: format!("{title} (redirect file {old})"),
            });
        }
    }
    for (alias, routes) in aliases {
        if let [route] = routes {
            names.entry(alias).or_insert_with(|| route.clone());
        }
    }
    let json = |value: &str| serde_json::to_string(value).expect("a string serializes");
    let script = super::references::minify(include_str!("old-urls.js"))
        .replace("__ALL_PAGES__", &json(all_pages))
        .replace("__GRAPH__", &json(graph))
        .replace("__MAP__", &json(MAP_ROUTE));
    files.insert(
        MAP_ROUTE.to_owned(),
        serde_json::to_string(&names).expect("a map of strings serializes"),
    );
    Ok(Redirects {
        files,
        script: format!("<script data-lsg-old-urls>{script}</script>"),
    })
}

/// A file that opens `route` and keeps the fragment, and without JavaScript opens `route`.
fn page(title: &str, route: &str) -> String {
    let href = escape(route);
    let target = serde_json::to_string(route)
        .expect("a string serializes")
        .replace('<', "\\u003c");
    let title = escape(title);
    format!(
        "<!doctype html>\n<html lang=\"en\"><head><meta charset=\"utf-8\"><title>{title}</title><meta name=\"robots\" content=\"noindex\"><link rel=\"canonical\" href=\"{href}\"><meta http-equiv=\"refresh\" content=\"0; url={href}\"><script>location.replace({target} + location.hash)</script></head><body><a href=\"{href}\">{title}</a></body></html>\n"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn none(_: &str) -> bool {
        false
    }

    #[test]
    fn digest_file_for_each_readable_page() {
        let redirects = plan(
            [("GitP/A/Session", "GitP/A/Session/")],
            [],
            true,
            "pages.html",
            "graph.html",
            none,
        )
        .unwrap();
        let old = format!("p-{}.html", digest("GitP/A/Session"));
        let file = &redirects.files[&old];
        assert!(file.contains("location.replace(\"GitP/A/Session/\" + location.hash)"));
        assert!(file.contains("<link rel=\"canonical\" href=\"GitP/A/Session/\">"));
        assert!(file.contains("content=\"0; url=GitP/A/Session/\""));
        assert!(file.contains("noindex"));
        assert_eq!(
            redirects.files[MAP_ROUTE],
            r#"{"gitp/a/session":"GitP/A/Session/"}"#
        );
    }

    #[test]
    fn no_digest_files_without_readable_urls_or_at_the_digest_file() {
        let title = "Notes/Rename with /rename";
        let route = format!("p-{}.html", digest(title));
        let readable = plan([(title, route.as_str())], [], true, "a", "b", none).unwrap();
        assert_eq!(readable.files.len(), 1);
        let default = plan([("A", "p-1.html")], [], false, "a", "b", none).unwrap();
        assert_eq!(default.files.len(), 1);
    }

    #[test]
    fn aliases_naming_one_page() {
        let one = vec!["A/".to_owned()];
        let two = vec!["A/".to_owned(), "B/".to_owned()];
        let redirects = plan(
            [("A", "A/"), ("B", "B/")],
            [
                ("first".to_owned(), one.as_slice()),
                ("both".to_owned(), two.as_slice()),
                ("b".to_owned(), one.as_slice()),
            ],
            true,
            "a",
            "b",
            none,
        )
        .unwrap();
        assert_eq!(
            redirects.files[MAP_ROUTE],
            r#"{"a":"A/","b":"B/","first":"A/"}"#
        );
    }

    #[test]
    fn collision_with_a_generated_file() {
        let old = format!("p-{}.html", digest("A"));
        let error = plan([("A", "A/")], [], true, "a", "b", |route| route == old)
            .err()
            .unwrap();
        assert!(error.to_string().contains(&old), "{error}");
    }

    #[test]
    fn script_names_lists_and_map() {
        let redirects = plan([], [], true, "pages.html", "graph.html", none).unwrap();
        assert!(
            redirects
                .script
                .contains("location.replace(\"pages.html\")")
        );
        assert!(
            redirects
                .script
                .contains("location.replace(\"graph.html\")")
        );
        assert!(redirects.script.contains("fetch(\"lsg-old-urls.json\")"));
        assert!(!redirects.script.contains("__"));
    }
}
