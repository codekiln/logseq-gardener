//! Build plugins: the publication view, derived rows, contributed metadata and process plugins.
use logseq_gardener_sdk::contributions::{BuildPlugin, BuildPlugins, Contribution};
use logseq_gardener_sdk::journals::JournalDate;
use logseq_gardener_sdk::page_titles::FilenameFormat;
use logseq_gardener_sdk::publication::PluginView;
use logseq_gardener_sdk::publishing::NamespaceSelection;
use logseq_gardener_sdk::search::SearchOptions;
use logseq_gardener_sdk::site::{SiteError, SiteOptions, publish_site_with_options};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;

static NEXT: AtomicUsize = AtomicUsize::new(0);

const CONFIG: &str = r#"{:publishing/all-pages-public? true :journal/page-title-format "yyyy-MM-dd EEE" :publishing/page-properties? true :block-hidden-properties #{:secret}}"#;
const HOME_BLOCK: &str = "11111111-1111-4111-8111-111111111111";
const SHARED: &str = "33333333-3333-4333-8333-333333333333";

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "lsg-build-plugins-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(root.join("garden/logseq")).unwrap();
        fs::write(root.join("garden/logseq/config.edn"), CONFIG).unwrap();
        let fixture = Self(root);
        fixture.write(
            "pages/Home.md",
            &format!("- Intro\n- Episodes\n  id:: {HOME_BLOCK}\n\t- Authored child\n- Outro\n"),
        );
        fixture.write(
            "pages/Show___Episode 1.md",
            "recording:: https://example.com/one.mp3\nsecret:: HIDDEN_SECRET_VALUE\n\n- Episode one\n\t- Chapter A\n\t\t- Detail\n- Second\n  id:: 22222222-2222-4222-8222-222222222222\n  collapsed:: true\n",
        );
        fixture.write(
            "pages/Show___Episode 2.md",
            &format!("- Dup A\n  id:: {SHARED}\n- Embeds {{{{embed (({HOME_BLOCK}))}}}}\n"),
        );
        fixture.write(
            "pages/Show___Other.md",
            &format!("- Dup B\n  id:: {SHARED}\n"),
        );
        fixture.write(
            "pages/Show___Private.md",
            "public:: false\nrecording:: https://example.com/WITHHELD.mp3\n\n- WITHHELD_BODY\n",
        );
        fixture.write("pages/Other.md", "- EXCLUDED_BODY\n");
        fixture.write(
            "journals/2026_10_24.md",
            "- A journal about [[Show/Episode 1]]\n",
        );
        fixture
    }

    fn write(&self, name: &str, text: &str) {
        let path = self.0.join("garden").join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    /// Write a file outside the garden, for plugin folders.
    fn plugin_file(&self, name: &str, text: &str) -> PathBuf {
        let path = self.0.join("plugins").join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, text).unwrap();
        #[cfg(unix)]
        if name.ends_with(".sh") {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
        }
        path
    }

    fn publish(&self, site: &str, options: SiteOptions) -> Result<PathBuf, SiteError> {
        let output = self.0.join(site);
        publish_site_with_options(
            self.0.join("garden"),
            &output,
            FilenameFormat::TripleLowbar,
            &NamespaceSelection::new(&["Home", "Show"], &[]).unwrap(),
            &SiteOptions {
                journals: true,
                build_date: JournalDate::new(2026, 10, 25),
                ..options
            },
        )
        .map(|_| output)
    }

    fn plugins(&self, site: &str, plugins: Vec<Recorder>) -> Result<PathBuf, SiteError> {
        let mut build_plugins = BuildPlugins::default();
        for plugin in plugins {
            build_plugins.push(plugin);
        }
        self.publish(
            site,
            SiteOptions {
                build_plugins,
                ..SiteOptions::default()
            },
        )
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// An in-process plugin that records the views it receives and returns fixed contributions.
#[derive(Clone)]
struct Recorder {
    id: String,
    properties: Vec<String>,
    blocks: bool,
    contributions: Value,
    delay: Duration,
    views: Arc<Mutex<Vec<String>>>,
    calls: Arc<AtomicUsize>,
}

impl Recorder {
    fn new(id: &str, contributions: Value) -> Self {
        Self {
            id: id.to_owned(),
            properties: Vec::new(),
            blocks: true,
            contributions,
            delay: Duration::ZERO,
            views: Arc::default(),
            calls: Arc::default(),
        }
    }
}

impl BuildPlugin for Recorder {
    fn id(&self) -> &str {
        &self.id
    }
    fn page_properties(&self) -> &[String] {
        &self.properties
    }
    fn wants_blocks(&self) -> bool {
        self.blocks
    }
    fn contribute(&self, view: &PluginView<'_>) -> Result<Vec<Contribution>, String> {
        std::thread::sleep(self.delay);
        self.calls.fetch_add(1, Ordering::SeqCst);
        self.views
            .lock()
            .unwrap()
            .push(serde_json::to_string(view).unwrap());
        if self.contributions == json!("fail") {
            return Err("boom".to_owned());
        }
        Ok(serde_json::from_value(self.contributions.clone()).unwrap())
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

/// The URL of a page in `lsg-site.json`.
fn route(site: &Path, id: &str) -> String {
    site_manifest(site)["pages"]
        .as_array()
        .unwrap()
        .iter()
        .find(|page| page["id"] == id && page["kind"] != "home")
        .unwrap()["url"]
        .as_str()
        .unwrap()
        .to_owned()
}

fn page(site: &Path, id: &str) -> String {
    fs::read_to_string(site.join(route(site, id))).unwrap()
}

fn rows(contributions: Value) -> Value {
    json!(contributions)
}

#[test]
fn rows_follow_the_target_blocks_children_and_links_resolve() {
    let fixture = Fixture::new();
    let plugin = Recorder::new(
        "episodes",
        rows(json!([{
            "kind": "rows", "pageId": "home", "blockId": HOME_BLOCK,
            "rows": [
                {"content": [{"page": "show/episode 1"}, {"text": " — 42 min"}], "children": [
                    {"content": [{"page": "show/episode 1", "block": "1.1", "label": "Chapter"}]}
                ]},
                {"content": [{"text": "[[Secret]] <b>x</b>"}, {"url": "https://example.com/a", "label": "site"}, {"url": "mailto:a@example.com", "label": "mail"}]}
            ]
        }])),
    );
    let calls = Arc::clone(&plugin.calls);
    let site = fixture.plugins("site", vec![plugin]).unwrap();
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    let home = page(&site, "home");
    let episode = route(&site, "show/episode 1");
    let derived = home
        .find("<ul class=\"lsg-derived\" data-lsg-plugin=\"episodes\">")
        .expect("derived rows");
    let child = home.find("Authored child").unwrap();
    let outro = home.find("Outro").unwrap();
    assert!(child < derived && derived < outro, "{home}");
    assert!(home.contains(&format!(
        "<a class=\"page-ref\" href=\"{episode}\">Show/Episode 1</a> — 42 min"
    )));
    assert!(home.contains("[[Secret]] &lt;b&gt;x&lt;/b&gt;"), "{home}");
    assert!(!home.contains("<b>x</b>"));
    assert!(home.contains("<a class=\"external-link\" target=\"_blank\" rel=\"noopener noreferrer\" href=\"https://example.com/a\">site</a>"));
    assert!(home.contains("<a href=\"mailto:a@example.com\">mail</a>"));
    assert!(home.contains("<style data-lsg-derived>"));
    // The block link carries the source anchor that the target page now emits.
    let start = home
        .find(&format!("<a href=\"{episode}#source-block-"))
        .unwrap();
    let fragment = &home[start + episode.len() + 10..];
    let fragment = &fragment[..fragment.find('"').unwrap()];
    let target = page(&site, "show/episode 1");
    assert!(
        target.contains(&format!("<span id=\"{fragment}\"></span>")),
        "{fragment}"
    );
    let anchor = target.find(&format!("id=\"{fragment}\"")).unwrap();
    assert!(
        target[anchor..]
            .trim_start_matches(|c| c != '>')
            .contains("Chapter A")
    );
    // Derived rows carry no block anchor or id of their own.
    let list = &home[derived..];
    let list = &list[..list.find("</ul></li>").unwrap_or(list.len())];
    assert!(!list.contains(" id=\""), "{list}");
    assert!(!page(&site, "show/episode 2").contains("<style data-lsg-derived>"));
}

#[test]
fn view_holds_published_pages_and_blocks_only() {
    let fixture = Fixture::new();
    let mut full = Recorder::new("full", json!([]));
    full.properties = vec!["recording".to_owned(), "secret".to_owned()];
    let mut pages_only = Recorder::new("pages-only", json!([]));
    pages_only.blocks = false;
    let (full_views, pages_views) = (Arc::clone(&full.views), Arc::clone(&pages_only.views));
    let site = fixture
        .plugins("site", vec![full.clone(), pages_only.clone()])
        .unwrap();
    let text = full_views.lock().unwrap()[0].clone();
    let view: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(view["apiVersion"], 1);
    assert_eq!(view["revision"].as_str().unwrap().len(), 16);
    for hidden in [
        "WITHHELD",
        "EXCLUDED_BODY",
        "HIDDEN_SECRET_VALUE",
        ".md",
        "show/private",
        "\"other\"",
    ] {
        assert!(!text.contains(hidden), "{hidden} in {text}");
    }
    let root = fixture.0.join("garden");
    assert!(!text.contains(root.to_str().unwrap()));
    let manifest = site_manifest(&site);
    let expected: BTreeSet<(String, String, String, String)> = manifest["pages"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|page| {
            matches!(
                page["kind"].as_str(),
                Some("page" | "journal" | "namespace")
            )
        })
        .map(|page| {
            (
                page["id"].as_str().unwrap().to_owned(),
                page["title"].as_str().unwrap().to_owned(),
                page["url"].as_str().unwrap().to_owned(),
                page["kind"].as_str().unwrap().to_owned(),
            )
        })
        .collect();
    let pages = view["pages"].as_array().unwrap();
    let ids: Vec<&str> = pages
        .iter()
        .map(|page| page["id"].as_str().unwrap())
        .collect();
    let mut sorted = ids.clone();
    sorted.sort_unstable();
    assert_eq!(ids, sorted);
    let actual: BTreeSet<_> = pages
        .iter()
        .map(|page| {
            (
                page["id"].as_str().unwrap().to_owned(),
                page["title"].as_str().unwrap().to_owned(),
                page["url"].as_str().unwrap().to_owned(),
                page["kind"].as_str().unwrap().to_owned(),
            )
        })
        .collect();
    assert_eq!(actual, expected);
    assert!(ids.contains(&"show"), "the generated namespace page");
    assert!(actual.iter().any(|page| page.3 == "journal"));
    let episode = pages
        .iter()
        .find(|page| page["id"] == "show/episode 1")
        .unwrap();
    assert_eq!(
        episode["properties"],
        json!({"recording": "https://example.com/one.mp3"})
    );
    let blocks = view["blocks"].as_array().unwrap();
    let of = |page: &str| -> Vec<&Value> {
        blocks
            .iter()
            .filter(|block| block["pageId"] == page)
            .collect()
    };
    let episode = of("show/episode 1");
    assert_eq!(
        episode[0],
        &json!({"id": "1", "pageId": "show/episode 1", "text": "Episode one", "children": ["1.1"]})
    );
    assert_eq!(episode[1]["children"], json!(["1.1.1"]));
    assert_eq!(
        episode[3],
        &json!({"id": "22222222-2222-4222-8222-222222222222", "pageId": "show/episode 1", "text": "Second", "children": []})
    );
    assert_eq!(of("home")[1]["id"], HOME_BLOCK);
    assert_eq!(of("show/episode 2")[0]["id"], "1");
    assert_eq!(of("show/other")[0]["id"], "1");
    assert!(!text.contains(&format!("\"id\":\"{SHARED}\"")));
    let pages_view: Value = serde_json::from_str(&pages_views.lock().unwrap()[0]).unwrap();
    assert_eq!(pages_view["blocks"], json!([]));
    assert_eq!(
        pages_view["pages"].as_array().unwrap()[0]["properties"],
        json!({})
    );
    // The same garden and options give the same view.
    fixture.plugins("again", vec![full]).unwrap();
    assert_eq!(full_views.lock().unwrap()[1], text);
}

#[test]
fn derived_rows_reach_no_index_and_no_other_page() {
    let fixture = Fixture::new();
    let search = || SiteOptions {
        search: Some(SearchOptions::default()),
        ..SiteOptions::default()
    };
    let plain = fixture.publish("plain", search()).unwrap();
    let mut build_plugins = BuildPlugins::default();
    build_plugins.push(Recorder::new(
        "episodes",
        json!([{
            "kind": "rows", "pageId": "home", "blockId": HOME_BLOCK,
            "rows": [{"content": [{"page": "show/episode 1"}, {"text": " derived-needle"}]}]
        }, {
            "kind": "rows", "pageId": "home",
            "rows": [{"content": [{"page": "show/episode 2", "label": "end-needle"}]}]
        }]),
    ));
    let site = fixture
        .publish(
            "site",
            SiteOptions {
                build_plugins,
                ..search()
            },
        )
        .unwrap();
    let (before, after) = (files(&plain), files(&site));
    let home = route(&site, "home");
    let changed: Vec<&String> = before
        .keys()
        .filter(|name| before.get(*name) != after.get(*name))
        .collect();
    assert_eq!(changed, [&"lsg-site.json".to_owned(), &home], "{changed:?}");
    assert_eq!(before.len(), after.len());
    for (name, contents) in &after {
        if name != &home {
            assert!(
                !String::from_utf8_lossy(contents).contains("needle"),
                "{name} holds a derived row"
            );
        }
    }
    let html = String::from_utf8_lossy(&after[&home]).into_owned();
    let end = html
        .rfind("data-lsg-plugin=\"episodes\"><li><span class=\"lsg-dot\"></span><a class=\"page-ref\" href=")
        .unwrap();
    assert!(
        html[..end].ends_with("<ul class=\"outline lsg-derived\" "),
        "{html}"
    );
    // The embed of the target block on Episode 2 shows its authored children only.
    let embed = String::from_utf8_lossy(&after[&route(&site, "show/episode 2")]).into_owned();
    assert!(embed.contains("Authored child") && !embed.contains("derived-needle"));
}

#[test]
fn a_plugin_that_contributes_nothing_changes_no_file() {
    let fixture = Fixture::new();
    let plain = fixture.publish("plain", SiteOptions::default()).unwrap();
    let site = fixture
        .plugins("site", vec![Recorder::new("noop", json!([]))])
        .unwrap();
    assert_eq!(files(&plain), files(&site));
}

#[test]
fn output_does_not_depend_on_which_plugin_finishes_first() {
    let fixture = Fixture::new();
    let end = |text: &str| json!([{"kind": "rows", "pageId": "home", "rows": [{"content": [{"text": text}]}]}]);
    let build = |site: &str, slow: &str| {
        let mut alpha = Recorder::new("alpha", end("from alpha"));
        let mut beta = Recorder::new("beta", end("from beta"));
        for plugin in [&mut alpha, &mut beta] {
            if plugin.id == slow {
                plugin.delay = Duration::from_millis(100);
            }
        }
        fixture.plugins(site, vec![beta, alpha]).unwrap()
    };
    let first = build("first", "alpha");
    let second = build("second", "beta");
    assert_eq!(files(&first), files(&second));
    let home = page(&first, "home");
    assert!(home.find("from alpha").unwrap() < home.find("from beta").unwrap());
}

#[test]
fn rejected_contributions_stop_the_build_before_output() {
    let fixture = Fixture::new();
    fixture.plugin_file("player/player.mjs", "export default {};\n");
    let manifest = fixture.plugin_file(
        "player/lsg-plugin.json",
        r#"{"id":"player","apiVersion":1,"version":"1","module":"player.mjs","activation":"after-load"}"#,
    );
    let cases: Vec<(Recorder, &str)> = vec![
        (
            Recorder::new("player", json!([])),
            "id \"player\" is also declared by plugin manifest",
        ),
        (
            Recorder::new(
                "rows",
                json!([{"kind": "rows", "pageId": "show/private", "rows": [{"content": [{"text": "x"}]}]}]),
            ),
            "build plugin rows: contributions[0]: page \"show/private\" is not in the view",
        ),
        (
            Recorder::new(
                "rows",
                json!([{"kind": "rows", "pageId": "home", "rows": [{"content": [{"page": "other"}]}]}]),
            ),
            "link to page \"other\", which is not in the view",
        ),
        (
            Recorder::new(
                "rows",
                json!([{"kind": "rows", "pageId": "home", "rows": []}]),
            ),
            "rows must not be empty",
        ),
        (
            Recorder::new(
                "rows",
                json!([{"kind": "rows", "pageId": "home", "rows": [{"content": [{"url": "javascript:alert(1)", "label": "x"}]}]}]),
            ),
            "must use the http, https or mailto scheme",
        ),
        (
            Recorder::new(
                "meta",
                json!([{"kind": "metadata", "pageId": "home", "values": {"a": 1}}]),
            ),
            "metadata needs a browser module",
        ),
        (
            Recorder::new("fails", json!("fail")),
            "build plugin fails: boom",
        ),
        (Recorder::new("Bad_Id", json!([])), "lower-case"),
    ];
    for (index, (plugin, expected)) in cases.into_iter().enumerate() {
        let mut build_plugins = BuildPlugins::default();
        build_plugins.push(plugin);
        let site = format!("site-{index}");
        let error = fixture
            .publish(
                &site,
                SiteOptions {
                    plugins: vec![manifest.clone()],
                    build_plugins,
                    ..SiteOptions::default()
                },
            )
            .unwrap_err();
        assert!(matches!(error, SiteError::BuildPlugin(_)), "{error}");
        assert!(error.to_string().contains(expected), "{error}");
        assert!(!fixture.0.join(&site).exists());
    }
    let mut pages_only = Recorder::new(
        "rows",
        json!([{"kind": "rows", "pageId": "home", "blockId": "1", "rows": [{"content": [{"text": "x"}]}]}]),
    );
    pages_only.blocks = false;
    let error = fixture.plugins("pages-only", vec![pages_only]).unwrap_err();
    assert!(
        error.to_string().contains("outside the plugin's view"),
        "{error}"
    );
}

#[cfg(unix)]
mod process {
    use super::*;

    const ANSWER: &str = r#"{"apiVersion":1,"contributions":[{"kind":"metadata","pageId":"show/episode 1","values":{"chapters":[{"start":0,"title":"Intro"}]}},{"kind":"rows","pageId":"home","rows":[{"content":[{"page":"show/episode 1"}]}]}]}"#;

    fn script(fixture: &Fixture, folder: &str, body: &str) -> PathBuf {
        fixture.plugin_file(
            &format!("{folder}/build.sh"),
            &format!("#!/bin/sh\n{body}\n"),
        )
    }

    fn manifest(fixture: &Fixture, folder: &str, extra: &str) -> PathBuf {
        fixture.plugin_file(
            &format!("{folder}/lsg-plugin.json"),
            &format!(
                r#"{{"id":"{folder}","apiVersion":1,"version":"1.0.0","pageProperties":["recording"],"build":{{"command":["./build.sh"]{extra}}}}}"#
            ),
        )
    }

    fn with(fixture: &Fixture, site: &str, manifests: Vec<PathBuf>) -> Result<PathBuf, SiteError> {
        fixture.publish(
            site,
            SiteOptions {
                plugins: manifests,
                ..SiteOptions::default()
            },
        )
    }

    fn metadata(html: &str) -> Value {
        let start = html
            .find("<script type=\"application/json\" data-lsg-page>")
            .unwrap();
        let json = &html[start..];
        let json = &json[json.find('>').unwrap() + 1..json.find("</script>").unwrap()];
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn process_plugin_runs_once_and_its_contributions_reach_the_page() {
        let fixture = Fixture::new();
        script(
            &fixture,
            "player",
            &format!("cat > view.json\necho run >> calls.txt\nprintf '%s' '{ANSWER}'"),
        );
        fixture.plugin_file("player/player.mjs", "export default {};\n");
        let manifest = fixture.plugin_file(
            "player/lsg-plugin.json",
            r#"{"id":"player","apiVersion":1,"version":"1.0.0","module":"player.mjs","activation":"on-demand","pageProperties":["recording"],"build":{"command":["./build.sh"],"blocks":true}}"#,
        );
        let site = with(&fixture, "site", vec![manifest]).unwrap();
        let folder = fixture.0.join("plugins/player");
        assert_eq!(
            fs::read_to_string(folder.join("calls.txt")).unwrap(),
            "run\n"
        );
        let request: Value =
            serde_json::from_slice(&fs::read(folder.join("view.json")).unwrap()).unwrap();
        assert_eq!(request["apiVersion"], 1);
        assert_eq!(
            request["plugin"],
            json!({"id": "player", "version": "1.0.0"})
        );
        assert!(!request["view"]["blocks"].as_array().unwrap().is_empty());
        let episode = page(&site, "show/episode 1");
        assert_eq!(
            metadata(&episode)["plugins"]["player"],
            json!({"recording": "https://example.com/one.mp3", "chapters": [{"start": 0, "title": "Intro"}]})
        );
        assert!(page(&site, "home").contains("data-lsg-plugin=\"player\""));
        assert_eq!(site_manifest(&site)["plugins"][0]["id"], "player");
    }

    #[test]
    fn build_only_plugin_adds_no_host_or_plugin_entry() {
        let fixture = Fixture::new();
        script(
            &fixture,
            "lister",
            r#"cat > /dev/null
printf '%s' '{"apiVersion":1,"contributions":[{"kind":"rows","pageId":"home","rows":[{"content":[{"text":"listed"}]}]}]}'"#,
        );
        let site = with(&fixture, "site", vec![manifest(&fixture, "lister", "")]).unwrap();
        assert!(site_manifest(&site).get("plugins").is_none());
        assert!(!site.join("plugins").exists());
        for (name, contents) in files(&site) {
            let text = String::from_utf8_lossy(&contents);
            assert!(
                !text.contains("data-lsg-page") && !text.contains("lsg-host-"),
                "{name}"
            );
        }
        assert!(page(&site, "home").contains("listed"));
        let noop = Fixture::new();
        script(
            &noop,
            "noop",
            r#"cat > /dev/null; printf '{"apiVersion":1,"contributions":[]}'"#,
        );
        let plain = noop.publish("plain", SiteOptions::default()).unwrap();
        let site = with(&noop, "site", vec![manifest(&noop, "noop", "")]).unwrap();
        assert_eq!(files(&plain), files(&site));
    }

    #[test]
    fn failing_process_plugins_stop_the_build() {
        let fixture = Fixture::new();
        for (folder, body, extra, expected) in [
            ("exits", "cat > /dev/null; exit 1", "", "exit status: 1"),
            (
                "garbled",
                "cat > /dev/null; echo '{'",
                "",
                "output is not JSON",
            ),
            (
                "future",
                r#"cat > /dev/null; echo '{"apiVersion":2,"contributions":[]}'"#,
                "",
                "apiVersion 2",
            ),
            (
                "slow",
                "sleep 5",
                r#","timeoutSeconds":1"#,
                "timeout of 1 seconds",
            ),
        ] {
            script(&fixture, folder, body);
            let started = std::time::Instant::now();
            let error =
                with(&fixture, folder, vec![manifest(&fixture, folder, extra)]).unwrap_err();
            let message = error.to_string();
            assert!(
                message.contains(&format!("build plugin {folder}:")),
                "{message}"
            );
            assert!(message.contains(expected), "{message}");
            assert!(!fixture.0.join(folder).exists());
            assert!(started.elapsed() < Duration::from_secs(4), "{folder}");
        }
    }
}

#[test]
fn view_follows_the_all_pages_public_setting() {
    let fixture = Fixture::new();
    fs::write(
        fixture.0.join("garden/logseq/config.edn"),
        r#"{:journal/page-title-format "yyyy-MM-dd EEE"}"#,
    )
    .unwrap();
    fixture.write(
        "pages/Home.md",
        &format!("public:: true\n\n- Episodes\n  id:: {HOME_BLOCK}\n"),
    );
    let plugin = Recorder::new("viewer", json!([]));
    let views = Arc::clone(&plugin.views);
    let site = fixture.plugins("site", vec![plugin]).unwrap();
    let view: Value = serde_json::from_str(&views.lock().unwrap()[0]).unwrap();
    let ids: Vec<&str> = view["pages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|page| page["id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, ["home"]);
    assert!(
        view["blocks"]
            .as_array()
            .unwrap()
            .iter()
            .all(|block| block["pageId"] == "home")
    );
    let manifest = site_manifest(&site);
    assert!(
        manifest["pages"]
            .as_array()
            .unwrap()
            .iter()
            .all(|page| page["id"] == "home" || page["id"].is_null())
    );
}
