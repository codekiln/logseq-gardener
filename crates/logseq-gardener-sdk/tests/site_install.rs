//! Installable gardens: the `:lsg/install` key, the manifest, the icon, the head tags and the
//! Install app item, with and without readable URLs, and the sites that do not opt in.
use logseq_gardener_sdk::journals::JournalDate;
use logseq_gardener_sdk::page_titles::FilenameFormat;
use logseq_gardener_sdk::publishing::NamespaceSelection;
use logseq_gardener_sdk::search::SearchOptions;
use logseq_gardener_sdk::site::{PageUrls, SiteError, SiteOptions, publish_site_with_options};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);

const INSTALL: &str =
    r#":lsg/install {:name "Ghost in the Patch" :short-name "GitP" :icon "assets/app-icon.png"}"#;
const BASE_CONFIG: &str =
    r#":publishing/all-pages-public? true :journal/page-title-format "yyyy-MM-dd EEE""#;

struct Fixture(PathBuf);

/// A PNG header for a square image, then filler: the checks read only the header.
fn png(side: u32, total: usize) -> Vec<u8> {
    let mut bytes = vec![0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a, 0, 0, 0, 13];
    bytes.extend(b"IHDR");
    bytes.extend(side.to_be_bytes());
    bytes.extend(side.to_be_bytes());
    bytes.resize(total, 7);
    bytes
}

impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "lsg-install-site-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).unwrap();
        let fixture = Self(fs::canonicalize(root).unwrap());
        fixture.config(INSTALL);
        fixture.write(
            "pages/Docs___Guide___Start.md",
            "- Start here. See [[Docs/Guide/Q&A: 2026]] and [[Other]].\n- A query {{query (page-tags [[Guide]])}}\n",
        );
        fixture.write(
            "pages/Docs___Guide___QA.md",
            "title:: Docs/Guide/Q&A: 2026\n\n- Questions. Back to [[Docs/Guide/Start]].\n",
        );
        fixture.write("pages/Other.md", "- Other links [[Docs/Guide/Start]].\n");
        fixture.write(
            "journals/2026_10_23.md",
            "- A journal about [[Docs/Guide/Start]].\n",
        );
        fixture.write_bytes("assets/app-icon.png", &png(512, 40_000));
        fixture
    }

    fn config(&self, extra: &str) {
        self.write("logseq/config.edn", &format!("{{{BASE_CONFIG} {extra}}}"));
    }

    fn write(&self, name: &str, text: &str) {
        self.write_bytes(name, text.as_bytes());
    }

    fn write_bytes(&self, name: &str, bytes: &[u8]) {
        let path = self.0.join("garden").join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, bytes).unwrap();
    }

    fn options(&self, readable: bool) -> SiteOptions {
        SiteOptions {
            journals: true,
            build_date: Some(JournalDate::new(2026, 10, 24).unwrap()),
            search: Some(SearchOptions::default()),
            page_urls: if readable {
                PageUrls::Readable
            } else {
                PageUrls::default()
            },
            old_url_redirects: true,
            ..SiteOptions::default()
        }
    }

    fn publish(&self, site: &str, readable: bool) -> Result<PathBuf, SiteError> {
        let output = self.0.join(site);
        let selection = NamespaceSelection::new(&["Docs", "Other"], &[]).unwrap();
        publish_site_with_options(
            self.0.join("garden"),
            &output,
            FilenameFormat::TripleLowbar,
            &selection,
            &self.options(readable),
        )
        .map(|_| output)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// Every file under `root`, relative and slash-separated, with its bytes.
fn files(root: &Path) -> BTreeMap<String, Vec<u8>> {
    fn visit(root: &Path, path: &Path, result: &mut BTreeMap<String, Vec<u8>>) {
        for entry in fs::read_dir(path).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                visit(root, &path, result);
            } else {
                let relative = path.strip_prefix(root).unwrap().to_string_lossy();
                let mut bytes = Vec::new();
                fs::File::open(&path)
                    .unwrap()
                    .read_to_end(&mut bytes)
                    .unwrap();
                result.insert(relative.replace('\\', "/"), bytes);
            }
        }
    }
    let mut result = BTreeMap::new();
    visit(root, root, &mut result);
    result
}

/// Fragments in `_refs/` are stored compressed, so decode loosely: they hold no page markup.
fn text(bytes: &[u8]) -> std::borrow::Cow<'_, str> {
    String::from_utf8_lossy(bytes)
}

/// A full page carries the shell's header; redirect pages and fragments do not.
fn is_full_page(route: &str, html: &str) -> bool {
    route.ends_with(".html") && html.contains("<header class=\"lsg-header")
}

