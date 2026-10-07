use std::fs;
use std::path::{Component, Path, PathBuf};

pub(super) struct ResolvedAsset {
    pub relative_path: PathBuf,
    pub contents: Vec<u8>,
    pub extension: String,
}

/// Resolve an explicitly requested file without following asset symlinks.
/// The caller must keep the source filesystem unchanged during publication.
pub(super) fn resolve_asset(
    root: &Path,
    source_relative: &Path,
    url: &str,
) -> Result<ResolvedAsset, String> {
    let decoded = decode_url(url)?;
    if decoded.is_empty()
        || decoded.starts_with('/')
        || decoded.ends_with('/')
        || decoded
            .chars()
            .any(|c| c.is_control() || matches!(c, '\\' | ':' | '?' | '#'))
    {
        return Err("asset URL must be a local relative path without a scheme or suffix".into());
    }
    if source_relative
        .components()
        .any(|c| !matches!(c, Component::Normal(_)))
    {
        return Err("source document must have a garden-relative path".into());
    }
    let mut relative = source_relative
        .parent()
        .ok_or("source document has no parent path")?
        .to_path_buf();
    for component in decoded.split('/') {
        match component {
            "" | "." => {}
            ".." => {
                if !relative.pop() {
                    return Err("asset path escapes the source garden".into());
                }
            }
            name => relative.push(name),
        }
    }
    if !relative.starts_with("assets") || relative.components().count() < 2 {
        return Err("asset path must name a file beneath the garden assets directory".into());
    }
    let extension = relative
        .extension()
        .and_then(|extension| extension.to_str())
        .ok_or("asset filename has no supported extension")?
        .to_ascii_lowercase();
    if !matches!(
        extension.as_str(),
        "png"
            | "jpg"
            | "jpeg"
            | "gif"
            | "webp"
            | "avif"
            | "ico"
            | "pdf"
            | "mp3"
            | "mp4"
            | "ogg"
            | "wav"
            | "webm"
    ) {
        return Err(format!("unsupported asset extension: {extension}"));
    }
    // Removing a trailing separator keeps symlink_metadata from following a
    // symlink supplied as the root itself.
    let mut path: PathBuf = root.components().collect();
    require_directory(&path)?;
    let mut components = relative.components().peekable();
    while let Some(component) = components.next() {
        path.push(component);
        if components.peek().is_some() {
            require_directory(&path)?;
        } else {
            let metadata = fs::symlink_metadata(&path)
                .map_err(|error| format!("cannot inspect asset file: {error}"))?;
            if !metadata.file_type().is_file() {
                return Err("asset must be a regular file without symlinks".into());
            }
        }
    }
    let contents = fs::read(&path).map_err(|error| format!("cannot read asset file: {error}"))?;
    Ok(ResolvedAsset {
        relative_path: relative,
        contents,
        extension,
    })
}

fn require_directory(path: &Path) -> Result<(), String> {
    let metadata = fs::symlink_metadata(path)
        .map_err(|error| format!("cannot inspect asset directory: {error}"))?;
    if metadata.file_type().is_dir() {
        Ok(())
    } else {
        Err("asset path must use ordinary directories without symlinks".into())
    }
}

fn decode_url(url: &str) -> Result<String, String> {
    let mut decoded = Vec::with_capacity(url.len());
    let mut source = url.as_bytes().iter().copied();
    while let Some(value) = source.next() {
        if value == b'%' {
            let high = source.next().and_then(hex_digit);
            let low = source.next().and_then(hex_digit);
            match (high, low) {
                (Some(high), Some(low)) => decoded.push(high * 16 + low),
                _ => return Err("asset URL contains an invalid percent escape".into()),
            }
        } else {
            decoded.push(value);
        }
    }
    String::from_utf8(decoded).map_err(|_| "asset URL decodes to invalid UTF-8".into())
}

