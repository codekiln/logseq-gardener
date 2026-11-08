//! The first-party `sequence-header` plugin: which pages get an entry and what it holds.
use logseq_gardener_sdk::page_titles::FilenameFormat;
use logseq_gardener_sdk::publishing::NamespaceSelection;
use logseq_gardener_sdk::site::{SiteOptions, publish_site_with_options};
use serde_json::{Value, json};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT: AtomicUsize = AtomicUsize::new(0);

struct Fixture(PathBuf);

impl Fixture {
    /// A garden whose `Show` pages form a sequence, with `extra` in its `config.edn` map.
    fn new(extra: &str) -> Self {
        let root = std::env::temp_dir().join(format!(
            "lsg-sequence-header-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        let fixture = Self(root);
        fixture.write(
            "logseq/config.edn",
            &format!("{{:publishing/all-pages-public? true {extra}}}"),
        );
        for (name, text) in [
            ("Show___One.md", "next:: [[show/two]]\n\n- One\n"),
            (
                "Show___Two.md",
                "prev:: [[Show/One]]\nnext:: [[First & <Three>]]\n\n- Two\n",
            ),
            (
                "Show___Three.md",
                "alias:: [[First & <Three>]]\nprev:: [[Show/Two]]\n\n- Three\n",
            ),
            (
                "Show___Withheld Neighbours.md",
                "prev:: [[Other]]\nnext:: [[Show/Private]]\n\n- Both neighbours are withheld\n",
            ),
            ("Show___Private.md", "public:: false\n\n- PRIVATE_BODY\n"),
            ("Other.md", "- Outside the selection\n"),
            ("Show___Alias A.md", "alias:: [[Shared]]\n\n- A\n"),
            ("Show___Alias B.md", "alias:: [[Shared]]\n\n- B\n"),
            (
                "Show___Ambiguous And Self.md",
                "prev:: [[Shared]]\nnext:: [[Show/Ambiguous And Self]]\n\n- Body\n",
            ),
            ("Show___Plain.md", "- No neighbours\n"),
        ] {
            fixture.write(&format!("pages/{name}"), text);
        }
        fixture
    }

    fn write(&self, name: &str, text: &str) {
        let path = self.0.join("garden").join(name);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    fn publish(&self, site: &str) -> PathBuf {
        let output = self.0.join(site);
        publish_site_with_options(
            self.0.join("garden"),
            &output,
            FilenameFormat::TripleLowbar,
            &NamespaceSelection::new(&["Show"], &[]).unwrap(),
            &SiteOptions::default(),
        )
        .unwrap();
        output
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn manifest(site: &Path) -> Value {
    serde_json::from_slice(&fs::read(site.join("lsg-site.json")).unwrap()).unwrap()
}

fn url(site: &Path, title: &str) -> String {
    manifest(site)["pages"]
        .as_array()
        .unwrap()
        .iter()
        .find(|page| page["title"] == title)
        .unwrap_or_else(|| panic!("{title} is published"))["url"]
        .as_str()
        .unwrap()
        .to_owned()
}

/// The page's `sequence-header` entry, or `Value::Null` without one.
fn entry(site: &Path, title: &str) -> Value {
    let html = fs::read_to_string(site.join(url(site, title))).unwrap();
    let start = html
        .find("<script type=\"application/json\" data-lsg-page>")
        .expect("pages carry metadata");
    let json = &html[start..];
    let json = &json[json.find('>').unwrap() + 1..json.find("</script>").unwrap()];
    let metadata: Value = serde_json::from_str(json).unwrap();
    metadata["plugins"]["sequence-header"].clone()
}

fn link(site: &Path, title: &str) -> Value {
    json!({"url": url(site, title), "title": title})
}

#[test]
fn sequence_header_entries_hold_each_published_neighbour() {
    let fixture = Fixture::new(":lsg/plugins [:sequence-header]");
    let site = fixture.publish("site");
    let embedded = Path::new(env!("CARGO_MANIFEST_DIR")).join("plugins/sequence-header");
    for file in ["sequence-header.mjs", "sequence-header.css"] {
        assert_eq!(
            fs::read(site.join("plugins/sequence-header").join(file)).unwrap(),
            fs::read(embedded.join(file)).unwrap(),
            "{file}"
        );
    }
    assert_eq!(
        manifest(&site)["plugins"],
        json!([{
            "id": "sequence-header",
            "version": logseq_gardener_sdk::VERSION,
            "module": "plugins/sequence-header/sequence-header.mjs",
            "styles": ["plugins/sequence-header/sequence-header.css"],
            "activation": "on-demand",
        }])
    );
    // Both neighbours, found through a lower-case name and an alias.
    assert_eq!(
        entry(&site, "Show/Two"),
        json!({"prev": link(&site, "Show/One"), "next": link(&site, "Show/Three")})
    );
    // One neighbour.
    assert_eq!(
        entry(&site, "Show/One"),
        json!({"next": link(&site, "Show/Two")})
    );
    assert_eq!(
        entry(&site, "Show/Three"),
        json!({"prev": link(&site, "Show/Two")})
    );
    // Withheld, excluded, ambiguous and self targets, and a page without neighbours, give no
    // entry, so the browser imports no plugin code there.
    for title in [
        "Show/Withheld Neighbours",
        "Show/Ambiguous And Self",
        "Show/Plain",
        "Show",
    ] {
        assert_eq!(entry(&site, title), Value::Null, "{title}");
    }
    for path in walk(&site) {
        let text = String::from_utf8_lossy(&fs::read(&path).unwrap()).into_owned();
        assert!(!text.contains("PRIVATE_BODY"), "{}", path.display());
    }
}

#[test]
fn sequence_header_leaves_out_a_direction_the_garden_hides() {
    let fixture = Fixture::new(":lsg/plugins [:sequence-header] :block-hidden-properties #{:next}");
    let site = fixture.publish("site");
    assert_eq!(
        entry(&site, "Show/Two"),
        json!({"prev": link(&site, "Show/One")})
    );
    assert_eq!(entry(&site, "Show/One"), Value::Null);
}

#[test]
fn sequence_header_leaves_the_page_sequence_region_as_it_is() {
    let with = Fixture::new(":lsg/plugins [:sequence-header]");
    let without = Fixture::new("");
    let (with_site, without_site) = (with.publish("site"), without.publish("site"));
    let region = |site: &Path| {
        let html = fs::read_to_string(site.join(url(site, "Show/Two"))).unwrap();
        let start = html.find("<nav class=\"page-sequence\"").unwrap();
        html[start..start + html[start..].find("</nav>").unwrap()].to_owned()
    };
    assert_eq!(region(&with_site), region(&without_site));
}

fn walk(root: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    for entry in fs::read_dir(root).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            found.extend(walk(&path));
        } else {
            found.push(path);
        }
    }
    found
}
