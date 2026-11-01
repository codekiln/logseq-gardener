//! Readable page URLs: page folders, links that resolve from them, `lsg-site.json`, search and
//! plugin metadata routes, and the unsafe-title and collision checks.
use logseq_gardener_sdk::journals::JournalDate;
use logseq_gardener_sdk::page_titles::FilenameFormat;
use logseq_gardener_sdk::publishing::NamespaceSelection;
use logseq_gardener_sdk::search::SearchOptions;
use logseq_gardener_sdk::site::{PageUrls, SiteError, SiteOptions, publish_site_with_options};
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);

const BLOCK: &str = "6730a1b2-0000-4000-8000-0000000000aa";
const START: &str = "Docs/Guide/Start/";
const ENCODED: &str = "Docs/Guide/Q%26A%3A%202026/";

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "lsg-readable-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).unwrap();
        // The stylesheet path must have no symlink components, such as macOS's /var.
        let fixture = Self(fs::canonicalize(root).unwrap());
        fixture.write(
            "logseq/config.edn",
            r#"{:publishing/all-pages-public? true :journal/page-title-format "yyyy-MM-dd EEE" :publishing/page-properties? true}"#,
        );
        fixture.write(
            "pages/Docs___Guide___Start.md",
            &format!(
                "recording:: https://example.com/start.mp3\n\n- Start here. See [[Docs/Guide/Q&A: 2026]] and [[Other]].\n\t- Nested child with ![logo](../assets/logo.png)\n- A block with an id\n  id:: {BLOCK}\n- The unicornword lives on this page\n"
            ),
        );
        fixture.write(
            "pages/Docs___Guide___QA.md",
            "title:: Docs/Guide/Q&A: 2026\n\n- Questions and answers. Back to [[Docs/Guide/Start]].\n",
        );
        fixture.write(
            "pages/Other.md",
            &format!("- Other quotes (({BLOCK})) and links [[Docs/Guide/Start]].\n"),
        );
        fixture.write(
            "pages/Docs___Private.md",
            "public:: false\n- PRIVATE_SENTINEL\n",
        );
        fixture.write(
            "journals/2026_10_23.md",
            "- A journal entry about [[Docs/Guide/Start]].\n",
        );
        fixture.write("assets/logo.png", "PNG");
        fixture.write(
            "player/player.mjs",
            "export default { id: 'player', apiVersion: 1, setup() {} };\n",
        );
        fixture.write(
            "player/lsg-plugin.json",
            r#"{"id":"player","apiVersion":1,"version":"1.0.0","module":"player.mjs","activation":"after-load","pageProperties":["recording"]}"#,
        );
        fixture.write("theme.css", "main { color: teal; }\n");
        fixture
    }

    fn write(&self, name: &str, text: &str) {
        let path = self.0.join("garden").join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    fn options(&self) -> SiteOptions {
        SiteOptions {
            journals: true,
            build_date: Some(JournalDate::new(2026, 10, 24).unwrap()),
            search: Some(SearchOptions::default()),
            stylesheet: Some(self.0.join("garden/theme.css")),
            page_urls: PageUrls::Readable,
            ..SiteOptions::default()
        }
    }

    fn publish(&self, site: &str, options: &SiteOptions) -> Result<PathBuf, SiteError> {
        let output = self.0.join(site);
        let selection = NamespaceSelection::new(&["Docs", "Other"], &[]).unwrap();
        publish_site_with_options(
            self.0.join("garden"),
            &output,
            FilenameFormat::TripleLowbar,
            &selection,
            options,
        )
        .map(|_| output)
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn files(root: &Path) -> Vec<String> {
    fn visit(root: &Path, path: &Path, result: &mut Vec<String>) {
        for entry in fs::read_dir(path).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                visit(root, &path, result);
            } else {
                let relative = path.strip_prefix(root).unwrap().to_string_lossy();
                result.push(relative.replace('\\', "/"));
            }
        }
    }
    let mut result = Vec::new();
    visit(root, root, &mut result);
    result.sort();
    result
}

