//! Logseq's All pages table: the first 40 rows of the default view in static HTML, the
//! rest of that view on static paging pages, and every entry in one list data file that
//! the script fetches when a reader sorts, searches or shows journals.

use std::fmt::Write;

use super::render::escape;
use super::{digest, page_status};
use crate::page_list::{Column, PageKind, PageListEntry};

/// The All pages script, written once at the site root and loaded only by All pages.
pub(super) const SCRIPT_ROUTE: &str = "lsg-all-pages.js";
/// Every entry, journals included, fetched on the first sort, search or toggle.
pub(super) const LIST_ROUTE: &str = "_pages/list.json";
pub(super) const LIST_DIRECTORY: &str = "_pages";
const STYLES: &str = include_str!("all-pages.css");
const ROWS: usize = 40;

/// The script with the shared fuzzy score, without comments or indentation, as
/// `references::script` writes its own.
pub(super) fn script() -> String {
    let mut script = String::new();
    let source = include_str!("all-pages.js").replacen(
        super::search::FUZZY_MARKER,
        include_str!("fuzzy.js"),
        1,
    );
    for line in source.lines().map(str::trim) {
        if line.is_empty() || line.starts_with("//") {
            continue;
        }
        if script.ends_with(|c: char| !";{},".contains(c)) {
            script.push('\n');
        }
        script.push_str(line);
    }
    script.push('\n');
    script
}

/// Styles and the deferred script for the `<head>` of All pages screens.
pub(super) fn head() -> String {
    format!("<style>{STYLES}</style><script defer src=\"{SCRIPT_ROUTE}\"></script>")
}

/// The route of a page of the default view: the index for the first, then `pages-<n>.html`.
fn page_route(index_route: &str, page: usize) -> String {
    if page == 1 {
        index_route.to_owned()
    } else {
        format!("pages-{page}.html")
    }
}

/// `yyyy-MM-dd HH:mm` and the RFC 3339 value of a Unix time, in UTC.
fn utc(seconds: i64) -> (String, String) {
    let days = seconds.div_euclid(86_400);
    let minutes = seconds.rem_euclid(86_400) / 60;
    // Howard Hinnant's civil_from_days.
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    let (hour, minute) = (minutes / 60, minutes % 60);
    (
        format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}"),
        format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:00Z"),
    )
}

fn date_cell(class: &str, seconds: Option<i64>) -> String {
    match seconds {
        Some(seconds) => {
            let (shown, value) = utc(seconds);
            format!("<td class=\"{class}\"><time datetime=\"{value}\">{shown}</time></td>")
        }
        None => format!("<td class=\"{class}\"></td>"),
    }
}

fn row(entry: &PageListEntry<String>, dates: bool) -> String {
    let icon = entry.icon.as_deref().map(escape).unwrap_or_default();
    // Generated namespace rows keep their link style and accessible name, without a badge.
    let generated = if entry.kind == PageKind::Namespace {
        page_status::link_attributes(true, &entry.title).replacen(
            " class=\"namespace-link\"",
            "",
            1,
        )
    } else {
        String::new()
    };
    let class = if entry.kind == PageKind::Namespace {
        "page-ref namespace-link"
    } else {
        "page-ref"
    };
    let mut html = format!(
        "<tr><td class=\"icon\">{icon}</td><td class=\"name\"><a class=\"{class}\"{generated} href=\"{}\">{}</a></td><td class=\"backlinks\">{}</td>",
        escape(&entry.id),
        escape(&entry.title),
        entry.backlinks
    );
    if dates {
        html.push_str(&date_cell("created-at", entry.created));
        html.push_str(&date_cell("updated-at", entry.updated));
    }
    html.push_str("</tr>");
    html
}

fn header(label: &str, column: &str, sorted: Option<bool>) -> String {
    let caret = match sorted {
        Some(true) => "<span class=\"caret\" aria-hidden=\"true\">▾</span>",
        Some(false) => "<span class=\"caret\" aria-hidden=\"true\">▴</span>",
        None => "",
    };
    let sort = match sorted {
        Some(true) => " aria-sort=\"descending\"",
        Some(false) => " aria-sort=\"ascending\"",
        None => "",
    };
    format!(
        "<th class=\"{column}\"{sort}><button type=\"button\" class=\"sort\" data-sort=\"{column}\">{label}{caret}</button></th>"
    )
}

