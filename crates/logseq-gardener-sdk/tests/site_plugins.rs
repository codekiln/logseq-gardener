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
