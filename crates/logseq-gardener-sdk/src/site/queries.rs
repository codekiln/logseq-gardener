//! Query boxes: `{{query …}}` and `#+BEGIN_QUERY` blocks with Logseq's header, clause
//! row, results grouped by page or a table, and its three error forms. The SDK's
//! [`crate::query`] evaluates every query during the build; this module only writes the
//! results. Each box keeps up to [`IN_PAGE_GZIP`] of its results in the page and writes the
//! rest to fragment files that `lsg-queries.js` loads as the reader scrolls.
use std::cell::{OnceCell, RefCell};
use std::collections::BTreeMap;
use std::io::Write;
use std::path::Path;
use std::thread::JoinHandle;

use flate2::{Compression, write::GzEncoder};

use super::references::{Context, REFS, References};
use super::render::{QueryBox, QueryBoxes, escape};
use crate::git_dates::GitDates;
use crate::journals::{JournalDate, JournalFormats};
use crate::query::{
    AdvancedBody, AdvancedQuery, EvaluateOptions, QueryErrorKind, QueryGraph, QueryResults,
    QueryTitle, SimpleQuery, parse_advanced, parse_simple, seed,
};
use crate::references::BlockId;

/// The query script, written once at the site root and linked only from pages with a query.
pub(super) const SCRIPT_ROUTE: &str = "lsg-queries.js";
/// The query styles, written once at the site root and linked only from pages with a query.
pub(super) const STYLES_ROUTE: &str = "lsg-queries.css";
/// `queries.js` without comments or indentation.
pub(super) fn script() -> String {
    super::references::minify(include_str!("queries.js"))
}
pub(super) const STYLES: &str = include_str!("queries.css");
/// Every query box carries this class; pages that contain it link the script and styles.
pub(super) const MARKER: &str = "class=\"custom-query";
/// The part of a query's results kept in the page, measured gzip.
const IN_PAGE_GZIP: usize = 30_000;
/// The most results a fragment file holds, measured gzip.
const FRAGMENT_GZIP: usize = 100_000;

const SEARCH: &str = "<svg class=\"lsg-query-icon\" width=\"12\" height=\"12\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><path d=\"M10 10m-7 0a7 7 0 1 0 14 0a7 7 0 1 0 -14 0\"/><path d=\"M21 21l-6 -6\"/></svg>";
const TABLE: &str = "<svg width=\"20\" height=\"20\" viewBox=\"0 0 24 24\" fill=\"none\" stroke=\"currentColor\" stroke-width=\"2\" stroke-linecap=\"round\" stroke-linejoin=\"round\" aria-hidden=\"true\"><path d=\"M3 5a2 2 0 0 1 2 -2h14a2 2 0 0 1 2 2v14a2 2 0 0 1 -2 2h-14a2 2 0 0 1 -2 -2v-14z\"/><path d=\"M3 10h18\"/><path d=\"M10 3v18\"/></svg>";

/// The `<head>` tags of a page with a query.
pub(super) fn head() -> String {
    format!(
        "<link rel=\"stylesheet\" href=\"{STYLES_ROUTE}\"><script defer src=\"{SCRIPT_ROUTE}\"></script>"
    )
}

/// The git history and its diagnostic.
type Dates = (Option<GitDates>, Option<String>);

/// The git history, read on its own thread and joined the first time a table or the page
/// list needs a date.
pub(super) struct LazyDates {
    pending: RefCell<Option<JoinHandle<Dates>>>,
    ready: OnceCell<Dates>,
}

impl LazyDates {
    pub fn new(pending: JoinHandle<Dates>) -> Self {
        Self {
            pending: RefCell::new(Some(pending)),
            ready: OnceCell::new(),
        }
    }

    pub fn get(&self) -> &Dates {
        self.ready.get_or_init(|| {
            self.pending
                .borrow_mut()
                .take()
                .expect("dates are read once")
                .join()
                .expect("git history reader panicked")
        })
    }
}

