//! Which pages a publication contains under Logseq's graph-wide
//! `:publishing/all-pages-public?` setting and the older `:all-pages-public?`.

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use logseq_gardener_sdk::page_titles::FilenameFormat;
use logseq_gardener_sdk::publishing::NamespaceSelection;
use logseq_gardener_sdk::site::{SiteOptions, SiteReport, publish_site_with_options};

static NEXT: AtomicUsize = AtomicUsize::new(0);

struct Fixture(PathBuf);

impl Fixture {
    /// A garden with one page of each kind; `config` is `logseq/config.edn`, or none.
    fn new(config: Option<&str>) -> Self {
        let root = std::env::temp_dir().join(format!(
            "lsg-public-pages-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(root.join("garden/pages")).unwrap();
        let fixture = Self(root);
        if let Some(config) = config {
            fixture.write("logseq/config.edn", config);
        }
        fixture.write("pages/Notes.md", "- UNMARKED_PARENT_SENTINEL\n");
        fixture.write(
            "pages/Notes___Marked.md",
            "public:: true\n\n- MARKED_SENTINEL [[Notes/Unmarked]] [[Notes/Nickname]]\n",
        );
        fixture.write("pages/Notes___Unmarked.md", "- UNMARKED_SENTINEL\n");
        fixture.write(
            "pages/Notes___Private.md",
            "public:: false\n\n- PRIVATE_SENTINEL\n",
        );
        fixture.write(
            "pages/Notes___Late.md",
            "- LATE_SENTINEL\n  public:: true\n",
        );
        fixture.write(
            "pages/Notes___Aliased.md",
            "alias:: Notes/Nickname\n\n- ALIASED_SENTINEL\n",
        );
        fixture.write(
            "pages/Notes___Nickname.md",
            "public:: true\n\n- NICKNAME_SENTINEL\n",
        );
        fixture.write(
            "pages/Elsewhere.md",
            "public:: true\n\n- ELSEWHERE_SENTINEL\n",
        );
        fixture.write(
            "journals/2026_10_30.md",
            "public:: true\n\n- MARKED_JOURNAL_SENTINEL\n",
        );
        fixture.write("journals/2026_10_31.md", "- UNMARKED_JOURNAL_SENTINEL\n");
        fixture
    }

    fn write(&self, name: &str, text: &str) {
        let path = self.0.join("garden").join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    fn publish(&self) -> (SiteReport, String) {
        let report = publish_site_with_options(
            self.0.join("garden"),
            self.0.join("site"),
            FilenameFormat::TripleLowbar,
            &NamespaceSelection::new(&["Notes"], &[]).unwrap(),
            &SiteOptions {
                journals: true,
                ..SiteOptions::default()
            },
        )
        .unwrap();
        let mut text = String::new();
        for entry in fs::read_dir(self.0.join("site")).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_some_and(|e| e == "html" || e == "json") {
                text.push_str(&fs::read_to_string(path).unwrap());
            }
        }
        (report, text)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

const PAGE_SENTINELS: [&str; 7] = [
    "UNMARKED_PARENT_SENTINEL",
    "MARKED_SENTINEL",
    "UNMARKED_SENTINEL",
    "LATE_SENTINEL",
    "ALIASED_SENTINEL",
    "NICKNAME_SENTINEL",
    "UNMARKED_JOURNAL_SENTINEL",
];

/// Logseq publishes only pages marked `public:: true`, and pages aliased to them.
fn assert_marked_pages_only(config: &str) {
    let fixture = Fixture::new(Some(config));
    let (report, text) = fixture.publish();
    for sentinel in [
        "MARKED_SENTINEL",
        "NICKNAME_SENTINEL",
        "ALIASED_SENTINEL",
        "MARKED_JOURNAL_SENTINEL",
    ] {
        assert!(text.contains(sentinel), "{config}: {sentinel} missing");
    }
    for sentinel in [
        "UNMARKED_PARENT_SENTINEL",
        "UNMARKED_SENTINEL",
        "LATE_SENTINEL",
        "PRIVATE_SENTINEL",
        "UNMARKED_JOURNAL_SENTINEL",
        "ELSEWHERE_SENTINEL",
    ] {
        assert!(!text.contains(sentinel), "{config}: {sentinel} published");
    }
    assert_eq!((report.pages, report.journals), (3, 1), "{config}");
    // Notes, Notes/Unmarked, Notes/Late, Notes/Private and the unmarked journal.
    assert_eq!(report.withheld_pages, 5, "{config}");
    assert!(
        report
            .diagnostics
            .iter()
            .any(|d| d.source.ends_with("Notes___Unmarked.md")
                && d.message.contains("public:: true")),
        "{config}: {:?}",
        report.diagnostics
    );
}

/// Every selected page is published except those marked `public:: false`.
fn assert_all_pages(config: Option<&str>) {
    let fixture = Fixture::new(config);
    let (report, text) = fixture.publish();
    for sentinel in PAGE_SENTINELS.iter().chain(&["MARKED_JOURNAL_SENTINEL"]) {
        assert!(text.contains(sentinel), "{config:?}: {sentinel} missing");
    }
    for sentinel in ["PRIVATE_SENTINEL", "ELSEWHERE_SENTINEL"] {
        assert!(!text.contains(sentinel), "{config:?}: {sentinel} published");
    }
    assert_eq!((report.pages, report.journals), (6, 2), "{config:?}");
    assert_eq!(report.withheld_pages, 1, "{config:?}");
}

#[test]
fn setting_absent_publishes_only_marked_pages() {
    assert_marked_pages_only("{:file/name-format :triple-lowbar}");
    assert_marked_pages_only("{;; :publishing/all-pages-public? true\n}");
}

#[test]
fn setting_false_publishes_only_marked_pages() {
    assert_marked_pages_only("{:publishing/all-pages-public? false}");
    assert_marked_pages_only("{:all-pages-public? false}");
}

#[test]
fn setting_true_publishes_every_selected_page_except_private_ones() {
    assert_all_pages(Some("{:publishing/all-pages-public? true}"));
}

#[test]
fn older_setting_true_publishes_every_selected_page() {
    assert_all_pages(Some("{:all-pages-public? true}"));
    // Logseq's export reads the older key whenever the newer one is not true.
    assert_all_pages(Some(
        "{:publishing/all-pages-public? false :all-pages-public? true}",
    ));
}

#[test]
fn garden_without_config_publishes_every_selected_page() {
    assert_all_pages(None);
}

#[test]
fn malformed_setting_publishes_only_marked_pages_with_a_diagnostic() {
    let fixture = Fixture::new(Some(r#"{:publishing/all-pages-public? "yes"}"#));
    let (report, text) = fixture.publish();
    assert!(text.contains("MARKED_SENTINEL") && !text.contains("UNMARKED_SENTINEL"));
    assert!(
        report
            .diagnostics
            .iter()
            .any(|d| d.message.contains(":publishing/all-pages-public?")),
        "{:?}",
        report.diagnostics
    );
}
