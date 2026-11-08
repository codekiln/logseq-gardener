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
            "lsg-advanced-queries-{}-{}",
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

const QUERY: &str = "[:find (pull ?p [*]) :where [?p :block/name] [?p :block/properties ?props] [(get ?props :logseq-entity) ?entities] [(contains? ?entities \"Logseq/Entity/Podcast/Episode\")] [(get ?props :public) ?public] [(= ?public true)]]";
const TRANSFORM: &str = "(fn [rows] (sort-by (fn [row] (get-in row [:block/properties :podcast-published-at])) (fn [a b] (compare b a)) rows))";
const VIEW: &str = "(fn [rows] (into [\"ul\"] (map (fn [row] [\"li\" [\"a\" {:href (str \"#/page/\" (clojure.string/replace (:block/name row) \"/\" \"%2F\"))} (:block/original-name row)]]) rows)))";

fn block(map: &str) -> String {
    let indented = map.replace('\n', "\n  ");
    format!("- #+BEGIN_QUERY\n  {indented}\n  #+END_QUERY\n")
}

fn gitpa_map(extra: &str) -> String {
    format!(
        "{{:title [:h2 \"Recent Ceremonies\"] :query {QUERY} :result-transform {TRANSFORM} :view {VIEW}{extra}}}"
    )
}

fn episode(fixture: &Fixture, name: &str, public: &str, at: &str) {
    fixture.write(
        &format!("pages/Notes___{name}.md"),
        &format!(
            "logseq-entity:: [[Logseq/Entity/Podcast/Episode]]\npublic:: {public}\npodcast-published-at:: {at}\n\n- notes on {name}\n"
        ),
    );
}

fn eighteen() -> Fixture {
    let fixture = Fixture::new();
    for n in 0..18 {
        episode(
            &fixture,
            &format!("Ep{n:02}"),
            "true",
            &format!("2026-01-{:02}T10:00:00Z", n + 1),
        );
    }
    episode(&fixture, "Draft", "false", "2026-02-01T10:00:00Z");
    episode(&fixture, "Quoted", "\"true\"", "2026-02-02T10:00:00Z");
    fixture.write("pages/Notes___NoProps.md", "- plain page\n");
    fixture.write(
        "pages/Notes___Other.md",
        "logseq-entity:: [[Other/Type]]\npublic:: true\n\n- other type\n",
    );
    // Not in a selected namespace, so withheld from the site.
    fixture.write(
        "pages/Secret___Ep.md",
        "logseq-entity:: [[Logseq/Entity/Podcast/Episode]]\npublic:: true\npodcast-published-at:: 2027-01-01T00:00:00Z\n\n- secret\n",
    );
    fixture
}

fn links(html: &str) -> Vec<(String, String)> {
    let start = html.find("<ul><li>").expect("a list");
    let end = start + html[start..].find("</ul>").unwrap();
    html[start..end]
        .split("<a href=\"")
        .skip(1)
        .map(|part| {
            let (href, rest) = part.split_once("\">").unwrap();
            let text = rest.split_once("</a>").unwrap().0;
            (href.to_owned(), text.to_owned())
        })
        .collect()
}

#[test]
fn gitpa_form_lists_public_episodes_newest_first() {
    let fixture = eighteen();
    fixture.write(
        "pages/Notes___Home.md",
        &block(&gitpa_map(" :table-view? false")),
    );
    let report = fixture.publish();
    let html = fixture.html("Notes/Home");
    assert!(html.contains("<div class=\"custom-query-title\"><h2>Recent Ceremonies</h2></div>"));
    assert!(html.contains("<span class=\"results-count\">18 results</span>"));
    // No table or list toggle.
    assert!(!html.contains("lsg-query-view"));
    let found = links(&html);
    assert_eq!(found.len(), 18);
    let titles: Vec<&str> = found.iter().map(|(_, text)| text.as_str()).collect();
    let expected: Vec<String> = (0..18).rev().map(|n| format!("Notes/Ep{n:02}")).collect();
    assert_eq!(titles, expected);
    // Each link is the page's published URL.
    assert_eq!(found[0].0, format!("{}.html", Fixture::route("Notes/Ep17")));
    // The pages the data holds nothing for are left out, and the withheld page too.
    for left_out in ["Draft", "Quoted", "NoProps", "Other"] {
        assert!(!html.contains(&format!("Notes/{left_out}")), "{left_out}");
    }
    assert!(!html.contains("Secret"));
    // The box adds no diagnostic.
    assert!(
        report
            .diagnostics
            .iter()
            .all(|d| !d.source.ends_with("Notes___Home.md")),
        "{:?}",
        report.diagnostics
    );
    // Links create no reference: the target page has no Linked References.
    assert!(!fixture.html("Notes/Ep00").contains("Linked Reference"));
}