fn has_install_tags(html: &str) -> bool {
    html.contains("<link rel=\"manifest\"")
}

const HEAD: &str = "<link rel=\"manifest\" href=\"lsg.webmanifest\"><link rel=\"apple-touch-icon\" href=\"lsg-icon.png\"><meta name=\"theme-color\" media=\"(prefers-color-scheme: light)\" content=\"#fbfcfd\"><meta name=\"theme-color\" media=\"(prefers-color-scheme: dark)\" content=\"#151718\"><script defer src=\"lsg-install.js\"></script>";

#[test]
fn the_key_writes_the_manifest_the_icon_and_the_script() {
    let fixture = Fixture::new();
    let output = fixture.publish("site", false).unwrap();
    let files = files(&output);
    assert_eq!(
        text(&files["lsg.webmanifest"]),
        r##"{"name":"Ghost in the Patch","short_name":"GitP","start_url":"./","scope":"./","display":"standalone","theme_color":"#fbfcfd","background_color":"#fbfcfd","icons":[{"src":"lsg-icon.png","type":"image/png","sizes":"512x512","purpose":"any"}]}"##
    );
    assert_eq!(
        files["lsg-icon.png"],
        fs::read(fixture.0.join("garden/assets/app-icon.png")).unwrap(),
        "the icon is copied unchanged"
    );
    let script = text(&files["lsg-install.js"]).into_owned();
    assert!(script.contains("beforeinstallprompt") && script.contains("prompt()"));
    assert!(script.len() <= 1500, "{} bytes", script.len());
    // The icon is a site file, not a copied asset, and no service worker is written.
    assert!(
        !files
            .keys()
            .any(|name| name.contains("service-worker") || name.starts_with("sw."))
    );
    let site: Value = serde_json::from_slice(&files["lsg-site.json"]).unwrap();
    let listed = site.to_string();
    assert!(
        !listed.contains("lsg.webmanifest") && !listed.contains("lsg-icon.png"),
        "{listed}"
    );
}

#[test]
fn every_full_page_has_the_tags_and_nothing_else_does() {
    for readable in [false, true] {
        let fixture = Fixture::new();
        let output = fixture.publish("site", readable).unwrap();
        let files = files(&output);
        let mut full = Vec::new();
        for (route, bytes) in &files {
            if !route.ends_with(".html") {
                continue;
            }
            let html = text(bytes).into_owned();
            if is_full_page(route, &html) {
                assert!(html.contains(HEAD), "{route} lacks the install head");
                assert_eq!(html.matches("<link rel=\"manifest\"").count(), 1, "{route}");
                assert_eq!(html.matches("data-lsg-install").count(), 1, "{route}");
                assert!(!html.contains("apple-mobile-web-app"), "{route}");
                // After <base> on readable pages, so the URLs resolve to the site root.
                if let Some(base) = html.find("<base href") {
                    assert!(
                        base < html.find("<link rel=\"manifest\"").unwrap(),
                        "{route}"
                    );
                }
                full.push(route.as_str());
            } else {
                assert!(!has_install_tags(&html), "{route} is not a full page");
                assert!(!html.contains("data-lsg-install"), "{route}");
            }
        }
        for expected in ["index.html", "pages.html", "graph.html"] {
            assert!(full.contains(&expected), "{expected} in {full:?}");
        }
        let kinds = |predicate: fn(&str) -> bool| full.iter().any(|route| predicate(route));
        assert!(kinds(|route| route.starts_with("journals")) || full.contains(&"index.html"));
        if readable {
            for expected in [
                "Docs/Guide/Start/index.html",
                "Docs/Guide/index.html",
                "Docs/index.html",
                "Other/index.html",
            ] {
                assert!(full.contains(&expected), "{expected} in {full:?}");
            }
            let nested = text(&files["Docs/Guide/Start/index.html"]);
            assert!(nested.contains("<base href=\"../../../\">"));
        } else {
            assert!(full.iter().any(|route| route.starts_with("p-")), "{full:?}");
        }
        // Redirect pages and reference fragments exist and carry no tag.
        assert!(files.keys().any(|route| route.starts_with("_refs/")));
        if readable {
            assert!(
                files.iter().any(|(route, bytes)| route.starts_with("p-")
                    && text(bytes).contains("http-equiv=\"refresh\"")),
                "an old-address redirect page exists"
            );
        }
    }
}

