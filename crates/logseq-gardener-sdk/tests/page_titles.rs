use std::path::PathBuf;

use logseq_gardener_sdk::garden::{DocumentKind, GardenDocument};
use logseq_gardener_sdk::page_titles::{FilenameFormat, PageTitleError, page_title};

fn document(path: &str, source: &str) -> GardenDocument {
    GardenDocument {
        relative_path: PathBuf::from(path),
        kind: DocumentKind::Page,
        source: source.to_owned(),
        parsed: lsdoc::parse_to_projection(source),
    }
}

#[test]
fn saved_logseq_fixture_titles() {
    // Inputs from the recorded OG relationship experiment; expected original
    // names are in current-og-relationships/.../relationships.json.
    let fixtures = [
        (
            "Title Override",
            include_str!(
                "../../../openspec/changes/parser-comparison-evidence/experiments/parser-comparison/fixtures/pages/Title Override.md"
            ),
            "Chosen Name",
        ),
        (
            "Graph Cases",
            include_str!(
                "../../../openspec/changes/parser-comparison-evidence/experiments/parser-comparison/fixtures/pages/Graph Cases.md"
            ),
            "Graph Cases",
        ),
        (
            "Alias Source",
            include_str!(
                "../../../openspec/changes/parser-comparison-baseline/experiments/mldoc-baseline/fixtures/garden/pages/Alias Source.md"
            ),
            "Alias Source",
        ),
        (
            "Code Fence",
            include_str!(
                "../../../openspec/changes/parser-comparison-baseline/experiments/mldoc-baseline/fixtures/garden/pages/Code Fence.md"
            ),
            "Code Fence",
        ),
        (
            "Media",
            include_str!(
                "../../../openspec/changes/parser-comparison-baseline/experiments/mldoc-baseline/fixtures/garden/pages/Media.md"
            ),
            "Media",
        ),
        (
            "Target Page",
            include_str!(
                "../../../openspec/changes/parser-comparison-baseline/experiments/mldoc-baseline/fixtures/garden/pages/Target Page.md"
            ),
            "Target Page",
        ),
    ];
    for (stem, source, expected) in fixtures {
        let doc = document(&format!("pages/{stem}.md"), source);
        for format in [FilenameFormat::Legacy, FilenameFormat::TripleLowbar] {
            assert_eq!(page_title(&doc, format).unwrap(), expected, "{stem}");
        }
    }
    let doc = document(
        "pages/Names___Nested.md",
        include_str!(
            "../../../openspec/changes/parser-comparison-baseline/experiments/mldoc-baseline/fixtures/garden/pages/Names___Nested.md"
        ),
    );
    assert_eq!(
        page_title(&doc, FilenameFormat::Legacy).unwrap(),
        "Names___Nested"
    );
    assert_eq!(
        page_title(&doc, FilenameFormat::TripleLowbar).unwrap(),
        "Names/Nested"
    );
}

#[test]
fn decoding_matches_format_specific_logseq_order() {
    for (stem, legacy, triple) in [
        ("%C3%A9.%2F%252F", "é//%2F", "%C3%A9./%2F"),
        ("Bad%2X.%2F", "Bad%2X/%2F", "Bad%2X."),
        ("%E9%20", "%E9%20", "%E9 "),
        ("____A______B___", "____A______B___", "_A/B"),
        ("A%2fB%252F", "A/B%2F", "A/B%2F"),
        ("Résumé___日本+%23", "Résumé___日本+#", "Résumé/日本+#"),
        ("End%", "End%", "End%"),
        ("%C0%AF", "%C0%AF", "%C0%AF"),
    ] {
        let doc = document(&format!("pages/subdir/{stem}.md"), "- Content\n");
        assert_eq!(
            page_title(&doc, FilenameFormat::Legacy).unwrap(),
            legacy,
            "{stem}"
        );
        assert_eq!(
            page_title(&doc, FilenameFormat::TripleLowbar).unwrap(),
            triple,
            "{stem}"
        );
    }
}

#[test]
fn only_the_leading_properties_block_supplies_a_title() {
    for format in [FilenameFormat::Legacy, FilenameFormat::TripleLowbar] {
        let doc = document(
            "pages/Example.md",
            "title:: First\nTITLE:: /Chosen___Name/\n- Content\n",
        );
        assert_eq!(page_title(&doc, format).unwrap(), "/Chosen___Name/");
        for source in [
            "- Content\n  title:: Bullet title\n",
            "Paragraph\n\ntitle:: Later title\n",
            "```markdown\ntitle:: Literal\n```\n",
            "- Content\n\ntitle:: Later root property\n",
        ] {
            assert_eq!(
                page_title(&document("pages/Example.md", source), format).unwrap(),
                "Example",
                "{source}"
            );
        }
    }
}

