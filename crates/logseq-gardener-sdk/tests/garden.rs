use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use logseq_gardener_sdk::ast::{Block, Inline};
use logseq_gardener_sdk::garden::{DocumentKind, GardenLoadError, load_garden};

static NEXT_FIXTURE: AtomicUsize = AtomicUsize::new(0);

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "lsg-loader-{}-{}",
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
}

impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn retains_exact_source_outline_properties_and_references() {
    let fixture = Fixture::new();
    let source = concat!(
        "title:: Example\r\n",
        "- Parent [[Visible/Page]]  \r\n",
        "\t- Child café ((12345678-1234-1234-1234-123456789abc))\r\n",
        "\t  id:: abcdefab-1234-1234-1234-123456789abc\r\n",
        "- Literal `[[Inline/Secret]]`\r\n",
        "```text\r\n[[Fence/Secret]] ((abcdefab-1234-1234-1234-123456789abc))\r\n```\r\n",
    );
    let path = fixture.write("pages/Example.md", source);
    let garden = load_garden(&fixture.0).unwrap();
    assert_eq!(garden.documents.len(), 1);
    let document = &garden.documents[0];
    assert_eq!(document.relative_path, Path::new("pages/Example.md"));
    assert_eq!(document.kind, DocumentKind::Page);
    assert_eq!(document.source, source);
    assert_eq!(fs::read(&path).unwrap(), source.as_bytes());
    assert_eq!(document.parsed.refs.page, ["Visible/Page"]);
    assert_eq!(
        document.parsed.refs.block,
        ["12345678-1234-1234-1234-123456789abc"]
    );
    let blocks = &document.parsed.blocks;
    assert!(blocks.iter().any(|block| matches!(block,
        Block::Properties { props, .. } if props.iter().any(|property| property.0 == "title" && property.1 == "Example")
    )));
    let levels: Vec<_> = blocks
        .iter()
        .filter_map(|block| match block {
            Block::Bullet { level, .. } => Some(*level),
            _ => None,
        })
        .collect();
    assert!(
        levels.windows(2).any(|levels| levels[1] > levels[0]),
        "{blocks:#?}"
    );
    assert!(blocks.iter().any(|block| matches!(block,
        Block::Src { lang, code, .. } if lang == "text" && code.contains("[[Fence/Secret]]")
    )));
    assert!(blocks.iter().any(|block| matches!(block,
        Block::Bullet { inline, .. } if inline.iter().any(|inline| matches!(inline, Inline::Code { text, .. } if text == "[[Inline/Secret]]"))
    )));
}

#[test]
fn loads_only_visible_markdown_in_deterministic_path_order() {
    let fixture = Fixture::new();
    fixture.write("pages/z.md", "- z");
    fixture.write("pages/nested/b.md", "- b");
    fixture.write("pages/a.md", "- a");
    fixture.write("journals/2026_10_06.md", "- journal");
    for path in [
        "pages/.hidden.md",
        "pages/.hidden/visible.md",
        "journals/.hidden.md",
        "pages/ignored.org",
        "pages/ignored.MD",
        "pages/ignored.txt",
        "assets/image.md",
        "stray.md",
    ] {
        fixture.write(path, [0xff]); // A skipped file must never be read or decoded.
    }
    let first = load_garden(&fixture.0).unwrap();
    let second = load_garden(&fixture.0).unwrap();
    assert_eq!(first, second);
    let paths: Vec<_> = first
        .documents
        .iter()
        .map(|document| document.relative_path.as_path())
        .collect();
    assert_eq!(
        paths,
        [
            "journals/2026_10_06.md",
            "pages/a.md",
            "pages/nested/b.md",
            "pages/z.md"
        ]
        .map(Path::new)
    );
    assert_eq!(first.documents[0].kind, DocumentKind::Journal);
    assert!(
        first.documents[1..]
            .iter()
            .all(|document| document.kind == DocumentKind::Page)
    );
}

#[test]
fn accepts_empty_pages_or_journals_but_rejects_unrelated_roots() {
    for name in ["pages", "journals"] {
        let fixture = Fixture::new();
        fs::create_dir(fixture.0.join(name)).unwrap();
        assert!(load_garden(&fixture.0).unwrap().documents.is_empty());
    }
    let fixture = Fixture::new();
    let error = load_garden(&fixture.0).unwrap_err();
    assert_eq!(error.path(), fixture.0);
    assert!(matches!(error, GardenLoadError::InvalidInput { .. }));
}

