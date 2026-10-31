use logseq_gardener_sdk::page_titles::FilenameFormat;
use logseq_gardener_sdk::publishing::NamespaceSelection;
use logseq_gardener_sdk::site::{SiteOptions, SiteReport, publish_site_with_options};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);

struct Fixture(PathBuf);

impl Fixture {
    fn new(config: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "lsg-shell-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(root.join("garden/pages")).unwrap();
        fs::create_dir_all(root.join("garden/logseq")).unwrap();
        // These gardens publish every page, so the tests can focus on other settings.
        let config = config.replacen('{', "{:publishing/all-pages-public? true ", 1);
        fs::write(root.join("garden/logseq/config.edn"), config).unwrap();
        for (file, text) in [
            ("tmux.md", "- TMUX_BODY\n"),
            (
                "gh-dash.md",
                "alias:: GitHub/CLI/Extension/gh-dash\n- DASH_BODY\n",
            ),
            ("Private.md", "public:: false\n- SECRET\n"),
            ("Other.md", "- OTHER_BODY\n"),
        ] {
            fs::write(root.join("garden/pages").join(file), text).unwrap();
        }
        Self(root)
    }

    fn publish(&self, homepage: Option<&str>) -> SiteReport {
        publish_site_with_options(
            self.0.join("garden"),
            self.0.join("site"),
            FilenameFormat::TripleLowbar,
            &NamespaceSelection::new(&["tmux", "gh-dash", "Private", "Other"], &[]).unwrap(),
            &SiteOptions {
                stylesheet: None,
                homepage: homepage.map(str::to_owned),
                ..SiteOptions::default()
            },
        )
        .unwrap()
    }

    fn page(&self, marker: &str) -> String {
        fs::read_dir(self.0.join("site"))
            .unwrap()
            .flatten()
            .filter(|entry| entry.file_name().to_string_lossy().starts_with("p-"))
            .map(|entry| fs::read_to_string(entry.path()).unwrap())
            .find(|html| html.contains(marker))
            .unwrap()
    }

    fn file(&self, name: &str) -> String {
        fs::read_to_string(self.0.join("site").join(name)).unwrap()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

const CONFIG: &str =
    r#"{:favorites ["github/cli/extension/gh-dash" "tmux" "private" "missing" "TMUX" "gh-dash"]}"#;

fn before<'a>(html: &'a str, marker: &str) -> &'a str {
    &html[..html.find(marker).unwrap()]
}

#[test]
fn header_and_sidebar_follow_logseq_order_with_resolved_favorites() {
    let f = Fixture::new(CONFIG);
    let report = f.publish(None);
    let tmux = f.page("TMUX_BODY");
    let frame = before(&tmux, "<main>");
    let order = [
        "aria-label=\"Toggle left sidebar\"",
        "aria-label=\"Home\"",
        "aria-label=\"More\"",
        "aria-label=\"Toggle right sidebar\"",
        "data-lsg-all-pages",
        "Favorites</h2>",
        ">GitHub/CLI/Extension/gh-dash</a>",
        "aria-current=\"page\">tmux</a>",
    ];
    let positions: Vec<_> = order.iter().map(|item| frame.find(item).unwrap()).collect();
    assert!(
        positions.windows(2).all(|pair| pair[0] < pair[1]),
        "{frame}"
    );
    assert_eq!(frame.matches(">tmux</a>").count(), 1);
    let alias = frame.find(">GitHub/CLI/Extension/gh-dash</a>").unwrap();
    assert!(frame[alias + 1..].contains(">gh-dash</a>"), "{frame}");
    assert!(!frame.contains("SECRET") && !frame.contains("missing"));
    assert!(frame.contains("<a href=\"index.html\" data-lsg-all-pages>"));
    assert!(tmux.contains("<script defer src=\"lsg-shell.js\"></script>"));
    let diagnostics: Vec<_> = report
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.message.starts_with("Favorite"))
        .map(|diagnostic| diagnostic.message.clone())
        .collect();
    assert_eq!(diagnostics.len(), 2, "{diagnostics:?}");
    assert!(
        diagnostics
            .iter()
            .all(|m| m.contains("names no published page"))
    );
    // The build strips the script's comment lines and indentation.
    let shell = f.file("lsg-shell.js");
    assert!(shell.contains("window.logseqGardenerShell = true;"));
    assert!(shell.contains("window.logseqGardenerCards(event)"));
    assert!(!shell.contains("\n  "));
    let index = f.file("index.html");
    let frame = before(&index, "<main>");
    assert!(!frame.contains("aria-label=\"Home\""));
    assert!(frame.contains("data-lsg-all-pages aria-current=\"page\""));
}

#[test]
fn homepage_marks_its_favorite_and_all_pages_opens_the_page_list() {
    let f = Fixture::new(CONFIG);
    f.publish(Some("tmux"));
    let index = f.file("index.html");
    let frame = before(&index, "<main>");
    assert!(!frame.contains("aria-label=\"Home\""));
    assert!(frame.contains("<a href=\"pages.html\" data-lsg-all-pages>"));
    assert!(frame.contains("aria-current=\"page\">tmux</a>"));
    let tmux = f.page("TMUX_BODY");
    assert!(before(&tmux, "<main>").contains("aria-label=\"Home\""));
    assert!(before(&f.file("pages.html"), "<main>").contains("data-lsg-all-pages aria-current"));
}

