use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use logseq_gardener_sdk::page_titles::FilenameFormat;
use logseq_gardener_sdk::publishing::NamespaceSelection;
use logseq_gardener_sdk::site::publish_site;

static NEXT_FIXTURE: AtomicUsize = AtomicUsize::new(0);

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "lsg-site-{}-{}",
            std::process::id(),
            NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }

    fn write(&self, relative: &str, contents: impl AsRef<[u8]>) -> PathBuf {
        let path = self.0.join(relative);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, contents).unwrap();
        path
    }

    fn garden(&self) -> PathBuf {
        self.0.join("garden")
    }

    fn output(&self) -> PathBuf {
        self.0.join("site")
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

fn files(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn visit(root: &Path, directory: &Path, result: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for entry in fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();
            let metadata = fs::symlink_metadata(&path).unwrap();
            if metadata.is_dir() {
                visit(root, &path, result);
            } else if metadata.is_file() {
                result.insert(
                    path.strip_prefix(root).unwrap().to_owned(),
                    fs::read(path).unwrap(),
                );
            }
        }
    }
    let mut result = BTreeMap::new();
    visit(root, root, &mut result);
    result
}

fn route(title: &str) -> String {
    let hash = title
        .as_bytes()
        .iter()
        .fold(0xcbf2_9ce4_8422_2325u64, |hash, byte| {
            (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
        });
    format!("p-{hash:016x}.html")
}

fn html(root: &Path, title: &str) -> String {
    fs::read_to_string(root.join(route(title))).unwrap()
}

#[test]
fn selects_display_titles_and_withholds_private_content_without_source_changes() {
    let fixture = Fixture::new();
    fixture.write(
        "garden/pages/renamed.md",
        concat!(
            "title:: Notes/Start\n",
            "- Read [[notes/other]] and [[Notes/Private/Hidden]].\n",
            "- {{embed [[Notes/Private/Hidden]]}}\n",
            "- ((12345678-1234-1234-1234-123456789abc))\n",
            "- ![image](../assets/shared.png)\n",
            "- ![same](../assets/shared.png)\n",
        ),
    );
    fixture.write("garden/pages/Notes___Other.md", "- Other visible page\n");
    fixture.write(
        "garden/pages/Notes___Private___Hidden.md",
        concat!(
            "public:: true\n",
            "- EXCLUDED_PAGE_SENTINEL\n",
            "  id:: 12345678-1234-1234-1234-123456789abc\n",
            "  - EXCLUDED_BLOCK_SENTINEL\n",
            "- ![secret](../assets/excluded.png)\n",
        ),
    );
    fixture.write(
        "garden/pages/Notes___Withheld.md",
        concat!(
            "- Otherwise visible\n",
            "  - WITHHELD_CHILD_SENTINEL\n",
            "    public:: false\n",
            "- ![private](../assets/private.png)\n",
        ),
    );
    fixture.write(
        "garden/pages/Notes___Literal.md",
        "```text\npublic:: false\n```\n",
    );
    fixture.write("garden/journals/2026_10_07.md", "- JOURNAL_SENTINEL\n");
    fixture.write("garden/assets/shared.png", "SELECTED_ASSET_CONTENT");
    fixture.write("garden/assets/excluded.png", "EXCLUDED_ASSET_SENTINEL");
    fixture.write("garden/assets/private.png", "WITHHELD_ASSET_SENTINEL");
    let before = files(&fixture.garden());
    let selection = NamespaceSelection::new(&["Notes"], &["Notes/Private"]).unwrap();
    let report = publish_site(
        fixture.garden(),
        fixture.output(),
        FilenameFormat::TripleLowbar,
        &selection,
    )
    .unwrap();
    assert_eq!(report.pages, 3);
    assert_eq!(report.assets, 1);
    assert_eq!(report.skipped_journals, 1);
    assert_eq!(report.withheld_pages, 1);
    assert_eq!(files(&fixture.garden()), before);
    let index = fs::read_to_string(fixture.output().join("index.html")).unwrap();
    for title in ["Notes/Start", "Notes/Other", "Notes/Literal"] {
        assert!(index.contains(title));
        assert!(index.contains(&route(title)));
    }
    assert!(!index.contains("Notes/Private/Hidden"));
    assert!(!index.contains("Notes/Withheld"));
    let start = html(&fixture.output(), "Notes/Start");
    assert!(start.contains(&format!("href=\"{}\"", route("Notes/Other"))));
    assert!(
        !fixture
            .output()
            .join(route("Notes/Private/Hidden"))
            .exists()
    );
    assert!(html(&fixture.output(), "Notes/Literal").contains("public:: false"));
    let output = files(&fixture.output());
    for contents in output.values() {
        let text = String::from_utf8_lossy(contents);
        for sentinel in [
            "EXCLUDED_PAGE_SENTINEL",
            "EXCLUDED_BLOCK_SENTINEL",
            "EXCLUDED_ASSET_SENTINEL",
            "WITHHELD_CHILD_SENTINEL",
            "WITHHELD_ASSET_SENTINEL",
            "JOURNAL_SENTINEL",
        ] {
            assert!(!text.contains(sentinel), "leaked {sentinel}");
        }
    }
    let asset_paths: Vec<_> = output
        .iter()
        .filter(|(_, content)| content.as_slice() == b"SELECTED_ASSET_CONTENT")
        .map(|(path, _)| path)
        .collect();
    assert_eq!(asset_paths.len(), 1);
    let asset_route = asset_paths[0].to_string_lossy();
    assert!(start.contains(&format!("src=\"{asset_route}\"")));
    assert!(
        report
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.source == Path::new("pages/renamed.md"))
    );
    for construct in ["page reference", "block reference", "macro"] {
        assert!(
            report.diagnostics.iter().any(|diagnostic| diagnostic.source
                == Path::new("pages/renamed.md")
                && diagnostic.message.to_lowercase().contains(construct)),
            "missing {construct} diagnostic: {:?}",
            report.diagnostics
        );
    }
}

