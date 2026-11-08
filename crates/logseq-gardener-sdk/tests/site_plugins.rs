//! `lsg-site.json`, plugin manifests, plugin files and per-page plugin metadata.
use logseq_gardener_sdk::journals::JournalDate;
use logseq_gardener_sdk::page_titles::FilenameFormat;
use logseq_gardener_sdk::publishing::NamespaceSelection;
use logseq_gardener_sdk::site::{SiteError, SiteOptions, publish_site_with_options};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);

const CONFIG: &str = r#"{:publishing/all-pages-public? true :journal/page-title-format "yyyy-MM-dd EEE" :publishing/page-properties? true :block-hidden-properties #{:secret}}"#;

const PLAYER: &str = r#"{"id":"player","apiVersion":1,"version":"2.0.0","module":"player.mjs","styles":["player.css"],"activation":"after-load","pageProperties":["recording","secret"]}"#;

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "lsg-plugins-site-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(root.join("garden/logseq")).unwrap();
        fs::write(root.join("garden/logseq/config.edn"), CONFIG).unwrap();
        let fixture = Self(root);
        fixture.write(
            "pages/Show___Episode 1.md",
            "recording:: https://example.com/one.mp3\nsecret:: HIDDEN_SECRET_VALUE\n\n- Episode one links [[Show/Episode 2]] ![photo](../assets/shared.png)\n",
        );
        fixture.write(
            "pages/Show___Episode 2.md",
            "- Episode two without a recording\n",
        );
        fixture.write(
            "pages/Show___Private.md",
            "public:: false\nrecording:: https://example.com/WITHHELD_RECORDING.mp3\n\n- WITHHELD_BODY ![only](../assets/only-private.png)\n",
        );
        fixture.write(
            "pages/Other.md",
            "recording:: https://example.com/EXCLUDED_RECORDING.mp3\n\n- EXCLUDED_BODY\n",
        );
        fixture.write(
            "journals/2026_10_24.md",
            "- A journal about [[Show/Episode 1]]\n",
        );
        fixture.write("assets/shared.png", "PNG");
        fixture.write("assets/only-private.png", "PNG");
        fixture.write(
            "player/player.mjs",
            "export default { id: 'player', apiVersion: 1, setup() {} };\n",
        );
        fixture.write("player/player.css", "audio { width: 100%; }\n");
        fixture.write("player/lsg-plugin.json", PLAYER);
        fixture.write("other/badge.mjs", "export default {};\n");
        fixture.write(
            "other/lsg-plugin.json",
            r#"{"id":"badge","apiVersion":1,"version":"1","module":"badge.mjs","activation":"on-demand"}"#,
        );
        fixture
    }

    fn write(&self, name: &str, text: &str) {
        let path = self.0.join("garden").join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    fn manifest(&self, folder: &str) -> PathBuf {
        self.0.join("garden").join(folder).join("lsg-plugin.json")
    }

    fn publish(&self, site: &str, plugins: Vec<PathBuf>) -> Result<PathBuf, SiteError> {
        let output = self.0.join(site);
        publish_site_with_options(
            self.0.join("garden"),
            &output,
            FilenameFormat::TripleLowbar,
            &NamespaceSelection::new(&["Show"], &[]).unwrap(),
            &SiteOptions {
                journals: true,
                build_date: JournalDate::new(2026, 10, 25),
                plugins,
                ..SiteOptions::default()
            },
        )
        .map(|_| output)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn files(root: &Path) -> BTreeMap<String, Vec<u8>> {
    fn visit(root: &Path, path: &Path, result: &mut BTreeMap<String, Vec<u8>>) {
        for entry in fs::read_dir(path).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                visit(root, &path, result);
            } else {
                let name = path
                    .strip_prefix(root)
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");
                result.insert(name, fs::read(&path).unwrap());
            }
        }
    }
    let mut result = BTreeMap::new();
    visit(root, root, &mut result);
    result
}

fn site_manifest(site: &Path) -> Value {
    serde_json::from_slice(&fs::read(site.join("lsg-site.json")).unwrap()).unwrap()
}

fn metadata(html: &str) -> Value {
    let start = html
        .find("<script type=\"application/json\" data-lsg-page>")
        .unwrap();
    let json = &html[start..];
    let json = &json[json.find('>').unwrap() + 1..json.find("</script>").unwrap()];
    serde_json::from_str(json).unwrap()
}

