use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

use logseq_gardener_sdk::{
    page_titles::FilenameFormat,
    publishing::NamespaceSelection,
    site::{SiteOptions, publish_site_with_options},
};

mod support;
static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "lsg-page-nav-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(path.join("garden/pages")).unwrap();
        Self(path)
    }
    fn page(&self, file: &str, body: &str) {
        fs::write(self.0.join("garden/pages").join(file), body).unwrap();
    }
    fn publish(&self, homepage: Option<&str>) -> usize {
        publish_site_with_options(
            self.0.join("garden"),
            self.0.join("site"),
            FilenameFormat::TripleLowbar,
            &NamespaceSelection::new(&["Visible"], &["Visible/Excluded"]).unwrap(),
            &SiteOptions {
                homepage: homepage.map(str::to_owned),
                ..SiteOptions::default()
            },
        )
        .unwrap()
        .pages
    }
    fn html(&self, title: &str) -> String {
        fs::read_to_string(self.0.join("site").join(route(title))).unwrap()
    }
    /// The reference fragment files the page loads when a reader opens a section.
    fn fragments(&self, title: &str) -> String {
        let stem = format!("{}-", route(title).trim_end_matches(".html"));
        let mut paths: Vec<_> = fs::read_dir(self.0.join("site/_refs"))
            .unwrap()
            .flatten()
            .map(|entry| entry.path())
            .filter(|path| {
                path.file_name()
                    .unwrap()
                    .to_string_lossy()
                    .starts_with(&stem)
            })
            .collect();
        paths.sort();
        paths
            .into_iter()
            .map(|path| support::read_text(&path))
            .collect()
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
            (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
        });
    format!("p-{hash:016x}.html")
}
fn nav(html: &str) -> &str {
    html.split_once("<nav class=\"page-sequence\"")
        .map(|(_, rest)| rest.split_once("</nav>").unwrap().0)
        .unwrap_or("")
}

#[test]
fn explicit_sequence_resolves_aliases_to_canonical_titles_and_homepage_routes() {
    let f = Fixture::new();
    f.page(
        "Visible___Start.md",
        "alias:: [[Start alias]]\n- First page\n",
    );
    f.page("Visible___Middle.md", "PrEv:: [[START ALIAS]]\nnext:: [[Visible/End & <last>]]\n- Middle body\n- ![recording](https://example.test/full.mp3)\n");
    f.page(
        "Visible___End.md",
        "title:: Visible/End & <last>\nprev:: [[Visible/Middle]]\n- Last page\n",
    );
    let source = fs::read(f.0.join("garden/pages/Visible___Middle.md")).unwrap();
    assert_eq!(f.publish(Some("Visible/Middle")), 3);
    assert_eq!(
        fs::read(f.0.join("garden/pages/Visible___Middle.md")).unwrap(),
        source
    );
    let middle = f.html("Visible/Middle");
    let sequence = nav(&middle);
    assert!(sequence.contains(&format!(
        "rel=\"prev\" href=\"{}\">Previous: Visible/Start",
        route("Visible/Start")
    )));
    assert!(sequence.contains(&format!(
        "rel=\"next\" href=\"{}\">Next: Visible/End &amp; &lt;last&gt;",
        route("Visible/End & <last>")
    )));
    assert!(middle.find("Middle body").unwrap() < middle.find("class=\"page-sequence\"").unwrap());
    assert!(
        middle.find("class=\"page-sequence\"").unwrap()
            < middle.find("<section class=\"page-hierarchy ").unwrap()
    );
    assert!(middle.contains("<audio controls"));
    // Logseq's order: Linked References, Hierarchy, Unlinked References.
    let linked = middle.find("data-refs=\"linked\"").unwrap();
    let hierarchy = middle.find("<section class=\"page-hierarchy ").unwrap();
    assert!(linked < hierarchy);
    assert!(hierarchy < middle.find("data-refs=\"unlinked\"").unwrap());
    assert!(f.fragments("Visible/Middle").contains("Page reference"));
    let home = fs::read_to_string(f.0.join("site/index.html")).unwrap();
    assert_eq!(
        &home[home.find("<main>").unwrap()..],
        &middle[middle.find("<main>").unwrap()..]
    );
    assert!(!nav(&f.html("Visible/End & <last>")).contains("rel=\"next\""));
    assert!(nav(&f.html("Visible/Start")).is_empty());
}

