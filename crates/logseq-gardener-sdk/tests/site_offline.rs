//! Offline reading: the `:lsg/offline` key, the worker, the page script, the script tag and the
//! two More menu items, with and without readable URLs, and the sites that do not opt in.
use logseq_gardener_sdk::journals::JournalDate;
use logseq_gardener_sdk::offline::{MENU_ITEMS, PAGE_HEAD, SCRIPT_ROUTE, WORKER_ROUTE};
use logseq_gardener_sdk::page_titles::FilenameFormat;
use logseq_gardener_sdk::publishing::NamespaceSelection;
use logseq_gardener_sdk::search::SearchOptions;
use logseq_gardener_sdk::site::{PageUrls, SiteError, SiteOptions, publish_site_with_options};
use std::collections::BTreeMap;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);

const ON: &str = ":lsg/offline true";
const OFF: &str = ":lsg/offline false";
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
            "lsg-offline-site-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).unwrap();
        let fixture = Self(fs::canonicalize(root).unwrap());
        fixture.config(ON);
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

const OFFLINE_FILES: [&str; 2] = [WORKER_ROUTE, SCRIPT_ROUTE];

/// Remove what the key adds to a page: the script tag and the two menu items.
fn strip(html: &str) -> String {
    html.replace(PAGE_HEAD, "").replace(MENU_ITEMS, "")
}

#[test]
fn true_writes_the_worker_and_the_page_script() {
    let fixture = Fixture::new();
    let files = files(&fixture.publish("site", false).unwrap());
    let worker = text(&files[WORKER_ROUTE]).into_owned();
    assert!(worker.contains("lsg-offline:") && worker.contains("PAGES = 250"));
    assert!(worker.len() <= 5_000, "{} bytes", worker.len());
    let script = text(&files[SCRIPT_ROUTE]).into_owned();
    assert!(script.contains("data-lsg-offline-save") && script.len() <= 2_000);
    // Neither file is a page, and the site manifest lists no offline file.
    let site = text(&files["lsg-site.json"]).into_owned();
    assert!(
        !site.contains(WORKER_ROUTE) && !site.contains(SCRIPT_ROUTE),
        "{site}"
    );
}

#[test]
fn every_full_page_has_the_tag_and_the_items_and_nothing_else_does() {
    for readable in [false, true] {
        let fixture = Fixture::new();
        let files = files(&fixture.publish("site", readable).unwrap());
        let mut full = Vec::new();
        for (route, bytes) in &files {
            if !route.ends_with(".html") {
                if route != SCRIPT_ROUTE {
                    assert!(!text(bytes).contains("data-lsg-offline-save"), "{route}");
                }
                continue;
            }
            let html = text(bytes).into_owned();
            if is_full_page(route, &html) {
                assert_eq!(html.matches(PAGE_HEAD).count(), 1, "{route}");
                assert_eq!(html.matches(MENU_ITEMS).count(), 1, "{route}");
                assert_eq!(html.matches("lsg-offline.js").count(), 1, "{route}");
                assert!(!html.contains(WORKER_ROUTE), "{route} names the worker");
                if let Some(base) = html.find("<base href") {
                    assert!(base < html.find(PAGE_HEAD).unwrap(), "{route}");
                }
                full.push(route.as_str());
            } else {
                assert!(!html.contains("lsg-offline"), "{route} is not a full page");
            }
        }
        for expected in ["index.html", "pages.html", "graph.html"] {
            assert!(full.contains(&expected), "{expected} in {full:?}");
        }
        assert!(
            full.iter().any(|route| route.starts_with("journals")) || full.contains(&"index.html")
        );
        if readable {
            for expected in [
                "Docs/Guide/Start/index.html",
                "Docs/Guide/index.html",
                "Docs/index.html",
                "Other/index.html",
            ] {
                assert!(full.contains(&expected), "{expected} in {full:?}");
            }
            assert!(
                files.iter().any(|(route, bytes)| route.starts_with("p-")
                    && text(bytes).contains("http-equiv=\"refresh\"")),
                "an old-address redirect page exists"
            );
        }
        assert!(files.keys().any(|route| route.starts_with("_refs/")));
        for (route, bytes) in files
            .iter()
            .filter(|(route, _)| route.starts_with("_refs/"))
        {
            assert!(!text(bytes).contains("lsg-offline"), "{route}");
        }
    }
}