#[test]
fn renders_readable_markup_and_escapes_source_html_and_unsafe_links() {
    let fixture = Fixture::new();
    fixture.write(
        "garden/pages/Notes___Markup.md",
        concat!(
            "# Heading\n\n",
            "- Parent **strong** and *emphasis* and `literal <tag>`\n",
            "  - Child item\n\n",
            "> A quoted paragraph\n\n",
            "| Name | Value |\n| --- | --- |\n| Cell | Content |\n\n",
            "```rust\nlet x = \"<unsafe>\";\n```\n\n",
            "<script>alert('SOURCE_HTML_SENTINEL')</script>\n\n",
            "- [danger](javascript:alert)\n",
            "- [safe](https://example.com/?a=1&b=2)\n",
            "- ![remote](http://example.com/remote.png)\n",
            "- {{query (page-property public true)}}\n",
        ),
    );
    let selection = NamespaceSelection::new(&["Notes"], &[]).unwrap();
    let report = publish_site(
        fixture.garden(),
        fixture.output(),
        FilenameFormat::TripleLowbar,
        &selection,
    )
    .unwrap();
    let page = html(&fixture.output(), "Notes/Markup");
    for tag in [
        "<h1",
        "<ul",
        "<li",
        "<strong",
        "<em",
        "<code",
        "<pre",
        "<blockquote",
        "<table",
    ] {
        assert!(page.contains(tag), "missing {tag}: {page}");
    }
    for content in [
        "Heading",
        "Child item",
        "Cell",
        "SOURCE_HTML_SENTINEL",
        "&lt;script&gt;",
        "&lt;unsafe&gt;",
        "literal &lt;tag&gt;",
    ] {
        assert!(page.contains(content), "missing {content}: {page}");
    }
    assert!(!page.contains("<script>"));
    assert!(!page.contains("href=\"javascript:"));
    assert!(!page.contains("src=\"http://example.com/remote.png\""));
    assert!(page.contains("href=\"https://example.com/?a=1&amp;b=2\""));
    assert!(report.diagnostics.len() >= 3, "{:?}", report.diagnostics);
    assert_eq!(report.assets, 0);
}

