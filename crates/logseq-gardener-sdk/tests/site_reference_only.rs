//! Pages that exist only because published blocks name them (reference-only pages).
use logseq_gardener_sdk::journals::JournalDate;
use logseq_gardener_sdk::page_titles::FilenameFormat;
use logseq_gardener_sdk::publishing::NamespaceSelection;
use logseq_gardener_sdk::site::{
    PageUrls, SearchOptions, SiteOptions, SiteReport, publish_site_with_options,
};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
mod support;

static NEXT: AtomicUsize = AtomicUsize::new(0);

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "lsg-reference-only-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&root).unwrap();
        let fixture = Self(root);
        fixture.write(
            "logseq/config.edn",
            "{:publishing/all-pages-public? true :hidden [\"/pages/Draft.md\"]}",
        );
        fixture.write(
            "pages/Deno.md",
            "tags:: company\n\n- Deploy with [[Deno Deploy]]\n- A chord (C#/D♭)\n- Settings: [[Deno/Desktop/Settings/Developer]]\n- [[Deno/Private]] and [[Draft]] and [[Deno/Q]]\n",
        );
        fixture.write(
            "pages/Deno___Q___Why install deno globally.md",
            "- So [[deno deploy]] can run it\n- Noted on [[Nov 18th, 2024]]\n",
        );
        fixture.write("pages/Sourcegraph.md", "tags:: company\n\n- Code search\n");
        fixture.write(
            "pages/Deno___Private.md",
            "public:: false\n\n- Only this private page names [[Secret Project]]\n",
        );
        fixture.write("pages/Draft.md", "- A draft :hidden keeps out\n");
        fixture.write("pages/mise.md", "- A file outside a narrow selection\n");
        fixture
    }

    fn write(&self, path: &str, text: &str) {
        let path = self.0.join("garden").join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    fn publish_with(&self, output: &str, roots: &[&str], options: SiteOptions) -> SiteReport {
        publish_site_with_options(
            self.0.join("garden"),
            self.0.join(output),
            FilenameFormat::TripleLowbar,
            &NamespaceSelection::new(roots, &[]).unwrap(),
            &options,
        )
        .unwrap()
    }

    fn publish(&self, output: &str) -> SiteReport {
        self.publish_with(
            output,
            &["Deno", "Sourcegraph", "mise", "Draft"],
            SiteOptions {
                journals: true,
                build_date: JournalDate::new(2026, 10, 25),
                search: Some(SearchOptions::default()),
                ..SiteOptions::default()
            },
        )
    }

    fn page(&self, output: &str, title: &str) -> String {
        fs::read_to_string(self.0.join(output).join(route(title))).unwrap()
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn route(title: &str) -> String {
    let hash = title
        .as_bytes()
        .iter()
        .fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
            (hash ^ u64::from(*byte)).wrapping_mul(0x0100_0000_01b3)
        });
    format!("p-{hash:016x}.html")
}

fn files(root: &Path) -> BTreeMap<String, Vec<u8>> {
    let mut out = BTreeMap::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(folder) = stack.pop() {
        for entry in fs::read_dir(folder).unwrap().flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                let name = path
                    .strip_prefix(root)
                    .unwrap()
                    .to_string_lossy()
                    .into_owned();
                out.insert(name, fs::read(&path).unwrap());
            }
        }
    }
    out
}

