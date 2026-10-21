use logseq_gardener_sdk::{
    page_titles::FilenameFormat,
    publishing::NamespaceSelection,
    site::{SiteOptions, publish_site_with_options},
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
        let path = std::env::temp_dir().join(format!(
            "lsg-homepage-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(path.join("garden/pages")).unwrap();
        fs::create_dir_all(path.join("garden/assets")).unwrap();
        fs::write(
            path.join("garden/pages/Home.md"),
            "alias:: Welcome\n- HOMEPAGE_CONTENT [[Notes]]\n- ![art](../assets/art.png)\n",
        )
        .unwrap();
        fs::write(path.join("garden/pages/Notes.md"), "- OTHER_CONTENT\n").unwrap();
        fs::write(
            path.join("garden/pages/Hidden.md"),
            "public:: false\n- HIDDEN_SENTINEL\n",
        )
        .unwrap();
        fs::write(path.join("garden/assets/art.png"), "ART_CONTENT").unwrap();
        Self(path)
    }
    fn publish(
        &self,
        homepage: Option<&str>,
    ) -> Result<logseq_gardener_sdk::site::SiteReport, logseq_gardener_sdk::site::SiteError> {
        publish_site_with_options(
            self.0.join("garden"),
            self.0.join("output"),
            FilenameFormat::TripleLowbar,
            &NamespaceSelection::new(&["Home", "Notes", "Hidden"], &[]).unwrap(),
            &SiteOptions {
                homepage: homepage.map(str::to_owned),
                ..SiteOptions::default()
            },
        )
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
#[test]
fn homepage_title_and_alias_keep_content_links_media_and_original_route() {
    for selector in ["Home", " welcome "] {
        let f = Fixture::new();
        let report = f.publish(Some(selector)).unwrap();
        assert_eq!(report.pages, 2);
        assert_eq!(report.assets, 1);
        let html = fs::read_to_string(f.0.join("output/index.html")).unwrap();
        assert!(
            html.contains("HOMEPAGE_CONTENT")
                && html.contains("<img")
                && html.contains("pages.html")
        );
        assert!(!html.contains("HIDDEN_SENTINEL"));
        let index = fs::read_to_string(f.0.join("output/pages.html")).unwrap();
        assert!(index.contains(">Home</a>") && index.contains(">Notes</a>"));
        let originals: Vec<_> = fs::read_dir(f.0.join("output"))
            .unwrap()
            .flatten()
            .filter(|entry| entry.file_name().to_string_lossy().starts_with("p-"))
            .collect();
        assert_eq!(originals.len(), 2);
        assert!(
            originals
                .iter()
                .any(|entry| fs::read_to_string(entry.path()).unwrap() == html)
        );
        assert!(
            fs::read_to_string(f.0.join("garden/pages/Home.md"))
                .unwrap()
                .contains("alias:: Welcome")
        );
    }
}
#[test]
fn unavailable_private_excluded_or_ambiguous_homepage_fails_before_output() {
    for selector in ["Missing", "Hidden", "NotSelected", "Shared"] {
        let f = Fixture::new();
        fs::write(
            f.0.join("garden/pages/NotSelected.md"),
            "- EXCLUDED_SENTINEL\n",
        )
        .unwrap();
        fs::write(f.0.join("garden/pages/Home.md"), "alias:: Shared\n- home\n").unwrap();
        fs::write(
            f.0.join("garden/pages/Notes.md"),
            "alias:: Shared\n- notes\n",
        )
        .unwrap();
        let error = f.publish(Some(selector)).unwrap_err();
        assert!(error.to_string().contains("one selected visible page"));
        assert!(!f.0.join("output").exists());
    }
}
#[test]
fn omitted_homepage_retains_index_behavior() {
    let f = Fixture::new();
    f.publish(None).unwrap();
    let index = fs::read_to_string(f.0.join("output/index.html")).unwrap();
    assert!(index.contains("Garden index") && index.contains(">Home</a>"));
    assert!(!index.contains("HOMEPAGE_CONTENT"));
    assert!(!f.0.join("output/pages.html").exists());
}