fn paging(index_route: &str, page: usize, pages: usize) -> String {
    let mut html = String::from("<nav class=\"paging\" aria-label=\"Pages\">");
    if page > 1 {
        let _ = write!(
            html,
            "<a href=\"{}\" aria-label=\"First page\">«</a><a href=\"{}\" rel=\"prev\">◂ Prev</a>",
            page_route(index_route, 1),
            page_route(index_route, page - 1)
        );
    }
    let _ = write!(html, "<span class=\"page-number\">{page}/{pages}</span>");
    if page < pages {
        let _ = write!(
            html,
            "<a href=\"{}\" rel=\"next\">Next ▸</a><a href=\"{}\" aria-label=\"Last page\">»</a>",
            page_route(index_route, page + 1),
            page_route(index_route, pages)
        );
    }
    html.push_str("</nav>");
    html
}

/// The default view: journals hidden, `Updated At` newest first when the list has dates,
/// otherwise `Page name` ascending.
fn default_view(entries: &[PageListEntry<String>], dates: bool) -> Vec<&PageListEntry<String>> {
    let mut view: Vec<_> = entries
        .iter()
        .filter(|entry| entry.kind != PageKind::Journal)
        .collect();
    let (column, descending) = if dates {
        (Column::Updated, true)
    } else {
        (Column::Name, false)
    };
    view.sort_by(|left, right| left.cmp_by(right, column, descending));
    view
}

/// The All pages screens: `(route, <main> content)` for the index and each paging page.
pub(super) fn screens(
    entries: &[PageListEntry<String>],
    index_route: &str,
) -> Vec<(String, String)> {
    let dates = entries.iter().any(|entry| entry.updated.is_some());
    let view = default_view(entries, dates);
    let pages = view.len().div_ceil(ROWS).max(1);
    let mut head_row = String::from(
        "<thead><tr><th class=\"icon\"><span class=\"visually-hidden\">Icon</span></th>",
    );
    head_row.push_str(&header("Page name", "name", (!dates).then_some(false)));
    head_row.push_str(&header("Backlinks", "backlinks", None));
    if dates {
        head_row.push_str(&header("Created At", "created-at", None));
        head_row.push_str(&header("Updated At", "updated-at", Some(true)));
    }
    head_row.push_str("</tr></thead>");
    let empty = if view.is_empty() {
        "<p>No pages matched the publication policy.</p>"
    } else {
        ""
    };
    (1..=pages)
        .map(|page| {
            let rows: String = view
                .iter()
                .skip((page - 1) * ROWS)
                .take(ROWS)
                .map(|entry| row(entry, dates))
                .collect();
            let nav = paging(index_route, page, pages);
            let html = format!(
                "<div class=\"all-pages\" data-list-src=\"{LIST_ROUTE}\" data-sort=\"{}\" data-desc=\"{dates}\"><h1 class=\"title\">All pages</h1><p class=\"total\">Total {} pages</p><div class=\"actions\"><input type=\"search\" class=\"search\" placeholder=\"Search page names\" aria-label=\"Search page names\"><button type=\"button\" class=\"journal-toggle\" aria-pressed=\"false\" title=\"Show journals\" aria-label=\"Show journals\">📅</button>{nav}</div><div class=\"table-wrapper\"><table class=\"all-pages-table\">{head_row}<tbody>{rows}</tbody></table></div>{empty}{nav}</div>",
                if dates { "updated-at" } else { "name" },
                view.len(),
            );
            (page_route(index_route, page), html)
        })
        .collect()
}