#[test]
fn a_page_with_no_file_gets_a_page_that_links_reach() {
    let f = Fixture::new();
    let report = f.publish("site");
    // Deno Deploy, D♭), company, Deno/Desktop, Deno/Desktop/Settings, .../Developer,
    // Draft and the journal Nov 18th, 2024.
    assert_eq!(report.reference_only_pages, 8, "{:?}", report.diagnostics);
    let deploy = f.page("site", "Deno Deploy");
    assert!(deploy.contains("<h1>Deno Deploy</h1>"));
    assert!(deploy.contains("<h2>2 Linked References</h2>"), "{deploy}");
    assert!(deploy.contains("unlinked-references"));
    assert!(!deploy.contains("namespace-notice") && !deploy.contains("page-properties"));
    // Links in either case open the page with the ordinary link style.
    let deno = f.page("site", "Deno");
    let link = format!("<a href=\"{}\">Deno Deploy</a>", route("Deno Deploy"));
    assert!(deno.contains(&link), "{deno}");
    let question = f.page("site", "Deno/Q/Why install deno globally");
    assert!(question.contains(&format!(
        "<a href=\"{}\">deno deploy</a>",
        route("Deno Deploy")
    )));
    // A tag drops a slash at either end.
    assert!(deno.contains(&format!(
        "<a class=\"tag\" href=\"{}\">#/D♭)</a>",
        route("D♭)")
    )));
    // A generated namespace page keeps its Namespace style in page text.
    assert!(deno.contains(&format!(
        "<a class=\"namespace-link\" aria-label=\"Deno/Q (Namespace)\" href=\"{}\">",
        route("Deno/Q")
    )));
    // Withheld names stay unresolved with a diagnostic; a hidden file counts as no file.
    assert!(deno.contains("<span class=\"page-reference unresolved\"><span>Deno/Private</span>"));
    assert!(deno.contains(&format!("<a href=\"{}\">Draft</a>", route("Draft"))));
    let unresolved: Vec<&str> = report
        .diagnostics
        .iter()
        .filter(|d| d.message.starts_with("Unresolved"))
        .map(|d| d.message.as_str())
        .collect();
    assert_eq!(
        unresolved,
        ["Unresolved or excluded page reference: Deno/Private"]
    );
    // A name only a private page writes gets no page and appears nowhere.
    let site = files(&f.0.join("site"));
    assert!(
        site.values()
            .all(|bytes| !String::from_utf8_lossy(bytes).contains("Secret Project"))
    );
    // Tag page and namespace parents.
    let company = f.page("site", "company");
    assert!(company.contains("Pages tagged with \"company\""));
    assert!(company.contains(&format!(
        "href=\"{}\">Sourcegraph</a>",
        route("Sourcegraph")
    )));
    let settings = f.page("site", "Deno/Desktop/Settings");
    assert!(
        settings.contains("<h2>1 Linked Reference</h2>"),
        "{settings}"
    );
}

#[test]
fn other_views_list_and_find_reference_only_pages() {
    let f = Fixture::new();
    f.publish("site");
    let site = f.0.join("site");
    // All pages: kind `r`, no dates; the journal is a journal row.
    let list = fs::read_to_string(site.join("_pages/list.json")).unwrap();
    assert!(
        list.contains("[\"Deno Deploy\",null,\"r\",2,null,null]"),
        "{list}"
    );
    assert!(list.contains("[\"Nov 18th, 2024\",null,\"j\""), "{list}");
    // The site manifest and the journals feed.
    let manifest = fs::read_to_string(site.join("lsg-site.json")).unwrap();
    assert!(manifest.contains("\"kind\":\"reference\""), "{manifest}");
    let feed: String = ["index.html", "journals-2.html"]
        .iter()
        .filter_map(|name| fs::read_to_string(site.join(name)).ok())
        .collect();
    assert!(feed.contains(&format!(
        "<a class=\"journal-title\" href=\"{}\">Nov 18th, 2024</a>",
        route("Nov 18th, 2024")
    )));
    // Search: the title index holds the title without its computable route.
    let titles = files(&site)
        .into_iter()
        .find(|(name, _)| name.starts_with("lsg-search-titles-"))
        .unwrap()
        .1;
    assert!(String::from_utf8_lossy(&titles).contains("[\"Deno Deploy\"]"));
    // The graph flags the node as having a page: it opens.
    let neighbourhood =
        fs::read_to_string(site.join(format!("_graph/p/{}.json", &route("Deno Deploy")[2..18])))
            .unwrap();
    let json: serde_json::Value = serde_json::from_str(&neighbourhood).unwrap();
    assert_eq!(json["t"][0], "Deno Deploy");
    assert_eq!(json["f"][0].as_u64().unwrap() & 64, 0);
}

#[test]
fn a_narrow_selection_publishes_the_names_its_pages_write() {
    let f = Fixture::new();
    f.write("pages/Deno___Tools.md", "- Install with [[mise]]\n");
    let report = f.publish_with("site", &["Deno"], SiteOptions::default());
    let deploy = f.page("site", "Deno Deploy");
    assert!(deploy.contains("<h2>2 Linked References</h2>"));
    // `mise` has a file outside the selection: it stays unresolved and gets no page.
    assert!(!f.0.join("site").join(route("mise")).exists());
    assert!(
        report
            .diagnostics
            .iter()
            .any(|d| d.message == "Unresolved or excluded page reference: mise")
    );
}

