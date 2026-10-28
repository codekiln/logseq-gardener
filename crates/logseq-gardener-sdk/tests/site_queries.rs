use logseq_gardener_sdk::{
    journals::JournalDate,
    page_titles::FilenameFormat,
    publishing::NamespaceSelection,
    site::{SiteOptions, SiteReport, publish_site_with_options},
};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};
static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "lsg-queries-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(root.join("garden/pages")).unwrap();
        fs::create_dir_all(root.join("garden/journals")).unwrap();
        Self(root)
    }
    fn write(&self, path: &str, text: &str) {
        let path = self.0.join("garden").join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }
    fn publish(&self) -> SiteReport {
        publish_site_with_options(
            self.0.join("garden"),
            self.0.join("site"),
            FilenameFormat::TripleLowbar,
            &NamespaceSelection::new(&["Notes"], &[]).unwrap(),
            &SiteOptions {
                journals: true,
                build_date: JournalDate::new(2026, 10, 27),
                ..SiteOptions::default()
            },
        )
        .unwrap()
    }
    fn route(title: &str) -> String {
        let hash = title
            .as_bytes()
            .iter()
            .fold(0xcbf2_9ce4_8422_2325_u64, |h, b| {
                (h ^ u64::from(*b)).wrapping_mul(0x100000001b3)
            });
        format!("p-{hash:016x}")
    }
    fn html(&self, title: &str) -> String {
        fs::read_to_string(self.0.join(format!("site/{}.html", Self::route(title)))).unwrap()
    }
    fn file(&self, route: &str) -> String {
        fs::read_to_string(self.0.join("site").join(route)).unwrap()
    }
    fn fragments(&self, title: &str) -> Vec<String> {
        let stem = format!("{}-q", Self::route(title));
        let mut names: Vec<_> = fs::read_dir(self.0.join("site/_refs"))
            .into_iter()
            .flatten()
            .flatten()
            .map(|entry| entry.file_name().to_string_lossy().into_owned())
            .filter(|name| name.starts_with(&stem))
            .collect();
        names.sort();
        names
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn garden() -> Fixture {
    let fixture = Fixture::new();
    fixture.write(
        "pages/Notes___tmux.md",
        "alias:: terminal multiplexer\ntags:: [[Notes/Tool]]\n\n- tmux splits panes\n",
    );
    fixture.write(
        "pages/Notes___yazi.md",
        "tags:: [[Notes/Tool]]\nkind:: file manager\n\n- TODO try [[Notes/tmux]] with yazi\n\t- child names [[Notes/tmux]] again\n- yazi is a lightweight database of files\n",
    );
    fixture.write("pages/Notes___Tool.md", "- Tools I use\n");
    fixture.write("pages/Notes___Plain.md", "- No query on this page\n");
    fixture.write(
        "journals/2026_10_20.md",
        "- DOING read about [[Notes/tmux]]\n",
    );
    fixture.write("journals/2026_08_01.md", "- old note on [[Notes/tmux]]\n");
    fixture
}

#[test]
fn simple_query_shows_header_clause_row_and_groups() {
    let fixture = garden();
    fixture.write(
        "pages/Notes___Queries.md",
        "- Tools\n\t- {{query (and [[Notes/tmux]] (not [[Notes/Plain]]))}}\n",
    );
    let report = fixture.publish();
    let html = fixture.html("Notes/Queries");
    assert!(html.contains("<div class=\"custom-query\" data-lsg-query=\"q1\">"));
    assert!(html.contains("Live query</div>"));
    // tmux's properties and its block, the task on yazi, and the two journals.
    assert!(html.contains("<span class=\"results-count\">5 results</span>"));
    assert!(html.contains(
        "<div class=\"query-builder\">( AND [[Notes/tmux]] ( NOT [[Notes/Plain]] ) )</div>"
    ));
    // Journals first, newest first, then pages by name.
    let order: Vec<_> = [
        "Oct 20th, 2026",
        "Aug 1st, 2026",
        "Notes/tmux",
        "Notes/yazi",
    ]
    .iter()
    .map(|title| html.find(&format!(">{title}</a></summary>")).expect(title))
    .collect();
    assert!(order.windows(2).all(|pair| pair[0] < pair[1]), "{order:?}");
    let groups = html.matches("class=\"custom-query-page-result\"").count();
    assert_eq!(groups, 4);
    // The child of the task repeats the reference, but its parent is already a result: it
    // shows once, under the task.
    assert_eq!(html.matches("child names").count(), 1);
    assert!(html.contains("<link rel=\"stylesheet\" href=\"lsg-queries.css\">"));
    assert!(!html.contains("{{query"));
    assert!(fixture.file("lsg-queries.js").len() <= 2000);
    // A table view for the toggle, fetched from a fragment.
    let alt = fixture
        .fragments("Notes/Queries")
        .into_iter()
        .find(|name| name.contains("-alt-"))
        .unwrap();
    assert!(html.contains(&format!("data-alt=\"_refs/{alt}\"")));
    assert!(
        fixture
            .file(&format!("_refs/{alt}"))
            .contains("<table class=\"table-auto query-table\">")
    );
    assert!(
        !report
            .diagnostics
            .iter()
            .any(|d| d.source.ends_with("Notes___Queries.md")),
        "{:?}",
        report.diagnostics
    );
    // Pages without a query are unchanged by this feature.
    let plain = fixture.html("Notes/Plain");
    assert!(!plain.contains("lsg-queries"));
}

#[test]
fn page_queries_render_tables_and_chosen_columns() {
    let fixture = garden();
    fixture.write(
        "pages/Notes___Queries.md",
        "- {{query (page-tags [[Notes/Tool]])}}\n- {{query (page-tags [[Notes/Tool]])}}\n  query-properties:: [:page :kind]\n  query-sort-by:: page\n  query-sort-desc:: false\n",
    );
    fixture.publish();
    let html = fixture.html("Notes/Queries");
    assert_eq!(html.matches("Live query for pages").count(), 2);
    assert_eq!(html.matches("2 results").count(), 2);
    assert!(!html.contains("lsg-query-view"));
    let second = &html[html.rfind("data-lsg-query=\"q2\"").unwrap()..];
    assert!(second.contains("<thead><tr><th aria-sort=\"ascending\"><button type=\"button\" class=\"sort\">page</button></th><th><button type=\"button\" class=\"sort\">kind</button></th></tr></thead>"));
    let tmux = second.find(">Notes/tmux</a>").unwrap();
    let yazi = second.find(">Notes/yazi</a>").unwrap();
    assert!(tmux < yazi);
    assert!(second.contains("<td>file manager</td>"));
}

#[test]
fn relative_dates_text_tasks_and_empty_results() {
    let fixture = garden();
    fixture.write(
        "pages/Notes___Queries.md",
        "- {{query (between -30d today)}}\n- {{query \"lightweight database\"}}\n- {{query (task TODO DOING)}}\n- {{query [[Notes/Missing]]}}\n- {{query }}\n",
    );
    fixture.publish();
    let html = fixture.html("Notes/Queries");
    let boxes: Vec<_> = html.split("data-lsg-query=").skip(1).collect();
    assert_eq!(boxes.len(), 5);
    assert!(boxes[0].contains("1 result<"), "{}", boxes[0]);
    assert!(boxes[0].contains("between: -30d ~ today"));
    assert!(boxes[1].contains("1 result<"));
    assert!(boxes[2].contains("2 results"));
    assert!(boxes[3].contains("<div class=\"lsg-query-empty\">No matched result</div>"));
    assert!(boxes[3].contains("<span class=\"results-count\"></span>"));
    // A blank query shows its header and nothing else.
    assert!(!boxes[4].contains("No matched result"));
    assert!(boxes[4].contains("<div class=\"custom-query-results\"></div>"));
}

#[test]
fn advanced_queries_titles_collapse_and_errors() {
    let fixture = garden();
    fixture.write(
        "pages/Notes___Queries.md",
        concat!(
            "- #+BEGIN_QUERY\n  {:title [:h3 \"Blocks that mention tmux\"]\n   :query [[Notes/tmux]]\n   :collapsed? true}\n  #+END_QUERY\n",
            "- #+BEGIN_QUERY\n  {:title \"Open tasks\" :query [:find (pull ?b [*]) :where [?b :block/marker \"TODO\"]]}\n  #+END_QUERY\n",
            "- #+BEGIN_QUERY\n  {:title \"Broken\" :query (and [[Notes/tmux]]}\n  #+END_QUERY\n",
            "- {{query (and [[Notes/tmux]] \"x\"}}\n",
            "- {{query (between created-at -7d today)}}\n",
            "- #+BEGIN_QUERY\n  {:title \"Target\" :query [[Notes/Query Target]]}\n  #+END_QUERY\n",
        ),
    );
    fixture.write("pages/Notes___Query Target.md", "- target text\n");
    let report = fixture.publish();
    let html = fixture.html("Notes/Queries");
    // Collapsed: title, no count until opened, results only in fragments.
    assert!(html.contains("<details class=\"custom-query lsg-query-collapsed\" data-lsg-query=\"q1\" data-count=\"5 results\"><summary class=\"th\"><div class=\"title\"><div class=\"custom-query-title\"><h3>Blocks that mention tmux</h3></div></div><div class=\"flex\"><span class=\"results-count\"></span>"));
    assert!(!html.contains("tmux splits panes"));
    let first = fixture
        .fragments("Notes/Queries")
        .into_iter()
        .find(|name| name.contains("-q1-1"))
        .unwrap();
    assert!(
        fixture
            .file(&format!("_refs/{first}"))
            .contains("tmux splits panes")
    );
    assert!(html.contains("<div class=\"custom-query-title\">Open tasks</div>"));
    assert!(html.contains("<div class=\"warning my-1\">Query failed:<p>This published site evaluates queries in the simple query language only"));
    assert!(html.contains("<h5>Invalid query:</h5>"));
    assert!(html.contains(
        "<h5>Query Error:</h5><pre><code>(and [[Notes/tmux]] &quot;x&quot;</code></pre>"
    ));
    assert!(html.contains("Query failed:<p>the published site does not support the query form (between created-at)</p>"));
    let messages: Vec<_> = report
        .diagnostics
        .iter()
        .filter(|d| d.source.ends_with("Notes___Queries.md"))
        .map(|d| d.message.clone())
        .collect();
    assert_eq!(messages.len(), 4, "{messages:?}");
    assert!(messages.iter().all(|m| m.contains("(line ")));
    // A page named only inside an advanced query has no linked references from it.
    assert!(
        !fixture
            .html("Notes/Query Target")
            .contains("Linked Reference")
    );
}

#[test]
fn large_results_keep_the_rest_in_fragments() {
    let fixture = garden();
    let mut source = String::new();
    for page in 0..120 {
        let mut text = String::new();
        for block in 0..20 {
            text.push_str(&format!(
                "- block {block} on page {page} names [[Notes/tmux]] with some padding text {}\n",
                "lorem ipsum dolor sit amet ".repeat(4)
            ));
        }
        fixture.write(&format!("pages/Notes___Many {page:03}.md"), &text);
    }
    source.push_str("- {{query [[Notes/tmux]]}}\n");
    fixture.write("pages/Notes___Queries.md", &source);
    fixture.publish();
    let html = fixture.html("Notes/Queries");
    assert!(html.contains("2405 results"));
    let fragments: Vec<_> = fixture
        .fragments("Notes/Queries")
        .into_iter()
        .filter(|name| !name.contains("-alt-"))
        .collect();
    assert!(!fragments.is_empty());
    let in_page = html
        .matches("names <span class=\"page-reference\">")
        .count();
    assert!(in_page > 0 && in_page < 2400, "{in_page}");
    assert!(html.contains(&format!("data-next=\"_refs/{}\"", fragments[0])));
    // Every block is in the page or a fragment, once.
    let mut all = html.clone();
    for name in &fragments {
        all.push_str(&fixture.file(&format!("_refs/{name}")));
    }
    // The 2,400 filler blocks and yazi's child, shown under its task.
    assert_eq!(
        all.matches("names <span class=\"page-reference\">").count(),
        2401
    );
}

#[test]
fn datalog_default_journal_queries_are_skipped_with_a_diagnostic() {
    let fixture = garden();
    fixture.write(
        "logseq/config.edn",
        "{:default-queries {:journals [{:title \"NOW\" :query [:find (pull ?b [*]) :where [?b :block/marker ?m]]} {:title \"Tasks\" :query (task TODO DOING)}]}}",
    );
    fixture.write("journals/2026_10_27.md", "- today\n");
    let report = fixture.publish();
    let skipped = report
        .diagnostics
        .iter()
        .filter(|d| d.message.starts_with("Default journal query skipped"))
        .count();
    assert_eq!(skipped, 1);
    // The simple-language default query renders collapsed with its count under today's journal.
    let today = fixture.html("Oct 27th, 2026");
    assert!(today.contains(
        "<div class=\"lsg-default-queries\"><details class=\"custom-query lsg-query-collapsed\""
    ));
    assert!(today.contains("<span class=\"results-count\">2 results</span>"));
    assert!(fixture.file("index.html").contains("lsg-default-queries"));
}