/// A page's HTML without the tags plugins add.
fn without_plugin_tags(html: &str) -> String {
    let Some(start) = html.find("<script type=\"application/json\" data-lsg-page>") else {
        return html.to_owned();
    };
    let end = html.find("</head>").unwrap();
    format!("{}{}", &html[..start], &html[end..])
}

#[test]
fn site_manifest_lists_every_page_once_and_leaves_out_withheld_content() {
    let fixture = Fixture::new();
    let site = fixture.publish("site", Vec::new()).unwrap();
    let manifest = site_manifest(&site);
    assert_eq!(manifest["apiVersion"], 1);
    assert!(manifest["revision"].as_str().unwrap().len() == 16);
    assert!(manifest.get("plugins").is_none());
    let pages = manifest["pages"].as_array().unwrap();
    let urls: Vec<&str> = pages
        .iter()
        .map(|page| page["url"].as_str().unwrap())
        .collect();
    let mut sorted = urls.clone();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(urls, sorted, "pages sorted by URL, each once");
    for url in &urls {
        assert!(site.join(url).is_file(), "{url} exists");
    }
    let by_title = |title: &str| pages.iter().find(|page| page["title"] == title).unwrap();
    let episode = by_title("Show/Episode 1");
    assert_eq!(episode["id"], "show/episode 1");
    assert_eq!(episode["kind"], "page");
    assert_eq!(by_title("Show")["kind"], "namespace");
    assert_eq!(by_title("2026-10-24 Sat")["kind"], "journal");
    assert_eq!(by_title("Journals")["kind"], "home");
    assert_eq!(by_title("Journals")["id"], Value::Null);
    assert_eq!(by_title("All pages")["kind"], "list");
    assert!(!urls.contains(&"graph.html"));
    // Every generated page apart from the graph view is listed.
    let html: Vec<String> = files(&site)
        .into_keys()
        .filter(|name| name.ends_with(".html") && !name.contains('/') && name != "graph.html")
        .collect();
    assert_eq!(html.len(), urls.len(), "{html:?} vs {urls:?}");
    let text = String::from_utf8(fs::read(site.join("lsg-site.json")).unwrap()).unwrap();
    for withheld in ["Private", "WITHHELD", "Other", "EXCLUDED"] {
        assert!(!text.contains(withheld), "{withheld} in {text}");
    }
    let assets = manifest["assets"].as_array().unwrap();
    assert_eq!(assets.len(), 1, "{assets:?}");
    assert!(site.join(assets[0].as_str().unwrap()).is_file());
}

#[test]
fn builds_without_plugins_add_no_page_metadata_and_are_reproducible() {
    let fixture = Fixture::new();
    let first = fixture.publish("first", Vec::new()).unwrap();
    let second = fixture.publish("second", Vec::new()).unwrap();
    let first_files = files(&first);
    assert_eq!(first_files, files(&second));
    assert!(!first.join("plugins").exists());
    for (name, contents) in &first_files {
        let text = String::from_utf8_lossy(contents);
        assert!(!text.contains("data-lsg-page"), "{name}");
        assert!(!text.contains("lsg-host"), "{name}");
        assert!(!name.starts_with("lsg-host"), "{name}");
    }
}