#[test]
fn email_fallback_preserves_the_address_and_reports_unsupported_syntax() {
    let fixture = Fixture::new();
    fixture.write("garden/pages/Notes___Email.md", "- <reader@example.com>\n");
    let selection = NamespaceSelection::new(&["Notes"], &[]).unwrap();
    let report = publish_site(
        fixture.garden(),
        fixture.output(),
        FilenameFormat::TripleLowbar,
        &selection,
    )
    .unwrap();
    let page = html(&fixture.output(), "Notes/Email");
    assert!(
        page.contains("reader@example.com"),
        "missing email address: {page}"
    );
    assert!(!page.contains("local_part"), "parser fields leaked: {page}");
    assert!(report.diagnostics.iter().any(|diagnostic| {
        diagnostic.source == Path::new("pages/Notes___Email.md")
            && diagnostic.message.contains("email syntax")
    }));
}

#[test]
fn refuses_existing_or_source_descendant_output_without_changing_files() {
    let fixture = Fixture::new();
    fixture.write("garden/pages/Notes.md", "- Input\n");
    fixture.write("site/keep.txt", "EXISTING_OUTPUT_SENTINEL");
    let selection = NamespaceSelection::new(&["Notes"], &[]).unwrap();
    let before = files(&fixture.0);
    assert!(
        publish_site(
            fixture.garden(),
            fixture.output(),
            FilenameFormat::TripleLowbar,
            &selection
        )
        .is_err()
    );
    assert_eq!(files(&fixture.0), before);
    let nested = fixture.garden().join("new-site");
    assert!(
        publish_site(
            fixture.garden(),
            &nested,
            FilenameFormat::TripleLowbar,
            &selection
        )
        .is_err()
    );
    assert!(!nested.exists());
    assert_eq!(files(&fixture.0), before);
    let absent_parent = fixture.0.join("absent/site");
    assert!(
        publish_site(
            fixture.garden(),
            &absent_parent,
            FilenameFormat::TripleLowbar,
            &selection
        )
        .is_err()
    );
    assert!(!absent_parent.exists());
}

#[test]
fn rejects_lowercase_title_ambiguity_before_creating_output() {
    let fixture = Fixture::new();
    fixture.write("garden/pages/a.md", "title:: Notes/Same\n- First\n");
    fixture.write("garden/pages/b.md", "title:: Notes/same\n- Second\n");
    let selection = NamespaceSelection::new(&["Notes"], &[]).unwrap();
    assert!(
        publish_site(
            fixture.garden(),
            fixture.output(),
            FilenameFormat::TripleLowbar,
            &selection
        )
        .is_err()
    );
    assert!(!fixture.output().exists());
}

#[test]
fn stable_page_routes_survive_selection_changes() {
    let fixture = Fixture::new();
    fixture.write("garden/pages/Notes___Start.md", "- Start\n");
    fixture.write("garden/pages/Notes___Other.md", "- Other\n");
    let all = NamespaceSelection::new(&["Notes"], &[]).unwrap();
    let single = NamespaceSelection::new(&["Notes/Start"], &[]).unwrap();
    let second = fixture.0.join("second-site");
    publish_site(
        fixture.garden(),
        fixture.output(),
        FilenameFormat::TripleLowbar,
        &all,
    )
    .unwrap();
    publish_site(
        fixture.garden(),
        &second,
        FilenameFormat::TripleLowbar,
        &single,
    )
    .unwrap();
    assert!(fixture.output().join(route("Notes/Start")).exists());
    assert!(second.join(route("Notes/Start")).exists());
    assert!(!second.join(route("Notes/Other")).exists());
}

