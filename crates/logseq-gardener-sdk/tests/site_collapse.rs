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
            "lsg-collapse-{}-{}",
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
    fn route(title: &str) -> String {
        let hash = title
            .as_bytes()
            .iter()
            .fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
                (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
            });
        format!("p-{hash:016x}.html")
    }
    fn html(&self, title: &str) -> String {
        fs::read_to_string(self.0.join("site").join(Self::route(title))).unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
const DISCLOSURE: &str = "<details class=\"outline-disclosure\">";
const BULLET: &str = "<a class=\"bullet\" href=\"#\"></a>";

#[test]
fn nested_closed_outlines_keep_skipped_levels_and_following_siblings_separate() {
    let f = Fixture::new();
    f.write("pages/Notes___Start.md", "- Dialogue\n  collapsed:: true\n      - Outer child\n        collapsed:: true\n            - Inner transcript\n      - Outer sibling\n- Following sibling\n");
    f.publish();
    let html = f.html("Notes/Start").replace(BULLET, "");
    assert!(html.contains(&format!("{DISCLOSURE}<summary>Dialogue</summary><ul><li class=\"outline-collapsed\">{DISCLOSURE}<summary>Outer child</summary><ul><li>Inner transcript</li></ul></details></li><li>Outer sibling</li></ul></details></li><li>Following sibling")), "{html}");
    assert_eq!(html.matches(DISCLOSURE).count(), 2);
    assert!(!html.contains("outline-disclosure\" open"));
    assert!(html.contains("<details class=\"references-section\" open><summary><span class=\"ref-chevron\"></span><h2>Hierarchy</h2></summary>"));
}

#[test]
fn false_absent_leaf_page_properties_and_literal_examples_do_not_collapse() {
    let f = Fixture::new();
    f.write("pages/Notes___Start.md", "collapsed:: true\n- False parent\n  collapsed:: false\n  - Expanded child\n- Absent parent\n  - Also expanded\n- Leaf\n  collapsed:: true\n- `collapsed:: true`\n  - Literal child\n- Code example\n  ~~~text\n  collapsed:: true\n  ~~~\n  - Code sibling\n");
    f.publish();
    let html = f.html("Notes/Start");
    assert!(!html.contains(DISCLOSURE), "{html}");
    for text in [
        "Expanded child",
        "Also expanded",
        "Leaf",
        "Literal child",
        "Code sibling",
        "collapsed:: true",
    ] {
        assert!(html.contains(text), "{html}");
    }
}

#[test]
fn rich_summary_and_descendant_media_keep_safe_content_and_assets() {
    let f = Fixture::new();
    f.write("pages/Notes___Start.md", "- ### **Dialogue** [Reference](https://example.com/?q=a&b=c) <unsafe>\n  collapsed:: true\n  - ![Art](../assets/art.png)\n  - ![Sound](https://example.com/full.mp3)\n  - {{embed [[Notes/Other]]}}\n");
    f.write(
        "pages/nested/Other.md",
        "title:: Notes/Other\n- Embedded parent\n  collapsed:: true\n  - Embedded child ![Embedded art](../../assets/embedded.png)\n",
    );
    f.write("assets/art.png", "IMAGE_BYTES");
    f.write("assets/embedded.png", "EMBEDDED_IMAGE_BYTES");
    let report = f.publish();
    let html = f.html("Notes/Start");
    assert_eq!(report.assets, 2);
    assert_eq!(html.matches("<img src=\"assets/").count(), 2);
    let summary = html
        .split_once("<summary>")
        .unwrap()
        .1
        .split_once("</summary>")
        .unwrap()
        .0;
    assert!(summary.contains("<h3><strong>Dialogue</strong> <a class=\"external-link\" target=\"_blank\" rel=\"noopener noreferrer\" href=\"https://example.com/?q=a&amp;b=c\">Reference</a>"),"{html}");
    assert!(!summary.contains("<unsafe>"));
    assert!(html.contains("<img src=\"assets/"));
    assert!(html.contains("<audio controls preload=\"none\""));
    assert!(
        html.contains(&format!(
            "<div class=\"blocks-container\"><ul><li class=\"outline-collapsed\">{DISCLOSURE}<summary><a class=\"bullet\" href=\"#\"></a>Embedded parent</summary>"
        )),
        "{html}"
    );
}

#[test]
fn flow_embed_parent_retains_content_outside_phrasing_summary() {
    let f = Fixture::new();
    f.write(
        "pages/Notes___Start.md",
        "- Before [unsupported content] {{embed [[Notes/Other]]}} after\n  collapsed:: true\n  - Own child\n",
    );
    f.write("pages/Notes___Other.md", "- Embedded content\n");
    f.publish();
    let html = f.html("Notes/Start");
    let summary = html
        .split_once("<summary>")
        .unwrap()
        .1
        .split_once("</summary>")
        .unwrap()
        .0;
    assert!(summary.contains("Before") && summary.contains("after"));
    assert!(!summary.contains("<section") && !summary.contains("<ul"));
    assert!(summary.contains("[unsupported content] [embedded content]"));
    assert!(
        html.contains(
            "</summary>Before [unsupported content] <div class=\"embed-page\" data-level=\"1\">"
        ),
        "{html}"
    );
    assert!(html.contains("Embedded content") && html.contains("Own child"));
    assert!(!html.contains("<strong>Before <div"));
}

#[test]
fn original_descendant_anchors_survive_collapse_and_repeated_block_embeds() {
    let f = Fixture::new();
    let id = "12345678-1234-1234-1234-123456789abc";
    f.write("pages/Notes___Start.md", &format!("- Closed\n  collapsed:: true\n  - Destination\n    id:: {id}\n    collapsed:: true\n    - Deep content\n- (({id}))\n- {{{{embed (({id}))}}}}\n- {{{{embed (({id}))}}}}\n"));
    f.publish();
    let html = f.html("Notes/Start");
    assert_eq!(html.matches(&format!("id=\"block-{id}\"")).count(), 1);
    assert!(html.find(DISCLOSURE).unwrap() < html.find(&format!("id=\"block-{id}\"")).unwrap());
    assert!(html.contains(&format!("#block-{id}")));
    assert_eq!(html.matches("Deep content").count(), 3);
    assert_eq!(html.matches(DISCLOSURE).count(), 4);
    assert!(html.contains("window.logseqGardenerRevealFragment"));
}

#[test]
fn private_descendant_withholds_closed_page_and_assets_before_expansion() {
    let f = Fixture::new();
    f.write("pages/Notes___Start.md", "- {{embed [[Notes/Private]]}}\n");
    f.write("pages/Notes___Private.md", "- Closed\n  collapsed:: true\n  - PRIVATE_SENTINEL\n    public:: false\n    - ![secret](../assets/secret.png)\n");
    f.write("assets/secret.png", "SECRET_BYTES");
    let report = f.publish();
    assert_eq!(report.pages, 1);
    assert_eq!(report.assets, 0);
    assert!(!f.html("Notes/Start").contains("PRIVATE_SENTINEL"));
    assert!(
        !f.0.join("site")
            .join(Fixture::route("Notes/Private"))
            .exists()
    );
}

#[test]
fn outline_blocks_break_long_words_to_fit_a_phone_screen() {
    let f = Fixture::new();
    f.write(
        "pages/Notes___Start.md",
        "- Parent\n\t- [[Ableton/Live/Mode/Arrangement]]\n",
    );
    f.publish();
    let page = f.html("Notes/Start");
    assert!(page.contains("<link rel=\"stylesheet\" href=\"lsg-site.css\">"));
    let styles = fs::read_to_string(f.0.join("site/lsg-site.css")).unwrap();
    let style = &styles[styles.find(".outline li{").unwrap()..];
    let rule = &style[..style.find('}').unwrap()];
    assert!(rule.contains("overflow-wrap:anywhere"), "{rule}");
}

/// The class Logseq's rule gives the item of a block whose bullet it hides.
const EMPTY: &str = "<li class=\"lsg-empty\">";

#[test]
fn a_block_with_no_text_hides_its_bullet_as_logseq_does() {
    let f = Fixture::new();
    // The page's first and last blocks keep their bullets, as do a block with children
    // and a block with a property Logseq shows. A nested block keeps no such exception.
    f.write(
        "pages/Notes___Start.md",
        "- first\n- # Frontmatter and body\n\t- id:: 6add3ffb-e52b-4e8f-908a-9958186e4dfb\n\t- text\n\t-\n\t- kind:: note\n-\n\t- child\n- \n- ---\n- last\n",
    );
    f.write("pages/Notes___Ends.md", "-\n- middle\n-\n");
    f.publish();
    let html = f.html("Notes/Start");
    assert_eq!(html.matches(EMPTY).count(), 4, "{html}");
    let outline = html.split_once("<ul class=\"outline\">").unwrap().1;
    assert!(
        outline.starts_with(&format!("<li>{BULLET}first</li>")),
        "{html}"
    );
    assert!(outline.contains(&format!("<ul>{EMPTY}<span id=\"block-6add3ffb-e52b-4e8f-908a-9958186e4dfb\"></span>{BULLET}</li><li>{BULLET}text</li>{EMPTY}{BULLET}</li><li>{BULLET}</li></ul>")), "{html}");
    assert!(outline.contains(&format!("<li>{BULLET}<ul><li>{BULLET}child</li></ul></li>{EMPTY}{BULLET}</li>{EMPTY}{BULLET}<hr></li><li>{BULLET}last</li></ul>")), "{html}");
    let ends = f.html("Notes/Ends");
    assert!(!ends.contains(EMPTY), "{ends}");
}

#[test]
fn an_embedded_block_with_no_text_hides_its_bullet() {
    let f = Fixture::new();
    f.write("pages/Notes___Host.md", "- {{embed [[Notes/Embedded]]}}\n");
    f.write("pages/Notes___Embedded.md", "tags:: x\n\n-\n- a\n-\n- b\n");
    f.publish();
    let host = f.html("Notes/Host");
    let embed = host
        .split_once("<div class=\"blocks-container\">")
        .unwrap()
        .1;
    assert!(
        embed.starts_with(&format!(
            "<ul>{EMPTY}{BULLET}</li><li>{BULLET}a</li>{EMPTY}{BULLET}</li><li>{BULLET}b</li></ul>"
        )),
        "{host}"
    );
}
