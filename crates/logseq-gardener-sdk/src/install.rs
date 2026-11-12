//! Installable gardens: the settings, the icon check, the manifest and the page tags.
//!
//! A garden turns installation on with `:lsg/install` in its own `logseq/config.edn`; see
//! [`crate::garden_settings::read_install`]. Every rule lives here and in that reader. The
//! publisher writes the files and the tags that these functions return.
//!
//! ```
//! use logseq_gardener_sdk::garden_settings::parse_install;
//!
//! let settings = parse_install(
//!     r#"{:lsg/install {:name "Notes" :icon "app-icon.png"}}"#,
//!     std::path::Path::new("."),
//! );
//! // The icon file is checked, so a missing file stops the build with a message.
//! assert!(settings.unwrap_err().to_string().contains("app-icon.png"));
//! ```

use std::fmt;
use std::fs;
use std::io::Read;
use std::path::{Component, Path};

/// The manifest, written at the site root.
pub const MANIFEST_ROUTE: &str = "lsg.webmanifest";
/// The author's icon, copied unchanged to the site root.
pub const ICON_ROUTE: &str = "lsg-icon.png";
/// The script that shows the Install app item, loaded by installable pages only.
pub const SCRIPT_ROUTE: &str = "lsg-install.js";

/// The largest accepted icon file. KB is 1,000, as in `scripts/parity/budgets.toml`.
pub const ICON_MAX_BYTES: u64 = 256_000;
/// The smallest accepted icon side, in pixels.
pub const ICON_MIN_SIDE: u32 = 512;
/// The largest accepted icon side, in pixels.
pub const ICON_MAX_SIDE: u32 = 1024;
/// The longest `:name`, in characters.
pub const NAME_MAX_CHARS: usize = 45;
/// The longest `:short-name`, and the longest `:name` that needs no `:short-name`.
pub const SHORT_NAME_MAX_CHARS: usize = 12;

const PNG_SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];

/// An `:lsg/install` value that cannot be used. The build stops before it writes output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallError {
    /// What is wrong, naming the key or the icon path.
    pub problem: String,
}

impl fmt::Display for InstallError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "logseq/config.edn: :lsg/install {}", self.problem)
    }
}

impl std::error::Error for InstallError {}

/// A checked icon: a PNG inside the garden folder, square, 512 to 1024 pixels and at most
/// 256 KB. Its only constructor reads and checks the file, so no code holds an unchecked icon.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Icon {
    path: String,
    side: u32,
    bytes: Vec<u8>,
}

impl Icon {
    /// Read the icon at `path`, relative to the garden folder `root`.
    ///
    /// # Errors
    ///
    /// Returns an error naming the path and the check that failed: a path outside the garden
    /// folder, a missing or unreadable file, a file over 256 KB, a file that is not a PNG, an
    /// image that is not square, or a side outside 512 to 1024 pixels.
    pub fn read(root: &Path, path: &str) -> Result<Self, InstallError> {
        let fail = |check: &str| InstallError {
            problem: format!(":icon {path:?}: {check}"),
        };
        let relative = Path::new(path);
        let inside = !path.is_empty()
            && !path.contains('\\')
            && relative
                .components()
                .all(|part| matches!(part, Component::Normal(_) | Component::CurDir));
        if !inside {
            return Err(fail(
                "the path must be inside the garden folder, relative to it, with no .. segment",
            ));
        }
        let full = root.join(relative);
        // A symlink may point out of the garden folder; the real path must stay inside.
        let real = fs::canonicalize(&full).map_err(|_| fail("the file does not exist"))?;
        let real_root =
            fs::canonicalize(root).map_err(|_| fail("the garden folder is unreadable"))?;
        if !real.starts_with(&real_root) {
            return Err(fail(
                "the file resolves to a place outside the garden folder",
            ));
        }
        let meta = fs::metadata(&real).map_err(|_| fail("the file does not exist"))?;
        if !meta.is_file() {
            return Err(fail("the path is not a file"));
        }
        if meta.len() > ICON_MAX_BYTES {
            return Err(fail(&format!(
                "the file is {} KB; the limit is 256 KB",
                meta.len().div_ceil(1000)
            )));
        }
        let mut bytes = Vec::new();
        fs::File::open(&real)
            .and_then(|file| file.take(ICON_MAX_BYTES + 1).read_to_end(&mut bytes))
            .map_err(|_| fail("the file cannot be read"))?;
        let (width, height) = png_size(&bytes)
            .ok_or_else(|| fail("the file is not a PNG image; use a square PNG"))?;
        if width != height {
            return Err(fail(&format!(
                "the image is {width} by {height} pixels; it must be square"
            )));
        }
        if !(ICON_MIN_SIDE..=ICON_MAX_SIDE).contains(&width) {
            return Err(fail(&format!(
                "the image is {width} by {height} pixels; each side must be {ICON_MIN_SIDE} to {ICON_MAX_SIDE}"
            )));
        }
        Ok(Self {
            path: path.to_owned(),
            side: width,
            bytes,
        })
    }