#[test]
fn every_page_resolves_the_script_and_the_worker_to_the_site_root() {
    for readable in [false, true] {
        let fixture = Fixture::new();
        let files = files(&fixture.publish("site", readable).unwrap());
        for (route, bytes) in &files {
            let html = text(bytes).into_owned();
            if !is_full_page(route, &html) {
                continue;
            }
            let base = html
                .split("<base href=\"")
                .nth(1)
                .map_or("", |rest| rest.split('"').next().unwrap());
            let folder = route.rsplit_once('/').map_or("", |(folder, _)| folder);
            let mut parts: Vec<&str> = folder.split('/').filter(|p| !p.is_empty()).collect();
            if !base.is_empty() {
                parts.truncate(parts.len().saturating_sub(base.matches("../").count()));
            }
            // The script finds the worker beside itself, at the root of the site.
            for target in OFFLINE_FILES {
                let resolved = parts
                    .iter()
                    .copied()
                    .chain([target])
                    .collect::<Vec<_>>()
                    .join("/");
                assert!(
                    files.contains_key(&resolved),
                    "{route}: {target} resolves to {resolved}"
                );
                assert_eq!(resolved, target, "{route}");
            }
        }
    }
}

#[test]
fn false_writes_only_the_removing_worker() {
    let fixture = Fixture::new();
    fixture.config(OFF);
    for readable in [false, true] {
        let files = files(
            &fixture
                .publish(&format!("off-{readable}"), readable)
                .unwrap(),
        );
        let worker = text(&files[WORKER_ROUTE]).into_owned();
        assert!(worker.contains("unregister()") && worker.contains("caches.delete("));
        assert!(!worker.contains("fetch"), "{worker}");
        assert!(!files.contains_key(SCRIPT_ROUTE));
        for (route, bytes) in &files {
            let html = text(bytes);
            assert!(
                !html.contains("lsg-offline.js") && !html.contains("data-lsg-offline"),
                "{route}"
            );
        }
    }
}

#[test]
fn a_garden_without_the_key_has_no_offline_files_or_markup() {
    let fixture = Fixture::new();
    fixture.config("");
    for readable in [false, true] {
        let files = files(
            &fixture
                .publish(&format!("plain-{readable}"), readable)
                .unwrap(),
        );
        for name in OFFLINE_FILES {
            assert!(!files.contains_key(name), "{name}");
        }
        for (route, bytes) in &files {
            let contents = text(bytes);
            for marker in [
                "lsg-sw.js",
                "lsg-offline",
                "serviceWorker",
                "data-lsg-offline",
            ] {
                assert!(!contents.contains(marker), "{route} holds {marker}");
            }
        }
    }
}

#[test]
fn the_key_adds_only_its_files_the_tag_and_the_items() {
    // The same garden with the key, less its files, the tag and the menu items, is the garden
    // without it: a garden without the key publishes the site it published before the key existed.
    for (key, added) in [(ON, &OFFLINE_FILES[..]), (OFF, &OFFLINE_FILES[..1])] {
        for readable in [false, true] {
            let fixture = Fixture::new();
            fixture.config(key);
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
                if added.contains(&route.as_str()) {
                    continue;
                }
                assert!(!OFFLINE_FILES.contains(&route.as_str()), "{key}: {route}");
                let bytes = if route.ends_with(".html") {
                    strip(&text(&bytes)).into_bytes()
                } else {
                    bytes
                };
                stripped.insert(route, bytes);
            }
            let mut other = without;
            // `lsg-site.json` records a revision from every file; it differs when files do.
            for map in [&mut stripped, &mut other] {
                map.remove("lsg-site.json");
            }
            assert_eq!(
                stripped.keys().collect::<Vec<_>>(),
                other.keys().collect::<Vec<_>>()
            );
            for (route, bytes) in &stripped {
                assert!(bytes == &other[route], "{key} {readable}: {route} differs");
            }
        }
    }
}