#[test]
fn every_url_in_the_manifest_and_the_head_resolves_to_a_file() {
    for readable in [false, true] {
        let fixture = Fixture::new();
        let output = fixture.publish("site", readable).unwrap();
        let files = files(&output);
        let manifest: Value = serde_json::from_slice(&files["lsg.webmanifest"]).unwrap();
        for url in [
            manifest["start_url"].as_str().unwrap(),
            manifest["scope"].as_str().unwrap(),
            manifest["icons"][0]["src"].as_str().unwrap(),
        ] {
            assert!(!url.starts_with('/') && !url.contains("://"), "{url}");
        }
        assert_eq!(manifest["start_url"], "./");
        assert_eq!(manifest["icons"][0]["src"], "lsg-icon.png");
        assert!(files.contains_key("index.html") && files.contains_key("lsg-icon.png"));
        // Each head URL, resolved against the page's <base> or its own folder, names a file.
        for (route, bytes) in &files {
            let html = text(bytes).into_owned();
            if !route.ends_with(".html") || !is_full_page(route, &html) {
                continue;
            }
            let base = html
                .split("<base href=\"")
                .nth(1)
                .map_or("", |rest| rest.split('"').next().unwrap());
            let folder = route.rsplit_once('/').map_or("", |(folder, _)| folder);
            for target in ["lsg.webmanifest", "lsg-icon.png", "lsg-install.js"] {
                let resolved = if base.is_empty() {
                    if folder.is_empty() {
                        target.to_owned()
                    } else {
                        format!("{folder}/{target}")
                    }
                } else {
                    let up = base.matches("../").count();
                    let mut parts: Vec<&str> =
                        folder.split('/').filter(|p| !p.is_empty()).collect();
                    parts.truncate(parts.len().saturating_sub(up));
                    parts.push(target);
                    parts.join("/")
                };
                assert!(
                    files.contains_key(&resolved),
                    "{route}: {target} resolves to {resolved}"
                );
            }
        }
    }
}

#[test]
fn a_garden_without_the_key_has_no_install_markup_or_files() {
    let fixture = Fixture::new();
    fixture.config("");
    for readable in [false, true] {
        let output = fixture
            .publish(&format!("plain-{readable}"), readable)
            .unwrap();
        let files = files(&output);
        for name in ["lsg.webmanifest", "lsg-icon.png", "lsg-install.js"] {
            assert!(!files.contains_key(name), "{name}");
        }
        for (route, bytes) in &files {
            let contents = String::from_utf8_lossy(bytes);
            for marker in [
                "lsg.webmanifest",
                "lsg-icon.png",
                "lsg-install.js",
                "data-lsg-install",
                "theme-color",
                "apple-touch-icon",
            ] {
                assert!(!contents.contains(marker), "{route} holds {marker}");
            }
        }
    }
}

#[test]
fn the_key_changes_only_the_install_files_the_head_and_the_menu_item() {
    // The same garden with the key, less the three files, the head tags and the menu item,
    // is the garden without it: the key adds nothing else.
    for readable in [false, true] {
        let fixture = Fixture::new();
        let with = files(
            &fixture
                .publish(&format!("with-{readable}"), readable)
                .unwrap(),
        );
        fixture.config("");
        let without = files(
            &fixture
                .publish(&format!("without-{readable}"), readable)
                .unwrap(),
        );
        let mut stripped = BTreeMap::new();
        for (route, bytes) in with {
            if ["lsg.webmanifest", "lsg-icon.png", "lsg-install.js"].contains(&route.as_str()) {
                continue;
            }
            let bytes = if route.ends_with(".html") {
                text(&bytes)
                    .replace(HEAD, "")
                    .replace("<button type=\"button\" role=\"menuitem\" data-lsg-install hidden>Install app</button>", "")
                    .into_bytes()
            } else {
                bytes
            };
            stripped.insert(route, bytes);
        }
        let mut other = without;
        // `lsg-site.json` records a revision from the page contents; it differs when pages do.
        for map in [&mut stripped, &mut other] {
            map.remove("lsg-site.json");
        }
        assert_eq!(
            stripped.keys().collect::<Vec<_>>(),
            other.keys().collect::<Vec<_>>(),
            "same files"
        );
        for (route, bytes) in &stripped {
            assert!(bytes == &other[route], "{route} differs");
        }
    }
}

#[test]
fn a_page_edit_leaves_the_install_files_and_references_unchanged() {
    let fixture = Fixture::new();
    let before = files(&fixture.publish("before", true).unwrap());
    fixture.write(
        "pages/Other.md",
        "- Other links [[Docs/Guide/Start]].\n- A new block\n",
    );
    let after = files(&fixture.publish("after", true).unwrap());
    for name in ["lsg.webmanifest", "lsg-icon.png", "lsg-install.js"] {
        assert!(before[name] == after[name], "{name}");
    }
    for route in ["Docs/Guide/Start/index.html", "index.html"] {
        let tags = |files: &BTreeMap<String, Vec<u8>>| text(&files[route]).matches(HEAD).count();
        assert_eq!((tags(&before), tags(&after)), (1, 1), "{route}");
    }
}