#[test]
fn plugins_add_files_metadata_and_only_those_tags() {
    let fixture = Fixture::new();
    let plain = files(&fixture.publish("plain", Vec::new()).unwrap());
    let site = fixture
        .publish(
            "site",
            vec![fixture.manifest("player"), fixture.manifest("other")],
        )
        .unwrap();
    let output = files(&site);
    assert_eq!(
        output["plugins/player/player.mjs"],
        b"export default { id: 'player', apiVersion: 1, setup() {} };\n"
    );
    assert_eq!(
        output["plugins/player/player.css"],
        b"audio { width: 100%; }\n"
    );
    assert!(output.contains_key("plugins/badge/badge.mjs"));
    let manifest = site_manifest(&site);
    assert_eq!(
        manifest["plugins"],
        serde_json::json!([
            {"id": "badge", "version": "1", "module": "plugins/badge/badge.mjs", "styles": [], "activation": "on-demand"},
            {"id": "player", "version": "2.0.0", "module": "plugins/player/player.mjs", "styles": ["plugins/player/player.css"], "activation": "after-load"}
        ])
    );
    let revision = manifest["revision"].as_str().unwrap();
    let hosts: Vec<&String> = output
        .keys()
        .filter(|name| name.starts_with("lsg-host-"))
        .collect();
    assert_eq!(hosts.len(), 1);
    assert!(hosts[0].ends_with(".mjs"));
    // The host applies the reader's remembered Properties choice to each adopted page before
    // the other page initializers. scripts/plugins/browser.py checks the result in a browser.
    let host = String::from_utf8_lossy(&output[hosts[0].as_str()]);
    let properties = host.find("window.logseqGardenerInitProperties?.(m)");
    let outline = host.find("window.logseqGardenerInitOutline?.(m)");
    assert!(properties.is_some() && properties < outline, "{host}");
    for page in manifest["pages"].as_array().unwrap() {
        let url = page["url"].as_str().unwrap();
        let html = String::from_utf8(output[url].clone()).unwrap();
        let data = metadata(&html);
        assert_eq!(data["url"], page["url"]);
        assert_eq!(data["id"], page["id"]);
        assert_eq!(data["title"], page["title"]);
        assert_eq!(data["revision"], revision);
        assert!(html.contains(&format!("import('./{}')", hosts[0])), "{url}");
        // The plugin tags are the only difference from a build without plugins.
        assert_eq!(
            without_plugin_tags(&html),
            String::from_utf8(plain[url].clone()).unwrap(),
            "{url}"
        );
    }
    assert!(!String::from_utf8_lossy(&output["graph.html"]).contains("data-lsg-page"));
    let page = |title: &str| {
        let url = manifest["pages"]
            .as_array()
            .unwrap()
            .iter()
            .find(|page| page["title"] == title)
            .unwrap()["url"]
            .as_str()
            .unwrap()
            .to_owned();
        metadata(&String::from_utf8(output[&url].clone()).unwrap())
    };
    assert_eq!(
        page("Show/Episode 1")["plugins"],
        serde_json::json!({"player": {"recording": "https://example.com/one.mp3"}})
    );
    assert_eq!(page("Show/Episode 2")["plugins"], serde_json::json!({}));
    for (name, contents) in &output {
        let text = String::from_utf8_lossy(contents);
        for hidden in [
            "HIDDEN_SECRET_VALUE",
            "WITHHELD_RECORDING",
            "EXCLUDED_RECORDING",
        ] {
            assert!(!text.contains(hidden), "{hidden} in {name}");
        }
    }
}

#[test]
fn plugin_argument_order_does_not_change_output() {
    let fixture = Fixture::new();
    let forward = fixture
        .publish(
            "forward",
            vec![fixture.manifest("player"), fixture.manifest("other")],
        )
        .unwrap();
    let backward = fixture
        .publish(
            "backward",
            vec![fixture.manifest("other"), fixture.manifest("player")],
        )
        .unwrap();
    assert_eq!(files(&forward), files(&backward));
}

#[test]
fn invalid_manifests_fail_before_output_exists() {
    let fixture = Fixture::new();
    let cases = [
        (
            "version",
            PLAYER.replace("\"apiVersion\":1", "\"apiVersion\":2"),
            "supports apiVersion 1",
        ),
        (
            "escape",
            PLAYER.replace("\"player.mjs\"", "\"../player/player.mjs\""),
            "\"../player/player.mjs\"",
        ),
        (
            "missing",
            PLAYER.replace("\"player.css\"", "\"missing.css\""),
            "missing.css",
        ),
    ];
    for (folder, json, expected) in cases {
        fixture.write(&format!("{folder}/player.mjs"), "");
        fixture.write(&format!("{folder}/player.css"), "");
        fixture.write(&format!("{folder}/lsg-plugin.json"), &json);
        let error = fixture
            .publish(folder, vec![fixture.manifest(folder)])
            .unwrap_err();
        let message = error.to_string();
        assert!(matches!(error, SiteError::InvalidPlugin(_)), "{message}");
        assert!(
            message.contains(&format!("{folder}/lsg-plugin.json"))
                || message.contains(&format!("{folder}\\lsg-plugin.json")),
            "{message}"
        );
        assert!(message.contains(expected), "{message}");
        assert!(!fixture.0.join(folder).exists(), "{folder} output");
    }
    fixture.write("copy/player.mjs", "");
    fixture.write("copy/player.css", "");
    fixture.write("copy/lsg-plugin.json", PLAYER);
    let error = fixture
        .publish(
            "duplicate",
            vec![fixture.manifest("player"), fixture.manifest("copy")],
        )
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("player/lsg-plugin.json") && error.contains("copy/lsg-plugin.json"),
        "{error}"
    );
    assert!(!fixture.0.join("duplicate").exists());
}