#[test]
fn rejects_missing_traversal_and_unsupported_assets_with_diagnostics() {
    let fixture = Fixture::new();
    fixture.write(
        "garden/pages/Notes.md",
        concat!(
            "- ![outside](../../outside.png)\n",
            "- ![encoded](%2e%2e/%2e%2e/outside.png)\n",
            "- ![missing](../assets/missing.png)\n",
            "- ![suffix](../assets/valid.png?secret=1)\n",
            "- ![unsupported](../assets/source.html)\n",
        ),
    );
    fixture.write("outside.png", "OUTSIDE_ASSET_SENTINEL");
    fixture.write("garden/assets/valid.png", "SUFFIX_ASSET_SENTINEL");
    fixture.write("garden/assets/source.html", "UNSUPPORTED_ASSET_SENTINEL");
    let selection = NamespaceSelection::new(&["Notes"], &[]).unwrap();
    let report = publish_site(
        fixture.garden(),
        fixture.output(),
        FilenameFormat::TripleLowbar,
        &selection,
    )
    .unwrap();
    assert_eq!(report.assets, 0);
    assert!(report.diagnostics.len() >= 5, "{:?}", report.diagnostics);
    for contents in files(&fixture.output()).values() {
        let text = String::from_utf8_lossy(contents);
        for sentinel in [
            "OUTSIDE_ASSET_SENTINEL",
            "SUFFIX_ASSET_SENTINEL",
            "UNSUPPORTED_ASSET_SENTINEL",
        ] {
            assert!(!text.contains(sentinel));
        }
    }
}

#[cfg(unix)]
#[test]
fn rejects_asset_file_and_directory_symlinks() {
    use std::os::unix::fs::symlink;
    let fixture = Fixture::new();
    fixture.write(
        "garden/pages/Notes.md",
        "- ![file](../assets/link.png)\n- ![directory](../assets/linked/image.png)\n",
    );
    let outside = fixture.write("outside/image.png", "SYMLINK_ASSET_SENTINEL");
    fs::create_dir_all(fixture.garden().join("assets")).unwrap();
    symlink(&outside, fixture.garden().join("assets/link.png")).unwrap();
    symlink(
        outside.parent().unwrap(),
        fixture.garden().join("assets/linked"),
    )
    .unwrap();
    let selection = NamespaceSelection::new(&["Notes"], &[]).unwrap();
    let report = publish_site(
        fixture.garden(),
        fixture.output(),
        FilenameFormat::TripleLowbar,
        &selection,
    )
    .unwrap();
    assert_eq!(report.assets, 0);
    assert!(report.diagnostics.len() >= 2);
    assert!(
        files(&fixture.output())
            .values()
            .all(|contents| !String::from_utf8_lossy(contents).contains("SYMLINK_ASSET_SENTINEL"))
    );
}

