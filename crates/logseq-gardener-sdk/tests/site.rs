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
            "- ![remote](https://example.com/remote.png)\n",
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
    assert!(!page.contains("src=\"https://example.com/remote.png\""));
    assert!(page.contains("href=\"https://example.com/?a=1&amp;b=2\""));
    assert!(report.diagnostics.len() >= 3, "{:?}", report.diagnostics);
    assert_eq!(report.assets, 0);
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