const EPISODE_PAGE: &str = "logseq-entity:: [[Logseq/Entity/Podcast/Episode/Show]], [[Logseq/Entity/Podcast/Episode]]\n\n- # The First Episode\n\t- Notes about the session.\n\t- ![Episode recording](../assets/episode-one.mp3)\n";

impl Fixture {
    /// The fixture with podcast episodes and `extra` added to its `config.edn` map.
    fn with_episodes(extra: &str) -> Self {
        let fixture = Self::new();
        fixture.config(extra);
        fixture.write("pages/Show___Podcast One.md", EPISODE_PAGE);
        fixture.write(
            "pages/Show___Podcast Two.md",
            "logseq-entity:: [[logseq/entity/podcast/episode]]\n\n- # Second\n\t- ![Remote](https://example.com/two.mp3)\n",
        );
        fixture.write(
            "pages/Show___Typed Without Recording.md",
            "logseq-entity:: [[Logseq/Entity/Podcast/Episode]]\n\n- # A show page\n\t- No audio.\n",
        );
        fixture.write(
            "pages/Show___Withheld Episode.md",
            "public:: false\nlogseq-entity:: [[Logseq/Entity/Podcast/Episode]]\n\n- # Withheld\n\t- ![r](https://example.com/WITHHELD_EPISODE.mp3)\n",
        );
        fixture.write("assets/episode-one.mp3", "ID3");
        fixture
    }

    fn config(&self, extra: &str) {
        let config = CONFIG.strip_suffix('}').unwrap().to_owned() + " " + extra + "}";
        fs::write(self.0.join("garden/logseq/config.edn"), config).unwrap();
    }

    fn publish_report(
        &self,
        site: &str,
        plugins: Vec<PathBuf>,
    ) -> Result<logseq_gardener_sdk::site::SiteReport, SiteError> {
        publish_site_with_options(
            self.0.join("garden"),
            self.0.join(site),
            FilenameFormat::TripleLowbar,
            &NamespaceSelection::new(&["Show"], &[]).unwrap(),
            &SiteOptions {
                journals: true,
                build_date: JournalDate::new(2026, 10, 25),
                plugins,
                ..SiteOptions::default()
            },
        )
    }
}

fn recorder() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/plugins/recorder/lsg-plugin.json")
}

fn page_metadata(site: &Path, title: &str) -> Value {
    let manifest = site_manifest(site);
    let url = manifest["pages"]
        .as_array()
        .unwrap()
        .iter()
        .find(|page| page["title"] == title)
        .unwrap_or_else(|| panic!("{title} is published"))["url"]
        .as_str()
        .unwrap()
        .to_owned();
    metadata(&fs::read_to_string(site.join(url)).unwrap())
}

