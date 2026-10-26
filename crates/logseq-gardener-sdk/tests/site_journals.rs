use logseq_gardener_sdk::journals::JournalDate;
use logseq_gardener_sdk::page_titles::FilenameFormat;
use logseq_gardener_sdk::publishing::NamespaceSelection;
use logseq_gardener_sdk::site::{SiteError, SiteOptions, SiteReport, publish_site_with_options};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);

const CONFIG: &str =
    r#"{:journal/page-title-format "yyyy-MM-dd EEE" :publishing/page-properties? true}"#;

struct Fixture(PathBuf);

impl Fixture {
    fn new(config: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "lsg-journals-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(root.join("garden/logseq")).unwrap();
        fs::write(root.join("garden/logseq/config.edn"), config).unwrap();
        let fixture = Self(root);
        fixture.write("pages/tmux.md", "- TMUX_BODY [[2026-10-23 Fri]]\n");
        fixture.write(
            "pages/Docker___Zsh.md",
            "- Installed on [[Feb 1st, 2025]]\n",
        );
        fixture.write("journals/2025_02_01.md", "- FEB_FIRST\n");
        fixture.write(
            "journals/2019_08_07.md",
            "alias:: 1987\nlogseq-entity:: Year\n\n- OLD_JOURNAL\n",
        );
        fixture.write("journals/2026_10_21.md", "- OCT_21 [[tmux]]\n");
        fixture.write("journals/2026_10_22.md", "- OCT_22\n");
        fixture.write("journals/2026_10_23.md", "- OCT_23_FILED\n");
        fixture.write("journals/2026_10_24.md", "- OCT_24\n");
        fixture.write("journals/2026_10_26.md", "- FUTURE_JOURNAL\n");
        fixture.write(
            "journals/2026_10_20.md",
            "- PRIVATE_JOURNAL\n  public:: false\n",
        );
        fixture
    }

