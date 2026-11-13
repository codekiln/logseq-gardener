//! Offline reading: the limits, the cache name, the build identifier, the worker sources and the
//! page markup.
//!
//! A garden turns offline reading on with `:lsg/offline true` in its own `logseq/config.edn`;
//! see [`crate::garden_settings::read_offline`]. Every rule lives here and in that reader. The
//! publisher writes the files and the markup that these functions return.
//!
//! ```
//! use logseq_gardener_sdk::offline::{OfflineSetting, build_id, cache_name, worker};
//!
//! assert_eq!(cache_name("/gitpa/"), "lsg-offline:/gitpa/");
//! let id = build_id(["index.html", "p-1.html"]);
//! assert!(worker(OfflineSetting::On, &id).unwrap().contains(&id));
//! assert!(worker(OfflineSetting::Absent, &id).is_none());
//! ```

use std::fmt;

/// The worker, written at the site root for `true` and `false`.
pub const WORKER_ROUTE: &str = "lsg-sw.js";
/// The page script with the menu items and the saved-copy notice, written for `true` only.
pub const SCRIPT_ROUTE: &str = "lsg-offline.js";

/// The most responses the worker keeps for one garden.
pub const MAX_COPIES: usize = 250;
/// The most bytes the worker keeps for one garden: 32 MB. MB is 1,000,000, as in
/// `scripts/parity/budgets.toml`.
pub const MAX_BYTES: u64 = 32_000_000;
/// The largest response the worker keeps: 256 KB.
pub const MAX_RESPONSE_BYTES: u64 = 256_000;
/// How long a copy is kept after it was saved: 30 days, in milliseconds.
pub const MAX_AGE_MS: u64 = 30 * 24 * 60 * 60 * 1000;
/// How long a navigation waits for the network before a stored copy answers, in milliseconds.
pub const NETWORK_WAIT_MS: u64 = 4_000;

/// The garden's `:lsg/offline` value. A garden holds exactly one of these, so no caller can
/// hold a malformed setting.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum OfflineSetting {
    /// No key: the site is the same as a site published before offline reading existed.
    #[default]
    Absent,
    /// `true`: the site offers offline reading.
    On,
    /// `false`: the site publishes a worker that deletes readers' copies and unregisters itself.
    Off,
}

impl OfflineSetting {
    /// Whether full pages carry the script tag and the two menu items.
    #[must_use]
    pub fn on(self) -> bool {
        self == Self::On
    }
}

/// An `:lsg/offline` value that cannot be used. The build stops before it writes output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OfflineError {
    /// What is wrong with the value.
    pub problem: String,
}

impl fmt::Display for OfflineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "logseq/config.edn: :lsg/offline {}", self.problem)
    }
}

impl std::error::Error for OfflineError {}

/// The Cache Storage name for the garden whose worker has the scope path `scope`, such as
/// `/gitpa/`. Every cache operation uses this exact name, so `/a/` and `/ab/` never touch each
/// other's copies.
#[must_use]
pub fn cache_name(scope: &str) -> String {
    format!("lsg-offline:{scope}")
}

/// The build identifier written into the worker: a digest of the `lsg` version and the
/// published page addresses, in any order.
///
/// A publish that edits a block keeps the addresses and so keeps the worker unchanged. A
/// publish that adds or removes a page changes it, which makes readers' browsers install the
/// new worker and check their stored pages.
#[must_use]
pub fn build_id<'a>(addresses: impl IntoIterator<Item = &'a str>) -> String {
    let mut addresses: Vec<&str> = addresses.into_iter().collect();
    addresses.sort_unstable();
    addresses.dedup();
    let mut text = format!("lsg {}\n", crate::VERSION);
    for address in addresses {
        text.push_str(&format!("{}:{address}\n", address.len()));
    }
    crate::search::digest(&text)
}

/// The contents of `lsg-sw.js` for `setting`: the visited-page worker for `On`, the removing
/// worker for `Off` and no file for `Absent`. A pure function of the setting and `build`.
#[must_use]
pub fn worker(setting: OfflineSetting, build: &str) -> Option<String> {
    match setting {
        OfflineSetting::Absent => None,
        OfflineSetting::On => Some(minify(
            &include_str!("site/offline-sw.js")
                .replace("__BUILD__", build)
                .replace("__PAGES__", &MAX_COPIES.to_string())
                .replace("__BYTES__", &MAX_BYTES.to_string())
                .replace("__ITEM__", &MAX_RESPONSE_BYTES.to_string())
                .replace("__AGE__", &MAX_AGE_MS.to_string())
                .replace("__WAIT__", &NETWORK_WAIT_MS.to_string()),
        )),
        OfflineSetting::Off => Some(minify(include_str!("site/offline-remove-sw.js"))),
    }
}

/// The contents of `lsg-offline.js`, written for `On` only.
#[must_use]
pub fn script() -> String {
    minify(include_str!("site/offline.js"))
}

/// The deferred script tag every full page of an `On` site carries. The URL is relative to the
/// site root, which a readable page's `<base>` names.
pub const PAGE_HEAD: &str = "<script defer src=\"lsg-offline.js\"></script>";