    /// The path in `:icon`, relative to the garden folder.
    #[must_use]
    pub fn path(&self) -> &str {
        &self.path
    }

    /// The side of the square image, in pixels.
    #[must_use]
    pub fn side(&self) -> u32 {
        self.side
    }

    /// The file's bytes, written unchanged to `lsg-icon.png`.
    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

/// The width and height in a PNG's header, or `None` when the bytes do not start like a PNG.
fn png_size(bytes: &[u8]) -> Option<(u32, u32)> {
    let header = bytes.get(..24)?;
    if header[..8] != PNG_SIGNATURE || &header[12..16] != b"IHDR" || header[8..12] != [0, 0, 0, 13]
    {
        return None;
    }
    let number = |at: usize| {
        u32::from_be_bytes([header[at], header[at + 1], header[at + 2], header[at + 3]])
    };
    Some((number(16), number(20)))
}

/// The checked `:lsg/install` settings. Every value is valid: names are trimmed and in range,
/// the short name is filled, and the icon has passed its checks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallSettings {
    name: String,
    short_name: String,
    icon: Icon,
}

impl InstallSettings {
    /// Check the names and keep the icon.
    ///
    /// `name` and `short_name` are trimmed. `short_name` defaults to `name` when `name` has at
    /// most 12 characters.
    ///
    /// # Errors
    ///
    /// Returns an error when a name is out of range, or when `name` has more than 12 characters
    /// and no short name is given.
    pub fn new(name: &str, short_name: Option<&str>, icon: Icon) -> Result<Self, InstallError> {
        let fail = |problem: String| InstallError { problem };
        let name = name.trim();
        let length = name.chars().count();
        if !(1..=NAME_MAX_CHARS).contains(&length) {
            return Err(fail(format!(
                ":name must hold 1 to {NAME_MAX_CHARS} characters, not {length}"
            )));
        }
        let short_name = match short_name.map(str::trim) {
            Some(short) => {
                let length = short.chars().count();
                if !(1..=SHORT_NAME_MAX_CHARS).contains(&length) {
                    return Err(fail(format!(
                        ":short-name must hold 1 to {SHORT_NAME_MAX_CHARS} characters, not {length}"
                    )));
                }
                short
            }
            None if length <= SHORT_NAME_MAX_CHARS => name,
            None => {
                return Err(fail(format!(
                    ":name {name:?} has {length} characters; add a :short-name of at most {SHORT_NAME_MAX_CHARS} characters, which Home Screens show under the icon"
                )));
            }
        };
        Ok(Self {
            name: name.to_owned(),
            short_name: short_name.to_owned(),
            icon,
        })
    }

    /// The app's name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The name that Home Screens show under the icon.
    #[must_use]
    pub fn short_name(&self) -> &str {
        &self.short_name
    }

    /// The checked icon.
    #[must_use]
    pub fn icon(&self) -> &Icon {
        &self.icon
    }