#[test]
fn a_page_with_the_query_links_the_same_scripts_and_styles_as_any_query_page() {
    let fixture = eighteen();
    fixture.write("pages/Notes___Home.md", &block(&gitpa_map("")));
    fixture.write("pages/Notes___Simple.md", "- {{query [[Notes/Ep00]]}}\n");
    fixture.publish();
    let resources = |html: &str| -> Vec<String> {
        let mut found = Vec::new();
        for tag in ["<script", "<link"] {
            for part in html.split(tag).skip(1) {
                let end = part.find('>').unwrap();
                found.push(format!("{tag}{}", &part[..end]));
            }
        }
        found.sort();
        found
    };
    let home = fixture.html("Notes/Home");
    assert_eq!(resources(&home), resources(&fixture.html("Notes/Simple")));
    // The view writes no element of its own that runs code.
    let body = &home[home.find("custom-query-results").unwrap()..];
    let body = &body[..body.find("</ul>").unwrap()];
    assert!(!body.contains("<script") && !body.contains("<style"));
}

#[test]
fn text_and_attributes_are_escaped() {
    let fixture = Fixture::new();
    episode(&fixture, "A <b>&", "true", "2026-01-01T00:00:00Z");
    fixture.write(
        "pages/Notes___Home.md",
        &block(&format!(
            "{{:query {QUERY} :view (fn [rows] [\"p\" {{:title \"x\\\"y\"}} (:block/original-name (first rows)) \" & <i>\"])}}"
        )),
    );
    fixture.publish();
    let html = fixture.html("Notes/Home");
    assert!(
        html.contains("<p title=\"x&quot;y\">Notes/A &lt;b&gt;&amp; &amp; &lt;i&gt;</p>"),
        "{html}"
    );
}

#[test]
fn an_empty_result_calls_no_view_and_shows_no_count() {
    let fixture = Fixture::new();
    fixture.write("pages/Notes___Plain.md", "- nothing\n");
    // The view would throw if it ran.
    fixture.write(
        "pages/Notes___Home.md",
        &block(&format!(
            "{{:title \"Empty\" :query {QUERY} :view (fn [rows] (compare \"a\" 1))}}"
        )),
    );
    let report = fixture.publish();
    let html = fixture.html("Notes/Home");
    assert!(html.contains("<div class=\"custom-query-title\">Empty</div>"));
    assert!(html.contains("<span class=\"results-count\"></span>"));
    assert!(html.contains("<div class=\"lsg-query-empty\">No matched result</div>"));
    assert!(!html.contains("Custom view failed"));
    assert!(
        report
            .diagnostics
            .iter()
            .all(|d| !d.source.ends_with("Notes___Home.md"))
    );
}

#[test]
fn view_links_resolve_names_and_leave_unmatched_ones() {
    let fixture = Fixture::new();
    fixture.write("pages/Notes___Target.md", "- target\n");
    fixture.write(
        "pages/Notes___Home.md",
        &block(
            "{:query [:find (pull ?p [*]) :where [?p :block/name \"notes/home\"]]\n :view (fn [rows] [\"ul\" [\"li\" [\"a\" {:href \"#/page/notes%2Ftarget\"} \"T\"]] [\"li\" [\"a\" {:href \"#/page/Secret%2FHidden\"} \"H\"]] [\"li\" [\"a\" {:href \"#/page/NotAPage\"} \"N\"]]])}",
        ),
    );
    fixture.publish();
    let html = fixture.html("Notes/Home");
    assert!(html.contains(&format!(
        "<a href=\"{}.html\">T</a>",
        Fixture::route("Notes/Target")
    )));
    assert!(html.contains("<a href=\"#/page/Secret%2FHidden\">H</a>"));
    assert!(html.contains("<a href=\"#/page/NotAPage\">N</a>"));
    assert!(!fixture.html("Notes/Target").contains("Linked Reference"));
}

#[test]
fn rows_without_a_view_show_the_page_table() {
    let fixture = eighteen();
    fixture.write(
        "pages/Notes___Home.md",
        &block(&format!(
            "{{:title \"Rows\" :query {QUERY} :result-transform {TRANSFORM}}}"
        )),
    );
    fixture.publish();
    let html = fixture.html("Notes/Home");
    assert!(html.contains("<span class=\"results-count\">18 results</span>"));
    assert!(html.contains("query-table"));
    let first = html.find("Notes/Ep17").unwrap();
    let last = html.find("Notes/Ep00").unwrap();
    assert!(first < last);
}

