//! Logseq's journals feed as static batch files: three journals, then seven per file.

use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

use super::render::escape;
use crate::journals::JournalDate;

/// Feed loader written once at the site root.
pub(super) const SCRIPT_ROUTE: &str = "lsg-journals.js";
pub(super) const SCRIPT: &str = include_str!("journals.js");
/// Styles added to the `<head>` of feed pages only. `lsg-shell.js` loads the feed
/// loader after the page's `load` event, so it stays out of first paint.
pub(super) const HEAD: &str = concat!(
    "<style>.lsg-journal{margin:0 0 4rem}.lsg-journal-title{margin:.5rem 0}.lsg-journal-title a{color:inherit;text-decoration:none}.lsg-journal-title a:hover{text-decoration:underline}",
    ".lsg-journal-references{margin-top:2rem}.lsg-journal-references>summary{cursor:pointer;font-size:1.1rem;font-weight:600}.lsg-journals-earlier{display:block;margin:2rem 0;color:var(--lsg-muted)}</style>",
);

const FIRST_BATCH: usize = 3;
const LATER_BATCH: usize = 7;

/// The UTC date of `SOURCE_DATE_EPOCH`, or of the system clock when it is unset or invalid.
pub(super) fn build_date() -> JournalDate {
    JournalDate::from_unix_days(build_seconds().div_euclid(86_400))
}

/// Unix seconds of `SOURCE_DATE_EPOCH`, or of the system clock when it is unset or invalid.
pub(super) fn build_seconds() -> i64 {
    std::env::var("SOURCE_DATE_EPOCH")
        .ok()
        .and_then(|value| value.trim().parse::<i64>().ok())
        .unwrap_or_else(|| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_or(0, |elapsed| elapsed.as_secs() as i64)
        })
}

/// Which journals appear in which batch file.
pub(super) struct Feed {
    batches: Vec<Vec<String>>,
    first: &'static str,
}

impl Feed {
    /// Plan the feed for published journals dated on or before `build_date`.
    ///
    /// The feed is empty when journals are disabled or none is dated in range.
    pub fn plan(
        journals: &BTreeMap<String, JournalDate>,
        build_date: JournalDate,
        enabled: bool,
        has_homepage: bool,
    ) -> Self {
        let mut listed = journals
            .iter()
            .filter(|(_, date)| enabled && **date <= build_date)
            .collect::<Vec<_>>();
        listed.sort_by(|left, right| right.1.cmp(left.1).then_with(|| left.0.cmp(right.0)));
        let mut routes = listed.into_iter().map(|(route, _)| route.clone());
        let mut batches = Vec::new();
        let first = routes.by_ref().take(FIRST_BATCH).collect::<Vec<_>>();
        if !first.is_empty() {
            batches.push(first);
        }
        let rest = routes.collect::<Vec<_>>();
        batches.extend(rest.chunks(LATER_BATCH).map(<[String]>::to_vec));
        Self {
            batches,
            first: if has_homepage {
                "journals.html"
            } else {
                "index.html"
            },
        }
    }

    pub fn is_empty(&self) -> bool {
        self.batches.is_empty()
    }

    /// Whether the feed's first batch holds `index.html`.
    pub fn home(&self) -> bool {
        !self.is_empty() && self.first == "index.html"
    }

