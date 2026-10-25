use logseq_gardener_sdk::{
    page_titles::FilenameFormat,
    publishing::NamespaceSelection,
    site::{SearchOptions, SiteOptions, publish_site_with_options},
};
use serde_json::Value;
use std::{
    collections::BTreeSet,
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};
static NEXT: AtomicUsize = AtomicUsize::new(0);
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "lsg-search-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(root.join("garden/pages")).unwrap();
        Self(root)
    }
    fn write(&self, name: &str, source: &str) {
        fs::write(self.0.join("garden/pages").join(name), source).unwrap();
    }
    fn publish(&self, directory: &str, search: Option<SearchOptions>) -> PathBuf {
        let output = self.0.join(directory);
        publish_site_with_options(
            self.0.join("garden"),
            &output,
            FilenameFormat::TripleLowbar,
            &NamespaceSelection::new(&["Public"], &["Public/Excluded"]).unwrap(),
            &SiteOptions {
                search,
                ..SiteOptions::default()
            },
        )
        .unwrap();
        output
    }
    fn corpus(output: &std::path::Path) -> (Value, String) {
        let manifest_path = fs::read_dir(output)
            .unwrap()
            .flatten()
            .find(|entry| {
                entry
                    .file_name()
                    .to_string_lossy()
                    .starts_with("lsg-search-manifest-")
            })
            .unwrap()
            .path();
        let manifest: Value =
            serde_json::from_str(&fs::read_to_string(manifest_path).unwrap()).unwrap();
        let raw =
            fs::read_to_string(output.join(manifest["corpus"]["href"].as_str().unwrap())).unwrap();
        assert_eq!(
            manifest["corpus"]["bytes"].as_u64().unwrap() as usize,
            raw.len()
        );
        let corpus: Value = serde_json::from_str(&raw).unwrap();
        assert_eq!(manifest["revision"], corpus["revision"]);
        (corpus, raw)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
#[test]
fn selected_originals_safe_projection_aliases_and_anchor_destinations() {
    let f = Fixture::new();
    let source = "alias:: Welcome, Shared\nsecret:: PROPERTY_SENTINEL\n- Parent **safe** [[Public/Target]] [[PrivateRefSentinel]] #PrivateTagSentinel\n  collapsed:: true\n\t- child needle `codeword`\n\t  id:: 12345678-1234-1234-1234-123456789abc\n- before {{embed [[Public/Target]]}} after\n- ((87654321-4321-4321-4321-cba987654321))\n- {{query PRIVATE_QUERY_SENTINEL}}\n- ![ASSET_LABEL_SENTINEL](../assets/ASSET_PATH_SENTINEL.png)\n- <b>INLINE_HTML_SENTINEL</b>\n- hidden placeholder\n  search-exclude:: true\n\t- EXCLUDED_SUBTREE_SENTINEL\n- visible sibling\n";
    f.write("Public.md", source);
    f.write(
        "Public___Target.md",
        "alias:: Shared\n- TARGET_ORIGINAL_SENTINEL\n  id:: 87654321-4321-4321-4321-cba987654321\n",
    );
    f.write("Public___Excluded.md", "- EXCLUDED_PAGE_SENTINEL\n");
    f.write(
        "Public___Private.md",
        "public:: false\n- PRIVATE_PAGE_SENTINEL\n",
    );
    f.write("PrivateRefSentinel.md", "- OUTSIDE_SELECTION_SENTINEL\n");
    let output = f.publish("site", Some(SearchOptions::default()));
    let (corpus, raw) = Fixture::corpus(&output);
    for sentinel in [
        "PROPERTY_SENTINEL",
        "PrivateRefSentinel",
        "PrivateTagSentinel",
        "PRIVATE_QUERY_SENTINEL",
        "ASSET_LABEL_SENTINEL",
        "ASSET_PATH_SENTINEL",
        "INLINE_HTML_SENTINEL",
        "hidden placeholder",
        "EXCLUDED_SUBTREE_SENTINEL",
        "EXCLUDED_PAGE_SENTINEL",
        "PRIVATE_PAGE_SENTINEL",
        "OUTSIDE_SELECTION_SENTINEL",
    ] {
        assert!(!raw.contains(sentinel), "leaked {sentinel}: {raw}");
    }
    assert_eq!(raw.matches("TARGET_ORIGINAL_SENTINEL").count(), 1);
    assert_eq!(corpus["pages"].as_array().unwrap().len(), 2);
    assert!(raw.contains("Welcome") && !raw.contains("Shared"));
    let records = corpus["content"].as_array().unwrap();
    assert!(records.iter().any(|r| r["text"] == "before after"));
    let child = records
        .iter()
        .find(|r| r["text"] == "child needle codeword")
        .unwrap();
    assert!(
        child["href"]
            .as_str()
            .unwrap()
            .ends_with("#block-12345678-1234-1234-1234-123456789abc")
    );
    assert_eq!(child["ancestors"][0], "Parent safe Public/Target");
    for record in records {
        let (route, fragment) = record["href"].as_str().unwrap().split_once('#').unwrap();
        let html = fs::read_to_string(output.join(route)).unwrap();
        assert!(
            html.contains(&format!("id=\"{fragment}\"")),
            "missing {fragment}"
        );
        assert!(html.contains("data-lsg-fragments"));
    }
    for page in corpus["pages"].as_array().unwrap() {
        let html = fs::read_to_string(output.join(page["href"].as_str().unwrap())).unwrap();
        assert!(
            html.contains("rel=\"lsg-search\"")
                && html.contains(corpus["revision"].as_str().unwrap())
        );
        assert!(!html.contains("src=\"lsg-search.mjs\""));
    }
    assert_eq!(
        fs::read_to_string(f.0.join("garden/pages/Public.md")).unwrap(),
        source
    );
    let repeated = f.publish("repeat", Some(SearchOptions::default()));
    assert_eq!(Fixture::corpus(&repeated).1, raw);
}
#[test]
fn body_exclusion_and_default_opt_out() {
    let f = Fixture::new();
    f.write("Public.md", "- BODY_SENTINEL\n");
    let output = f.publish("disabled", None);
    assert!(
        !fs::read_to_string(output.join("index.html"))
            .unwrap()
            .contains("rel=\"lsg-search\"")
    );
    let output = f.publish(
        "excluded",
        Some(SearchOptions {
            excluded_page_bodies: BTreeSet::from(["public".to_owned()]),
        }),
    );
    let (corpus, raw) = Fixture::corpus(&output);
    assert!(!raw.contains("BODY_SENTINEL"));
    assert_eq!(corpus["pages"].as_array().unwrap().len(), 1);
    assert_eq!(corpus["content"], serde_json::json!([]));
}
#[test]
fn duplicate_uuid_uses_offsets_and_large_original_outlines_have_real_destinations() {
    let f = Fixture::new();
    let mut source = "- first\n  id:: 12345678-1234-1234-1234-123456789abc\n- second\n  id:: 12345678-1234-1234-1234-123456789abc\n".to_owned();
    for n in 0..10_050 {
        source.push_str(&format!("- line {n}\n"));
    }
    f.write("Public.md", &source);
    let output = f.publish("site", Some(SearchOptions::default()));
    let (corpus, raw) = Fixture::corpus(&output);
    assert!(!raw.contains("#block-12345678"));
    assert!(raw.contains("line 10049"));
    let records = corpus["content"].as_array().unwrap();
    let html =
        fs::read_to_string(output.join(corpus["pages"][0]["href"].as_str().unwrap())).unwrap();
    let ids: BTreeSet<_> = html
        .split("id=\"")
        .skip(1)
        .filter_map(|part| part.split('"').next())
        .collect();
    for record in records {
        let fragment = record["href"].as_str().unwrap().split_once('#').unwrap().1;
        assert!(ids.contains(fragment));
    }
}