    fn write(&self, name: &str, text: &str) {
        let path = self.0.join("garden").join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    fn publish_with(&self, options: SiteOptions) -> Result<SiteReport, SiteError> {
        let _ = fs::remove_dir_all(self.0.join("site"));
        publish_site_with_options(
            self.0.join("garden"),
            self.0.join("site"),
            FilenameFormat::TripleLowbar,
            &NamespaceSelection::new(&["tmux", "Docker"], &[]).unwrap(),
            &options,
        )
    }

    fn publish(&self) -> SiteReport {
        self.publish_with(journals(None)).unwrap()
    }

    /// The page whose own HTML contains `marker`, followed by its reference fragments.
    fn page(&self, marker: &str) -> String {
        let site = self.0.join("site");
        let (name, mut html) = fs::read_dir(&site)
            .unwrap()
            .flatten()
            .filter(|entry| entry.file_name().to_string_lossy().starts_with("p-"))
            .map(|entry| {
                let name = entry.file_name().into_string().unwrap();
                (name, fs::read_to_string(entry.path()).unwrap())
            })
            .find(|(_, html)| html.contains(marker))
            .unwrap_or_else(|| panic!("no page contains {marker}"));
        let stem = format!("{}-", name.trim_end_matches(".html"));
        let mut fragments: Vec<_> = fs::read_dir(site.join("_refs"))
            .into_iter()
            .flatten()
            .flatten()
            .filter(|entry| entry.file_name().to_string_lossy().starts_with(&stem))
            .map(|entry| entry.path())
            .collect();
        fragments.sort();
        for fragment in fragments {
            html.push_str(&fs::read_to_string(fragment).unwrap());
        }
        html
    }

    fn file(&self, name: &str) -> String {
        fs::read_to_string(self.0.join("site").join(name)).unwrap()
    }

    fn exists(&self, name: &str) -> bool {
        self.0.join("site").join(name).exists()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn journals(homepage: Option<&str>) -> SiteOptions {
    SiteOptions {
        homepage: homepage.map(str::to_owned),
        journals: true,
        build_date: JournalDate::new(2026, 10, 25),
        ..SiteOptions::default()
    }
}

fn feed_titles(html: &str) -> Vec<&str> {
    html.split("class=\"journal-title\" href=\"")
        .skip(1)
        .map(|part| {
            let start = part.find('>').unwrap() + 1;
            &part[start..part.find('<').unwrap()]
        })
        .collect()
}

#[test]
fn journal_pages_use_journal_titles_aliases_and_date_links() {
    let f = Fixture::new(CONFIG);
    let report = f.publish();
    assert_eq!((report.pages, report.journals), (2, 7));
    assert_eq!((report.withheld_pages, report.skipped_journals), (1, 0));
    let journal = f.page("OCT_23_FILED");
    assert!(journal.contains("<title>2026-10-23 Fri</title>"));
    assert!(journal.contains("<h1>2026-10-23 Fri</h1>"));
    assert!(journal.contains("id=\"linked-references\""));
    assert!(journal.contains("TMUX_BODY"), "linked reference from tmux");
    assert!(!journal.contains("class=\"namespace-hierarchy"));
    let old = f.page("OLD_JOURNAL");
    assert!(old.contains("<h1>2019-08-07 Wed</h1>") && old.contains("logseq-entity"));
    let route = |marker: &str| {
        fs::read_dir(f.0.join("site"))
            .unwrap()
            .flatten()
            .find(|entry| {
                entry.path().is_file()
                    && fs::read_to_string(entry.path()).unwrap().contains(marker)
                    && entry.file_name().to_string_lossy().starts_with("p-")
            })
            .unwrap()
            .file_name()
            .into_string()
            .unwrap()
    };
    let docker = f.page("Installed on");
    assert!(docker.contains(&format!("href=\"{}\"", route("FEB_FIRST"))));
    assert!(
        docker.contains(">Feb 1st, 2025</a>"),
        "keeps the author's text"
    );
    let tmux = f.page("TMUX_BODY");
    assert!(tmux.contains(&format!("href=\"{}\"", route("OCT_23_FILED"))));
    f.write("pages/tmux.md", "- TMUX_BODY [[1987]]\n");
    f.publish();
    assert!(
        f.page("TMUX_BODY")
            .contains(&format!("href=\"{}\"", route("OLD_JOURNAL")))
    );
    for html in fs::read_dir(f.0.join("site")).unwrap().flatten() {
        let html = fs::read_to_string(html.path()).unwrap_or_default();
        assert!(!html.contains("PRIVATE_JOURNAL"));
    }
    // All pages hides journals until a reader shows them; the list data holds them.
    let pages = f.file("pages.html");
    assert!(!pages.contains(">2026-10-23 Fri</a>") && pages.contains(">tmux</a>"));
    assert!(f.file("_pages/list.json").contains("[\"2026-10-23 Fri\","));
    assert!(f.page("FUTURE_JOURNAL").contains("<h1>2026-10-26 Mon</h1>"));
}

#[test]
fn page_and_feed_outlines_get_the_outline_class_and_references_do_not() {
    let f = Fixture::new(CONFIG);
    f.publish();
    let journal = f.page("OCT_23_FILED");
    let (page, references) = journal.split_once("id=\"linked-references\"").unwrap();
    assert!(page.contains("<ul class=\"outline\"><li>"), "{page}");
    assert!(page.contains("<a class=\"bullet\" href=\"#\"></a>"));
    assert!(references.contains("TMUX_BODY"));
    assert!(!references.contains("class=\"outline\""), "{references}");
    // Copied blocks keep bullets, which open the block on its own page instead of zooming.
    assert!(
        references.contains("<ul class=\"refs-outline\">"),
        "{references}"
    );
    assert!(
        !references.contains("<a class=\"bullet\" href=\"#\">"),
        "{references}"
    );
    let feed = f.file("index.html");
    let article = feed.split("<article class=\"lsg-journal\"").nth(1).unwrap();
    assert!(article.contains("<ul class=\"outline\">"), "{article}");
    assert!(f.file("lsg-outline.js").contains("#zoom/"));
}

#[test]
fn home_page_feed_lists_three_then_seven_newest_first() {
    let f = Fixture::new(CONFIG);
    for day in 1..=9 {
        f.write(
            &format!("journals/2026_09_{day:02}.md"),
            &format!("- SEPTEMBER_{day}\n"),
        );
    }
    f.publish();
    let index = f.file("index.html");
    assert_eq!(
        feed_titles(&index),
        ["2026-10-24 Sat", "2026-10-23 Fri", "2026-10-22 Thu"]
    );
    assert!(index.contains("<title>Journals</title>"));
    assert!(index.contains("data-lsg-journals=\"lsg-journals.js\""));
    assert!(
        !index.contains("src=\"lsg-journals.js\""),
        "the shell loads the feed script after load"
    );
    assert!(index.contains("data-lsg-journals-earlier href=\"journals-2.html\">Earlier</a>"));
    assert!(!index.contains("source-block-"));
    let second = f.file("journals-2.html");
    assert_eq!(
        feed_titles(&second),
        [
            "2026-10-21 Wed",
            "2026-09-09 Wed",
            "2026-09-08 Tue",
            "2026-09-07 Mon",
            "2026-09-06 Sun",
            "2026-09-05 Sat",
            "2026-09-04 Fri"
        ]
    );
    assert!(second.contains("href=\"journals-3.html\">Earlier"));
    let third = f.file("journals-3.html");
    assert!(!third.contains("Earlier"));
    assert!(!f.exists("journals-4.html"));
    for html in [&index, &second, &third] {
        assert!(!html.contains("FUTURE_JOURNAL"));
    }
    assert!(
        !second.contains("data-lsg-journal-refs="),
        "unreferenced journals have no section"
    );
    assert!(
        index.contains("data-lsg-journal-refs=")
            && index.contains("Linked references — 1 page</summary><p><a href=\"")
    );
    assert!(f.exists("lsg-journals.js"));
    assert!(f.file("lsg-journals.js").len() <= 4_000);
}

#[test]
fn sidebar_lists_journals_first_and_marks_feed_pages() {
    let f = Fixture::new(CONFIG);
    f.publish();
    let tmux = f.page("TMUX_BODY");
    let journals = tmux.find("data-lsg-journals-link").unwrap();
    assert!(journals < tmux.find("data-lsg-all-pages").unwrap());
    assert!(tmux.contains("<a href=\"index.html\" data-lsg-journals-link>"));
    assert!(tmux.contains("<a href=\"pages.html\" data-lsg-all-pages>"));
    assert!(
        f.file("index.html")
            .contains("data-lsg-journals-link aria-current=\"page\"")
    );
    assert!(
        f.file("journals-2.html")
            .contains("data-lsg-journals-link aria-current=\"page\"")
    );
    assert!(f.file("lsg-shell.js").contains("gj:"));
}

#[test]
fn homepage_moves_the_feed_to_journals_html() {
    let f = Fixture::new(CONFIG);
    f.publish_with(journals(Some("tmux"))).unwrap();
    assert!(f.file("index.html").contains("TMUX_BODY"));
    assert_eq!(feed_titles(&f.file("journals.html"))[0], "2026-10-24 Sat");
    assert!(
        f.page("TMUX_BODY")
            .contains("<a href=\"journals.html\" data-lsg-journals-link>")
    );
}

#[test]
fn journals_stay_skipped_unless_requested() {
    let f = Fixture::new(CONFIG);
    let report = f.publish_with(SiteOptions::default()).unwrap();
    assert_eq!((report.journals, report.skipped_journals), (0, 8));
    assert!(f.file("index.html").contains(">tmux</a>"));
    assert!(!f.exists("pages.html") && !f.exists("lsg-journals.js"));
    assert!(!f.page("TMUX_BODY").contains("data-lsg-journals-link"));
}

#[test]
fn disabled_journals_publish_pages_without_a_feed() {
    let f = Fixture::new(
        r#"{:journal/page-title-format "yyyy-MM-dd EEE" :feature/enable-journals? false}"#,
    );
    let report = f.publish();
    assert_eq!(report.journals, 7);
    // All pages is the home page; journals stay in its list data until a reader shows them.
    assert!(
        f.file("index.html")
            .contains("<h1 class=\"title\">All pages</h1>")
    );
    assert!(!f.file("index.html").contains(">2026-10-23 Fri</a>"));
    assert!(f.file("_pages/list.json").contains("\"2026-10-23 Fri\""));
    assert!(!f.exists("journals-2.html") && !f.exists("pages.html"));
    assert!(!f.page("TMUX_BODY").contains("data-lsg-journals-link"));
}

#[test]
fn default_title_format_and_unparseable_names() {
    let f = Fixture::new("{}");
    f.write("journals/notes.md", "- NOT_A_DATE\n");
    let report = f.publish();
    assert_eq!(report.skipped_journals, 1);
    assert!(
        report
            .diagnostics
            .iter()
            .any(|d| d.source.ends_with("notes.md"))
    );
    assert!(f.page("OCT_23_FILED").contains("<h1>Oct 23rd, 2026</h1>"));
}

#[test]
fn page_and_journal_with_one_title_fail_before_output() {
    let f = Fixture::new(CONFIG);
    f.write("pages/2026-10-23 Fri.md", "- PAGE\n");
    let selection = NamespaceSelection::new(&["2026-10-23 Fri"], &[]).unwrap();
    let error = publish_site_with_options(
        f.0.join("garden"),
        f.0.join("site"),
        FilenameFormat::TripleLowbar,
        &selection,
        &journals(None),
    )
    .unwrap_err();
    assert!(matches!(error, SiteError::Collision { .. }));
    assert!(!f.exists(""));
}

#[test]
fn older_journals_leave_the_home_page_size_unchanged() {
    let f = Fixture::new(CONFIG);
    f.publish();
    let before = f.file("index.html").len();
    for offset in 0..1_000 {
        let date = JournalDate::from_unix_days(10_000 + offset);
        f.write(
            &format!(
                "journals/{:04}_{:02}_{:02}.md",
                date.year(),
                date.month(),
                date.day()
            ),
            "- OLDER\n",
        );
    }
    f.publish();
    assert_eq!(f.file("index.html").len(), before);
}