/// The two hidden More menu items of an `On` site; `lsg-offline.js` shows one of them.
pub const MENU_ITEMS: &str = "<button type=\"button\" role=\"menuitem\" data-lsg-offline-save hidden>Save pages for offline reading</button><button type=\"button\" role=\"menuitem\" data-lsg-offline-remove hidden>Remove offline copies</button>";

/// Drop comment lines and indentation, and join lines that end a statement.
fn minify(source: &str) -> String {
    let mut script = String::new();
    for line in source.lines().map(str::trim) {
        if line.is_empty() || line.starts_with("//") {
            continue;
        }
        if script.ends_with(|c: char| !";{},".contains(c)) {
            script.push('\n');
        }
        script.push_str(line);
    }
    script.push('\n');
    script
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gzip(text: &str) -> usize {
        use flate2::{Compression, write::GzEncoder};
        use std::io::Write;
        let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
        encoder.write_all(text.as_bytes()).unwrap();
        encoder.finish().unwrap().len()
    }

    #[test]
    fn neighbouring_scopes_get_names_no_prefix_test_confuses() {
        let (a, ab) = (cache_name("/a/"), cache_name("/ab/"));
        assert_eq!(
            (a.as_str(), ab.as_str()),
            ("lsg-offline:/a/", "lsg-offline:/ab/")
        );
        assert!(!ab.starts_with(&a) && !a.starts_with(&ab));
        assert_ne!(cache_name("/"), cache_name("/a/"));
    }

    #[test]
    fn the_build_id_follows_the_page_addresses_and_not_their_order() {
        let pages = ["index.html", "Docs/index.html", "Other/index.html"];
        let id = build_id(pages);
        assert_eq!(id.len(), 16);
        // A block edit keeps every address, so the identifier is the same.
        assert_eq!(build_id(pages), id);
        assert_eq!(build_id(pages.iter().rev().copied()), id);
        // Adding or removing a page changes it.
        assert_ne!(build_id([&pages[..], &["New/index.html"]].concat()), id);
        assert_ne!(build_id(pages[..2].iter().copied()), id);
        // Joining two addresses cannot spell a third set.
        assert_ne!(build_id(["a", "b"]), build_id(["a\nb"]));
    }

    #[test]
    fn the_worker_has_these_exact_bytes_for_a_fixed_identifier() {
        let on = worker(OfflineSetting::On, "0123456789abcdef").unwrap();
        assert!(on.starts_with("const BUILD = '0123456789abcdef', PAGES = 250, BYTES = 32000000, ITEM = 256000, AGE = 2592000000, WAIT = 4000;const SCOPE"), "{on}");
        assert!(!on.contains("__") && !on.contains("\n//"), "{on}");
        assert_eq!(
            on,
            worker(OfflineSetting::On, "0123456789abcdef").unwrap(),
            "pure"
        );
        assert_eq!(
            crate::search::digest(&on),
            "49a266053409b1f2",
            "the worker's bytes changed; update this digest after reviewing the change"
        );
        assert_ne!(on, worker(OfflineSetting::On, "fedcba9876543210").unwrap());
        assert!(on.len() <= 5_000, "{} bytes", on.len());
        assert!(gzip(&on) <= 2_000, "{} gzip", gzip(&on));
        for rule in [
            "'lsg-offline:' + SCOPE.pathname",
            "skipWaiting()",
            "clients.claim()",
            "navigationPreload",
            "method: 'HEAD'",
        ] {
            assert!(on.contains(rule), "{rule}");
        }
        // Every cache call names the garden's own cache; nothing lists caches or asks for
        // persistent storage.
        assert!(!on.contains("caches.keys") && !on.contains("persist("));
    }

    #[test]
    fn the_removing_worker_deletes_the_cache_and_unregisters() {
        let off = worker(OfflineSetting::Off, "ignored").unwrap();
        assert_eq!(
            off,
            "addEventListener('install', () => skipWaiting());addEventListener('activate', event => event.waitUntil((async () => {await caches.delete('lsg-offline:' + new URL(registration.scope).pathname);await registration.unregister();})()));\n"
        );
        assert_eq!(off, worker(OfflineSetting::Off, "other").unwrap());
        assert!(worker(OfflineSetting::Absent, "x").is_none());
    }

    #[test]
    fn the_page_script_and_markup_fit_their_budgets() {
        let script = script();
        assert!(script.len() <= 2_000, "{} bytes", script.len());
        assert!(gzip(&script) <= 1_000, "{} gzip", gzip(&script));
        assert!(!script.contains("persist(") && !script.contains("localStorage"));
        assert!(PAGE_HEAD.len() + MENU_ITEMS.len() <= 600);
        assert_eq!(
            MENU_ITEMS
                .matches("<button type=\"button\" role=\"menuitem\"")
                .count(),
            2
        );
        assert!(
            OfflineSetting::On.on() && !OfflineSetting::Off.on() && !OfflineSetting::Absent.on()
        );
    }
}