#[test]
fn unsafe_ambiguous_self_and_generated_sequence_targets_have_no_links() {
    let f = Fixture::new();
    f.page("Visible___Private.md", "public:: false\n- Confidential\n");
    f.page("Visible___Excluded.md", "- Excluded content\n");
    f.page("Visible___A.md", "alias:: [[Shared]]\n- A\n");
    f.page("Visible___B.md", "alias:: [[Shared]]\n- B\n");
    f.page("Visible___Generated___Leaf.md", "- Leaf\n");
    for (name, target) in [
        ("PrivateRef", "Visible/Private"),
        ("ExcludedRef", "Visible/Excluded"),
        ("Ambiguous", "Shared"),
        ("Missing", "Missing"),
        ("Synthetic", "Visible/Generated"),
        ("Self", "Visible/Self"),
    ] {
        f.page(
            &format!("Visible___{name}.md"),
            &format!("prev:: [[{target}]]\nnext:: [[{target}]]\n- Visible body\n"),
        );
    }
    f.publish(None);
    for name in [
        "PrivateRef",
        "ExcludedRef",
        "Ambiguous",
        "Missing",
        "Synthetic",
        "Self",
    ] {
        assert!(
            nav(&f.html(&format!("Visible/{name}"))).is_empty(),
            "{name}"
        );
    }
    assert!(!f.0.join("site").join(route("Visible/Private")).exists());
}

#[test]
fn only_last_leading_direct_property_value_supplies_a_neighbor() {
    let f = Fixture::new();
    f.page("Visible___Neighbor.md", "- Neighbor\n");
    f.page("Visible___Repeat.md", "prev:: [[Missing]]\nPrEv:: [[Visible/Neighbor]]\nnext:: [[Visible/Neighbor]]\nNEXT:: `[[Visible/Neighbor]]`\n- Body\n");
    f.page(
        "Visible___Nested.md",
        "- Parent\n  prev:: [[Visible/Neighbor]]\n- ~~~\n  next:: [[Visible/Neighbor]]\n  ~~~\n",
    );
    f.page("Visible___Malformed.md", "prev:: [[Visible/Neighbor]], [[Visible/Repeat]]\nnext:: [Neighbor]([[Visible/Neighbor]])\n- Body\n");
    f.publish(None);
    let repeated = f.html("Visible/Repeat");
    assert!(nav(&repeated).contains("rel=\"prev\""));
    assert!(!nav(&repeated).contains("rel=\"next\""));
    assert!(nav(&f.html("Visible/Nested")).is_empty());
    assert!(nav(&f.html("Visible/Malformed")).is_empty());
}

#[test]
fn namespace_status_preserves_index_names_and_selected_empty_pages() {
    let f = Fixture::new();
    f.page("Visible___Branch___Leaf.md", "- Leaf content\n");
    f.page("Visible___Empty.md", "");
    assert_eq!(f.publish(None), 2);
    let generated = f.html("Visible/Branch");
    assert!(generated.contains("<p class=\"namespace-notice\">Namespace"));
    assert!(
        generated.contains("class=\"namespace-link\" aria-label=\"Visible/Branch (Namespace)\"")
    );
    assert!(
        fs::read_to_string(f.0.join("site/lsg-site.css"))
            .unwrap()
            .contains("text-decoration-style:dotted")
    );
    assert!(
        !generated.contains("data-refs=\"linked\""),
        "no references, no section"
    );
    assert!(generated.contains("data-refs=\"unlinked\""));
    assert!(
        !f.html("Visible/Empty")
            .contains("<p class=\"namespace-notice\"")
    );
    let index = fs::read_to_string(f.0.join("site/index.html")).unwrap();
    // All pages rows keep the generated link style and name, without the badge.
    assert!(index.contains(&format!(
        "<a class=\"page-ref namespace-link\" aria-label=\"Visible/Branch (Namespace)\" href=\"{}\">Visible/Branch</a></td>",
        route("Visible/Branch")
    )));
    assert!(!index.contains("<span class=\"page-kind\">"));
    assert!(index.contains(&format!(
        "<a class=\"page-ref\" href=\"{}\">Visible/Empty</a></td>",
        route("Visible/Empty")
    )));
}

#[test]
fn namespace_status_does_not_disclose_unselected_source_existence() {
    let missing = Fixture::new();
    let stored = Fixture::new();
    for f in [&missing, &stored] {
        f.page("Visible___Branch___Leaf.md", "- Visible leaf\n");
    }
    stored.page("Visible___Branch.md", "- Secret unselected content\n");
    for f in [&missing, &stored] {
        publish_site_with_options(
            f.0.join("garden"),
            f.0.join("site"),
            FilenameFormat::TripleLowbar,
            &NamespaceSelection::new(&["Visible/Branch/Leaf"], &[]).unwrap(),
            &SiteOptions::default(),
        )
        .unwrap();
    }
    assert_eq!(
        missing.html("Visible/Branch"),
        stored.html("Visible/Branch")
    );
    assert!(
        !stored
            .html("Visible/Branch")
            .contains("Secret unselected content")
    );
}
