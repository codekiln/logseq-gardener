use logseq_gardener_sdk::{
    page_titles::FilenameFormat, publishing::NamespaceSelection, site::publish_site,
};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};
static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new(config: &str) -> Self {
        let p = std::env::temp_dir().join(format!(
            "lsg-properties-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(p.join("garden/pages")).unwrap();
        fs::create_dir_all(p.join("garden/logseq")).unwrap();
        fs::write(p.join("garden/logseq/config.edn"), config).unwrap();
        Self(p)
    }
    fn page(&self, name: &str, text: &str) {
        fs::write(
            self.0
                .join("garden/pages")
                .join(format!("{}.md", name.replace('/', "___"))),
            text,
        )
        .unwrap();
    }
    fn publish(&self) {
        publish_site(
            self.0.join("garden"),
            self.0.join("site"),
            FilenameFormat::TripleLowbar,
            &NamespaceSelection::new(&["Visible"], &["Visible/Excluded"]).unwrap(),
        )
        .unwrap();
    }
    fn html(&self, name: &str) -> String {
        fs::read_to_string(self.0.join("site").join(route(name))).unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
fn route(name: &str) -> String {
    format!(
        "p-{:016x}.html",
        name.as_bytes()
            .iter()
            .fold(0xcbf29ce484222325_u64, |h, b| (h ^ u64::from(*b))
                .wrapping_mul(0x100000001b3))
    )
}
fn properties(html: &str) -> &str {
    html.split_once("<section class=\"page-properties\"")
        .map(|(_, s)| s.split_once("</section>").unwrap().0)
        .unwrap_or("")
}
/// The Linked References header and the fragment files it loads.
fn backlinks(f: &Fixture, title: &str) -> String {
    let html = f.html(title);
    let header = html
        .split_once("data-refs=\"linked\"")
        .unwrap()
        .1
        .split_once("</section>")
        .unwrap()
        .0;
    let stem = format!("{}-linked-", route(title).trim_end_matches(".html"));
    let mut paths: Vec<_> = std::fs::read_dir(f.0.join("site/_refs"))
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
    header.to_owned()
        + &paths
            .into_iter()
            .map(|path| std::fs::read_to_string(path).unwrap())
            .collect::<String>()
}

#[test]
fn book_and_preset_properties_keep_order_scalars_aliases_and_avoid_duplicate_sequences() {
    let f = Fixture::new("{:publishing/page-properties? true}");
    f.page("Visible/Book/One", "alias:: [[First chapter]]\n- One\n");
    f.page("Visible/Book/Two","rating:: 4\nfinished:: false\nprev:: [[First chapter]]\nlabel:: A & B < C\ndownload:: https://media.example/preset.mbp?x=1&y=2\nnext:: [[Visible/Preset/016]]\n- Two\n");
    f.page(
        "Visible/Preset/016",
        "number:: 016\nprev:: [[Visible/Book/Two]]\n- Preset\n",
    );
    let source = fs::read(f.0.join("garden/pages/Visible___Book___Two.md")).unwrap();
    f.publish();
    let html = f.html("Visible/Book/Two");
    let props = properties(&html);
    assert!(props.contains("<summary id=\"page-properties\" tabindex=\"0\">Properties</summary>"));
    assert!(props.contains("<dt>rating</dt><dd>4</dd>"));
    assert!(props.contains("<dt>finished</dt><dd>false</dd>"));
    assert!(props.contains("A &amp; B &lt; C"));
    assert!(props.contains("href=\"https://media.example/preset.mbp?x=1&amp;y=2\""));
    assert!(props.contains("<a class=\"external-link\" target=\"_blank\" rel=\"noopener noreferrer\" href=\"https://media.example/preset.mbp"));
    assert!(props.contains(&format!(
        "href=\"{}\">Visible/Book/One",
        route("Visible/Book/One")
    )));
    assert!(props.find("<dt>rating").unwrap() < props.find("<dt>prev").unwrap());
    assert!(!html.contains("class=\"page-sequence\""));
    assert!(
        !f.html("Visible/Preset/016")
            .contains("class=\"page-sequence\"")
    );
    assert!(properties(&f.html("Visible/Book")).is_empty());
    assert_eq!(
        fs::read(f.0.join("garden/pages/Visible___Book___Two.md")).unwrap(),
        source
    );
}

#[test]
fn hidden_mixed_and_unsafe_values_supply_no_rows_or_property_backlinks() {
    let f = Fixture::new(
        "{:publishing/page-properties? true :block-hidden-properties #{:secret :next}}",
    );
    f.page("Visible/Target", "- Target\n");
    f.page("Visible/Private", "public:: false\n- Hidden\n");
    f.page("Visible/A", "alias:: [[Collision]]\n- A\n");
    f.page("Visible/B", "alias:: [[Collision]]\n- B\n");
    f.page("Visible/Source","mixed:: [[Visible/Target]], [[Visible/Private]]\nambiguous:: [[Collision]]\nsecret:: [[Visible/Target]]\nid:: [[Visible/Target]]\nlogseq-proxy-source:: [[Visible/Target]]\nnext:: [[Visible/Target]]\nunsafe:: {{embed [[Visible/Target]]}}\nasset:: ../assets/local.wav\nfile-path:: file:///sensitive\nblock-ref:: ((a5292345-1111-1111-1111-111111111111))\nlocal:: /Users/private/file\nwindows:: C:\\private\\file.wav\nunc:: \\\\server\\file.wav\nscript:: javascript:alert(1)\ndata:: data:text/plain,hidden\n- Public body\n");
    f.publish();
    let html = f.html("Visible/Source");
    assert!(properties(&html).is_empty());
    assert!(!html.contains("class=\"page-sequence\""));
    for text in [
        "Visible/Private",
        "Collision",
        "sensitive",
        "local.wav",
        "logseq-proxy-source",
        "Users/private",
        "javascript:",
        "data:text",
    ] {
        assert!(!html.contains(text), "{text}");
    }
    // Only withheld, hidden or unsafe values name the target, so it has no linked references.
    assert!(!f.html("Visible/Target").contains("data-refs=\"linked\""));
}

#[test]
fn property_backlinks_target_the_real_revealable_summary_and_resolve_bare_tags() {
    let f = Fixture::new(
        "{:publishing/page-properties? true :ref/linked-references-collapsed-threshold 0}",
    );
    f.page("Visible/Target", "- Target\n");
    f.page(
        "Visible/Source",
        "category:: [[Visible/Target]]\ntags:: Visible/Target\n- Source\n",
    );
    f.publish();
    let target = f.html("Visible/Target");
    assert!(backlinks(&f, "Visible/Target").contains(&format!(
        "href=\"{}#page-properties\">Page properties",
        route("Visible/Source")
    )));
    let source = f.html("Visible/Source");
    assert_eq!(source.matches("id=\"page-properties\"").count(), 1);
    assert!(properties(&source).contains("<dt>tags</dt><dd><a href="));
    assert!(target.contains("<h2>1 Linked Reference</h2>"));
}

#[test]
fn defaults_keep_sequence_and_label_property_source_links_honestly() {
    let f = Fixture::new("{}");
    f.page("Visible/One", "next:: [[Visible/Two]]\n- One\n");
    f.page("Visible/Two", "- Two\n");
    f.publish();
    let one = f.html("Visible/One");
    assert!(properties(&one).is_empty());
    assert!(!one.contains("rel=\"lsg-properties\""));
    assert!(!one.contains("data-lsg-properties-start"));
    assert!(f.html("Visible/One").contains("class=\"page-sequence\""));
    assert!(backlinks(&f, "Visible/Two").contains("Page reference"));
    assert!(!backlinks(&f, "Visible/Two").contains("#page-properties"));
}

#[test]
fn disclosure_sequence_overrides_last_values_and_missing_targets_work_generically() {
    let f = Fixture::new(
        "{:publishing/page-properties? true :publishing/page-properties-collapsed? false :publishing/page-sequence? false}",
    );
    f.page("Visible/One","custom:: old\ncustom:: newest\nprev:: [[Visible/Missing]]\nnext:: [[Visible/One]]\n- One\n");
    f.publish();
    let html = f.html("Visible/One");
    let props = properties(&html);
    assert!(props.contains("<details open>"));
    assert!(props.contains("newest"));
    assert!(!props.contains("old"));
    assert!(!props.contains("<dt>prev"));
    assert!(!props.contains("<dt>next"));
    assert!(!html.contains("class=\"page-sequence\""));
}

#[test]
fn properties_controller_is_available_on_empty_pages_and_restores_before_content() {
    let f = Fixture::new(
        "{:publishing/page-properties? true :publishing/page-properties-collapsed? false}",
    );
    f.page("Visible/Empty", "- Only content\n");
    f.page("Visible/Properties", "rating:: 4\n- Content\n");
    f.publish();
    for name in ["Visible/Empty", "Visible/Properties", "Visible"] {
        let html = f.html(name);
        let start = html.find("data-lsg-properties-start").unwrap();
        assert!(start < html.find("<body>").unwrap());
        let route = html
            .split("rel=\"lsg-properties\" href=\"")
            .nth(1)
            .unwrap()
            .split('"')
            .next()
            .unwrap();
        assert!(route.starts_with("lsg-properties-"));
        assert!(route.ends_with(".mjs"));
        assert!(f.0.join("site").join(route).is_file());
    }
    let html = f.html("Visible/Properties");
    let props = properties(&html);
    assert!(
        props
            .find("window.logseqGardenerInitProperties?.()")
            .unwrap()
            < props.find("<summary").unwrap()
    );
    assert!(props.contains("<details open>"));
}