#[test]
fn failures_name_the_form_and_the_diagnostic_carries_the_line() {
    let fixture = eighteen();
    let query = |extra: &str| block(&format!("{{:title \"T\" :query {QUERY} {extra}}}"));
    fixture.write(
        "pages/Notes___Home.md",
        &[
            query(":result-transform (fn [rows] (partition 2 rows))"),
            query(":view (fn [rows] (slurp \"x\"))"),
            query(":view (fn [rows] (compare \"a\" 1))"),
            query(":view (fn [rows] [\"script\" \"x\"])"),
            query(":view (fn [rows] [\"p\" {:onclick \"x()\"} \"x\"])"),
            query(":view (fn [rows] [\"a\" {:href \"javascript:alert(1)\"} \"x\"])"),
            query(":result-transform (fn [rows] (sort [\"a\" 1]))"),
            block("{:title \"Blocks\" :query [:find (pull ?b [*]) :where [?b :block/marker ?m]]}"),
        ]
        .concat(),
    );
    let report = fixture.publish();
    let html = fixture.html("Notes/Home");
    let warning = |label: &str, message: &str| {
        assert!(
            html.contains(&format!("<div class=\"warning my-1\">{label}<p>{message}")),
            "{label} {message}"
        );
    };
    warning(
        "Query failed:",
        "the published site does not support the query form (partition)",
    );
    warning(
        "Custom view failed:",
        "the published site does not support the query form (slurp)",
    );
    warning("Custom view failed:", "Cannot compare a to 1");
    // The failed view still shows how many rows it was given, as Logseq does.
    assert!(html.contains("<span class=\"results-count\">18 results</span></div></div><div class=\"custom-query-results\"><div class=\"warning my-1\">Custom view failed:<p>Cannot compare"));
    warning(
        "Custom view failed:",
        "the element &lt;script&gt; is not allowed in a view",
    );
    warning(
        "Custom view failed:",
        "the attribute onclick is not allowed in a view",
    );
    warning(
        "Custom view failed:",
        "the link scheme javascript: is not allowed in a view",
    );
    warning(
        "Query failed:",
        "the published site does not support the query form (:block/marker)",
    );
    assert!(!html.contains("<script>x"));
    // A transform that throws shows the untransformed rows, in page-name order.
    assert!(html.contains("<span class=\"results-count\">18 results</span>"));
    let messages: Vec<&String> = report
        .diagnostics
        .iter()
        .filter(|d| d.source.ends_with("Notes___Home.md"))
        .map(|d| &d.message)
        .collect();
    assert_eq!(messages.len(), 8, "{messages:?}");
    assert!(
        messages.iter().all(|m| m.contains("(line ")),
        "{messages:?}"
    );
    // The first query's unsupported form is on the line it is written on (block at line 1,
    // map on line 2).
    assert!(messages[0].contains("partition"), "{messages:?}");
    assert!(messages[0].ends_with("(line 2)"), "{messages:?}");
    assert!(messages[6].contains("Cannot compare") && messages[6].contains("untransformed"));
}

#[test]
fn a_view_over_the_size_limit_fails() {
    let fixture = eighteen();
    fixture.write(
        "pages/Notes___Home.md",
        &block(&format!(
            "{{:query {QUERY} :view (fn [rows] (into [\"ul\"] (map (fn [r] [\"li\" (clojure.string/join (map (fn [x] (str (:block/name r) \"................................................................\")) (take 20 (concat rows rows rows rows rows rows rows rows))))]) (concat rows rows rows rows rows rows rows rows))))}}"
        )),
    );
    fixture.publish();
    let html = fixture.html("Notes/Home");
    assert!(html.contains("Custom view failed:<p>the view wrote more than 128 KB of HTML; narrow the query or use take"));
}

#[test]
fn a_collapsed_query_loads_the_view_from_a_fragment() {
    let fixture = eighteen();
    fixture.write(
        "pages/Notes___Home.md",
        &block(&gitpa_map(" :collapsed? true")),
    );
    fixture.publish();
    let html = fixture.html("Notes/Home");
    assert!(html.contains("<details class=\"custom-query lsg-query-collapsed\""));
    assert!(html.contains("data-count=\"18 results\""));
    assert!(!html.contains("Notes/Ep00"));
    let first = fixture
        .fragments("Notes/Home")
        .into_iter()
        .find(|name| name.contains("-q1-1"))
        .unwrap();
    let fragment = fixture.file(&format!("_refs/{first}"));
    assert!(fragment.contains("<ul><li><a href="));
    assert_eq!(links(&fragment).len(), 18);
}

#[test]
fn simple_queries_keep_a_clear_message_for_functions() {
    let fixture = eighteen();
    fixture.write(
        "pages/Notes___Home.md",
        &[
            block("{:query [[Notes/Ep00]] :view (fn [rows] [\"p\" \"x\"])}"),
            block("{:query [[Notes/Ep00]] :result-transform (fn [rows] rows)}"),
        ]
        .concat(),
    );
    let report = fixture.publish();
    let html = fixture.html("Notes/Home");
    assert!(html.contains("Custom view failed:<p>Block results do not support :view functions when this site is built."));
    assert!(report.diagnostics.iter().any(|d| {
        d.message
            .contains("block results do not support functions when this site is built")
    }));
}

#[test]
fn repeated_builds_write_the_same_page() {
    let first = eighteen();
    first.write("pages/Notes___Home.md", &block(&gitpa_map("")));
    first.publish();
    let second = eighteen();
    second.write("pages/Notes___Home.md", &block(&gitpa_map("")));
    second.publish();
    assert_eq!(first.html("Notes/Home"), second.html("Notes/Home"));
}