#[test]
fn a_turned_on_first_party_plugin_writes_its_files_and_episode_entries() {
    let fixture = Fixture::with_episodes(":lsg/plugins [:podcast-episode]");
    let report = fixture.publish_report("site", Vec::new()).unwrap();
    let site = fixture.0.join("site");
    let output = files(&site);
    let embedded = Path::new(env!("CARGO_MANIFEST_DIR")).join("plugins/podcast-episode");
    for file in ["podcast-episode.mjs", "podcast-episode.css"] {
        assert_eq!(
            output[&format!("plugins/podcast-episode/{file}")],
            fs::read(embedded.join(file)).unwrap(),
            "{file}"
        );
    }
    assert!(!output.contains_key("plugins/podcast-episode/lsg-plugin.json"));
    assert_eq!(
        site_manifest(&site)["plugins"],
        serde_json::json!([{
            "id": "podcast-episode",
            "version": logseq_gardener_sdk::VERSION,
            "module": "plugins/podcast-episode/podcast-episode.mjs",
            "styles": ["plugins/podcast-episode/podcast-episode.css"],
            "activation": "on-demand",
        }])
    );
    let one = &page_metadata(&site, "Show/Podcast One")["plugins"]["podcast-episode"];
    assert_eq!(one["title"], "The First Episode");
    let recording = one["recording"].as_str().unwrap();
    assert!(
        recording.starts_with("assets/a-") && recording.ends_with(".mp3"),
        "{recording}"
    );
    assert_eq!(output[recording], b"ID3");
    assert_eq!(
        page_metadata(&site, "Show/Podcast Two")["plugins"],
        serde_json::json!({"podcast-episode": {"recording": "https://example.com/two.mp3", "title": "Second"}})
    );
    // Pages without the episode type, or without a recording, get no entry, so the browser
    // imports no plugin code there.
    for title in ["Show/Episode 1", "Show/Typed Without Recording", "Show"] {
        assert_eq!(
            page_metadata(&site, title)["plugins"],
            serde_json::json!({}),
            "{title}"
        );
    }
    let diagnostics: Vec<String> = report
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.message.starts_with("podcast-episode"))
        .map(|diagnostic| format!("{}: {}", diagnostic.source.display(), diagnostic.message))
        .collect();
    assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    assert!(
        diagnostics[0].contains("Show___Typed Without Recording.md"),
        "{diagnostics:?}"
    );
    for (name, contents) in &output {
        assert!(
            !String::from_utf8_lossy(contents).contains("WITHHELD_EPISODE"),
            "withheld recording in {name}"
        );
    }
}

/// Episode One of `tests/fixtures/parity-garden`, the fixture episode.
const FIXTURE_EPISODE: &str = include_str!(
    "../../../tests/fixtures/parity-garden/pages/Tools___Plugins___Show___Episode One.md"
);

fn page_html(site: &Path, title: &str) -> String {
    let manifest = site_manifest(site);
    let url = manifest["pages"]
        .as_array()
        .unwrap()
        .iter()
        .find(|page| page["title"] == title)
        .unwrap()["url"]
        .as_str()
        .unwrap()
        .to_owned();
    fs::read_to_string(site.join(url)).unwrap()
}

/// A page with `count` timestamped chapter headings, each `title_words` words long: one H3
/// in ten, the rest H4 under it.
fn long_episode(count: usize, title_words: usize) -> String {
    let mut page = String::from(
        "logseq-entity:: [[Logseq/Entity/Podcast/Episode]]\n\n- # Long\n\t- ![r](https://example.com/long.mp3)\n\t- ## Chapters\n",
    );
    let words = vec!["word"; title_words].join(" ");
    for number in 0..count {
        let (minutes, seconds) = (number * 47 / 60, number * 47 % 60);
        let stamp = if minutes >= 60 {
            format!("{:02}:{:02}:{seconds:02}", minutes / 60, minutes % 60)
        } else {
            format!("{minutes:02}:{seconds:02}")
        };
        if number % 10 == 0 {
            page.push_str(&format!("\t\t- ### {stamp} Part {number} {words}\n"));
        } else {
            page.push_str(&format!("\t\t\t- #### {stamp} Step {number} {words}\n"));
        }
    }
    page
}