    /// The contents of `lsg.webmanifest`: a pure function of these settings and the light
    /// header colour.
    ///
    /// Every URL is relative, there is no `id`, and non-ASCII characters are written as
    /// `\u` escapes, so the bytes are the same wherever the site is hosted and however a server
    /// labels the file's encoding.
    #[must_use]
    pub fn manifest(&self, colours: &Colours) -> String {
        let side = self.icon.side();
        format!(
            "{{\"name\":{},\"short_name\":{},\"start_url\":\"./\",\"scope\":\"./\",\"display\":\"standalone\",\"theme_color\":\"{light}\",\"background_color\":\"{light}\",\"icons\":[{{\"src\":\"{ICON_ROUTE}\",\"type\":\"image/png\",\"sizes\":\"{side}x{side}\",\"purpose\":\"any\"}}]}}",
            json_string(&self.name),
            json_string(&self.short_name),
            light = colours.light,
        )
    }
}

/// The header colours of the light and dark themes, which the manifest and the `theme-color`
/// elements use.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Colours {
    /// The light theme's `--lsg-header`, such as `#fbfcfd`.
    pub light: String,
    /// The dark theme's `--lsg-header`.
    pub dark: String,
}

impl Colours {
    /// Read both header colours from the site stylesheet's theme rules.
    ///
    /// # Panics
    ///
    /// Panics when the stylesheet lacks the light rule `:root{...}` or the dark rule
    /// `:root[data-lsg-theme="dark"]{...}` with a `--lsg-header` colour. A unit test reads the
    /// shipped `theme.css`, so a stylesheet edit that breaks this fails the tests.
    #[must_use]
    pub fn from_theme(css: &str) -> Self {
        let colour = |rule: &str| -> String {
            let start = css
                .find(rule)
                .unwrap_or_else(|| panic!("theme.css has no {rule} rule"));
            let body = &css[start + rule.len()..];
            let body = &body[..body.find('}').expect("theme rule is closed")];
            let value = body
                .split(';')
                .find_map(|declaration| declaration.trim().strip_prefix("--lsg-header:"))
                .unwrap_or_else(|| panic!("theme.css {rule} has no --lsg-header"));
            value.trim().to_owned()
        };
        Self {
            light: colour(":root{"),
            dark: colour(":root[data-lsg-theme=\"dark\"]{"),
        }
    }
}

/// The five head elements of an installable page: the manifest link, the `apple-touch-icon`
/// link, one `theme-color` element for each appearance, and the deferred install script. Every
/// URL is relative to the site root, which a readable page's `<base>` names.
#[must_use]
pub fn head(colours: &Colours) -> String {
    format!(
        "<link rel=\"manifest\" href=\"{MANIFEST_ROUTE}\"><link rel=\"apple-touch-icon\" href=\"{ICON_ROUTE}\"><meta name=\"theme-color\" media=\"(prefers-color-scheme: light)\" content=\"{}\"><meta name=\"theme-color\" media=\"(prefers-color-scheme: dark)\" content=\"{}\"><script defer src=\"{SCRIPT_ROUTE}\"></script>",
        colours.light, colours.dark
    )
}

/// The hidden Install app item that the More menu holds on installable pages.
pub const MENU_ITEM: &str =
    "<button type=\"button\" role=\"menuitem\" data-lsg-install hidden>Install app</button>";

