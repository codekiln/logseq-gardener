//! Logseq's header and left sidebar around every generated page.
use super::render::escape;
use crate::garden_settings::Favorite;

/// Shared behaviour file written once at the site root.
pub(super) const SCRIPT_ROUTE: &str = "lsg-shell.js";
pub(super) const SCRIPT: &str = include_str!("shell.js");

// Tabler Icons outline paths (MIT); see THIRD_PARTY_NOTICES.md.
const MENU: &str = "M4 6l16 0M4 12l16 0M4 18l16 0";
const HOME: &str = "M5 12l-2 0l9 -9l9 9l-2 0M5 12v7a2 2 0 0 0 2 2h10a2 2 0 0 0 2 -2v-7M9 21v-6a2 2 0 0 1 2 -2h2a2 2 0 0 1 2 2v6";
const DOTS: &str = "M4 12m-1 0a1 1 0 1 0 2 0a1 1 0 1 0 -2 0M12 12m-1 0a1 1 0 1 0 2 0a1 1 0 1 0 -2 0M20 12m-1 0a1 1 0 1 0 2 0a1 1 0 1 0 -2 0";
const SEARCH: &str = "M10 10m-7 0a7 7 0 1 0 14 0a7 7 0 1 0 -14 0M21 21l-6 -6";
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
    ".lsg-sidebar-shade{position:fixed;inset:0;z-index:25;background:rgba(0,0,0,.4)}.lsg-sidebar-shade[hidden]{display:none}",
    "@media(min-width:640px){.lsg-left-sidebar{z-index:10;width:246px}html[data-lsg-left-sidebar=open] body{padding-left:246px}html[data-lsg-left-sidebar=open] .lsg-header{margin-left:-246px}.lsg-sidebar-shade{display:none!important}}",
);

/// Navigation that is the same on every page apart from the current-page marks.
pub(super) struct Shell {
    /// Resolved favorites: output route and original title, in config order.
    pub favorites: Vec<Favorite<String>>,
    /// Route of the complete page index.
    pub all_pages: &'static str,
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
        format!(
            "<header class=\"lsg-header\"><button type=\"button\" class=\"lsg-button\" data-lsg-left-toggle aria-label=\"Toggle left sidebar\" aria-controls=\"lsg-left-sidebar\" aria-expanded=\"false\" hidden>{menu}</button><span class=\"lsg-header-space\"></span>{home_link}<button type=\"button\" class=\"lsg-button\" data-lsg-search aria-label=\"Search garden\" aria-haspopup=\"dialog\" aria-controls=\"lsg-search-dialog\" hidden>{search}</button><div class=\"lsg-more\"><button type=\"button\" class=\"lsg-button\" data-lsg-more aria-label=\"More\" aria-haspopup=\"menu\" aria-controls=\"lsg-more-menu\" aria-expanded=\"false\" hidden>{dots}</button><div id=\"lsg-more-menu\" role=\"menu\" aria-label=\"More\" hidden><button type=\"button\" role=\"menuitem\" data-lsg-theme-toggle>Toggle theme</button><button type=\"button\" role=\"menuitem\" data-lsg-theme-system hidden>Use system theme</button></div></div></header><aside id=\"lsg-left-sidebar\" class=\"lsg-left-sidebar\" aria-label=\"Left sidebar\"><nav aria-label=\"Navigation\"><ul><li><a href=\"{all}\" data-lsg-all-pages{all_current}>{files}<span>All pages</span></a></li></ul>{favorites}</nav></aside><div class=\"lsg-sidebar-shade\" data-lsg-sidebar-shade hidden></div>",
            menu = icon(MENU),
            dots = icon(DOTS),
            search = icon(SEARCH),
            files = icon(FILES),
            all = self.all_pages,
            all_current = current(route, self.all_pages),
        )
    }
}