#[test]
fn block_links_resolve_selected_nested_ids_without_exposing_other_targets() {
    let fixture = Fixture::new();
    let id = "67da7fc2-1c1f-46e9-84b0-dee824e45395";
    let duplicate = "12345678-1234-1234-1234-123456789abc";
    let excluded = "22345678-1234-1234-1234-123456789abc";
    let private = "32345678-1234-1234-1234-123456789abc";
    let root = "42345678-1234-1234-1234-123456789abc";
    fixture.write("garden/pages/Notes___Start.md", format!(
        "- Reference (({}))\n- Duplicate (({duplicate}))\n- Hidden (({excluded})) (({private}))\n- Root (({root}))\n- Invalid ((not-a-uuid))\n", id.to_uppercase()));
    fixture.write("garden/pages/Notes___Target.md", format!(
        "id:: {root}\n- Parent\n  - Nested **target** & <unsafe> [[Notes/Other]]\n    id:: {id}\n  - Duplicate one\n    id:: {duplicate}\n- Malformed\n  id:: not-a-uuid\n"));
    fixture.write(
        "garden/pages/Notes___Duplicate.md",
        format!("- Duplicate two\n  id:: {}\n", duplicate.to_uppercase()),
    );
    fixture.write(
        "garden/pages/Notes___Excluded.md",
        format!("- EXCLUDED SECRET ![x](../assets/private.png)\n  id:: {excluded}\n"),
    );
    fixture.write(
        "garden/pages/Notes___Private.md",
        format!("public:: false\n- PRIVATE SECRET\n  id:: {private}\n"),
    );
    fixture.write("garden/assets/private.png", b"SECRET ASSET");
    let before = files(&fixture.garden());
    let selection = NamespaceSelection::new(&["Notes"], &["Notes/Excluded"]).unwrap();
    let report = publish_site(
        fixture.garden(),
        fixture.output(),
        FilenameFormat::TripleLowbar,
        &selection,
    )
    .unwrap();
    let start = html(&fixture.output(), "Notes/Start");
    let target = html(&fixture.output(), "Notes/Target");
    assert!(
        start.contains(&format!("href=\"{}#block-{id}\"", route("Notes/Target"))),
        "{start}"
    );
    assert!(target.contains(&format!("id=\"block-{id}\"")), "{target}");
    assert!(
        start.contains("Nested target &amp; &lt;unsafe&gt; Notes/Other"),
        "{start}"
    );
    for unavailable in [duplicate, excluded, private, root] {
        assert!(!start.contains(&format!("#block-{unavailable}")));
        assert!(start.contains(&format!("(({unavailable}))")));
    }
    for contents in files(&fixture.output()).values() {
        let text = String::from_utf8_lossy(contents);
        for sentinel in [
            "EXCLUDED SECRET",
            "PRIVATE SECRET",
            "SECRET ASSET",
            "private.png",
            "id=\"block-not-a-uuid\"",
        ] {
            assert!(!text.contains(sentinel), "leaked {sentinel}");
        }
        assert!(!text.contains(&format!("id=\"block-{duplicate}\"")));
        assert!(!text.contains(&format!("id=\"block-{root}\"")));
    }
    assert_eq!(report.assets, 0);
    assert!(
        report
            .diagnostics
            .iter()
            .filter(|d| d.message.contains("block reference"))
            .count()
            >= 4
    );
    assert_eq!(before, files(&fixture.garden()));
}

#[test]
fn block_reference_labels_are_bounded_plain_text_and_ids_follow_property_precedence() {
    let fixture = Fixture::new();
    let id = "12345678-1234-1234-1234-123456789abc";
    fixture.write(
        "garden/pages/Notes___Start.md",
        format!("- (({id}))\n- [My label]((({id})))\n"),
    );
    fixture.write(
        "garden/pages/Notes___Target.md",
        format!(
            "- {} {{}} {{{{embed [[Notes/Hidden]]}}}}\n  id:: invalid\n  custom-id:: {id}\n",
            "a".repeat(220)
        ),
    );
    let report = publish_site(
        fixture.garden(),
        fixture.output(),
        FilenameFormat::TripleLowbar,
        &NamespaceSelection::new(&["Notes"], &[]).unwrap(),
    )
    .unwrap();
    let start = html(&fixture.output(), "Notes/Start");
    assert!(start.contains(&format!("{}…</a>", "a".repeat(160))));
    assert!(!start.contains("Notes/Hidden"));
    assert!(start.contains("My label</a>"), "{start}");
    assert!(html(&fixture.output(), "Notes/Target").contains(&format!("id=\"block-{id}\"")));
    assert_eq!(report.assets, 0);
}