#[test]
fn episode_chapters_come_from_timestamped_headings_and_bullets_and_their_blocks_are_anchored() {
    let fixture = Fixture::with_episodes(":lsg/plugins [:podcast-episode]");
    fixture.write("pages/Show___Fixture Episode.md", FIXTURE_EPISODE);
    fixture.write("assets/plugins-episode-one.mp3", "ID3");
    fixture.write("pages/Show___Long Episode.md", &long_episode(120, 6));
    let report = fixture.publish_report("site", Vec::new()).unwrap();
    let site = fixture.0.join("site");
    let entry = &page_metadata(&site, "Show/Fixture Episode")["plugins"]["podcast-episode"];
    let chapters = entry["chapters"].as_array().unwrap();
    let summary: Vec<(u64, &str, &Value)> = chapters
        .iter()
        .map(|c| {
            (
                c["seconds"].as_u64().unwrap(),
                c["title"].as_str().unwrap(),
                &c["parent"],
            )
        })
        .collect();
    assert_eq!(
        summary,
        [
            (0, "Opening", &Value::Null),
            (5, "the first voice comes in", &Value::from(0)),
            (90, "The middle", &Value::Null),
            (105, "a nested point", &Value::from(2)),
            (130, "a deeper point", &Value::from(3)),
            (3723, "An hour in", &Value::Null),
        ]
    );
    // Each chapter's anchor is on its block, so the module can find the block's timestamp.
    let html = page_html(&site, "Show/Fixture Episode");
    for (chapter, text) in chapters.iter().zip([
        "<h3>00:00 Opening</h3>",
        "00:05 - the first voice comes in",
        "<h3>01:30 The middle</h3>",
        "01:45 - a nested point",
        "02:10 - a deeper point",
        "<h3>1:02:03 An hour in</h3>",
    ]) {
        let anchor = chapter["anchor"].as_str().unwrap();
        let tag = format!("<span id=\"{anchor}\"></span><a class=\"bullet\" href=\"#\"></a>{text}");
        assert_eq!(html.matches(&tag).count(), 1, "{tag}");
    }
    // A timestamp later in a bullet stays text, and its block gains no anchor.
    assert!(html.contains("<li><a class=\"bullet\" href=\"#\"></a>A second voice joins at 01:28"));
    // About as many timestamps as the largest GitP/A episode stay in one entry.
    let long = &page_metadata(&site, "Show/Long Episode")["plugins"]["podcast-episode"];
    let chapters = long["chapters"].as_array().unwrap();
    assert_eq!(chapters.len(), 120);
    assert_eq!(chapters[11]["parent"], 10);
    assert_eq!(chapters[119]["seconds"], 119 * 47);
    assert!(serde_json::to_string(long).unwrap().len() < 64_000);
    assert!(
        !report
            .diagnostics
            .iter()
            .any(|d| d.message.contains("chapters and only")),
        "{:?}",
        report.diagnostics
    );
    // Without the key the same pages carry neither the entry nor the anchors.
    fixture.config("");
    let plain = fixture.publish("plain", Vec::new()).unwrap();
    assert!(!page_html(&plain, "Show/Fixture Episode").contains("source-block-"));
}

#[test]
fn chapters_past_the_entry_limit_are_left_out_with_one_diagnostic() {
    let fixture = Fixture::with_episodes(":lsg/plugins [:podcast-episode]");
    fixture.write("pages/Show___Long Episode.md", &long_episode(1200, 12));
    let report = fixture.publish_report("site", Vec::new()).unwrap();
    let site = fixture.0.join("site");
    let entry = &page_metadata(&site, "Show/Long Episode")["plugins"]["podcast-episode"];
    let kept = entry["chapters"].as_array().unwrap().len();
    let size = serde_json::to_string(entry).unwrap().len();
    assert!(kept > 100 && kept < 1200, "{kept}");
    assert!(size <= 64_000 && size > 60_000, "{size}");
    let diagnostics: Vec<&str> = report
        .diagnostics
        .iter()
        .filter(|d| d.message.contains("chapters and only"))
        .map(|d| d.message.as_str())
        .collect();
    assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
    assert!(
        diagnostics[0].contains(&format!("has 1200 chapters and only the first {kept} fit")),
        "{diagnostics:?}"
    );
    // Only the kept chapters' blocks gain anchors.
    assert_eq!(
        page_html(&site, "Show/Long Episode")
            .matches("id=\"source-block-")
            .count(),
        kept
    );
}