#[test]
fn identifies_missing_root_file_root_and_file_note_directory() {
    let fixture = Fixture::new();
    let file = fixture.write("file", "text");
    let absent = fixture.0.join("absent");
    for path in [&file, &absent] {
        let error = load_garden(path).unwrap_err();
        assert_eq!(error.path(), path);
        assert!(error.to_string().contains(&path.display().to_string()));
    }
    for name in ["pages", "journals"] {
        let garden = Fixture::new();
        let path = garden.write(name, "not a directory");
        let error = load_garden(&garden.0).unwrap_err();
        assert_eq!(error.path(), path);
    }
}

#[test]
fn invalid_utf8_stops_loading_and_identifies_note() {
    let fixture = Fixture::new();
    fixture.write("pages/a-valid.md", "- valid");
    let invalid = fixture.write("pages/z-invalid.md", [0xff]);
    let error = load_garden(&fixture.0).unwrap_err();
    assert_eq!(error.path(), invalid);
    assert!(
        matches!(error, GardenLoadError::Io { source, .. } if source.kind() == std::io::ErrorKind::InvalidData)
    );
}

#[cfg(unix)]
#[test]
fn rejects_symlink_root_note_directories_and_visible_entries() {
    use std::os::unix::fs::symlink;
    let target = Fixture::new();
    target.write("pages/real.md", "- real");
    let wrapper = Fixture::new();
    let root_link = wrapper.0.join("root");
    symlink(&target.0, &root_link).unwrap();
    assert_eq!(load_garden(&root_link).unwrap_err().path(), root_link);
    let trailing_root = PathBuf::from(format!("{}/", root_link.display()));
    assert_eq!(
        load_garden(&trailing_root).unwrap_err().path(),
        trailing_root
    );
    for name in ["pages", "journals"] {
        let fixture = Fixture::new();
        let path = fixture.0.join(name);
        symlink(target.0.join("pages"), &path).unwrap();
        assert_eq!(load_garden(&fixture.0).unwrap_err().path(), path);
    }
    for (name, destination) in [
        ("linked.md", target.0.join("pages/real.md")),
        ("linked.txt", target.0.join("pages/real.md")),
        ("linked-directory", target.0.join("pages")),
        ("broken.md", target.0.join("missing")),
    ] {
        let fixture = Fixture::new();
        fs::create_dir(fixture.0.join("pages")).unwrap();
        let path = fixture.0.join("pages").join(name);
        symlink(destination, &path).unwrap();
        assert_eq!(load_garden(&fixture.0).unwrap_err().path(), path);
    }
    let fixture = Fixture::new();
    fs::create_dir(fixture.0.join("pages")).unwrap();
    symlink(target.0.join("missing"), fixture.0.join("pages/.hidden")).unwrap();
    assert!(load_garden(&fixture.0).unwrap().documents.is_empty());
}

#[cfg(unix)]
#[test]
fn rejects_visible_special_files_without_attempting_to_read_them() {
    use std::os::unix::net::UnixListener;
    let fixture = Fixture::new();
    fs::create_dir(fixture.0.join("pages")).unwrap();
    let path = fixture.0.join("pages/socket.md");
    let _listener = UnixListener::bind(&path).unwrap();
    let error = load_garden(&fixture.0).unwrap_err();
    assert_eq!(error.path(), path);
    assert!(matches!(
        error,
        GardenLoadError::InvalidInput {
            reason: "special files are unsupported",
            ..
        }
    ));
}

#[cfg(unix)]
#[test]
fn unreadable_markdown_reports_its_path() {
    use std::os::unix::fs::PermissionsExt;
    let fixture = Fixture::new();
    let path = fixture.write("pages/unreadable.md", "- private");
    let permissions = fs::metadata(&path).unwrap().permissions();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o0)).unwrap();
    // A privileged test process can bypass file permissions; it cannot exercise this case.
    if fs::read_to_string(&path).is_ok() {
        fs::set_permissions(&path, permissions).unwrap();
        return;
    }
    let result = load_garden(&fixture.0);
    fs::set_permissions(&path, permissions).unwrap();
    let error = result.unwrap_err();
    assert_eq!(error.path(), path);
    assert!(
        matches!(error, GardenLoadError::Io { source, .. } if source.kind() == std::io::ErrorKind::PermissionDenied)
    );
}