/// A JSON string literal with every non-ASCII character written as `\u` escapes.
fn json_string(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 || (c as u32) > 0x7e => {
                let mut units = [0u16; 2];
                for unit in c.encode_utf16(&mut units) {
                    out.push_str(&format!("\\u{unit:04x}"));
                }
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A PNG header of the given size followed by filler, which is all the checks read.
    pub(crate) fn png(width: u32, height: u32, total: usize) -> Vec<u8> {
        let mut bytes = PNG_SIGNATURE.to_vec();
        bytes.extend([0, 0, 0, 13]);
        bytes.extend(b"IHDR");
        bytes.extend(width.to_be_bytes());
        bytes.extend(height.to_be_bytes());
        bytes.resize(total.max(bytes.len()), 0);
        bytes
    }

    fn garden(files: &[(&str, Vec<u8>)]) -> std::path::PathBuf {
        use std::sync::atomic::{AtomicUsize, Ordering};
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let root = std::env::temp_dir().join(format!(
            "lsg-install-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&root).unwrap();
        for (name, bytes) in files {
            let path = root.join(name);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, bytes).unwrap();
        }
        root
    }

    fn colours() -> Colours {
        Colours::from_theme(include_str!("site/theme.css"))
    }

    fn icon() -> Icon {
        let root = garden(&[("icon.png", png(512, 512, 2_000))]);
        Icon::read(&root, "icon.png").unwrap()
    }

    #[test]
    fn a_valid_icon_keeps_its_bytes_and_size() {
        let root = garden(&[("assets/app-icon.png", png(512, 512, 40_000))]);
        let icon = Icon::read(&root, "assets/app-icon.png").unwrap();
        assert_eq!(icon.side(), 512);
        assert_eq!(
            icon.bytes(),
            fs::read(root.join("assets/app-icon.png")).unwrap()
        );
        assert_eq!(icon.path(), "assets/app-icon.png");
        let largest = garden(&[("i.png", png(1024, 1024, 256_000))]);
        assert_eq!(Icon::read(&largest, "i.png").unwrap().side(), 1024);
    }

    #[test]
    fn a_bad_icon_names_the_path_and_the_check() {
        let root = garden(&[
            (
                "logo.svg",
                b"<svg xmlns='http://www.w3.org/2000/svg'/>".to_vec(),
            ),
            ("small.png", png(300, 300, 2_000)),
            ("wide.png", png(600, 512, 2_000)),
            ("big.png", png(512, 512, 300_000)),
            ("huge.png", png(2000, 2000, 2_000)),
            ("short.png", PNG_SIGNATURE.to_vec()),
        ]);
        let cases = [
            ("missing.png", "does not exist"),
            ("logo.svg", "not a PNG"),
            ("short.png", "not a PNG"),
            ("small.png", "300 by 300"),
            ("huge.png", "2000 by 2000"),
            ("wide.png", "600 by 512 pixels; it must be square"),
            ("big.png", "300 KB; the limit is 256 KB"),
            ("../icon.png", "inside the garden folder"),
            ("/etc/icon.png", "inside the garden folder"),
            ("", "inside the garden folder"),
        ];
        for (path, check) in cases {
            let error = Icon::read(&root, path).unwrap_err().to_string();
            assert!(error.contains(check), "{path}: {error}");
            assert!(
                error.contains("logseq/config.edn") && error.contains(":icon"),
                "{error}"
            );
            assert!(error.contains(&format!("{path:?}")), "{error}");
        }
    }

    #[cfg(unix)]
    #[test]
    fn a_symlink_out_of_the_garden_is_refused() {
        let outside = garden(&[("icon.png", png(512, 512, 2_000))]);
        let root = garden(&[]);
        std::os::unix::fs::symlink(outside.join("icon.png"), root.join("link.png")).unwrap();
        let error = Icon::read(&root, "link.png").unwrap_err().to_string();
        assert!(error.contains("outside the garden folder"), "{error}");
    }

    #[test]
    fn a_directory_is_not_an_icon() {
        let root = garden(&[("icons/a.png", png(512, 512, 2_000))]);
        let error = Icon::read(&root, "icons").unwrap_err().to_string();
        assert!(error.contains("not a file"), "{error}");
    }

    #[test]
    fn names_are_trimmed_and_the_short_name_is_filled() {
        let settings = InstallSettings::new("  Notes  ", None, icon()).unwrap();
        assert_eq!((settings.name(), settings.short_name()), ("Notes", "Notes"));
        let twelve = InstallSettings::new("123456789012", None, icon()).unwrap();
        assert_eq!(twelve.short_name(), "123456789012");
        let with_short =
            InstallSettings::new("Ghost in the Patch", Some(" GitP "), icon()).unwrap();
        assert_eq!(with_short.short_name(), "GitP");
    }

    #[test]
    fn name_errors_name_the_key_and_the_problem() {
        let long = "x".repeat(46);
        let cases = [
            ("", None, ":name must hold 1 to 45"),
            ("   ", None, ":name must hold 1 to 45"),
            (long.as_str(), Some("X"), ":name must hold 1 to 45"),
            ("Notes", Some(""), ":short-name must hold 1 to 12"),
            (
                "Notes",
                Some("1234567890123"),
                ":short-name must hold 1 to 12",
            ),
            (
                "Logseq Gardener Documentation",
                None,
                "add a :short-name of at most 12 characters",
            ),
            ("1234567890123", None, "add a :short-name"),
        ];
        for (name, short, problem) in cases {
            let error = InstallSettings::new(name, short, icon())
                .unwrap_err()
                .to_string();
            assert!(
                error.starts_with("logseq/config.edn: :lsg/install "),
                "{error}"
            );
            assert!(error.contains(problem), "{error}");
        }
    }

    #[test]
    fn the_manifest_has_these_exact_bytes_and_no_id() {
        let settings = InstallSettings::new("Ghost in the Patch", Some("GitP"), icon()).unwrap();
        let manifest = settings.manifest(&colours());
        assert_eq!(
            manifest,
            r##"{"name":"Ghost in the Patch","short_name":"GitP","start_url":"./","scope":"./","display":"standalone","theme_color":"#fbfcfd","background_color":"#fbfcfd","icons":[{"src":"lsg-icon.png","type":"image/png","sizes":"512x512","purpose":"any"}]}"##
        );
        let value: serde_json::Value = serde_json::from_str(&manifest).unwrap();
        assert!(value.get("id").is_none());
    }

    #[test]
    fn the_manifest_escapes_quotes_and_non_ascii_names() {
        let settings =
            InstallSettings::new("Le \"Jardin\" de Zoë \u{1f331}", Some("Jardin"), icon()).unwrap();
        let manifest = settings.manifest(&colours());
        assert!(manifest.is_ascii(), "{manifest}");
        assert!(
            manifest.contains(r#""name":"Le \"Jardin\" de Zo\u00eb \ud83c\udf31""#),
            "{manifest}"
        );
        let value: serde_json::Value = serde_json::from_str(&manifest).unwrap();
        assert_eq!(value["name"], "Le \"Jardin\" de Zoë \u{1f331}");
    }

    #[test]
    fn a_longest_name_stays_under_one_kilobyte() {
        let name = "\u{1f331}".repeat(NAME_MAX_CHARS);
        let short = "\u{1f331}".repeat(SHORT_NAME_MAX_CHARS);
        let root = garden(&[("i.png", png(1024, 1024, 2_000))]);
        let big = Icon::read(&root, "i.png").unwrap();
        let manifest = InstallSettings::new(&name, Some(&short), big)
            .unwrap()
            .manifest(&colours());
        assert!(manifest.len() <= 1000, "{} bytes", manifest.len());
        let plain = InstallSettings::new(&"x".repeat(NAME_MAX_CHARS), Some("x"), icon())
            .unwrap()
            .manifest(&colours());
        assert!(plain.len() < 500, "{} bytes", plain.len());
    }

    #[test]
    fn the_header_colours_come_from_the_shipped_theme() {
        assert_eq!(
            colours(),
            Colours {
                light: "#fbfcfd".into(),
                dark: "#151718".into()
            }
        );
    }

    #[test]
    fn the_head_holds_five_relative_elements() {
        let head = head(&colours());
        assert_eq!(head.matches("<link").count(), 2);
        assert_eq!(head.matches("<meta").count(), 2);
        assert!(
            head.contains("href=\"lsg.webmanifest\"") && head.contains("href=\"lsg-icon.png\"")
        );
        assert!(
            head.contains("src=\"lsg-install.js\"></script>") && head.contains("<script defer")
        );
        assert!(!head.contains("apple-mobile-web-app"));
        assert!(!head.contains("=\"/") && !head.contains("http"), "{head}");
        assert!(head.len() < 400, "{} bytes", head.len());
    }
}
