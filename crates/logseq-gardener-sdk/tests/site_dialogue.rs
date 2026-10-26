use logseq_gardener_sdk::{
    page_titles::FilenameFormat, publishing::NamespaceSelection, site::publish_site,
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
            "lsg-dialogue-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(root.join("garden/pages")).unwrap();
        Self(root)
    }
    fn write(&self, path: &str, text: &str) {
        let path = self.0.join("garden").join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }
    fn publish(&self) {
        publish_site(
            self.0.join("garden"),
            self.0.join("site"),
            FilenameFormat::TripleLowbar,
            &NamespaceSelection::new(&["Notes"], &["Notes/Excluded"]).unwrap(),
        )
        .unwrap();
    }
    fn html(&self, title: &str) -> String {
        let hash = title
            .as_bytes()
            .iter()
            .fold(0xcbf2_9ce4_8422_2325_u64, |h, b| {
                (h ^ u64::from(*b)).wrapping_mul(0x100000001b3)
            });
        let mut html = fs::read_to_string(self.0.join(format!("site/p-{hash:016x}.html"))).unwrap();
        // Reference sections load these fragments; their copies must stay inert too.
        let stem = format!("p-{hash:016x}-");
        let mut fragments: Vec<_> = fs::read_dir(self.0.join("site/_refs"))
            .into_iter()
            .flatten()
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| {
                path.file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with(&stem)
            })
            .collect();
        fragments.sort();
        for fragment in fragments {
            html.push_str(&fs::read_to_string(fragment).unwrap());
        }
        html
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
const AUDIO: &str = "https://media.example.org/interview.mp3?download=1&edition=2";
#[test]
fn original_speakers_bind_to_later_recording_and_preserve_source_and_disclosures() {
    let f = Fixture::new();
    let original = format!(
        "tags:: Interview\n- Ada & Bob · 0:00 — Hello **world**\n- **Another voice** · 27:30 — Dialogue\n  collapsed:: true\n  - Child speech\n- *Guest* · 00:27:30 — Same moment\n- ![Interview]({AUDIO})\n- [Download]({AUDIO})\n- ![Same recording]({AUDIO}#t=2)\n"
    );
    f.write("pages/Notes___Interview.md", &original);
    f.write("pages/Notes___Ordinary.md", "- Plain notes\n");
    f.publish();
    let html = f.html("Notes/Interview");
    assert_eq!(html.matches(" data-lsg-dialogue-time=").count(), 3);
    assert!(html.contains("Ada &amp; Bob · <a class=\"dialogue-timestamp\""));
    assert!(html.contains("href=\"https://media.example.org/interview.mp3?download=1&amp;edition=2#t=1650\">27:30</a> — Dialogue"));
    assert!(html.contains("#t=1650\">00:27:30</a>"));
    assert!(html.contains("#t=0\">0:00</a> — Hello <strong>world</strong>"));
    let summary = html
        .split("<details class=\"outline-disclosure\"><summary>")
        .nth(1)
        .unwrap()
        .split("</summary>")
        .next()
        .unwrap();
    assert!(summary.contains("<strong>Another voice</strong>"));
    assert!(summary.contains("class=\"bullet\""));
    assert!(summary.contains("data-lsg-dialogue-time=\"1650\""));
    assert_eq!(html.matches(" data-lsg-recording").count(), 2);
    assert!(html.contains("controls preload=\"none\""));
    assert!(
        html.contains(
            "src=\"https://media.example.org/interview.mp3?download=1&amp;edition=2#t=2\""
        )
    );
    assert!(html.contains("<script data-lsg-dialogue-start>"));
    assert!(!f.html("Notes/Ordinary").contains("data-lsg-dialogue-start"));
    assert!(
        !fs::read_to_string(f.0.join("site/index.html"))
            .unwrap()
            .contains("data-lsg-dialogue-start")
    );
    assert!(f.0.join("site/dialogue.js").exists());
    assert_eq!(
        fs::read_to_string(f.0.join("garden/pages/Notes___Interview.md")).unwrap(),
        original
    );
}
#[test]
fn literal_boundaries_and_invalid_times_do_not_load_playback_code() {
    let f = Fixture::new();
    f.write("pages/Notes.md", &format!("- Clock 12:30 is prose\n- <2026-10-20 Tue>\n- `Ada · 0:10 — code`\n- [Ada · 0:10 — link](https://example.org)\n- ## Ada · 0:10 — heading\n- **Ada (written)** · 0:10 — Written\n- **Ada** · written — Text\n- **Ada** · 0:60 — Invalid\n- Ada · 1:60:00 — Invalid\n- Ada · 99999999999999999999:00 — Invalid\n- Ada · 150119987579017:00 — Unsafe integer\n- Code\n  ~~~text\n  Ada · 0:10 — literal\n  ~~~\n- ![Interview]({AUDIO})\n"));
    f.publish();
    let html = f.html("Notes");
    assert!(!html.contains(" data-lsg-dialogue-time="));
    assert!(!html.contains(" data-lsg-recording"));
    assert!(!html.contains("data-lsg-dialogue-start"));
    assert!(!f.0.join("site/dialogue.js").exists());
    assert!(html.contains("<code>Ada · 0:10 — code</code>"));
    assert!(html.contains("<h2>Ada · 0:10 — heading</h2>"));
    assert!(html.contains("<strong>Ada (written)</strong> · 0:10 — Written"));
}
#[test]
fn multiple_original_recordings_remain_ambiguous() {
    let f = Fixture::new();
    f.write("pages/Notes.md", &format!("- Ada · 1:00 — Speech\n- ![First]({AUDIO})\n- ![Second](https://media.example.org/other.wav)\n"));
    f.publish();
    let html = f.html("Notes");
    assert!(!html.contains(" data-lsg-dialogue-time="));
    assert!(html.contains("Ada · 1:00 — Speech"));
    assert!(!f.0.join("site/dialogue.js").exists());
}
#[test]
fn transclusions_and_backlinks_keep_dialogue_and_copied_recordings_inert() {
    let f = Fixture::new();
    f.write("pages/Notes___Source.md", &format!("- **Ada** · 0:30 — [[Notes/Target]]\n  id:: 12345678-1234-1234-1234-123456789abc\n  - ![Interview]({AUDIO})\n"));
    f.write("pages/Notes___Target.md", "- Target page\n");
    f.write("pages/Notes___Host.md", "- {{embed [[Notes/Source]]}}\n- {{embed ((12345678-1234-1234-1234-123456789abc))}}\n- Guest · 0:20 — No original audio\n");
    f.publish();
    let source = f.html("Notes/Source");
    assert_eq!(source.matches(" data-lsg-dialogue-time=").count(), 1);
    for title in ["Notes/Host", "Notes/Target"] {
        let html = f.html(title);
        assert!(!html.contains(" data-lsg-dialogue-time="), "{title}");
        assert!(!html.contains(" data-lsg-recording"), "{title}");
        assert!(!html.contains("data-lsg-dialogue-start"), "{title}");
        assert!(html.contains("<strong>Ada</strong> · 0:30 —"));
        assert!(html.contains("<audio controls"));
    }
}
#[test]
fn local_recording_resolution_and_selection_do_not_add_authority_from_hidden_pages() {
    let f = Fixture::new();
    f.write(
        "pages/Notes___Local.md",
        "- Narrator · 1:05 — Local recording\n- ![Local](../assets/interview.wav)\n",
    );
    f.write("assets/interview.wav", "RIFF fixture");
    f.write(
        "pages/Notes___NoAudio.md",
        "- Voice · 1:05 — No recording here\n",
    );
    f.write("pages/Notes___Private.md", "public:: false\n- SECRET SPEAKER · 1:00 — PRIVATE_DIALOGUE_SENTINEL\n- ![Private](https://media.example.org/private.mp3)\n");
    f.write("pages/Notes___Excluded.md", "- Excluded · 1:00 — EXCLUDED_DIALOGUE_SENTINEL\n- ![Excluded](https://media.example.org/excluded.mp3)\n");
    f.publish();
    let html = f.html("Notes/Local");
    assert!(html.contains("#t=65\">1:05</a>"));
    assert!(html.contains("href=\"assets/a-"));
    assert!(html.contains(".wav#t=65\""));
    assert!(!f.html("Notes/NoAudio").contains(" data-lsg-dialogue-time="));
    for entry in fs::read_dir(f.0.join("site")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|e| e == "html") {
            let html = fs::read_to_string(path).unwrap();
            assert!(!html.contains("PRIVATE_DIALOGUE_SENTINEL"));
            assert!(!html.contains("EXCLUDED_DIALOGUE_SENTINEL"));
        }
    }
}