/// The page file of a readable route.
fn read(output: &Path, route: &str) -> String {
    fs::read_to_string(output.join(decode(route)).join("index.html"))
        .unwrap_or_else(|error| panic!("{route}: {error}"))
}

/// Percent-decode a URL path.
fn decode(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut out = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            out.push(u8::from_str_radix(&text[i + 1..i + 3], 16).unwrap());
            i += 3;
        } else {
            out.push(bytes[i]);
            i += 1;
        }
    }
    String::from_utf8(out).unwrap()
}

/// Every relative `href` and `src` in `html` at output path `file`, resolved against the
/// document's base, as an output path: a URL ending in `/` names the folder's `index.html`.
/// `None` marks a URL that climbs above the output root.
fn targets(file: &str, html: &str) -> Vec<(String, Option<String>)> {
    let mut folder: Vec<String> = file.split('/').map(str::to_owned).collect();
    folder.pop();
    let mut html = html.to_owned();
    if let Some(start) = html.find("<base href=\"") {
        let rest = &html[start + 12..];
        let end = rest.find('"').unwrap();
        for part in rest[..end].split('/').filter(|part| !part.is_empty()) {
            assert_eq!(part, "..", "{file}: base must climb to the root");
            folder
                .pop()
                .unwrap_or_else(|| panic!("{file}: base climbs above the root"));
        }
        html.replace_range(start..start + 12 + end + 2, "");
    }
    let mut found = Vec::new();
    for attribute in ["href=\"", "src=\""] {
        for (at, _) in html.match_indices(attribute) {
            let rest = &html[at + attribute.len()..];
            let value = rest[..rest.find('"').unwrap()].replace("&amp;", "&");
            if value.starts_with('#') || value.contains(':') || value.starts_with('/') {
                continue;
            }
            let path = value.split(['#', '?']).next().unwrap();
            let mut segments = folder.clone();
            let mut valid = true;
            for part in path.split('/') {
                match part {
                    "" | "." => {}
                    ".." => valid &= segments.pop().is_some(),
                    part => segments.push(decode(part)),
                }
            }
            if path.is_empty() || path.ends_with('/') {
                segments.push("index.html".to_owned());
            }
            found.push((value, valid.then(|| segments.join("/"))));
        }
    }
    found
}

#[test]
fn pages_live_in_title_folders_and_lists_stay_at_the_root() {
    let f = Fixture::new();
    let output = f.publish("site", &f.options()).unwrap();
    let all = files(&output);
    for file in [
        "Docs/Guide/Start/index.html",
        "Docs/Guide/Q&A: 2026/index.html",
        "Docs/Guide/index.html",
        "Docs/index.html",
        "Other/index.html",
        "2026-10-23 Fri/index.html",
        "index.html",
        "pages.html",
        "graph.html",
        "custom.css",
        "lsg-site.json",
    ] {
        assert!(
            all.iter().any(|path| path == file),
            "{file} missing: {all:?}"
        );
    }
    assert!(
        !all.iter().any(|path| path.starts_with("p-")),
        "digest page files remain: {all:?}"
    );
    assert!(!all.iter().any(|path| path.starts_with("Docs/Private")));
    assert!(
        all.iter()
            .any(|path| path.starts_with("_refs/p-") && path.ends_with("-linked-1.html")),
        "{all:?}"
    );
    let start = read(&output, START);
    assert!(start.starts_with(
        "<!doctype html>\n<html lang=\"en\"><head><meta charset=\"utf-8\"><base href=\"../../../\"><script data-lsg-readable>"
    ));
    assert!(start.contains(&format!("href=\"{ENCODED}\"")));
    assert!(start.contains("href=\"Other/\""));
    assert!(start.contains("href=\"Docs/Guide/\""));
    assert!(start.contains("src=\"assets/a-"));
    assert!(start.contains("<link rel=\"stylesheet\" href=\"custom.css\">"));
    assert!(!start.contains("<use href=\"#lsg-i-graph\"/>"));
    assert!(start.contains("data-lsg-slides"));
    let other = read(&output, "Other/");
    assert!(other.contains("<base href=\"../\">"));
    assert!(other.contains(&format!("href=\"{START}#block-{BLOCK}\"")));
    let home = fs::read_to_string(output.join("index.html")).unwrap();
    assert!(!home.contains("<base"));
    assert!(home.contains("<script data-lsg-readable>"));
    assert!(home.contains("href=\"2026-10-23%20Fri/\""));
}