#[test]
fn a_reference_only_address_that_collides_is_left_out() {
    let f = Fixture::new();
    f.write(
        "logseq/config.edn",
        "{:publishing/all-pages-public? true :publishing/page-properties? true}",
    );
    // A property value naming `assets/` is omitted for safety, so the row uses `plugins`.
    f.write(
        "pages/Deno___Logo.md",
        "see-also:: [[Plugins/Logo]]\n\n- [[Assets/Logo]]\n",
    );
    let report = f.publish_with(
        "site",
        &["Deno"],
        SiteOptions {
            page_urls: PageUrls::Readable,
            ..SiteOptions::default()
        },
    );
    assert!(
        report
            .diagnostics
            .iter()
            .any(|d| d.message.contains("\"Assets/Logo\" left out")
                && d.message.contains("assets")),
        "{:?}",
        report.diagnostics
    );
    let logo = fs::read_to_string(f.0.join("site/Deno/Logo/index.html")).unwrap();
    assert!(logo.contains("<span class=\"page-reference unresolved\"><span>Assets/Logo</span>"));
    assert!(logo.contains(
        "<dd><span class=\"page-reference unresolved no-brackets\"><span>Plugins/Logo</span></span></dd>"
    ));
    assert!(f.0.join("site/Deno Deploy/index.html").exists());
}

#[test]
fn unlinked_references_of_a_reference_only_page_hold_one_file() {
    let f = Fixture::new();
    f.write(
        "pages/Deno___Widget.md",
        "- [[Widget]] is a page no file holds\n",
    );
    for page in 0..60 {
        f.write(
            &format!("pages/Deno___Mention {page:02}.md"),
            "- A Widget mentioned in passing\n",
        );
    }
    f.publish_with("site", &["Deno"], SiteOptions::default());
    let stem = route("Widget").trim_end_matches(".html").to_owned();
    let refs = f.0.join("site/_refs");
    assert!(refs.join(format!("{stem}-unlinked-1.html.gz")).exists());
    assert!(!refs.join(format!("{stem}-unlinked-2.html.gz")).exists());
    let first = support::read_text(&refs.join(format!("{stem}-unlinked-1.html.gz")));
    assert!(first.contains("\"count\":60"), "{first}");
    assert!(first.contains("Showing the first 50 of 60 pages"));
}

#[test]
fn adding_a_name_rewrites_only_the_files_that_depend_on_it() {
    let f = Fixture::new();
    f.publish("before");
    let deno = fs::read_to_string(f.0.join("garden/pages/Deno.md")).unwrap();
    f.write(
        "pages/Deno.md",
        &(deno.clone() + "- Try [[Brand New Name]]\n"),
    );
    f.publish("after");
    let (old, new) = (files(&f.0.join("before")), files(&f.0.join("after")));
    let changed: Vec<&String> = new
        .iter()
        .filter(|(name, bytes)| old.get(*name) != Some(*bytes))
        .map(|(name, _)| name)
        .collect();
    let brand = route("Brand New Name");
    let brand_stem = &brand[..brand.len() - ".html".len()];
    let allowed = |name: &str| {
        name == route("Deno")
            || name.starts_with(brand_stem)
            || name.starts_with(&format!("_refs/{brand_stem}"))
            || name.starts_with(&format!("_graph/p/{}", &brand[2..18]))
            || name.starts_with("_graph/")
            || name.starts_with("_refs/")
            || name.starts_with("lsg-search-")
            || [
                "lsg-site.json",
                "_pages/list.json",
                "pages.html",
                "index.html",
            ]
            .contains(&name)
    };
    assert!(changed.iter().any(|name| **name == brand));
    let others: Vec<_> = changed.iter().filter(|name| !allowed(name)).collect();
    assert!(others.is_empty(), "{others:?}");
    // Every other page file stays the same, Deno Deploy's included.
    assert_eq!(old[&route("Deno Deploy")], new[&route("Deno Deploy")]);
    // Removing the last reference removes the page.
    f.write("pages/Deno.md", &deno);
    f.publish("removed");
    assert!(!f.0.join("removed").join(&brand).exists());
}