#[test]
fn installable_pages_grow_by_a_fixed_small_amount() {
    let fixture = Fixture::new();
    let with = files(&fixture.publish("with", true).unwrap());
    fixture.config("");
    let without = files(&fixture.publish("without", true).unwrap());
    let mut checked = 0;
    for (route, bytes) in &with {
        if !is_full_page(route, &text(bytes)) {
            continue;
        }
        let growth = bytes.len() - without[route].len();
        assert!(growth <= 500, "{route} grew by {growth} bytes");
        checked += 1;
    }
    assert!(checked >= 7, "{checked} pages checked");
}

#[test]
fn key_and_icon_errors_stop_the_build_before_output() {
    let cases = [
        (
            r#":lsg/install {:name "Notes" :shortname "N" :icon "assets/app-icon.png"}"#,
            vec![":shortname", ":name, :short-name, :icon"],
        ),
        (
            r#":lsg/install {:name "Logseq Gardener Documentation" :icon "assets/app-icon.png"}"#,
            vec![":short-name of at most 12 characters"],
        ),
        (
            r#":lsg/install {:name "Notes" :icon "assets/missing.png"}"#,
            vec!["assets/missing.png", "does not exist"],
        ),
        (
            r#":lsg/install {:name "Notes" :icon "../icon.png"}"#,
            vec!["../icon.png", "inside the garden folder"],
        ),
        (
            r#":lsg/install {:name "Notes" :icon "assets/logo.svg"}"#,
            vec!["assets/logo.svg", "not a PNG"],
        ),
        (
            r#":lsg/install {:name "Notes" :icon "assets/small.png"}"#,
            vec!["300 by 300"],
        ),
        (
            r#":lsg/install {:name "Notes" :icon "assets/big.png"}"#,
            vec!["300 KB; the limit is 256 KB"],
        ),
        (
            r#":default-home {:lsg/install {:name "Notes" :icon "assets/app-icon.png"}}"#,
            vec!["cannot sit inside :default-home"],
        ),
    ];
    for (config, expected) in cases {
        let fixture = Fixture::new();
        fixture.write("assets/logo.svg", "<svg/>");
        fixture.write_bytes("assets/small.png", &png(300, 2_000));
        fixture.write_bytes("assets/big.png", &png(512, 300_000));
        fixture.config(config);
        let output = fixture.0.join("site");
        let error = fixture.publish("site", false).unwrap_err();
        assert!(matches!(error, SiteError::Install(_)), "{error:?}");
        let message = error.to_string();
        assert!(
            message.starts_with("logseq/config.edn: :lsg/install "),
            "{message}"
        );
        for part in expected {
            assert!(message.contains(part), "{config}: {message}");
        }
        assert!(!output.exists(), "{config}: output was created");
    }
}

#[test]
fn a_page_named_for_the_manifest_fails_under_readable_urls() {
    let fixture = Fixture::new();
    fixture.write(
        "pages/lsg.webmanifest.md",
        "- A page named like the manifest\n",
    );
    let selection = NamespaceSelection::new(&["Docs", "Other", "lsg.webmanifest"], &[]).unwrap();
    let output = fixture.0.join("site");
    let error = publish_site_with_options(
        fixture.0.join("garden"),
        &output,
        FilenameFormat::TripleLowbar,
        &selection,
        &fixture.options(true),
    )
    .unwrap_err();
    let message = error.to_string();
    assert!(message.contains("lsg.webmanifest"), "{message}");
    assert!(!output.exists());
    // The same garden without the key has no manifest to collide with.
    fixture.config("");
    assert!(
        publish_site_with_options(
            fixture.0.join("garden"),
            fixture.0.join("plain"),
            FilenameFormat::TripleLowbar,
            &selection,
            &fixture.options(true),
        )
        .is_ok()
    );
}

#[test]
fn a_renamed_icon_with_spaces_and_a_dot_segment_is_accepted() {
    let fixture = Fixture::new();
    fixture.write_bytes("assets/my icon.png", &png(1024, 5_000));
    fixture.config(r#":lsg/install {:name "Notes" :icon "./assets/my icon.png"}"#);
    let output = fixture.publish("site", false).unwrap();
    let manifest = fs::read_to_string(output.join("lsg.webmanifest")).unwrap();
    assert!(
        manifest.contains("\"sizes\":\"1024x1024\"")
            && manifest.contains("\"short_name\":\"Notes\"")
    );
}