#[test]
fn every_link_and_resource_resolves_to_an_output_file() {
    let f = Fixture::new();
    let output = f.publish("site", &f.options()).unwrap();
    let all = files(&output);
    let mut checked = 0;
    for file in all
        .iter()
        .filter(|path| path.ends_with(".html") && !path.starts_with("flashcards/"))
    {
        let html = fs::read_to_string(output.join(file)).unwrap();
        for (value, target) in targets(file, &html) {
            let target = target.unwrap_or_else(|| panic!("{file}: {value} leaves the site"));
            assert!(
                all.contains(&target),
                "{file}: {value} resolves to missing {target}"
            );
            checked += 1;
        }
    }
    assert!(checked > 100, "only {checked} links checked");
}

#[test]
fn site_manifest_lists_folder_urls() {
    let f = Fixture::new();
    let output = f.publish("site", &f.options()).unwrap();
    let site: Value =
        serde_json::from_slice(&fs::read(output.join("lsg-site.json")).unwrap()).unwrap();
    let pages = site["pages"].as_array().unwrap();
    let url = |title: &str| {
        pages
            .iter()
            .find(|page| page["title"] == title && page["kind"] != "home")
            .unwrap_or_else(|| panic!("{title} missing: {pages:?}"))["url"]
            .as_str()
            .unwrap()
            .to_owned()
    };
    assert_eq!(url("Docs/Guide/Start"), START);
    assert_eq!(url("Docs/Guide/Q&A: 2026"), ENCODED);
    assert_eq!(url("Docs"), "Docs/");
    assert_eq!(url("2026-10-23 Fri"), "2026-10-23%20Fri/");
    let all = files(&output);
    for page in pages {
        let url = page["url"].as_str().unwrap();
        let file = if url.ends_with('/') {
            format!("{}index.html", decode(url))
        } else {
            url.to_owned()
        };
        assert!(all.contains(&file), "{url} has no file {file}");
        assert_eq!(
            url.ends_with('/'),
            !matches!(page["kind"].as_str(), Some("home" | "list")),
            "{page}"
        );
    }
}

