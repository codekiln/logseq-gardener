//! Links from help text to pages on the documentation site.
//!
//! The site publishes the `docs` graph with readable page URLs, so a page's address is the base
//! URL followed by the route that the SDK's [`readable_url`] gives the page's title. Help prints
//! these addresses and never fetches them.

use logseq_gardener_sdk::page_urls::readable_url;

/// The documentation site, published from the `docs` graph on `main`.
const BASE_URL: &str = "https://codekiln.github.io/logseq-gardener/";

/// The documentation site address of the page titled `title`.
///
/// # Panics
///
/// Panics when `title` cannot have a readable URL. Help links only fixed page titles, and the
/// help tests render every link.
pub fn page_url(title: &str) -> String {
    let route = readable_url(title)
        .unwrap_or_else(|error| panic!("help links a page with no readable URL: {error}"));
    format!("{BASE_URL}{route}")
}

/// The page title followed by its address in parentheses, for a "See ..." sentence in help.
pub fn page_link(title: &str) -> String {
    format!("{title} ({})", page_url(title))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_url_uses_the_readable_route() {
        assert_eq!(
            page_url("lsg/Ref/Static Search Data"),
            format!("{BASE_URL}lsg/Ref/Static%20Search%20Data/")
        );
    }

    #[test]
    fn page_link_names_the_title_and_its_url() {
        assert_eq!(
            page_link("lsg/Ref/Publish Options"),
            format!("lsg/Ref/Publish Options ({BASE_URL}lsg/Ref/Publish%20Options/)")
        );
    }
}