/// Everything a query box needs: the reference index, the build date and the page context.
pub(super) struct Queries<'r, 'a> {
    refs: &'r References<'a>,
    graph: OnceCell<QueryGraph<'r, String>>,
    resolve: &'r dyn Fn(&str) -> Option<String>,
    formats: &'r JournalFormats,
    today: JournalDate,
    context: Context<'r, 'a>,
    dates: &'r LazyDates,
    files: RefCell<Vec<(String, String)>>,
    numbers: RefCell<BTreeMap<String, usize>>,
}

/// How one box shows its results.
struct View {
    table: bool,
    toggle: bool,
    grouped: bool,
    collapsed: bool,
    /// Default journal queries run while collapsed and show their count.
    count_when_collapsed: bool,
    /// The block's `query-properties`, `query-sort-by` and `query-sort-desc`.
    columns: Option<Vec<String>>,
    sort_by: Option<String>,
    descending: bool,
}

impl<'r, 'a> Queries<'r, 'a> {
    pub fn new(
        refs: &'r References<'a>,
        resolve: &'r dyn Fn(&str) -> Option<String>,
        formats: &'r JournalFormats,
        today: JournalDate,
        context: Context<'r, 'a>,
        dates: &'r LazyDates,
    ) -> Self {
        Self {
            refs,
            graph: OnceCell::new(),
            resolve,
            formats,
            today,
            context,
            dates,
            files: RefCell::new(Vec::new()),
            numbers: RefCell::new(BTreeMap::new()),
        }
    }

    /// Fragment files written since the last call.
    pub fn take_files(&self) -> Vec<(String, String)> {
        std::mem::take(&mut self.files.borrow_mut())
    }

