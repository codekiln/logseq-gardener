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
    assert_eq!(f.file("lsg-shell.js"), include_str!("../src/site/shell.js"));
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
fn shell_cost_is_fixed_per_page() {
    let f = Fixture::new(CONFIG);
    f.publish(None);
    assert!(include_str!("../src/site/shell.js").len() <= 6_500);
    let other = f.page("OTHER_BODY");
    let frame = &other[other.find("<body>").unwrap()..other.find("<main>").unwrap()];
    assert!(frame.len() <= 3_000, "{}", frame.len());
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