#[test]
fn contents_special_case_precedes_the_title_override() {
    for format in [FilenameFormat::Legacy, FilenameFormat::TripleLowbar] {
        for path in ["pages/contents.md", "pages/contents.extra.md"] {
            assert_eq!(
                page_title(&document(path, "title:: Different\n"), format).unwrap(),
                "Contents"
            );
        }
        for path in [
            "pages/Contents.md",
            "pages/nested/contents.md",
            "pages/contents___child.md",
        ] {
            assert_eq!(
                page_title(&document(path, "title:: Different\n"), format).unwrap(),
                "Different"
            );
        }
    }
}

#[test]
fn invalid_paths_and_journals_carry_the_source_path() {
    for path in [
        "",
        "/pages/A.md",
        "pages",
        "pages/A.MD",
        "pages/A.txt",
        "other/A.md",
        "./pages/A.md",
        "pages/../A.md",
        "pages/./A.md",
        "pages//A.md",
        "pages/A.md/",
        "journals/A.md",
    ] {
        let doc = document(path, "title:: Override\n");
        let error = page_title(&doc, FilenameFormat::Legacy).unwrap_err();
        assert!(
            matches!(error, PageTitleError::InvalidPath { .. }),
            "{path}: {error}"
        );
        assert_eq!(error.path(), doc.relative_path);
    }
    let mut doc = document("journals/2026_10_03.md", "title:: Override\n");
    doc.kind = DocumentKind::Journal;
    let error = page_title(&doc, FilenameFormat::TripleLowbar).unwrap_err();
    assert!(matches!(error, PageTitleError::UnsupportedJournal { .. }));
    assert_eq!(error.path(), doc.relative_path);
    assert!(error.to_string().contains("date-format configuration"));
}

#[test]
fn empty_titles_fail_and_extraction_preserves_the_document() {
    for (path, source, format, fails) in [
        (
            "pages/______.md",
            "- Content\n",
            FilenameFormat::TripleLowbar,
            true,
        ),
        (
            "pages/______.md",
            "- Content\n",
            FilenameFormat::Legacy,
            false,
        ),
        ("pages/%20.md", "- Content\n", FilenameFormat::Legacy, true),
        (
            "pages/A.md",
            "title:: \n",
            FilenameFormat::TripleLowbar,
            true,
        ),
        (
            "pages/A.md",
            "title:: A\r\n- [[B]]\r\n",
            FilenameFormat::TripleLowbar,
            false,
        ),
    ] {
        let doc = document(path, source);
        let before = doc.clone();
        let result = page_title(&doc, format);
        if fails {
            let error = result.unwrap_err();
            assert!(matches!(error, PageTitleError::EmptyTitle { .. }));
            assert_eq!(error.path(), doc.relative_path);
        } else {
            assert!(result.is_ok());
        }
        assert_eq!(doc, before);
    }
}

#[cfg(unix)]
#[test]
fn non_utf8_stems_report_an_explicit_error() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;
    let mut doc = document("pages/A.md", "title:: Override\n");
    doc.relative_path = OsString::from_vec(b"pages/\xff.md".to_vec()).into();
    let before = doc.clone();
    let error = page_title(&doc, FilenameFormat::Legacy).unwrap_err();
    assert!(matches!(error, PageTitleError::NonUtf8Stem { .. }));
    assert_eq!(error.path(), doc.relative_path);
    assert_eq!(doc, before);
}

#[cfg(windows)]
#[test]
fn native_windows_paths_preserve_contents_behavior() {
    for path in [r"pages\contents.md", r"pages\contents.extra.md"] {
        assert_eq!(
            page_title(
                &document(path, "title:: Different\n"),
                FilenameFormat::TripleLowbar
            )
            .unwrap(),
            "Contents"
        );
    }
    assert_eq!(
        page_title(
            &document(r"pages\nested\contents.md", "title:: Different\n"),
            FilenameFormat::TripleLowbar
        )
        .unwrap(),
        "Different"
    );
    for path in [r"pages\.\A.md", r"pages\\A.md", r"pages\..\A.md"] {
        assert!(matches!(
            page_title(&document(path, ""), FilenameFormat::Legacy),
            Err(PageTitleError::InvalidPath { .. })
        ));
    }
}