    pub fn graph(&self) -> &QueryGraph<'r, String> {
        self.graph.get_or_init(|| {
            QueryGraph::new(
                self.refs.index(),
                self.refs.sources(),
                self.refs.dates(),
                self.resolve,
                self.formats,
                self.today,
            )
        })
    }

    fn next_id(&self, page: &str) -> String {
        let mut numbers = self.numbers.borrow_mut();
        let number = numbers.entry(page.to_owned()).or_default();
        *number += 1;
        format!("q{number}")
    }

    fn block_id(&self, page: &str, block: Option<usize>) -> Option<BlockId> {
        let block = block?;
        self.refs
            .index()
            .blocks_of(&page.to_owned())
            .into_iter()
            .find(|id| self.refs.index().block(*id).anchor == Some(block))
    }

    fn property(&self, id: Option<BlockId>, key: &str) -> Option<&str> {
        id.and_then(|id| self.graph().block_property(id, key))
    }

    fn flag(&self, id: Option<BlockId>, key: &str) -> Option<bool> {
        self.property(id, key)
            .map(|value| value.eq_ignore_ascii_case("true"))
    }

    fn line(&self, page: &str, block: Option<usize>) -> String {
        let line = self
            .refs
            .sources()
            .get(page)
            .zip(block)
            .map(|(document, byte)| {
                document.source[..byte.min(document.source.len())]
                    .matches('\n')
                    .count()
                    + 1
            });
        line.map(|line| format!(" (line {line})"))
            .unwrap_or_default()
    }

    fn view(&self, id: Option<BlockId>, pages: bool) -> View {
        let columns = self.property(id, "query-properties").map(|value| {
            value
                .trim_matches(|c| c == '[' || c == ']')
                .split(|c: char| c.is_whitespace() || c == ',')
                .map(|key| key.trim_start_matches(':').to_lowercase())
                .filter(|key| !key.is_empty())
                .collect()
        });
        View {
            table: pages || self.flag(id, "query-table") == Some(true),
            toggle: !pages,
            grouped: true,
            collapsed: self.flag(id, "collapsed") == Some(true),
            count_when_collapsed: false,
            columns,
            sort_by: self
                .property(id, "query-sort-by")
                .map(|key| key.trim_start_matches(':').to_lowercase()),
            descending: self.flag(id, "query-sort-desc") != Some(false),
        }
    }

    fn evaluate(
        &self,
        query: &SimpleQuery,
        id: Option<BlockId>,
        page: &str,
        block: Option<usize>,
        keep_children: bool,
    ) -> QueryResults<String> {
        self.graph().evaluate(
            query,
            &EvaluateOptions {
                own_block: id,
                keep_children,
                seed: seed(&format!("{page}#{}", block.unwrap_or_default())),
            },
        )
    }

    /// One query box. `body` replaces the results when the query cannot run.
    #[allow(clippy::too_many_arguments)]
    fn render_box(
        &self,
        page: &str,
        title: &str,
        builder: Option<String>,
        results: Option<&QueryResults<String>>,
        body: Option<String>,
        view: &View,
        asset: &mut dyn FnMut(&Path, &str) -> Result<String, String>,
        diagnostics: &mut Vec<String>,
    ) -> String {
        let id = self.next_id(page);
        let stem = format!("{REFS}/{}-{id}", page.trim_end_matches(".html"));
        let count = results.map_or(0, QueryResults::len);
        let count_text = match count {
            0 => String::new(),
            1 => "1 result".to_owned(),
            n => format!("{n} results"),
        };
        let toggle = |table: bool, alt: &str| {
            format!(
                "<button type=\"button\" class=\"lsg-query-view\" aria-label=\"{}\" data-alt=\"{}\">{TABLE}</button>",
                if table {
                    "Switch to list view"
                } else {
                    "Switch to table view"
                },
                escape(alt)
            )
        };
        let mut tools = String::new();
        let mut results_html = String::new();
        if let Some(body) = body {
            results_html = body;
        } else if let Some(results) = results.filter(|results| !results.is_empty()) {
            let main = self.parts(results, view.table, view, asset, diagnostics);
            results_html = self.write(&stem, "", main, !view.collapsed, view.table).0;
            if view.toggle && matches!(results, QueryResults::Blocks(_)) {
                let other = self.parts(results, !view.table, view, asset, diagnostics);
                if let (_, Some(alt)) = self.write(&stem, "-alt", other, false, !view.table) {
                    tools.push_str(&toggle(view.table, &alt));
                }
            }
        } else if results.is_some() {
            results_html = "<div class=\"lsg-query-empty\">No matched result</div>".to_owned();
        }
        let shown = if view.collapsed && !view.count_when_collapsed {
            String::new()
        } else {
            count_text.clone()
        };
        let count_span = format!("<span class=\"results-count\">{shown}</span>");
        let builder = builder
            .map(|text| format!("<div class=\"query-builder\">{}</div>", escape(&text)))
            .unwrap_or_default();
        if view.collapsed {
            format!(
                "<details class=\"custom-query lsg-query-collapsed\" data-lsg-query=\"{id}\" data-count=\"{count_text}\"><summary class=\"th\"><div class=\"title\">{title}</div><div class=\"flex\">{count_span}{tools}</div></summary>{builder}<div class=\"custom-query-results\">{results_html}</div></details>"
            )
        } else {
            format!(
                "<div class=\"custom-query\" data-lsg-query=\"{id}\"><div class=\"th\"><div class=\"title\">{title}</div><div class=\"flex\">{count_span}{tools}</div></div>{builder}<div class=\"custom-query-results\">{results_html}</div></div>"
            )
        }
    }

    /// The results as parts that can split across files: groups or blocks for a list, and
    /// a header plus rows for a table.
    fn parts(
        &self,
        results: &QueryResults<String>,
        table: bool,
        view: &View,
        asset: &mut dyn FnMut(&Path, &str) -> Result<String, String>,
        diagnostics: &mut Vec<String>,
    ) -> Parts {
        if table {
            return self.table(results, view);
        }
        let QueryResults::Blocks(blocks) = results else {
            return self.table(results, view);
        };
        let graph = self.graph();
        let mut items = Vec::new();
        if view.grouped {
            for (page, ids) in graph.group_by_page(blocks) {
                items.push(self.refs.query_group(
                    &page,
                    &ids,
                    true,
                    &self.context,
                    asset,
                    diagnostics,
                ));
            }
        } else {
            for &id in blocks {
                let page = self.refs.index().page(id).clone();
                items.push(self.refs.query_group(
                    &page,
                    &[id],
                    false,
                    &self.context,
                    asset,
                    diagnostics,
                ));
            }
        }
        Parts {
            head: String::new(),
            items,
            columns: 0,
        }
    }

    fn cell_value(&self, value: &str) -> String {
        value
            .split(',')
            .map(|part| {
                let name = part.trim();
                let bare = name
                    .strip_prefix("[[")
                    .and_then(|n| n.strip_suffix("]]"))
                    .unwrap_or_else(|| name.strip_prefix('#').unwrap_or(name));
                match (self.resolve)(bare) {
                    Some(route) if name.starts_with("[[") || name.starts_with('#') => format!(
                        "<a class=\"page-ref\" href=\"{}\">{}</a>",
                        escape(&route),
                        escape(bare)
                    ),
                    _ => escape(name),
                }
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn page_cell(&self, page: &str) -> String {
        format!(
            "<a class=\"page-ref\" href=\"{}\">{}</a>",
            escape(page),
            escape(self.refs.page_title(page))
        )
    }

    fn file_dates(&self, page: &str) -> Option<crate::git_dates::FileDates> {
        let document = self.refs.sources().get(page)?;
        let path = document.relative_path.to_str()?.replace('\\', "/");
        self.dates.get().0.as_ref()?.get(&path)
    }

    fn table(&self, results: &QueryResults<String>, view: &View) -> Parts {
        let graph = self.graph();
        // Rows as (sort keys by column, cells by column).
        let mut columns: Vec<String> = Vec::new();
        let mut rows: Vec<BTreeMap<String, (String, String)>> = Vec::new();
        let add = |key: &str, columns: &mut Vec<String>| {
            if !columns.iter().any(|c| c == key) {
                columns.push(key.to_owned());
            }
        };
        let date = |seconds: Option<i64>| -> (String, String) {
            seconds.map_or((String::new(), String::new()), |s| {
                let (shown, _) = utc(s);
                (shown.clone(), shown)
            })
        };
        match results {
            QueryResults::Blocks(blocks) => {
                add("block", &mut columns);
                add("page", &mut columns);
                for &id in blocks {
                    let page = self.refs.index().page(id).clone();
                    let mut row = BTreeMap::new();
                    let first = first_line(graph.block_text(id));
                    row.insert("block".to_owned(), (first.clone(), escape(&first)));
                    row.insert(
                        "page".to_owned(),
                        (
                            self.refs.page_title(&page).to_lowercase(),
                            self.page_cell(&page),
                        ),
                    );
                    for (key, value) in graph.block_properties(id) {
                        if super::properties::is_internal(key) {
                            continue;
                        }
                        add(key, &mut columns);
                        row.insert(key.clone(), (value.to_lowercase(), self.cell_value(value)));
                    }
                    let dates = self.file_dates(&page);
                    row.insert("created-at".into(), date(dates.map(|d| d.created)));
                    row.insert("updated-at".into(), date(dates.map(|d| d.updated)));
                    rows.push(row);
                }
            }
            QueryResults::Pages(pages) => {
                add("page", &mut columns);
                for page in pages {
                    let mut row = BTreeMap::new();
                    row.insert(
                        "page".to_owned(),
                        (
                            self.refs.page_title(page).to_lowercase(),
                            self.page_cell(page),
                        ),
                    );
                    for (key, value) in graph.page_property_list(page) {
                        if super::properties::is_internal(key) {
                            continue;
                        }
                        add(key, &mut columns);
                        row.insert(key.clone(), (value.to_lowercase(), self.cell_value(value)));
                    }
                    let dates = self.file_dates(page);
                    row.insert("created-at".into(), date(dates.map(|d| d.created)));
                    row.insert("updated-at".into(), date(dates.map(|d| d.updated)));
                    rows.push(row);
                }
                add("created-at", &mut columns);
                add("updated-at", &mut columns);
            }
        }
        if let Some(chosen) = &view.columns {
            columns = chosen.clone();
        }
        let sort = view
            .sort_by
            .clone()
            .unwrap_or_else(|| "updated-at".to_owned());
        rows.sort_by(|a, b| {
            let key = |row: &BTreeMap<String, (String, String)>| {
                row.get(&sort)
                    .map(|(key, _)| key.clone())
                    .unwrap_or_default()
            };
            let order = key(a).cmp(&key(b));
            if view.descending {
                order.reverse()
            } else {
                order
            }
        });
        let head = columns
            .iter()
            .map(|column| {
                let sorted = if *column == sort {
                    if view.descending {
                        " aria-sort=\"descending\""
                    } else {
                        " aria-sort=\"ascending\""
                    }
                } else {
                    ""
                };
                format!(
                    "<th{sorted}><button type=\"button\" class=\"sort\">{}</button></th>",
                    escape(column)
                )
            })
            .collect::<String>();
        let items = rows
            .iter()
            .map(|row| {
                let cells: String = columns
                    .iter()
                    .map(|column| {
                        format!(
                            "<td>{}</td>",
                            row.get(column)
                                .map(|(_, html)| html.as_str())
                                .unwrap_or_default()
                        )
                    })
                    .collect();
                format!("<tr>{cells}</tr>")
            })
            .collect();
        Parts {
            head: format!("<thead><tr>{head}</tr></thead>"),
            items,
            columns: columns.len(),
        }
    }

    /// Write the parts: up to [`IN_PAGE_GZIP`] in the returned markup when `in_page` is set
    /// and the rest in fragment files, each loading the next. Without `in_page`, every part
    /// goes to the files and the markup is a link to the first. `suffix` names an
    /// alternative view's files. Returns the markup and the first file's route.
    fn write(
        &self,
        stem: &str,
        suffix: &str,
        parts: Parts,
        in_page: bool,
        table: bool,
    ) -> (String, Option<String>) {
        let mut page_items = Vec::new();
        let mut files: Vec<Vec<String>> = Vec::new();
        let mut size = 0;
        for item in parts.items {
            let item_size = gzip_size(&item);
            if files.is_empty() && in_page {
                if page_items.is_empty() || size + item_size <= IN_PAGE_GZIP {
                    size += item_size;
                    page_items.push(item);
                    continue;
                }
                size = 0;
                files.push(Vec::new());
            } else if files.is_empty()
                || size + item_size > FRAGMENT_GZIP && !files.last().unwrap().is_empty()
            {
                size = 0;
                files.push(Vec::new());
            }
            size += item_size;
            files.last_mut().unwrap().push(item);
        }
        let route = |number: usize| format!("{stem}{suffix}-{number}.html");
        let link = |number: usize| {
            format!(
                "<a class=\"lsg-query-more\" href=\"{0}\" data-next=\"{0}\">More results</a>",
                route(number)
            )
        };
        let more = |number: usize| {
            if table {
                format!(
                    "<tr class=\"lsg-query-more-row\"><td colspan=\"{}\">{}</td></tr>",
                    parts.columns.max(1),
                    link(number)
                )
            } else {
                link(number)
            }
        };
        let wrap = |items: &[String], next: Option<usize>, first: bool| {
            let mut body = items.concat();
            if let Some(next) = next {
                body.push_str(&more(next));
            }
            match (table, first) {
                (true, true) => format!(
                    "<div class=\"table-wrapper\"><table class=\"table-auto query-table\">{}<tbody>{body}</tbody></table></div>",
                    parts.head
                ),
                (true, false) => format!("<table><tbody>{body}</tbody></table>"),
                (false, _) => body,
            }
        };
        let total = files.len();
        for (position, items) in files.iter().enumerate() {
            let number = position + 1;
            let next = (number < total).then_some(number + 1);
            let body = wrap(items, next, !in_page && number == 1);
            self.files.borrow_mut().push((
                route(number),
                format!(
                    "<!doctype html>\n<html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><base href=\"../\"><title>Query results</title></head><body><main><div class=\"query-fragment\">{body}</div></main></body></html>\n"
                ),
            ));
        }
        let first = (total > 0).then(|| route(1));
        let markup = if in_page {
            wrap(&page_items, first.as_ref().map(|_| 1), true)
        } else if total > 0 {
            link(1)
        } else {
            String::new()
        };
        (markup, first)
    }

    fn error_box(heading: &str, text: &str) -> String {
        format!(
            "<div class=\"lsg-query-error\"><h5>{heading}</h5><pre><code>{}</code></pre></div>",
            escape(text)
        )
    }

    fn failed(label: &str, message: &str) -> String {
        format!(
            "<div class=\"warning my-1\">{label}<p>{}</p></div>",
            escape(message)
        )
    }

    fn title_html(title: Option<&QueryTitle>) -> String {
        match title {
            Some(QueryTitle::Text(text)) => {
                format!("<div class=\"custom-query-title\">{}</div>", escape(text))
            }
            Some(QueryTitle::Element { tag, text }) => {
                let tag = match tag.as_str() {
                    "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "p" | "span" | "b" | "strong"
                    | "i" | "em" | "div" | "code" => tag.as_str(),
                    _ => "div",
                };
                format!(
                    "<div class=\"custom-query-title\"><{tag}>{}</{tag}></div>",
                    escape(text)
                )
            }
            None => "<div class=\"custom-query-title\"></div>".to_owned(),
        }
    }

    fn simple_title(pages: bool) -> String {
        format!(
            "{SEARCH} {}",
            if pages {
                "Live query for pages"
            } else {
                "Live query"
            }
        )
    }

    /// A default journal query: an advanced map rendered collapsed with its count, or
    /// nothing when it has no results. Datalog maps are skipped with a diagnostic.
    pub fn default_query(
        &self,
        page: &str,
        text: &str,
        asset: &mut dyn FnMut(&Path, &str) -> Result<String, String>,
    ) -> QueryBox {
        let mut diagnostics = Vec::new();
        let query = match parse_advanced(text, self.today, self.formats) {
            Ok(query) => query,
            Err(error) => {
                diagnostics.push(format!("Default journal query skipped: {error}"));
                return QueryBox {
                    html: String::new(),
                    diagnostics,
                };
            }
        };
        let AdvancedBody::Simple(simple) = &query.query else {
            return QueryBox {
                html: String::new(),
                diagnostics,
            };
        };
        let results = self.evaluate(simple, None, page, None, !query.remove_block_children);
        if results.is_empty() {
            return QueryBox {
                html: String::new(),
                diagnostics,
            };
        }
        let pages = matches!(results, QueryResults::Pages(_));
        let view = View {
            table: pages || query.table_view == Some(true),
            toggle: !pages && query.table_view.is_none(),
            grouped: query.group_by_page != Some(false) && simple.group_by_page != Some(false),
            collapsed: true,
            count_when_collapsed: true,
            columns: None,
            sort_by: None,
            descending: true,
        };
        let html = self.render_box(
            page,
            &Self::title_html(query.title.as_ref()),
            None,
            Some(&results),
            None,
            &view,
            asset,
            &mut diagnostics,
        );
        QueryBox { html, diagnostics }
    }

    fn advanced_box(
        &self,
        page: &str,
        block: Option<usize>,
        query: &AdvancedQuery,
        asset: &mut dyn FnMut(&Path, &str) -> Result<String, String>,
    ) -> QueryBox {
        let mut diagnostics = Vec::new();
        let id = self.block_id(page, block);
        let title = Self::title_html(query.title.as_ref());
        let line = self.line(page, block);
        let mut view = self.view(id, false);
        view.collapsed |= query.collapsed;
        let failure = if query.query == AdvancedBody::Datalog {
            Some((
                "Query failed:",
                "This published site evaluates queries in the simple query language only; Datalog queries are not supported.",
            ))
        } else if query.inputs || query.rules {
            Some((
                "Query failed:",
                "This published site does not support :inputs or :rules.",
            ))
        } else if query.view {
            Some((
                "Custom view failed:",
                "This published site does not run :view functions.",
            ))
        } else {
            None
        };
        if let Some((label, message)) = failure {
            diagnostics.push(format!("{label} {message}{line}"));
            view.toggle = false;
            let html = self.render_box(
                page,
                &title,
                None,
                None,
                Some(Self::failed(label, message)),
                &view,
                asset,
                &mut diagnostics,
            );
            return QueryBox { html, diagnostics };
        }
        if query.result_transform {
            diagnostics.push(format!(
                "Query :result-transform ignored; results shown untransformed{line}"
            ));
        }
        let AdvancedBody::Simple(simple) = &query.query else {
            let html = self.render_box(
                page,
                &title,
                None,
                None,
                None,
                &view,
                asset,
                &mut diagnostics,
            );
            return QueryBox { html, diagnostics };
        };
        let results = self.evaluate(simple, id, page, block, !query.remove_block_children);
        let pages = matches!(results, QueryResults::Pages(_));
        view.table |= pages || query.table_view == Some(true) || simple.table;
        view.toggle = !pages && query.table_view.is_none();
        view.grouped = query.group_by_page != Some(false)
            && simple.group_by_page != Some(false)
            && !query.result_transform;
        let html = self.render_box(
            page,
            &title,
            None,
            Some(&results),
            None,
            &view,
            asset,
            &mut diagnostics,
        );
        QueryBox { html, diagnostics }
    }
}

impl QueryBoxes for Queries<'_, '_> {
    fn simple(
        &self,
        page: &str,
        block: Option<usize>,
        text: &str,
        asset: &mut dyn FnMut(&Path, &str) -> Result<String, String>,
    ) -> QueryBox {
        let mut diagnostics = Vec::new();
        let line = self.line(page, block);
        let query = match parse_simple(text, self.today, self.formats) {
            Ok(query) => query,
            Err(error) => {
                if let QueryErrorKind::Unsupported(_) = error.kind {
                    diagnostics.push(format!("Query failed: {error}{line}"));
                    let view = View {
                        toggle: false,
                        ..self.view(None, false)
                    };
                    let html = self.render_box(
                        page,
                        &Self::simple_title(false),
                        None,
                        None,
                        Some(Self::failed("Query failed:", &error.to_string())),
                        &view,
                        asset,
                        &mut diagnostics,
                    );
                    return QueryBox { html, diagnostics };
                }
                diagnostics.push(format!("Query Error: {error}{line}"));
                return QueryBox {
                    html: Self::error_box("Query Error:", text),
                    diagnostics,
                };
            }
        };
        let id = self.block_id(page, block);
        let pages = query.returns_pages();
        let mut view = self.view(id, pages);
        view.table |= query.table;
        view.grouped = query.group_by_page != Some(false);
        let builder = Some(query.clause_text());
        let results = query
            .clause
            .is_some()
            .then(|| self.evaluate(&query, id, page, block, false));
        let html = self.render_box(
            page,
            &Self::simple_title(pages),
            builder,
            results.as_ref(),
            None,
            &view,
            asset,
            &mut diagnostics,
        );
        QueryBox { html, diagnostics }
    }

    fn advanced(
        &self,
        page: &str,
        block: Option<usize>,
        text: &str,
        asset: &mut dyn FnMut(&Path, &str) -> Result<String, String>,
    ) -> QueryBox {
        match parse_advanced(text, self.today, self.formats) {
            Ok(query) => self.advanced_box(page, block, &query, asset),
            Err(error) => {
                let line = self.line(page, block);
                let heading = match error.kind {
                    QueryErrorKind::InvalidMap(_) => "Invalid query:",
                    _ => "Query Error:",
                };
                QueryBox {
                    html: Self::error_box(heading, text.trim()),
                    diagnostics: vec![format!("{heading} {error}{line}")],
                }
            }
        }
    }
}

/// A result list or table split into pieces that can move to fragment files.
struct Parts {
    head: String,
    items: Vec<String>,
    columns: usize,
}

fn gzip_size(text: &str) -> usize {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(text.as_bytes())
        .expect("writing to memory");
    encoder.finish().expect("writing to memory").len()
}

/// A block's first line without its bullet, task marker or properties.
fn first_line(text: &str) -> String {
    let line = text.trim_start().lines().next().unwrap_or_default().trim();
    line.strip_prefix("- ").unwrap_or(line).trim().to_owned()
}

/// `yyyy-MM-dd HH:mm` in UTC.
fn utc(seconds: i64) -> (String, String) {
    let date = JournalDate::from_unix_days(seconds.div_euclid(86_400));
    let minutes = seconds.rem_euclid(86_400) / 60;
    let shown = format!(
        "{:04}-{:02}-{:02} {:02}:{:02}",
        date.year(),
        date.month(),
        date.day(),
        minutes / 60,
        minutes % 60
    );
    (shown.clone(), shown)
}