    /// Route of the first batch, when the feed is written.
    pub fn first_route(&self) -> Option<&'static str> {
        (!self.is_empty()).then_some(self.first)
    }

    /// Whether the journal at `route` appears in any batch.
    pub fn lists(&self, route: &str) -> bool {
        self.batches.iter().flatten().any(|listed| listed == route)
    }

    fn route(&self, batch: usize) -> String {
        if batch == 0 {
            self.first.to_owned()
        } else {
            format!("journals-{}.html", batch + 1)
        }
    }

    /// `<main>` contents for each batch file, from each journal's title, rendered
    /// blocks and count of referring pages.
    pub fn render(
        &self,
        journals: &BTreeMap<String, (String, String, usize)>,
    ) -> Vec<(String, String)> {
        self.batches
            .iter()
            .enumerate()
            .map(|(batch, routes)| {
                let mut html = format!("<div class=\"lsg-journals\" data-lsg-journals=\"{SCRIPT_ROUTE}\">");
                for route in routes {
                    let Some((title, body, references)) = journals.get(route) else {
                        continue;
                    };
                    let route = escape(route);
                    html.push_str(&format!(
                        "<article class=\"lsg-journal\"><h1 class=\"lsg-journal-title\"><a class=\"journal-title\" href=\"{route}\">{}</a></h1>{}",
                        escape(title),
                        without_source_anchors(body)
                    ));
                    if *references > 0 {
                        let label = if *references == 1 { "page" } else { "pages" };
                        html.push_str(&format!(
                            "<details class=\"lsg-journal-references\" data-lsg-journal-refs=\"{route}\"><summary>Linked references — {references} {label}</summary><p><a href=\"{route}#linked-references\">Open the linked references</a></p></details>"
                        ));
                    }
                    html.push_str("</article>");
                }
                if batch + 1 < self.batches.len() {
                    html.push_str(&format!(
                        "<a class=\"lsg-journals-earlier\" data-lsg-journals-earlier href=\"{}\">Earlier</a>",
                        self.route(batch + 1)
                    ));
                }
                html.push_str("</div>");
                (self.route(batch), html)
            })
            .collect()
    }
}

/// Drop per-page source anchors, which would repeat across journals in one feed page.
fn without_source_anchors(html: &str) -> String {
    const START: &str = "<span id=\"source-block-";
    const END: &str = "\"></span>";
    let mut result = String::with_capacity(html.len());
    let mut rest = html;
    while let Some(start) = rest.find(START) {
        result.push_str(&rest[..start]);
        let tail = &rest[start + START.len()..];
        match tail.find(END) {
            Some(end) if tail[..end].bytes().all(|byte| byte.is_ascii_digit()) => {
                rest = &tail[end + END.len()..];
            }
            _ => {
                result.push_str(START);
                rest = tail;
            }
        }
    }
    result.push_str(rest);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn journals(count: i64) -> BTreeMap<String, JournalDate> {
        (0..count)
            .map(|offset| {
                (
                    format!("p-{offset:03}.html"),
                    JournalDate::from_unix_days(20_000 - offset),
                )
            })
            .collect()
    }

    #[test]
    fn batches_three_then_seven_newest_first() {
        let today = JournalDate::from_unix_days(20_000);
        let feed = Feed::plan(&journals(12), today, true, false);
        assert_eq!(
            feed.batches.iter().map(Vec::len).collect::<Vec<_>>(),
            [3, 7, 2]
        );
        assert_eq!(feed.batches[0], ["p-000.html", "p-001.html", "p-002.html"]);
        assert_eq!(
            (feed.route(0), feed.route(2)),
            ("index.html".into(), "journals-3.html".into())
        );
        assert!(feed.home());
    }

    #[test]
    fn leaves_out_future_and_disabled_journals() {
        let yesterday = JournalDate::from_unix_days(19_999);
        let feed = Feed::plan(&journals(3), yesterday, true, true);
        assert_eq!(
            feed.batches,
            [vec!["p-001.html".to_owned(), "p-002.html".to_owned()]]
        );
        assert_eq!(feed.first_route(), Some("journals.html"));
        assert!(!feed.home());
        assert!(Feed::plan(&journals(3), yesterday, false, false).is_empty());
    }

    #[test]
    fn removes_only_source_anchors() {
        assert_eq!(
            without_source_anchors(
                "<li><span id=\"source-block-12\"></span>a<span id=\"block-x\"></span></li>"
            ),
            "<li>a<span id=\"block-x\"></span></li>"
        );
    }
}
