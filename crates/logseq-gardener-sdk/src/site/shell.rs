//! Logseq's header and left sidebar around every generated page.
use super::render::escape;
use crate::garden_settings::Favorite;

/// Shared behaviour file written once at the site root.
pub(super) const SCRIPT_ROUTE: &str = "lsg-shell.js";
/// `shell.js` without comments or indentation.
pub(super) fn script() -> String {
    super::references::minify(include_str!("shell.js"))
}
/// The right sidebar, loaded on first use.
pub(super) const SIDEBAR_ROUTE: &str = "lsg-sidebar.js";
pub(super) const SIDEBAR_STYLES_ROUTE: &str = "lsg-sidebar.css";
pub(super) const SIDEBAR_STYLES: &str = include_str!("sidebar.css");

/// `sidebar.js` without indentation or comment lines, which keeps it within its gzip budget.
pub(super) fn sidebar_script() -> String {
    compact(include_str!("sidebar.js"))
}

/// View as slides, loaded with the first Slides card.
pub(super) const SLIDES_ROUTE: &str = "lsg-slides.js";

/// `slides.js` with `slides.css` inside it, compacted like the sidebar script, so a deck
/// costs one request.
pub(super) fn slides_script() -> String {
    let styles = include_str!("slides.css").replace('\n', "");
    debug_assert!(!styles.contains(['\'', '\\']));
    compact(include_str!("slides.js")).replace("__LSG_SLIDES_CSS__", &styles)
}