#[test]
fn a_hidden_entity_property_still_chooses_episode_pages() {
    let fixture = Fixture::with_episodes(":lsg/plugins [:podcast-episode]");
    fixture.config(
        ":lsg/plugins [:podcast-episode] :block-hidden-properties #{:secret :logseq-entity}",
    );
    fixture.write(
        "pages/Show___Podcast One.md",
        &EPISODE_PAGE.replace("Podcast/Episode/Show", "Podcast/Episode/HIDDEN_TYPE_VALUE"),
    );
    let site = fixture.publish("site", Vec::new()).unwrap();
    let entry = &page_metadata(&site, "Show/Podcast One")["plugins"]["podcast-episode"];
    assert_eq!(entry["title"], "The First Episode");
    // The graph view draws the hidden property's page reference, as Logseq's does, with or
    // without plugins; the plugin adds the hidden value to no file.
    let holding = |site: &Path| -> Vec<String> {
        files(site)
            .into_iter()
            .filter(|(_, contents)| String::from_utf8_lossy(contents).contains("HIDDEN_TYPE_VALUE"))
            .map(|(name, _)| name)
            .collect()
    };
    let with_plugin = holding(&site);
    assert!(
        with_plugin.iter().all(|name| name.starts_with("_graph/")),
        "{with_plugin:?}"
    );
    fixture.config(":block-hidden-properties #{:secret :logseq-entity}");
    let without = fixture.publish("without", Vec::new()).unwrap();
    assert_eq!(with_plugin, holding(&without));
}

#[test]
fn an_empty_plugins_key_builds_the_same_site_as_no_key() {
    let fixture = Fixture::with_episodes("");
    let without = files(&fixture.publish("without", Vec::new()).unwrap());
    fixture.config(":lsg/plugins []");
    let empty = files(&fixture.publish("empty", Vec::new()).unwrap());
    assert_eq!(without, empty);
    assert!(!without.keys().any(|name| name.starts_with("plugins/")));
    assert!(
        without
            .values()
            .all(|contents| !String::from_utf8_lossy(contents).contains("data-lsg-page"))
    );
}

#[test]
fn first_party_and_manifest_plugins_give_the_same_output_in_any_order() {
    let fixture = Fixture::with_episodes(":lsg/plugins [:podcast-episode]");
    let forward = fixture
        .publish("forward", vec![recorder(), fixture.manifest("other")])
        .unwrap();
    let backward = fixture
        .publish("backward", vec![fixture.manifest("other"), recorder()])
        .unwrap();
    assert_eq!(files(&forward), files(&backward));
    let ids: Vec<Value> = site_manifest(&forward)["plugins"]
        .as_array()
        .unwrap()
        .iter()
        .map(|plugin| plugin["id"].clone())
        .collect();
    assert_eq!(ids, ["badge", "fixture-recorder", "podcast-episode"]);
}

#[test]
fn a_manifest_with_a_first_party_id_stops_before_output() {
    let fixture = Fixture::with_episodes(":lsg/plugins [:podcast-episode]");
    fixture.write("clash/clash.mjs", "export default {};\n");
    fixture.write(
        "clash/lsg-plugin.json",
        r#"{"id":"podcast-episode","apiVersion":1,"version":"1","module":"clash.mjs","activation":"on-demand"}"#,
    );
    let error = fixture
        .publish("clash-site", vec![fixture.manifest("clash")])
        .unwrap_err();
    let message = error.to_string();
    assert!(matches!(error, SiteError::InvalidPlugin(_)), "{message}");
    for part in [
        "\"podcast-episode\"",
        "logseq/config.edn",
        "lsg-plugin.json",
    ] {
        assert!(message.contains(part), "{part} in {message}");
    }
    assert!(!fixture.0.join("clash-site").exists());
    // Without the key, the manifest may use the id.
    fixture.config("");
    fixture
        .publish("clash-free", vec![fixture.manifest("clash")])
        .unwrap();
}

#[test]
fn a_malformed_plugins_key_stops_before_output() {
    let fixture = Fixture::with_episodes("");
    for (case, value, expected) in [
        ("misspelled", "[:podcast-epsiode]", ":podcast-epsiode"),
        ("string", r#"["podcast-episode"]"#, "must be a keyword"),
        ("repeated", "[:podcast-episode :podcast-episode]", "twice"),
        ("not-vector", ":podcast-episode", "must be a vector"),
    ] {
        fixture.config(&format!(":lsg/plugins {value}"));
        let error = fixture.publish(case, Vec::new()).unwrap_err();
        let message = error.to_string();
        assert!(matches!(error, SiteError::PluginKey(_)), "{message}");
        assert!(
            message.starts_with("logseq/config.edn: :lsg/plugins"),
            "{message}"
        );
        assert!(message.contains(expected), "{message}");
        assert!(message.contains(":podcast-episode"), "{message}");
        assert!(!fixture.0.join(case).exists(), "{case} output");
    }
}