#[test]
fn outline_heading_properties_and_task_presentation_remain_literal() {
    let f = Fixture::new();
    f.write("pages/Notes.md", &format!("- **Ada** · 0:10 — Spoken\n- **Ada** · 0:20 — Property heading\n  heading:: 2\n- TODO **Ada** · 0:30 — Task\n- DONE **Ada** · 0:40 — Done task\n- [#A] Ada · 0:50 — Prioritized\n- ![Interview]({AUDIO})\n"));
    f.publish();
    let html = f.html("Notes");
    assert_eq!(html.matches(" data-lsg-dialogue-time=").count(), 1);
    assert!(html.contains("<h2><strong>Ada</strong> · 0:20 — Property heading</h2>"));
    assert!(html.contains("class=\"task-checkbox\""));
    assert!(html.contains("class=\"task-done\""));
    assert!(html.contains("class=\"priority\""));
    assert!(html.contains("class=\"outline\""));
    assert!(html.contains("class=\"bullet\""));
    assert!(f.0.join("site/lsg-outline.js").exists());
}

#[test]
fn search_enabled_dialogue_pages_and_journal_feed_keep_initial_scripts_within_budget() {
    use logseq_gardener_sdk::{
        journals::JournalDate,
        site::{SearchOptions, SiteOptions, publish_site_with_options},
    };
    let f = Fixture::new();
    let source = format!("- Ada · 0:10 — Original dialogue\n- ![Recording]({AUDIO})\n");
    f.write("pages/Notes.md", &source);
    f.write("journals/2026_10_25.md", &source);
    publish_site_with_options(
        f.0.join("garden"),
        f.0.join("site"),
        FilenameFormat::TripleLowbar,
        &NamespaceSelection::new(&["Notes"], &[]).unwrap(),
        &SiteOptions {
            search: Some(SearchOptions::default()),
            journals: true,
            build_date: JournalDate::new(2026, 10, 25),
            ..SiteOptions::default()
        },
    )
    .unwrap();
    let mut dialogue_pages = 0;
    for entry in fs::read_dir(f.0.join("site")).unwrap().flatten() {
        if entry.path().extension().is_none_or(|ext| ext != "html") {
            continue;
        }
        let html = fs::read_to_string(entry.path()).unwrap();
        if !html.contains(" data-lsg-dialogue-time=") {
            continue;
        }
        dialogue_pages += 1;
        assert!(html.contains("rel=\"lsg-search\""));
        assert!(html.contains("data-lsg-dialogue-start"));
        assert!(html.contains("rel=\"lsg-dialogue\" href=\"dialogue.js\""));
        assert!(!html.contains("src=\"dialogue.js\""));
        // Like the existing conservative search budget, include inline, deferred,
        // and module scripts; scripts created after load are outside this total.
        let mut bytes = 0;
        for script in html.split("<script").skip(1) {
            let (attributes, content) = script.split_once('>').unwrap();
            if let Some(source) = attributes.split("src=\"").nth(1) {
                let source = source.split('"').next().unwrap();
                bytes += fs::read(f.0.join("site").join(source)).unwrap().len();
            } else {
                bytes += content.split("</script>").next().unwrap().len();
            }
        }
        assert!(bytes <= 16_000, "{}: {bytes}", entry.path().display());
    }
    assert_eq!(dialogue_pages, 3, "source page, journal page and feed");
    assert_eq!(
        fs::read_to_string(f.0.join("garden/pages/Notes.md")).unwrap(),
        source
    );
    assert_eq!(
        fs::read_to_string(f.0.join("garden/journals/2026_10_25.md")).unwrap(),
        source
    );
}