fn compact(script: &str) -> String {
    script
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

// Tabler Icons outline paths (MIT); see THIRD_PARTY_NOTICES.md.
const MENU: &str = "M4 6l16 0M4 12l16 0M4 18l16 0";
const HOME: &str = "M5 12l-2 0l9 -9l9 9l-2 0M5 12v7a2 2 0 0 0 2 2h10a2 2 0 0 0 2 -2v-7M9 21v-6a2 2 0 0 1 2 -2h2a2 2 0 0 1 2 2v6";
const DOTS: &str = "M4 12m-1 0a1 1 0 1 0 2 0a1 1 0 1 0 -2 0M12 12m-1 0a1 1 0 1 0 2 0a1 1 0 1 0 -2 0M20 12m-1 0a1 1 0 1 0 2 0a1 1 0 1 0 -2 0";
const SEARCH: &str = "M10 10m-7 0a7 7 0 1 0 14 0a7 7 0 1 0 -14 0M21 21l-6 -6";
const CALENDAR: &str = "M4 7a2 2 0 0 1 2 -2h12a2 2 0 0 1 2 2v12a2 2 0 0 1 -2 2h-12a2 2 0 0 1 -2 -2v-12zM16 3v4M8 3v4M4 11h16M11 15h1M12 15v3";
const SIDEBAR_RIGHT: &str =
    "M4 6a2 2 0 0 1 2 -2h12a2 2 0 0 1 2 2v12a2 2 0 0 1 -2 2h-12a2 2 0 0 1 -2 -2zM15 4l0 16";
const HIERARCHY: &str = "M10 5a2 2 0 1 0 4 0a2 2 0 1 0-4 0M3 19a2 2 0 1 0 4 0a2 2 0 1 0-4 0M17 19a2 2 0 1 0 4 0a2 2 0 1 0-4 0M6.5 17.5 12 13l5.5 4.5M12 7v6";
const FILES: &str = "M15 3v4a1 1 0 0 0 1 1h4M18 17h-7a2 2 0 0 1 -2 -2v-10a2 2 0 0 1 2 -2h4l5 5v7a2 2 0 0 1 -2 2zM16 17v2a2 2 0 0 1 -2 2h-7a2 2 0 0 1 -2 -2v-10a2 2 0 0 1 2 -2h2";

pub(super) const STYLES: &str = concat!(
    ".lsg-header{position:sticky;top:0;z-index:20;display:flex;align-items:center;gap:4px;height:48px;padding:0 8px;box-sizing:border-box;border-bottom:1px solid var(--lsg-border);background:var(--lsg-header)}",
    ".lsg-header-space{flex:1}",
    ".lsg-button{display:inline-flex;align-items:center;justify-content:center;width:36px;height:36px;padding:0;border:0;border-radius:6px;background:none;color:var(--lsg-muted);cursor:pointer}",
    ".lsg-button:hover{background:var(--lsg-surface);color:var(--lsg-text)}.lsg-button[hidden]{display:none}",
    ".lsg-icon{width:20px;height:20px;flex:none;fill:none;stroke:currentColor;stroke-width:2;stroke-linecap:round;stroke-linejoin:round}",
    ".lsg-more{position:relative}",
    "#lsg-more-menu{position:absolute;right:0;top:42px;z-index:40;min-width:12rem;padding:4px;background:var(--lsg-background);border:1px solid var(--lsg-border);border-radius:8px;box-shadow:0 4px 16px rgba(0,0,0,.15)}",
    "#lsg-more-menu button{display:block;width:100%;padding:6px 10px;border:0;border-radius:4px;background:none;color:var(--lsg-text);font:inherit;font-size:14px;text-align:left;cursor:pointer}",
    "#lsg-more-menu button:is(:hover,:focus){background:var(--lsg-surface)}#lsg-more-menu[hidden],#lsg-more-menu button[hidden]{display:none}",
    ".lsg-left-sidebar{display:none;position:fixed;top:48px;bottom:0;left:0;z-index:30;width:74vw;overflow-y:auto;padding:12px;box-sizing:border-box;background:var(--lsg-sidebar);font-size:14px}",
    "html[data-lsg-left-sidebar=open] .lsg-left-sidebar{display:block}",
    ".lsg-left-sidebar ul{list-style:none;margin:0;padding:0}.lsg-left-sidebar li{margin:0}",
    ".lsg-left-sidebar a{display:flex;align-items:center;gap:8px;padding:6px 8px;border-radius:6px;color:var(--lsg-text);text-decoration:none;overflow-wrap:anywhere}",
    ".lsg-left-sidebar a:hover{background:var(--lsg-surface)}.lsg-left-sidebar a[aria-current=page]{background:var(--lsg-surface);font-weight:600}",
    ".lsg-left-sidebar ul[aria-labelledby] a{display:block;padding-left:36px;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}.lsg-favorites-heading{margin:20px 8px 6px;color:var(--lsg-muted);font-size:12px;font-weight:600;letter-spacing:.05em;text-transform:uppercase}",
    ".lsg-shortcut{margin-left:auto;color:var(--lsg-muted);font-size:12px;letter-spacing:.1em}",
    ".lsg-badge{margin-left:4px;padding:0 12px;border-radius:9999px;background:var(--lsg-surface);font-size:12px;font-weight:500}",
    ".lsg-sidebar-shade{position:fixed;inset:0;z-index:25;background:rgba(0,0,0,.4)}.lsg-sidebar-shade[hidden]{display:none}",
    "@media(max-width:639.98px){.lsg-right-toggle,[data-lsg-slides]{display:none!important}}@media(min-width:640px){html[data-lsg-right-sidebar=open] body{padding-right:var(--lsg-right-width,40vw)}}",
    "@media(min-width:640px){.lsg-left-sidebar{z-index:10;width:246px}html[data-lsg-left-sidebar=open] body{padding-left:246px}html[data-lsg-left-sidebar=open] .lsg-header{margin-left:-246px}.lsg-sidebar-shade{display:none!important}}",
);

/// Navigation that is the same on every page apart from the current-page marks.
pub(super) struct Shell {
    /// `g g` opens the graph view; `:shortcuts {:go/graph-view false}` turns it off.
    pub graph_shortcut: bool,
    /// Resolved favorites: output route and original title, in config order.
    pub favorites: Vec<Favorite<String>>,
    /// Route of the complete page index.
    pub all_pages: &'static str,
    /// Route of the journals feed's first batch, when the feed is written.
    pub journals: Option<&'static str>,
    /// Route of the published `contents` page, shown by the Contents card.
    pub contents: Option<String>,
    /// Right sidebar cards for a first visit: `contents` or page routes.
    pub sidebar: Vec<String>,
    /// Cards due at the build time; `None` when flashcards are turned off.
    pub flashcards: Option<usize>,
    /// Whether page Properties are enabled for this publication.
    pub properties: bool,
}

fn icon(path: &str) -> String {
    format!(
        "<svg class=\"lsg-icon\" viewBox=\"0 0 24 24\" aria-hidden=\"true\"><path d=\"{path}\"/></svg>"
    )
}

fn current(route: &str, page: &str) -> &'static str {
    if route == page {
        " aria-current=\"page\""
    } else {
        ""
    }
}