#[test]
fn search_routes_are_page_folders_and_the_engine_accepts_them() {
    let f = Fixture::new();
    let output = f.publish("site", &f.options()).unwrap();
    let all = files(&output);
    let titles = all
        .iter()
        .find(|path| path.starts_with("lsg-search-titles-"))
        .unwrap();
    let titles: Value = serde_json::from_slice(&fs::read(output.join(titles)).unwrap()).unwrap();
    let routes: Vec<&str> = titles["pages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|page| page[1].as_str().unwrap())
        .collect();
    assert!(
        routes.contains(&START) && routes.contains(&ENCODED),
        "{routes:?}"
    );
    let engine = all
        .iter()
        .find(|path| path.starts_with("lsg-search-engine-"))
        .unwrap();
    let engine = fs::read_to_string(output.join(engine)).unwrap();
    assert!(engine.contains("/^(?:p-[a-f0-9]{16}\\.html|(?:(?:[\\w.~-]|%[0-9A-F]{2})+\\/)+)$/u"));
    assert!(!engine.contains("/^p-[a-f0-9]{16}\\.html$/u"));
}

#[test]
fn plugin_pages_carry_folder_urls_and_the_readable_host() {
    let f = Fixture::new();
    let options = SiteOptions {
        plugins: vec![f.0.join("garden/player/lsg-plugin.json")],
        ..f.options()
    };
    let output = f.publish("site", &options).unwrap();
    let start = read(&output, START);
    let metadata = start
        .split("<script type=\"application/json\" data-lsg-page>")
        .nth(1)
        .and_then(|rest| rest.split("</script>").next())
        .unwrap();
    let metadata: Value = serde_json::from_str(metadata).unwrap();
    assert_eq!(metadata["url"], START);
    assert_eq!(
        metadata["plugins"]["player"]["recording"],
        "https://example.com/start.mp3"
    );
    let host = files(&output)
        .into_iter()
        .find(|path| path.starts_with("lsg-host-"))
        .unwrap();
    let host = fs::read_to_string(output.join(host)).unwrap();
    assert!(host.contains("/^([^/]+\\.html|([^/]+\\/)+)?$/"));
    assert!(host.contains("'title,meta,base,[data-lsg-page],[data-lsg-host]'"));
    assert!(host.contains(".href = base.href;"));
}

#[test]
fn a_homepage_is_written_at_the_root_and_in_its_folder() {
    let f = Fixture::new();
    let options = SiteOptions {
        homepage: Some("Docs/Guide/Start".to_owned()),
        ..f.options()
    };
    let output = f.publish("site", &options).unwrap();
    let home = fs::read_to_string(output.join("index.html")).unwrap();
    assert!(home.contains("unicornword") && !home.contains("<base"));
    assert!(read(&output, START).contains("unicornword"));
}

#[test]
fn readable_scripts_replace_the_page_address_lines() {
    let f = Fixture::new();
    let output = f.publish("site", &f.options()).unwrap();
    let outline = fs::read_to_string(output.join("lsg-outline.js")).unwrap();
    assert!(outline.contains("page.href = location.href.split('#')[0];"));
    assert!(outline.contains("link.href = page.href + ZOOM + blockId(up);"));
    let graph = fs::read_to_string(output.join("lsg-graph.js")).unwrap();
    assert!(graph.contains("const titleAt = route =>"));
    assert!(graph.contains("const routeOf = title => foldable(title)"));
    assert!(graph.contains("'_graph/p/' + digestAt(route) + '.json'"));
}

/// A title with a literal slash, stored as `%2F` in its file name, whose segment
/// `Rename with ` ends with a space.
const SPACED: &str = "Docs/Q/Rename with /rename, then resume?";

#[test]
fn unsafe_titles_keep_their_digest_file() {
    let f = Fixture::new();
    f.write(
        "pages/Docs___Q___Rename with %2Frename, then resume%3F.md",
        "- SPACED_SENTINEL, see [[Docs/Guide/Start]]\n",
    );
    f.write(
        "pages/Backslash.md",
        "title:: Docs/A\\B\n\n- BACKSLASH_SENTINEL\n",
    );
    f.write(
        "pages/Docs___Links.md",
        &format!("- Links to [[{SPACED}]] and [[Docs/A\\B]]\n"),
    );
    let report = {
        let output = f.0.join("site");
        let selection = NamespaceSelection::new(&["Docs", "Other"], &[]).unwrap();
        publish_site_with_options(
            f.0.join("garden"),
            &output,
            FilenameFormat::TripleLowbar,
            &selection,
            &f.options(),
        )
        .unwrap()
    };
    let output = f.0.join("site");
    let all = files(&output);
    let site: Value =
        serde_json::from_slice(&fs::read(output.join("lsg-site.json")).unwrap()).unwrap();
    // The page's URL in `lsg-site.json`, which must be a digest file at the root.
    let digest_file = |title: &str| {
        let url = site["pages"]
            .as_array()
            .unwrap()
            .iter()
            .find(|page| page["title"] == title)
            .unwrap_or_else(|| panic!("{title:?} missing from lsg-site.json"))["url"]
            .as_str()
            .unwrap()
            .to_owned();
        assert!(
            url.starts_with("p-") && url.ends_with(".html") && !url.contains('/'),
            "{title:?} has URL {url}"
        );
        url
    };
    for (title, sentinel, problem) in [
        (
            SPACED,
            "SPACED_SENTINEL",
            "a segment starts or ends with whitespace",
        ),
        ("Docs/A\\B", "BACKSLASH_SENTINEL", "it contains a backslash"),
    ] {
        let file = digest_file(title);
        let html = fs::read_to_string(output.join(&file))
            .unwrap_or_else(|error| panic!("{file}: {error}: {all:?}"));
        assert!(html.contains(sentinel));
        assert!(!html.contains("<base"), "{file} sits at the root");
        assert!(html.contains("<script data-lsg-readable>"));
        assert!(
            read(&output, "Docs/Links/").contains(&format!("href=\"{file}\"")),
            "Docs/Links does not link to {file}"
        );
        assert!(
            report.diagnostics.iter().any(|diagnostic| diagnostic
                .message
                .contains(&format!("{title:?}"))
                && diagnostic.message.contains(problem)),
            "no diagnostic for {title:?}: {:?}",
            report.diagnostics
        );
    }
    // The prefix `Docs/Q/Rename with ` is a namespace page with an unsafe title too.
    assert!(all.contains(&digest_file("Docs/Q/Rename with ")), "{all:?}");
    assert!(all.contains(&"Docs/Q/index.html".to_owned()), "{all:?}");
    assert!(read(&output, START).contains("<base href=\"../../../\">"));
    for file in all
        .iter()
        .filter(|path| path.ends_with(".html") && !path.starts_with("flashcards/"))
    {
        let html = fs::read_to_string(output.join(file)).unwrap();
        for (value, target) in targets(file, &html) {
            let target = target.unwrap_or_else(|| panic!("{file}: {value} leaves the site"));
            assert!(
                all.contains(&target),
                "{file}: {value} resolves to missing {target}"
            );
        }
    }
}

#[test]
fn colliding_paths_fail_before_output_and_name_both_pages() {
    for (pages, expected) in [
        (
            [
                ("Tools___Vim.md", "- one\n"),
                ("Second.md", "title:: tools/vim\n\n- two\n"),
            ],
            "pages \"Tools/Vim\" and \"tools/vim\" cannot both have readable URLs: their paths differ only in letter case or Unicode normalization",
        ),
        (
            [
                ("Tools___A.md", "- one\n"),
                ("tools___B.md", "title:: tools/B\n\n- two\n"),
            ],
            "pages \"Tools\" and \"tools/B\" cannot both have readable URLs: their folders \"Tools\" and \"tools\" differ only in letter case",
        ),
        (
            [
                ("Tools.md", "- one\n"),
                ("Index.md", "title:: Tools/index.html\n\n- two\n"),
            ],
            "pages \"Tools\" and \"Tools/index.html\" cannot both have readable URLs: the folder of \"Tools/index.html\" is the index.html of \"Tools\"",
        ),
    ] {
        let f = Fixture::new();
        for (name, text) in pages {
            f.write(&format!("pages/{name}"), text);
        }
        let selection = NamespaceSelection::new(&["Tools", "tools"], &[]).unwrap();
        let output = f.0.join("site");
        let error = publish_site_with_options(
            f.0.join("garden"),
            &output,
            FilenameFormat::TripleLowbar,
            &selection,
            &f.options(),
        )
        .unwrap_err();
        assert!(error.to_string().starts_with(expected), "{error}");
        assert!(!output.exists());
    }
}

#[test]
fn a_page_named_like_a_generated_root_file_fails_before_output() {
    let f = Fixture::new();
    f.write(
        "pages/Docs.md",
        "title:: assets\n\n- the assets page ![logo](../assets/logo.png)\n",
    );
    let selection = NamespaceSelection::new(&["assets", "Docs"], &[]).unwrap();
    let output = f.0.join("site");
    let error = publish_site_with_options(
        f.0.join("garden"),
        &output,
        FilenameFormat::TripleLowbar,
        &selection,
        &f.options(),
    )
    .unwrap_err();
    assert_eq!(
        error.to_string(),
        "page \"assets\" cannot have a readable URL: the build writes assets at the output root"
    );
    assert!(!output.exists());
}

#[test]
fn default_layout_ignores_readable_checks() {
    let f = Fixture::new();
    f.write("pages/Unsafe.md", "title:: Docs/../Up\n\n- body\n");
    let options = SiteOptions {
        page_urls: PageUrls::Digest,
        ..f.options()
    };
    let output = f.publish("site", &options).unwrap();
    assert!(files(&output).iter().any(|path| path.starts_with("p-")));
    assert!(
        !fs::read_to_string(output.join("index.html"))
            .unwrap()
            .contains("data-lsg-readable")
    );
}
