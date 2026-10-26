use logseq_gardener_sdk::page_titles::FilenameFormat;
use logseq_gardener_sdk::publishing::NamespaceSelection;
use logseq_gardener_sdk::site::{SiteOptions, SiteReport, publish_site_with_options};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "lsg-all-pages-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(root.join("garden/logseq")).unwrap();
        fs::write(
            root.join("garden/logseq/config.edn"),
            r#"{:journal/page-title-format "yyyy-MM-dd EEE"}"#,
        )
        .unwrap();
        Self(root)
    }

    fn write(&self, name: &str, text: &str) {
        let path = self.0.join("garden").join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    /// Commit everything written so far at a Unix time.
    fn commit(&self, time: i64) {
        let date = format!("@{time} +0000");
        let garden = self.0.join("garden");
        for args in [
            &["init", "-q"][..],
            &["add", "."],
            &["commit", "-q", "-m", "c"],
        ] {
            let status = Command::new("git")
                .arg("-C")
                .arg(&garden)
                .args(args)
                .env("GIT_AUTHOR_DATE", &date)
                .env("GIT_COMMITTER_DATE", &date)
                .env("GIT_AUTHOR_NAME", "t")
                .env("GIT_AUTHOR_EMAIL", "t@example.com")
                .env("GIT_COMMITTER_NAME", "t")
                .env("GIT_COMMITTER_EMAIL", "t@example.com")
                .env("GIT_CONFIG_NOSYSTEM", "1")
                .env("HOME", &self.0)
                .status()
                .unwrap();
            assert!(status.success());
        }
    }

    fn publish(&self, output: &str) -> SiteReport {
        publish_site_with_options(
            self.0.join("garden"),
            self.0.join(output),
            FilenameFormat::TripleLowbar,
            &NamespaceSelection::new(&["mise", "tmux", "Notes"], &[]).unwrap(),
            &SiteOptions {
                journals: true,
                ..SiteOptions::default()
            },
        )
        .unwrap()
    }

    fn file(&self, output: &str, name: &str) -> String {
        fs::read_to_string(self.0.join(output).join(name)).unwrap()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn all_pages_lists_git_dates_newest_first_with_logseq_backlinks() {
    let f = Fixture::new();
    f.write("pages/mise.md", "icon:: 🧭\n\n- self [[mise]]\n- plain\n");
    f.write("pages/tmux___Command___kill.md", "- [[mise]]\n");
    f.commit(1_740_820_320);
    f.write("pages/Notes.md", "- [[mise]] and [[tmux/Command/kill]]\n");
    f.write("journals/2026_10_23.md", "- [[mise]]\n");
    f.commit(1_750_000_000);
    let report = f.publish("site");
    assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);
    // A journals feed is the home page, so All pages is pages.html.
    let html = f.file("site", "pages.html");
    assert!(html.contains("<title>All pages</title>"));
    assert!(html.contains("<h1 class=\"title\">All pages</h1>"));
    // mise, Notes, tmux/Command/kill and the generated tmux and tmux/Command; no journal.
    assert!(html.contains("Total 5 pages"));
    assert!(html.contains("<span class=\"page-number\">1/1</span>"));
    assert!(!html.contains(">2026-10-23 Fri</a>"));
    let order = ["Notes", "mise", "tmux/Command/kill", "tmux", "tmux/Command"]
        .map(|name| html.find(&format!(">{name}</a>")).unwrap());
    assert!(order.is_sorted(), "{order:?}");
    // Backlinks: own block, tmux/Command/kill, Notes and the journal.
    assert!(html.contains(">mise</a></td><td class=\"backlinks\">4</td>"));
    assert!(html.contains("<td class=\"icon\">🧭</td>"));
    assert!(html.contains(
        "<td class=\"created-at\"><time datetime=\"2025-03-01T09:12:00Z\">2025-03-01 09:12</time></td>"
    ));
    assert!(html.contains("<th class=\"updated-at\" aria-sort=\"descending\">"));
    assert!(html.contains("<script defer src=\"lsg-all-pages.js\"></script>"));
    let list = f.file("site", "_pages/list.json");
    assert!(list.contains("[\"2026-10-23 Fri\","));
    assert!(list.contains(",\"🧭\"]"));
    // The same commit builds the same list and order.
    f.publish("again");
    assert_eq!(html, f.file("again", "pages.html"));
    assert_eq!(list, f.file("again", "_pages/list.json"));
}

#[test]
fn many_pages_page_through_static_files() {
    let f = Fixture::new();
    for n in 0..85 {
        f.write(&format!("pages/Notes___{n:03}.md"), "- note\n");
    }
    f.publish("site");
    let first = f.file("site", "index.html");
    assert!(first.contains("Total 86 pages"));
    assert!(first.contains("<a href=\"pages-2.html\" rel=\"next\">Next ▸</a>"));
    let second = f.file("site", "pages-2.html");
    assert!(second.contains("<span class=\"page-number\">2/3</span>"));
    assert!(second.contains("<a href=\"index.html\" rel=\"prev\">◂ Prev</a>"));
    // Without git dates the view sorts by name: Notes, then Notes/000 to Notes/038 first.
    assert!(first.contains(">Notes/038</a>") && !first.contains(">Notes/039</a>"));
    assert!(second.contains(">Notes/039</a>") && second.contains(">Notes/078</a>"));
    let third = f.file("site", "pages-3.html");
    assert!(third.contains(">Notes/084</a>") && !third.contains("Next ▸"));
    assert!(!first.contains("Created At"));
}
