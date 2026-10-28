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
            "lsg-flashcards-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(root.join("garden/pages")).unwrap();
        fs::create_dir_all(root.join("garden/logseq")).unwrap();
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
        "- [[tmux]] keys\n\t- Zoom pane #card\n\t  card-repeats:: 2\n\t  card-next-schedule:: 2027-01-01T00:00:00.000Z\n\t  card-ease-factor:: 2.6\n\t  card-last-interval:: 4\n\t  card-last-reviewed:: 2026-10-01T00:00:00.000Z\n\t  card-last-score:: 5\n\t\t- `prefix z`\n\t- Copy mode [[Card]]\n\t\t- `prefix [`\n- Not a card `#card`\n",
    );
    fixture.write(
        "pages/Notes___Capitals.md",
        "- France {{cloze Paris}} #card\n- Overdue #card\n  card-repeats:: 1\n  card-next-schedule:: 2026-10-01T00:00:00.000Z\n",
    );
    fixture.write(
        "pages/Notes___Decks.md",
        "- {{cards [[tmux]]}}\n- {{cards Zoxide}}\n- {{cards}}\n- {{cards (and [[tmux]] (not [[Copy]]))}}\n",
    );
    fixture
}

#[test]
fn decks_render_counts_first_cards_and_empty_decks() {
    let fixture = garden();
    let report = fixture.publish();
    let decks = fixture.html("Notes/Decks");
    assert!(!decks.contains("{{cards"), "{decks}");
    assert!(
        decks.contains("<style>.cards-review{"),
        "widget styles inlined"
    );
    // tmux: two cards, the scheduled one not yet due.
    assert!(
        decks.contains("<span class=\"cards-query\">[[tmux]]</span>"),
        "{decks}"
    );
    assert!(
        decks.contains("title=\"Overdue/Total\">1/2</span>"),
        "{decks}"
    );
    assert!(
        decks.contains("Copy mode"),
        "first due card's front: {decks}"
    );
    assert!(!decks.contains("prefix ["), "the widget shows no answer");
    assert!(
        decks
            .contains("<code>Cards: Zoxide</code><div class=\"cards-none\">No matched cards</div>")
    );
    // `{{cards}}` is every card: four, three due.
    assert!(
        decks.contains("title=\"Overdue/Total\">3/4</span>"),
        "{decks}"
    );
    assert!(
        !decks.contains("lsg-flashcards.js"),
        "the review loads on use"
    );
    let no_diagnostic = |text: &str| !report.diagnostics.iter().any(|d| d.message.contains(text));
    assert!(no_diagnostic("card"), "{:?}", report.diagnostics);
    assert!(
        no_diagnostic("Unsupported macro"),
        "{:?}",
        report.diagnostics
    );
}

#[test]
fn deck_files_hold_schedules_and_cards_are_written_once() {
    let fixture = garden();
    fixture.publish();
    let list: serde_json::Value =
        serde_json::from_str(&fixture.file("flashcards/decks.json")).unwrap();
    let labels: Vec<&str> = list["decks"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry[1].as_str().unwrap())
        .collect();
    assert_eq!(
        labels,
        ["[[tmux]]", "Zoxide", "(and [[tmux]] (not [[Copy]]))", "All"]
    );
    let all: serde_json::Value =
        serde_json::from_str(&fixture.file("flashcards/decks/all.json")).unwrap();
    let rows = all["cards"].as_array().unwrap();
    assert_eq!(rows.len(), 4);
    // Due cards first, unscheduled before scheduled; the not-yet-due card last.
    assert_eq!(rows[2][2], "2026-10-01T00:00:00.000Z");
    assert_eq!(rows[3][1], 2);
    assert_eq!(rows[3][4], 2.6);
    let cards = fixture.file("flashcards/cards/0.html");
    assert_eq!(cards.matches("<article data-card=").count(), 4);
    assert!(
        cards.contains(
            "<div class=\"lsg-card-back\"><ul class=\"refs-outline\"><li><span class=\"bullet\">"
        ),
        "{cards}"
    );
    assert!(cards.contains("prefix z"));
    assert!(cards.contains("class=\"cloze\""));
    for key in [
        "card-repeats",
        "card-next-schedule",
        "card-ease-factor",
        "card-last-score",
    ] {
        assert!(!cards.contains(key), "{key} hidden in the review");
        assert!(
            !fixture.html("Notes/tmux").contains(key),
            "{key} hidden on the page"
        );
    }
}

#[test]
fn sidebar_shows_flashcards_with_the_due_badge() {
    let fixture = garden();
    fixture.publish();
    let page = fixture.html("Notes/tmux");
    let item = page
        .split("<li class=\"flashcards-nav\">")
        .nth(1)
        .and_then(|rest| rest.split("</li>").next())
        .expect("Flashcards item");
    assert!(item.contains("<span>Flashcards</span>"));
    assert!(item.contains(">g f</span>"));
    assert!(item.contains("data-lsg-due>3</span>"), "{item}");
    let flashcards = page.find("flashcards-nav").unwrap();
    assert!(flashcards < page.find("graph-view-nav").unwrap());
    assert!(
        page.contains("<li class=\"lsg-card\">"),
        "card blocks are styled"
    );
    assert!(
        fixture
            .file("lsg-flashcards.js")
            .contains("logseqGardenerCards")
    );
    assert!(fixture.file("lsg-flashcards.css").contains(".lsg-review"));
}

#[test]
fn flashcards_turned_off_leave_no_item_or_files() {
    let fixture = garden();
    fixture.write("logseq/config.edn", "{:feature/enable-flashcards? false}");
    fixture.publish();
    assert!(!fixture.html("Notes/tmux").contains("flashcards-nav"));
    assert!(!fixture.0.join("site/flashcards").exists());
    assert!(!fixture.0.join("site/lsg-flashcards.js").exists());
}