impl Shell {
    /// Header, sidebar and shade for the page at `route`; `home` marks index.html.
    pub(super) fn render(&self, route: &str, home: bool) -> String {
        let home_link = if home {
            String::new()
        } else {
            format!(
                "<a class=\"lsg-button\" href=\"index.html\" aria-label=\"Home\">{}</a>",
                icon(HOME)
            )
        };
        let mut favorites = String::new();
        if !self.favorites.is_empty() {
            favorites.push_str(
                "<h2 class=\"lsg-favorites-heading\" id=\"lsg-favorites\">Favorites</h2><ul aria-labelledby=\"lsg-favorites\">",
            );
            for favorite in &self.favorites {
                favorites.push_str(&format!(
                    "<li><a href=\"{}\"{}>{}</a></li>",
                    escape(&favorite.page),
                    current(route, &favorite.page),
                    escape(&favorite.title)
                ));
            }
            favorites.push_str("</ul>");
        }
        let journals = self.journals.map_or_else(String::new, |first| {
            let feed =
                route == first || (route.starts_with("journals-") && route.ends_with(".html"));
            format!(
                "<li><a href=\"{first}\" data-lsg-journals-link{}>{}<span>Journals</span></a></li>",
                if feed { " aria-current=\"page\"" } else { "" },
                icon(CALENDAR)
            )
        });
        let (graph_attribute, graph_hint) = if self.graph_shortcut {
            (
                " data-lsg-graph-shortcut",
                "<span class=\"lsg-shortcut\" aria-hidden=\"true\">g g</span>",
            )
        } else {
            ("", "")
        };
        let mut right = String::new();
        if let Some(contents) = &self.contents {
            right.push_str(&format!(" data-lsg-contents=\"{}\"", escape(contents)));
        }
        if !self.sidebar.is_empty() {
            right.push_str(&format!(
                " data-lsg-sidebar-default=\"{}\"",
                escape(&self.sidebar.join(" "))
            ));
        }
        let flashcards = self.flashcards.map_or_else(String::new, |due| {
            let badge = if due > 0 {
                format!("<span class=\"lsg-badge\" data-lsg-due>{due}</span>")
            } else {
                String::new()
            };
            format!(
                "<li class=\"flashcards-nav\"><a href=\"#\" role=\"button\" data-lsg-flashcards>{}<span>Flashcards</span><span class=\"lsg-shortcut\" aria-hidden=\"true\">g f</span>{badge}</a></li>",
                icon(super::flashcards::INFINITY)
            )
        });
        // View as slides opens a right-sidebar card, so it is on page routes only.
        let slides = if !home && route.starts_with("p-") {
            "<button type=\"button\" role=\"menuitem\" data-lsg-slides>View as slides</button>"
        } else {
            ""
        };
        format!(
            "<header class=\"lsg-header cp__header\"><button type=\"button\" class=\"lsg-button\" data-lsg-left-toggle aria-label=\"Toggle left sidebar\" aria-controls=\"lsg-left-sidebar\" aria-expanded=\"false\" hidden>{menu}</button><span class=\"lsg-header-space\"></span>{home_link}<a class=\"lsg-button button\" href=\"graph.html#/graph\" aria-label=\"Graph\"><svg class=\"lsg-icon\" viewBox=\"0 0 24 24\" aria-hidden=\"true\"><path id=\"lsg-i-graph\" d=\"{hierarchy}\"/></svg></a><button type=\"button\" class=\"lsg-button\" data-lsg-search aria-label=\"Search garden\" aria-haspopup=\"dialog\" aria-controls=\"lsg-search-dialog\" hidden>{search}</button><div class=\"lsg-more\"><button type=\"button\" class=\"lsg-button\" data-lsg-more aria-label=\"More\" aria-haspopup=\"menu\" aria-controls=\"lsg-more-menu\" aria-expanded=\"false\" hidden>{dots}</button><div id=\"lsg-more-menu\" role=\"menu\" aria-label=\"More\" hidden>{slides}<button type=\"button\" role=\"menuitem\" data-lsg-theme-toggle>Toggle theme</button><button type=\"button\" role=\"menuitem\" data-lsg-theme-system hidden>Use system theme</button></div></div><button type=\"button\" class=\"lsg-button lsg-right-toggle\" data-lsg-right-toggle aria-label=\"Toggle right sidebar\" aria-expanded=\"false\"{right} hidden>{sidebar_right}</button></header><aside id=\"lsg-left-sidebar\" class=\"lsg-left-sidebar\" aria-label=\"Left sidebar\"><nav aria-label=\"Navigation\"><ul>{journals}{flashcards}<li class=\"graph-view-nav\"><a href=\"graph.html\" data-lsg-graph-view{graph_attribute}{graph_current}><svg class=\"lsg-icon\" viewBox=\"0 0 24 24\" aria-hidden=\"true\"><use href=\"#lsg-i-graph\"/></svg><span>Graph view</span>{graph_hint}</a></li><li><a href=\"{all}\" data-lsg-all-pages{all_current}>{files}<span>All pages</span></a></li></ul>{favorites}</nav></aside><div class=\"lsg-sidebar-shade\" data-lsg-sidebar-shade hidden></div>",
            menu = icon(MENU),
            hierarchy = HIERARCHY,
            graph_current = current(route, "graph.html"),
            dots = icon(DOTS),
            search = icon(SEARCH),
            files = icon(FILES),
            sidebar_right = icon(SIDEBAR_RIGHT),
            all = self.all_pages,
            all_current = current(route, self.all_pages),
        )
    }
}