fn hex_digit(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    struct Fixture(PathBuf);

    impl Fixture {
        fn new() -> Self {
            static NEXT: AtomicU64 = AtomicU64::new(0);
            let path = std::env::temp_dir().join(format!(
                "gardener-assets-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir(&path).unwrap();
            fs::create_dir(path.join("assets")).unwrap();
            Self(path)
        }

        fn write(&self, path: &str, contents: &[u8]) {
            let path = self.0.join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, contents).unwrap();
        }

        fn resolve(&self, url: &str) -> Result<ResolvedAsset, String> {
            resolve_asset(&self.0, Path::new("pages/Start.md"), url)
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }

    #[test]
    fn resolves_nested_documents_and_strictly_decodes_once() {
        let fixture = Fixture::new();
        fixture.write("assets/nested/été + image.PNG", b"image contents");
        let asset = resolve_asset(
            &fixture.0,
            Path::new("pages/section/Start.md"),
            "../../assets%2fnested/%C3%A9t%C3%A9%20+%20image.PNG",
        )
        .unwrap();
        assert_eq!(
            asset.relative_path,
            Path::new("assets/nested/été + image.PNG")
        );
        assert_eq!(asset.extension, "png");
        assert_eq!(asset.contents, b"image contents");
        fixture.write("assets/%2e%2e.png", b"literal escape");
        assert_eq!(
            fixture
                .resolve("../assets/%252e%252e.png")
                .unwrap()
                .contents,
            b"literal escape"
        );
    }

    #[test]
    fn rejects_unsafe_urls_and_unsupported_types() {
        let fixture = Fixture::new();
        fixture.write("assets/good.png", b"safe");
        fixture.write("outside.png", b"private sentinel");
        for url in [
            "/assets/good.png",
            "%2fassets/good.png",
            "//host/image.png",
            "https://example.com/image.png",
            "file:../assets/good.png",
            "data:image/png;base64,abc",
            "../assets/good.png?download=1",
            "../assets/good.png%23fragment",
            "../assets/good.png%3fquery",
            "../assets/%00good.png",
            "../assets/%0Agood.png",
            "..\\assets\\good.png",
            "../assets%5cgood.png",
            "../outside.png",
            "../assets/../outside.png",
            "../assets/%2e%2e%2foutside.png",
            "../../assets/good.png",
            "../assets",
            "../assets/good.png/",
            "../assets/.",
            "../assets/../assets",
            "../assets/script.svg",
            "../assets/script.html",
            "../assets/script.js",
            "../assets/%",
            "../assets/%0",
            "../assets/%GG.png",
            "../assets/%FF.png",
        ] {
            assert!(fixture.resolve(url).is_err(), "accepted {url}");
        }
    }

    #[cfg(unix)]
    #[test]
    fn rejects_asset_directory_and_leaf_symlinks() {
        use std::os::unix::fs::symlink;
        let fixture = Fixture::new();
        fixture.write("private/nested.png", b"private sentinel");
        symlink(fixture.0.join("private"), fixture.0.join("assets/link")).unwrap();
        symlink(
            fixture.0.join("private/nested.png"),
            fixture.0.join("assets/leaf.png"),
        )
        .unwrap();
        assert!(fixture.resolve("../assets/link/nested.png").is_err());
        assert!(fixture.resolve("../assets/leaf.png").is_err());
        fs::remove_dir_all(fixture.0.join("assets")).unwrap();
        symlink(fixture.0.join("private"), fixture.0.join("assets")).unwrap();
        assert!(fixture.resolve("../assets/nested.png").is_err());
    }

    #[test]
    fn rejects_directories_as_files_and_forged_source_paths() {
        let fixture = Fixture::new();
        fs::create_dir(fixture.0.join("assets/directory.png")).unwrap();
        assert!(fixture.resolve("../assets/directory.png").is_err());
        assert!(
            resolve_asset(
                &fixture.0,
                Path::new("../pages/Start.md"),
                "../assets/good.png"
            )
            .is_err()
        );
    }
}