/// Every entry as compact JSON rows: `[title, route, kind, backlinks, created, updated, icon]`,
/// with kind `p`, `j` or `n` and absent values as `null`. The route is `null` when it is
/// the usual `p-<digest of the title>.html`.
pub(super) fn list_json(entries: &[PageListEntry<String>]) -> String {
    let rows: Vec<serde_json::Value> = entries
        .iter()
        .map(|entry| {
            let kind = match entry.kind {
                PageKind::Page => "p",
                PageKind::Journal => "j",
                PageKind::Namespace => "n",
            };
            // Routes are `p-<digest of the title>.html`, which the script computes itself.
            let route =
                (entry.id != format!("p-{}.html", digest(&entry.title))).then_some(&entry.id);
            let mut row = serde_json::json!([
                entry.title,
                route,
                kind,
                entry.backlinks,
                entry.created,
                entry.updated
            ]);
            if let Some(icon) = &entry.icon {
                row.as_array_mut()
                    .expect("row array")
                    .push(icon.clone().into());
            }
            row
        })
        .collect();
    // `<` appears only inside strings; escaping it keeps markup out of the file.
    serde_json::to_string(&rows)
        .expect("list JSON")
        .replace('<', "\\u003c")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entry(title: &str, kind: PageKind, updated: Option<i64>) -> PageListEntry<String> {
        PageListEntry {
            id: format!("p-{:02}.html", title.len()),
            title: title.to_owned(),
            kind,
            icon: None,
            backlinks: 1,
            sort_key: title.to_lowercase(),
            created: updated,
            updated,
        }
    }

    #[test]
    fn utc_dates() {
        assert_eq!(
            utc(1_740_820_320),
            (
                "2025-03-01 09:12".to_owned(),
                "2025-03-01T09:12:00Z".to_owned()
            )
        );
        assert_eq!(utc(0).0, "1970-01-01 00:00");
    }

    #[test]
    fn paging_links() {
        let entries: Vec<_> = (0..90)
            .map(|n| entry(&format!("page {n:03}"), PageKind::Page, Some(n)))
            .chain([entry("2026-10-23 Fri", PageKind::Journal, Some(1000))])
            .collect();
        let screens = screens(&entries, "pages.html");
        let routes: Vec<_> = screens.iter().map(|(route, _)| route.as_str()).collect();
        assert_eq!(routes, ["pages.html", "pages-2.html", "pages-3.html"]);
        let (_, first) = &screens[0];
        assert!(first.contains("Total 90 pages"));
        assert!(first.contains("<span class=\"page-number\">1/3</span><a href=\"pages-2.html\" rel=\"next\">Next ▸</a><a href=\"pages-3.html\" aria-label=\"Last page\">»</a>"));
        assert!(!first.contains("◂ Prev"));
        // Newest first, journals hidden.
        assert!(first.find("page 089").unwrap() < first.find("page 088").unwrap());
        assert!(!first.contains("2026-10-23 Fri"));
        assert_eq!(first.matches("<tr><td").count(), 40);
        let (_, middle) = &screens[1];
        assert!(middle.contains("<a href=\"pages.html\" aria-label=\"First page\">«</a><a href=\"pages.html\" rel=\"prev\">◂ Prev</a><span class=\"page-number\">2/3</span>"));
        assert!(middle.contains("page 049") && !middle.contains("page 050"));
        let (_, last) = &screens[2];
        assert_eq!(last.matches("<tr><td").count(), 10);
        assert!(!last.contains("Next ▸"));
        assert_eq!(last.matches("class=\"paging\"").count(), 2);
    }

    #[test]
    fn no_dates_sorts_by_name_without_date_columns() {
        let entries = [
            entry("b", PageKind::Page, None),
            entry("A", PageKind::Page, None),
            entry("tmux/Command", PageKind::Namespace, None),
        ];
        let screens = screens(&entries, "index.html");
        let (route, html) = &screens[0];
        assert_eq!(route, "index.html");
        assert!(!html.contains("Created At") && !html.contains("updated-at\""));
        assert!(html.find(">A<").unwrap() < html.find(">b<").unwrap());
        assert!(html.contains(
            "<a class=\"page-ref namespace-link\" aria-label=\"tmux/Command (Namespace)\""
        ));
        assert!(!html.contains("page-kind"));
        assert!(html.contains("1/1"));
    }

    #[test]
    fn list_rows_escape_and_keep_journals() {
        let mut journal = entry("2026-10-23 Fri", PageKind::Journal, Some(5));
        journal.icon = Some("\"🧭\"".to_owned());
        let mut page = entry("a</script>", PageKind::Namespace, None);
        page.id = format!("p-{}.html", digest(&page.title));
        let json = list_json(&[page, journal]);
        assert!(!json.contains('<'));
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(
            value[0],
            serde_json::json!(["a</script>", null, "n", 1, null, null])
        );
        assert_eq!(value[1][1], "p-14.html");
        assert_eq!(value[1][2], "j");
        assert_eq!(value[1][6], "\"🧭\"");
    }
}