#[test]
fn the_worker_changes_with_the_page_list_and_not_with_a_block_edit() {
    let fixture = Fixture::new();
    let before = files(&fixture.publish("before", true).unwrap());
    fixture.write(
        "pages/Other.md",
        "- Other links [[Docs/Guide/Start]].\n- A new block\n",
    );
    let edited = files(&fixture.publish("edited", true).unwrap());
    assert!(before["Other/index.html"] != edited["Other/index.html"]);
    for name in OFFLINE_FILES {
        assert!(
            before[name] == edited[name],
            "{name} changed after a block edit"
        );
    }
    fixture.write("pages/Docs___New.md", "- A new page\n");
    let added = files(&fixture.publish("added", true).unwrap());
    assert!(
        before[WORKER_ROUTE] != added[WORKER_ROUTE],
        "a new page changes the worker"
    );
    assert!(before[SCRIPT_ROUTE] == added[SCRIPT_ROUTE]);
    fs::remove_file(fixture.0.join("garden/pages/Docs___New.md")).unwrap();
    let removed = files(&fixture.publish("removed", true).unwrap());
    assert!(
        before[WORKER_ROUTE] == removed[WORKER_ROUTE],
        "the same pages give the same worker"
    );
}

#[test]
fn pages_grow_by_a_fixed_small_amount() {
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
        assert_eq!(growth, PAGE_HEAD.len() + MENU_ITEMS.len(), "{route}");
        assert!(growth <= 600, "{route} grew by {growth} bytes");
        checked += 1;
    }
    assert!(checked >= 7, "{checked} pages checked");
}

#[test]
fn the_key_works_with_and_without_install() {
    let fixture = Fixture::new();
    let mut png = vec![0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a, 0, 0, 0, 13];
    png.extend(b"IHDR");
    png.extend(512u32.to_be_bytes());
    png.extend(512u32.to_be_bytes());
    png.resize(2_000, 0);
    fixture.write_bytes("assets/app-icon.png", &png);
    fixture.config(&format!(
        "{ON} :lsg/install {{:name \"Notes\" :icon \"assets/app-icon.png\"}}"
    ));
    let both = files(&fixture.publish("both", true).unwrap());
    let only = {
        fixture.config(ON);
        files(&fixture.publish("only", true).unwrap())
    };
    assert!(both[WORKER_ROUTE] == only[WORKER_ROUTE] && both[SCRIPT_ROUTE] == only[SCRIPT_ROUTE]);
    let page = text(&both["index.html"]).into_owned();
    assert!(
        page.contains(PAGE_HEAD) && page.contains("data-lsg-install") && page.contains(MENU_ITEMS)
    );
}

#[test]
fn key_errors_stop_the_build_before_output() {
    for (config, problem) in [
        (
            r#":lsg/offline "yes""#,
            r#"must be true or false, not "yes""#,
        ),
        (":lsg/offline 1", "must be true or false, not 1"),
        (
            ":default-home {:lsg/offline true}",
            "is a top-level key; it cannot sit inside :default-home",
        ),
    ] {
        let fixture = Fixture::new();
        fixture.config(config);
        let output = fixture.0.join("site");
        let error = fixture.publish("site", false).unwrap_err();
        assert!(matches!(error, SiteError::Offline(_)), "{error:?}");
        assert_eq!(
            error.to_string(),
            format!("logseq/config.edn: :lsg/offline {problem}")
        );
        assert!(!output.exists(), "{config}: output was created");
    }
}

#[test]
fn pages_named_like_the_offline_files_fail_under_readable_urls() {
    for (key, name) in [
        (ON, "lsg-sw.js"),
        (OFF, "lsg-sw.js"),
        (ON, "lsg-offline.js"),
    ] {
        let fixture = Fixture::new();
        fixture.config(key);
        fixture.write(
            &format!("pages/{name}.md"),
            "- A page named like an offline file\n",
        );
        let selection = NamespaceSelection::new(&["Docs", "Other", name], &[]).unwrap();
        let output = fixture.0.join("site");
        let error = publish_site_with_options(
            fixture.0.join("garden"),
            &output,
            FilenameFormat::TripleLowbar,
            &selection,
            &fixture.options(true),
        )
        .unwrap_err();
        assert!(
            matches!(error, SiteError::PageUrl(_)),
            "{key} {name}: {error:?}"
        );
        assert!(error.to_string().contains(name), "{error}");
        assert!(!output.exists());
        // Without the key there is no file to collide with.
        fixture.config("");
        assert!(
            publish_site_with_options(
                fixture.0.join("garden"),
                fixture.0.join("plain"),
                FilenameFormat::TripleLowbar,
                &selection,
                &fixture.options(true),
            )
            .is_ok(),
            "{name}"
        );
    }
}
