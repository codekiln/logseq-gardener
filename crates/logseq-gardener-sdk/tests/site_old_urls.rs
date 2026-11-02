//! Old page URL redirects: digest files of the default layout that open readable pages, the
//! name map that `index.html` reads for Logseq `#/page/<name>` links, and the collision check.
use logseq_gardener_sdk::journals::JournalDate;
use logseq_gardener_sdk::page_titles::FilenameFormat;
use logseq_gardener_sdk::publishing::NamespaceSelection;
use logseq_gardener_sdk::site::{PageUrls, SiteError, SiteOptions, publish_site_with_options};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "lsg-old-urls-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).unwrap();
        let fixture = Self(root);
        fixture.write(
            "logseq/config.edn",
            r#"{:publishing/all-pages-public? true :journal/page-title-format "yyyy-MM-dd EEE"}"#,
        );
        fixture.write(
            "pages/GitP___A___Session.md",
            "alias:: Session, Shared\n\n- The session. See [[Other]].\n",
        );
        fixture.write("pages/Other.md", "alias:: Shared\n\n- Other page.\n");
        // `Claude/Artifact` is published under the folder spelling of the page `CLAUDE`.
        fixture.write("pages/CLAUDE.md", "- The upper-case page.\n");
        fixture.write("pages/Claude___Artifact.md", "- ARTIFACT_SENTINEL\n");
        // A title with a backslash keeps its digest file under readable URLs.
        fixture.write(
            "pages/Backslash.md",
            "title:: GitP/A\\B\n\n- BACKSLASH_SENTINEL\n",
        );
        // Logseq titles this page `SolveSpace`; releases before lsg did named its file
        // after `Solvespace`.
        fixture.write(
            "pages/Solvespace.md",
            "alias:: [[SolveSpace]]\n\n- SOLVESPACE_SENTINEL\n",
        );
        fixture.write("journals/2026_10_23.md", "- A journal entry.\n");
        fixture
    }

    fn write(&self, name: &str, text: &str) {
        let path = self.0.join("garden").join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    fn options(&self, page_urls: PageUrls, old_url_redirects: bool) -> SiteOptions {
        SiteOptions {
            journals: true,
            build_date: Some(JournalDate::new(2026, 10, 24).unwrap()),
            page_urls,
            old_url_redirects,
            ..SiteOptions::default()
        }
    }

    fn publish(&self, site: &str, options: &SiteOptions) -> Result<PathBuf, SiteError> {
        let output = self.0.join(site);
        let selection =
            NamespaceSelection::new(&["GitP", "Other", "CLAUDE", "Claude", "SolveSpace"], &[])
                .unwrap();
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

/// Title to `url` of every page, journal and namespace page in `lsg-site.json`.
fn urls(output: &Path) -> BTreeMap<String, String> {
    let site: Value =
        serde_json::from_slice(&fs::read(output.join("lsg-site.json")).unwrap()).unwrap();
    site["pages"]
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
                page["title"].as_str().unwrap().to_owned(),
                page["url"].as_str().unwrap().to_owned(),
            )
        })
        .collect()
}