#[test]
fn gardens_without_favorites_omit_the_heading_and_keep_controls_hidden_without_script() {
    let f = Fixture::new("{}");
    f.publish(None);
    let other = f.page("OTHER_BODY");
    let frame = before(&other, "<main>");
    assert!(!frame.contains("Favorites</h2>"));
    for control in [
        "data-lsg-left-toggle",
        "data-lsg-more",
        "data-lsg-right-toggle",
    ] {
        let start = frame.find(control).unwrap();
        assert!(frame[start..frame[start..].find('>').unwrap() + start].contains(" hidden"));
    }
    assert!(frame.contains("<div id=\"lsg-more-menu\" role=\"menu\" aria-label=\"More\" hidden>"));
}

#[test]
fn view_as_slides_is_the_first_more_item_on_page_routes_only() {
    let f = Fixture::new(CONFIG);
    f.publish(Some("Other"));
    let item = "<button type=\"button\" role=\"menuitem\" data-lsg-slides>View as slides</button>";
    let tmux = f.page("TMUX_BODY");
    assert!(
        tmux.contains(&format!("aria-label=\"More\" hidden>{item}<button type=\"button\" role=\"menuitem\" data-lsg-theme-toggle>")),
        "{tmux}"
    );
    // The home page, All pages and the graph view have no deck to show.
    for name in ["index.html", "pages.html", "graph.html"] {
        let html = f.file(name);
        assert!(html.contains("data-lsg-theme-toggle"), "{name}");
        assert!(!html.contains(item), "{name}");
    }
    // The right sidebar does not exist below 640px, so neither does the item.
    assert!(tmux.contains(
        "@media(max-width:639.98px){.lsg-right-toggle,[data-lsg-slides]{display:none!important}}"
    ));
}

#[test]
fn shell_cost_is_fixed_per_page() {
    let f = Fixture::new(CONFIG);
    f.publish(None);
    // The [shell] script_kb budget in scripts/parity/budgets.toml, raised for the
    // flashcard review's loader.
    assert!(f.file("lsg-shell.js").len() <= 6_900);
    let other = f.page("OTHER_BODY");
    let frame = &other[other.find("<body>").unwrap()..other.find("<main>").unwrap()];
    // The graph view's header link and sidebar item added about 300 bytes, and the
    // Flashcards item with its icon about 400; the [shell] frame_kb budget in
    // scripts/parity/budgets.toml is 6,000.
    assert!(frame.len() <= 3_700, "{}", frame.len());
}

#[test]
fn default_sidebar_cards_reach_the_toggle_and_unresolved_names_are_reported() {
    let f = Fixture::new(r#"{:default-home {:sidebar ["Contents" "tmux" "missing"]}}"#);
    let report = f.publish(None);
    let tmux = f.page("TMUX_BODY");
    let start = tmux.find("data-lsg-right-toggle").unwrap();
    let button = &tmux[start..start + tmux[start..].find('>').unwrap()];
    assert!(
        button.contains("data-lsg-sidebar-default=\"contents p-"),
        "{button}"
    );
    assert!(!button.contains("data-lsg-contents"), "{button}");
    let diagnostics: Vec<_> = report
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.message.starts_with("Default sidebar"))
        .collect();
    assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    assert!(diagnostics[0].message.contains("\"missing\""));
    let sidebar = f.file("lsg-sidebar.js");
    assert!(sidebar.contains("window.logseqGardenerSidebar"));
    assert!(!sidebar.contains("\n//") && !sidebar.contains("\n "));
    assert!(f.file("lsg-sidebar.css").contains(".lsg-right-sidebar"));
}

#[test]
fn graph_entry_points_use_logseqs_classes_and_follow_the_shortcut_setting() {
    let f = Fixture::new(CONFIG);
    f.publish(None);
    let tmux = f.page("TMUX_BODY");
    let frame = before(&tmux, "<main>");
    assert!(frame.contains("<header class=\"lsg-header cp__header\">"));
    let order = [
        "aria-label=\"Home\"",
        "href=\"graph.html#/graph\" aria-label=\"Graph\"",
        "aria-label=\"More\"",
        "<li class=\"graph-view-nav\"><a href=\"graph.html\" data-lsg-graph-view data-lsg-graph-shortcut>",
        "data-lsg-all-pages",
    ];
    let positions: Vec<_> = order.iter().map(|item| frame.find(item).unwrap()).collect();
    assert!(
        positions.windows(2).all(|pair| pair[0] < pair[1]),
        "{frame}"
    );
    assert!(frame.contains(">g g</span>"));
    // The graph page itself marks its sidebar item and exists.
    let graph = f.file("graph.html");
    assert!(
        before(&graph, "<main>")
            .contains("data-lsg-graph-view data-lsg-graph-shortcut aria-current=\"page\"")
    );
    assert!(graph.contains("class=\"graph-filters\""));
    assert!(graph.contains("<script defer src=\"lsg-graph.js\"></script>"));
    // No other page loads graph data or the graph script.
    assert!(!tmux.contains("_graph/") && !tmux.contains("lsg-graph.js"));

    let off = Fixture::new(r#"{:shortcuts {:go/graph-view false}}"#);
    off.publish(None);
    let frame = off.page("TMUX_BODY");
    let frame = before(&frame, "<main>");
    assert!(
        frame.contains("<li class=\"graph-view-nav\"><a href=\"graph.html\" data-lsg-graph-view>")
    );
    assert!(!frame.contains("g g"));
}