#[test]
fn aliases_use_selected_visible_candidates_and_preserve_source() {
    let fixture = Fixture::new();
    fixture.write("garden/pages/Publish___Start.md", "- [[Shared]] [Source label]([[Shared]]) [[Clash]] [[PrivateOnly]] [[ExcludedOnly]] [[Nested]] [[Literal]]\n");
    fixture.write("garden/pages/Publish___File.md", "title:: Publish/Chosen\nalias:: Shared, Shared, Clash\n\n- Selected content\n  alias:: Nested\n");
    fixture.write(
        "garden/pages/Publish___Clash.md",
        "title:: Clash\n\n- A real title\n",
    );
    fixture.write("garden/pages/Publish___Private.md", "alias:: Shared, PrivateOnly\npublic:: false\n\n- PRIVATE_TEXT_SECRET ![secret](../assets/private.png) ![hidden](https://private.invalid/HIDDEN_PRIVATE_URL.png)\n");
    fixture.write("garden/pages/Excluded___Secret.md", "alias:: Shared, ExcludedOnly, Publish/Invisible\n\n- EXCLUDED_TEXT_SECRET ![secret](../assets/excluded.png) ![hidden](https://excluded.invalid/HIDDEN_EXCLUDED_URL.png)\n");
    fixture.write(
        "garden/pages/Publish___Literal.md",
        "alias:: `[[Literal]]`\n\n- Literal page\n",
    );
    fixture.write("garden/assets/private.png", "PRIVATE_ASSET_SECRET");
    fixture.write("garden/assets/excluded.png", "EXCLUDED_ASSET_SECRET");
    let original = files(&fixture.garden());
    let selection = NamespaceSelection::new(&["Publish", "Clash"], &[]).unwrap();
    let report = publish_site(
        fixture.garden(),
        fixture.output(),
        FilenameFormat::TripleLowbar,
        &selection,
    )
    .unwrap();
    let generated = files(&fixture.output());
    let html = fs::read_to_string(fixture.output().join(route("Publish/Start"))).unwrap();
    assert!(html.contains(&format!("href=\"{}\">Shared</a>", route("Publish/Chosen"))));
    assert!(html.contains(&format!(
        "href=\"{}\">Source label</a>",
        route("Publish/Chosen")
    )));
    assert!(html.contains("<span class=\"unresolved\">Clash</span>"));
    assert!(
        report
            .diagnostics
            .iter()
            .any(|d| d.message == "Ambiguous selected page reference: Clash")
    );
    for unavailable in ["PrivateOnly", "ExcludedOnly", "Nested", "Literal"] {
        assert!(html.contains(&format!("<span class=\"unresolved\">{unavailable}</span>")));
    }
    assert_eq!(report.assets, 0);
    for content in generated.values() {
        let output = String::from_utf8_lossy(content);
        for sentinel in [
            "Publish/Private",
            "Excluded/Secret",
            "PRIVATE_TEXT_SECRET",
            "EXCLUDED_TEXT_SECRET",
            "PRIVATE_ASSET_SECRET",
            "EXCLUDED_ASSET_SECRET",
            "Publish/Invisible",
            "HIDDEN_PRIVATE_URL",
            "HIDDEN_EXCLUDED_URL",
        ] {
            assert!(!output.contains(sentinel), "output leaked {sentinel}");
        }
    }
    assert_eq!(original, files(&fixture.garden()));
    let second = fixture.0.join("site-again");
    publish_site(
        fixture.garden(),
        &second,
        FilenameFormat::TripleLowbar,
        &selection,
    )
    .unwrap();
    assert_eq!(generated, files(&second));
}

#[test]
fn normalized_title_collision_fails_before_output() {
    let fixture = Fixture::new();
    fixture.write("garden/pages/A.md", "title:: Café\n\n- A\n");
    fixture.write("garden/pages/B.md", "title:: CAFÉ\n\n- B\n");
    let selection = NamespaceSelection::new(&["Café", "CAFÉ"], &[]).unwrap();
    assert!(
        publish_site(
            fixture.garden(),
            fixture.output(),
            FilenameFormat::TripleLowbar,
            &selection
        )
        .is_err()
    );
    assert!(!fixture.output().exists());
}