fn root_files(output: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(output)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

/// The 64-bit FNV-1a digest that names a page `p-<digest>.html` in the default layout.
fn digest(title: &str) -> String {
    let mut value: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in title.bytes() {
        value = (value ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3);
    }
    format!("{value:016x}")
}

#[test]
fn every_default_layout_page_file_opens_the_readable_page() {
    let f = Fixture::new();
    let old = urls(
        &f.publish("old", &f.options(PageUrls::Digest, false))
            .unwrap(),
    );
    let output = f
        .publish("new", &f.options(PageUrls::Readable, true))
        .unwrap();
    let new = urls(&output);
    assert_eq!(
        old.keys().collect::<Vec<_>>(),
        new.keys().collect::<Vec<_>>()
    );
    assert_eq!(new["Claude/Artifact"], "CLAUDE/Artifact/");
    for (title, file) in &old {
        let route = &new[title];
        let html = fs::read_to_string(output.join(file)).unwrap();
        if file == route {
            assert_eq!(title, "GitP/A\\B");
            assert!(
                html.contains("BACKSLASH_SENTINEL"),
                "{title} is not its page"
            );
            continue;
        }
        assert!(
            html.contains(&format!("location.replace(\"{route}\" + location.hash)")),
            "{file} for {title}: {html}"
        );
        assert!(html.contains(&format!("<link rel=\"canonical\" href=\"{route}\">")));
        assert!(html.contains(&format!("content=\"0; url={route}\"")));
        assert!(html.contains("<meta name=\"robots\" content=\"noindex\">"));
    }
}

#[test]
fn the_title_spelling_of_earlier_releases_opens_the_page() {
    let f = Fixture::new();
    let output = f
        .publish("site", &f.options(PageUrls::Readable, true))
        .unwrap();
    assert_eq!(urls(&output)["SolveSpace"], "SolveSpace/");
    for title in ["SolveSpace", "Solvespace"] {
        let html = fs::read_to_string(output.join(format!("p-{}.html", digest(title)))).unwrap();
        assert!(
            html.contains("location.replace(\"SolveSpace/\" + location.hash)"),
            "{title}: {html}"
        );
    }
}

#[test]
fn index_reads_a_map_of_titles_journals_and_aliases() {
    let f = Fixture::new();
    let output = f
        .publish("site", &f.options(PageUrls::Readable, true))
        .unwrap();
    let map: BTreeMap<String, String> =
        serde_json::from_slice(&fs::read(output.join("lsg-old-urls.json")).unwrap()).unwrap();
    assert_eq!(map["gitp/a/session"], "GitP/A/Session/");
    assert_eq!(map["session"], "GitP/A/Session/");
    assert_eq!(map["claude/artifact"], "CLAUDE/Artifact/");
    assert_eq!(map["2026-10-23 fri"], "2026-10-23%20Fri/");
    assert_eq!(map["gitp/a"], "GitP/A/");
    assert!(
        !map.contains_key("shared"),
        "an alias of two pages: {map:?}"
    );
    let index = fs::read_to_string(output.join("index.html")).unwrap();
    assert_eq!(index.matches("<script data-lsg-old-urls>").count(), 1);
    assert!(index.contains("fetch(\"lsg-old-urls.json\")"));
    assert!(index.contains("location.replace(\"pages.html\")"));
    assert!(index.contains("location.replace(\"graph.html\")"));
    let pages = fs::read_to_string(output.join("pages.html")).unwrap();
    assert!(!pages.contains("data-lsg-old-urls"));
}

#[test]
fn digest_layout_redirects_only_hash_links() {
    let f = Fixture::new();
    let output = f
        .publish("site", &f.options(PageUrls::Digest, true))
        .unwrap();
    let pages: Vec<String> = urls(&output).into_values().collect();
    for name in root_files(&output) {
        if name.starts_with("p-") {
            assert!(pages.contains(&name), "{name} is not a page");
        }
    }
    let map: BTreeMap<String, String> =
        serde_json::from_slice(&fs::read(output.join("lsg-old-urls.json")).unwrap()).unwrap();
    assert_eq!(
        map["session"],
        format!("p-{}.html", digest("GitP/A/Session"))
    );
}

#[test]
fn without_the_option_nothing_is_added() {
    let f = Fixture::new();
    for urls in [PageUrls::Digest, PageUrls::Readable] {
        let site = format!("site-{urls:?}");
        let output = f.publish(&site, &f.options(urls, false)).unwrap();
        assert!(!output.join("lsg-old-urls.json").exists());
        let index = fs::read_to_string(output.join("index.html")).unwrap();
        assert!(!index.contains("data-lsg-old-urls"));
        if urls == PageUrls::Readable {
            let digests: Vec<String> = root_files(&output)
                .into_iter()
                .filter(|name| name.starts_with("p-"))
                .collect();
            assert_eq!(digests, [format!("p-{}.html", digest("GitP/A\\B"))]);
        }
    }
}

#[test]
fn a_page_named_like_a_redirect_file_fails_before_output() {
    let f = Fixture::new();
    let name = format!("p-{}.html", digest("Other"));
    f.write("pages/Named.md", &format!("title:: {name}\n\n- body\n"));
    let selection = NamespaceSelection::new(&["Other", &name], &[]).unwrap();
    let output = f.0.join("site");
    let error = publish_site_with_options(
        f.0.join("garden"),
        &output,
        FilenameFormat::TripleLowbar,
        &selection,
        &f.options(PageUrls::Readable, true),
    )
    .unwrap_err();
    assert!(error.to_string().contains(&name), "{error}");
    assert!(!output.exists());
}
