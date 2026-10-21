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
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "lsg-embed-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(root.join("garden/pages")).unwrap();
        Self(root)
    }
    fn write(&self, file: &str, text: &str) {
        let path = self.0.join("garden").join(file);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }
    fn publish(&self) -> logseq_gardener_sdk::site::SiteReport {
        publish_site(
            self.0.join("garden"),
            self.0.join("site"),
            FilenameFormat::TripleLowbar,
            &NamespaceSelection::new(&["Notes"], &["Notes/Excluded"]).unwrap(),
        )
        .unwrap()
    }
    fn html(&self, title: &str) -> String {
        let hash = title
            .as_bytes()
            .iter()
            .fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
                (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
            });
        fs::read_to_string(self.0.join(format!("site/p-{hash:016x}.html"))).unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
#[test]
fn page_alias_and_repeated_block_embeds_render_subtrees_and_original_links() {
    let f = Fixture::new();
    f.write("pages/Notes___Start.md", "- {{embed [[Friendly]]}}\n- {{embed ((12345678-1234-1234-1234-123456789abc))}}\n- {{embed ((12345678-1234-1234-1234-123456789abc))}}\n- ((12345678-1234-1234-1234-123456789abc))\n");
    f.write("pages/Notes___Other.md", "alias:: Friendly\n- Parent **strong**\n  id:: 12345678-1234-1234-1234-123456789abc\n  - Child content\n- Sibling content\n");
    let report = f.publish();
    let html = f.html("Notes/Start");
    assert!(report.diagnostics.is_empty(), "{:?}", report.diagnostics);
    assert_eq!(html.matches("Parent <strong>strong</strong>").count(), 3);
    assert_eq!(html.matches("Child content").count(), 3);
    assert_eq!(html.matches("Sibling content").count(), 1);
    assert!(!html.contains("id=\"block-"));
    assert!(html.contains("#block-12345678-1234-1234-1234-123456789abc"));
    assert_eq!(
        f.html("Notes/Other")
            .matches("id=\"block-12345678-1234-1234-1234-123456789abc\"")
            .count(),
        1
    );
}
#[test]
fn embedded_assets_use_original_source_and_diagnostics_name_the_source() {
    let f = Fixture::new();
    f.write(
        "pages/Notes___Start.md",
        "- {{embed [[Notes/Nested/Other]]}}\n",
    );
    // A source in a subdirectory demonstrates that URLs use the embedded owner.
    f.write("pages/nested/Other.md", "title:: Notes/Nested/Other\n- ![Art](../../assets/art.png)\n- ![Nope](../../assets/missing.png)\n");
    f.write("assets/art.png", "IMAGE_BYTES");
    let report = f.publish();
    assert_eq!(report.assets, 1);
    assert!(f.html("Notes/Start").contains("<img src=\"assets/"));
    assert!(
        report
            .diagnostics
            .iter()
            .any(|d| d.message.contains("pages/nested/Other.md")),
        "{:?}",
        report.diagnostics
    );
}
#[test]
fn private_excluded_missing_and_ambiguous_targets_keep_fallbacks() {
    let f = Fixture::new();
    f.write("pages/Notes___Start.md", "- {{embed [[Notes/Private]]}}\n- {{embed [[Notes/Excluded]]}}\n- {{embed [[Absent]]}}\n- {{embed [[Shared]]}}\n- {{embed ((12345678-1234-1234-1234-123456789abc))}}\n");
    f.write(
        "pages/Notes___Private.md",
        "public:: false\n- PRIVATE_SENTINEL https://private.example/audio.mp3\n",
    );
    f.write("pages/Notes___Excluded.md", "- EXCLUDED_SENTINEL\n");
    for name in ["A", "B"] {
        f.write(
            &format!("pages/Notes___{name}.md"),
            "alias:: Shared\n- AMBIGUOUS_CONTENT\n  id:: 12345678-1234-1234-1234-123456789abc\n",
        );
    }
    let report = f.publish();
    let html = f.html("Notes/Start");
    for sentinel in [
        "PRIVATE_SENTINEL",
        "private.example",
        "EXCLUDED_SENTINEL",
        "AMBIGUOUS_CONTENT",
    ] {
        assert!(!html.contains(sentinel));
    }
    assert_eq!(html.matches("{{embed").count(), 5);
    assert!(report.diagnostics.len() >= 5);
}
#[test]
fn cycles_and_expansion_budgets_terminate_with_diagnostics() {
    let f = Fixture::new();
    f.write("pages/Notes___Start.md", "- {{embed [[Notes/Cycle]]}}\n");
    f.write(
        "pages/Notes___Cycle.md",
        "- Retained content\n- {{embed [[Notes/Start]]}}\n",
    );
    f.write(
        "pages/Notes___Repeated.md",
        &"- {{embed [[Notes/Leaf]]}}\n".repeat(140),
    );
    f.write("pages/Notes___Leaf.md", "- Leaf body\n");
    let report = f.publish();
    assert!(f.html("Notes/Start").contains("Retained content"));
    assert_eq!(f.html("Notes/Repeated").matches("Leaf body").count(), 128);
    assert!(
        report
            .diagnostics
            .iter()
            .any(|d| d.message.contains("cycle or depth"))
    );
    assert!(
        report
            .diagnostics
            .iter()
            .any(|d| d.message.contains("expansion budget"))
    );
}

#[test]
fn paragraph_and_heading_embeds_accept_flow_content() {
    let f = Fixture::new();
    f.write("pages/Notes___Start.md", "Before {{embed [[Notes/Leaf]]}} after\n\n# {{embed [[Notes/Leaf]]}}\n\n- **{{embed [[Notes/Leaf]]}}**\n");
    f.write("pages/Notes___Leaf.md", "- Body\n");
    f.publish();
    let html = f.html("Notes/Start");
    assert_eq!(
        html.matches("<section class=\"transclusion\">").count(),
        2,
        "{html}"
    );
    assert!(!html.contains("<p>Before"));
    assert!(!html.contains("<h1><section"));
    assert!(!html.contains("<strong><section"));
}
#[test]
fn total_embedded_node_work_is_bounded() {
    let f = Fixture::new();
    f.write("pages/Notes___Start.md", "- {{embed [[Notes/Large]]}}\n");
    f.write("pages/Notes___Large.md", &"- Embedded body\n".repeat(6_000));
    let report = f.publish();
    assert!(
        report
            .diagnostics
            .iter()
            .any(|d| d.message.contains("total-work budget"))
    );
    assert!(f.html("Notes/Start").matches("Embedded body").count() < 6_000);
}
