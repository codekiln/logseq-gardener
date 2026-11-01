//! Helpers shared by the site tests.

use std::io::Read;
use std::path::Path;

/// A published text file as text: a `.gz` file, such as a Linked or Unlinked References
/// fragment, is checked to be gzip with no file name and a zero time, then decompressed.
pub fn read_text(path: &Path) -> String {
    let bytes = std::fs::read(path).unwrap();
    if path.extension().is_none_or(|extension| extension != "gz") {
        return String::from_utf8(bytes).unwrap();
    }
    // ID1 ID2, deflate, no flags (so no file name), MTIME 0.
    assert_eq!(&bytes[..8], &[0x1f, 0x8b, 8, 0, 0, 0, 0, 0], "{path:?}");
    let mut text = String::new();
    flate2::read::GzDecoder::new(bytes.as_slice())
        .read_to_string(&mut text)
        .unwrap();
    text
}
