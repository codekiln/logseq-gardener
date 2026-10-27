use logseq_gardener_sdk::journals::JournalDate;
use logseq_gardener_sdk::page_titles::FilenameFormat;
use logseq_gardener_sdk::publishing::NamespaceSelection;
use logseq_gardener_sdk::site::{SiteOptions, publish_site_with_options};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);

struct Fixture(PathBuf);

impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "lsg-graph-site-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(root.join("garden/logseq")).unwrap();
        fs::write(
            root.join("garden/logseq/config.edn"),
            "{:journal/page-title-format \"yyyy-MM-dd EEE\" :graph/forcesettings {:link-dist 90}}",
        )
        .unwrap();
        let fixture = Self(root);
        fixture.write(
            "pages/Foam.md",
            "tags:: Tool\n\n- Like [[Obsidian]] and [[Contrando]]\n",
        );
        fixture.write("pages/Obsidian.md", "- An editor, see [[Foam]]\n");
        fixture.write("pages/Tool.md", "- Tools\n");
        fixture.write(
            "pages/Secret.md",
            "public:: false\n\n- [[Secret Project]] and [[Foam]]\n",
        );
        fixture.write("pages/Other.md", "- [[Hidden Neighbour]]\n");
        fixture.write("journals/2026_10_23.md", "- Used [[Foam]] today\n");
        fixture
    }

    fn write(&self, name: &str, text: &str) {
        let path = self.0.join("garden").join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    fn publish(&self, output: &str, cache: Option<PathBuf>) -> PathBuf {
        let site = self.0.join(output);
        publish_site_with_options(
            self.0.join("garden"),
            &site,
            FilenameFormat::TripleLowbar,
            &NamespaceSelection::new(&["Foam", "Obsidian", "Tool", "Secret"], &[]).unwrap(),
            &SiteOptions {
                journals: true,
                build_date: JournalDate::new(2026, 10, 25),
                layout_cache: cache,
                ..SiteOptions::default()
            },
        )
        .unwrap();
        site
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// The titles and flags in a data file.
fn nodes(path: &Path) -> Vec<(String, u8)> {
    let mut bytes = Vec::new();
    flate2::read::GzDecoder::new(fs::File::open(path).unwrap())
        .read_to_end(&mut bytes)
        .unwrap();
    assert_eq!(&bytes[..4], b"LSGG");
    let n = u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize;
    let mut at = 40;
    let mut lengths = Vec::new();
    for _ in 0..n {
        let (mut value, mut shift) = (0_usize, 0);
        loop {
            let byte = bytes[at];
            at += 1;
            value |= usize::from(byte & 0x7f) << shift;
            if byte < 0x80 {
                break;
            }
            shift += 7;
        }
        lengths.push(value);
    }
    let mut titles = Vec::new();
    for length in lengths {
        titles.push(String::from_utf8(bytes[at..at + length].to_vec()).unwrap());
        at += length;
    }
    titles
        .into_iter()
        .zip(bytes[at..at + n].iter().copied())
        .collect()
}

fn route(title: &str) -> String {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in title.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

#[test]
fn graph_files_hold_published_pages_and_reference_only_pages_only() {
    let f = Fixture::new();
    let site = f.publish("site", None);
    let default = nodes(&site.join("_graph/default.bin"));
    let titles: Vec<&str> = default.iter().map(|(title, _)| title.as_str()).collect();
    for title in ["Foam", "Obsidian", "Tool", "Contrando"] {
        assert!(titles.contains(&title), "{title} in {titles:?}");
    }
    // The private page, what only it names and an unselected page never appear.
    for title in [
        "Secret",
        "Secret Project",
        "Other",
        "Hidden Neighbour",
        "2026-10-23 Fri",
    ] {
        assert!(!titles.contains(&title), "{title}");
    }
    let flags = |title: &str| default.iter().find(|(t, _)| t == title).unwrap().1;
    assert_ne!(flags("Contrando") & 64, 0, "reference-only");
    assert_ne!(flags("Tool") & 2, 0, "tag");
    // The journals graph adds the journal page.
    let journals = nodes(&site.join("_graph/journals.bin"));
    let journal = journals
        .iter()
        .find(|(t, _)| t == "2026-10-23 Fri")
        .unwrap();
    assert_ne!(journal.1 & 1, 0);
    // Foam's neighbourhood file flags the journal that only Show Journals brings in.
    let foam: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(site.join(format!("_graph/p/{}.json", route("Foam")))).unwrap(),
    )
    .unwrap();
    assert_eq!(foam["t"][0], "Foam");
    let at = foam["t"]
        .as_array()
        .unwrap()
        .iter()
        .position(|t| t == "2026-10-23 Fri")
        .unwrap();
    assert_ne!(foam["f"][at].as_u64().unwrap() & 128, 0);
    let everything: String = walk(&site)
        .into_iter()
        .map(|path| String::from_utf8_lossy(&fs::read(path).unwrap()).into_owned())
        .collect();
    assert!(!everything.contains("Secret Project"));
    // The forces from config.edn reach the page.
    let page = fs::read_to_string(site.join("graph.html")).unwrap();
    assert!(page.contains("&quot;link_distance&quot;:90"), "{page}");
    assert!(page.contains("_graph/journals.bin"));
}

fn walk(root: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut pending = vec![root.to_owned()];
    while let Some(directory) = pending.pop() {
        for entry in fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                pending.push(path);
            } else {
                found.push(path);
            }
        }
    }
    found
}

#[test]
fn the_same_garden_gives_identical_graph_files_and_reuses_the_cache() {
    let f = Fixture::new();
    let cache = f.0.join("cache");
    let first = f.publish("first", Some(cache.clone()));
    let cached: Vec<_> = fs::read_dir(&cache).unwrap().collect();
    assert_eq!(cached.len(), 2, "one layout per journals setting");
    let second = f.publish("second", Some(cache.clone()));
    let third = f.publish("third", None);
    for file in ["_graph/default.bin", "_graph/journals.bin"] {
        let bytes = fs::read(first.join(file)).unwrap();
        assert_eq!(bytes, fs::read(second.join(file)).unwrap(), "{file}");
        assert_eq!(bytes, fs::read(third.join(file)).unwrap(), "{file}");
    }
    // A change that adds no node or edge reuses the cached layout: emptying the cache's files
    // proves the build read them instead of laying the graph out again.
    f.write("pages/Tool.md", "- Tools, reworded\n");
    for entry in fs::read_dir(&cache).unwrap() {
        let path = entry.unwrap().path();
        let mut bytes = fs::read(&path).unwrap();
        bytes.iter_mut().for_each(|byte| *byte = 0);
        fs::write(&path, bytes).unwrap();
    }
    let fourth = f.publish("fourth", Some(cache));
    assert_ne!(
        fs::read(first.join("_graph/default.bin")).unwrap(),
        fs::read(fourth.join("_graph/default.bin")).unwrap(),
        "positions came from the cache"
    );
}
