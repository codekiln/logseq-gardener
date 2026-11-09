use logseq_gardener_sdk::{
    page_titles::FilenameFormat,
    publishing::NamespaceSelection,
    site::{SearchOptions, SiteOptions, publish_site_with_options},
};
use serde_json::Value;
use std::{
    collections::BTreeSet,
    fs,
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};
static NEXT: AtomicUsize = AtomicUsize::new(0);
fn digest(title: &str) -> String {
    let hash = title
        .as_bytes()
        .iter()
        .fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
            (hash ^ u64::from(*byte)).wrapping_mul(0x0100_0000_01b3)
        });
    format!("{hash:016x}")
}
struct Fixture(PathBuf);
impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "lsg-search-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(root.join("garden/pages")).unwrap();
        Self(root)
    }
    fn write(&self, name: &str, source: &str) {
        fs::write(self.0.join("garden/pages").join(name), source).unwrap();
    }
    fn publish(&self, directory: &str, search: Option<SearchOptions>) -> PathBuf {
        let output = self.0.join(directory);
        publish_site_with_options(
            self.0.join("garden"),
            &output,
            FilenameFormat::TripleLowbar,
            &NamespaceSelection::new(&["Public"], &["Public/Excluded"]).unwrap(),
            &SiteOptions {
                search,
                ..SiteOptions::default()
            },
        )
        .unwrap();
        output
    }
    // The format version 5 files read back as the version 1 corpus view these
    // assertions use; `raw` holds every search file plus the destinations.
    fn corpus(output: &std::path::Path) -> (Value, String) {
        let manifest_path = output.join("lsg-search-manifest.json");
        let manifest_raw = fs::read_to_string(manifest_path).unwrap();
        let manifest: Value = serde_json::from_str(&manifest_raw).unwrap();
        assert_eq!(manifest["format_version"], 5);
        let read = |kind: &str, digest: &Value| {
            let raw = fs::read_to_string(output.join(format!(
                "lsg-search-{kind}-{}.json",
                digest.as_str().unwrap()
            )))
            .unwrap();
            let value: Value = serde_json::from_str(&raw).unwrap();
            assert_eq!(value["format_version"], 5);
            (value, raw)
        };
        let (titles, mut raw) = read("titles", &manifest["titles"]);
        raw.push_str(&manifest_raw);
        let pages: Vec<_> = titles["pages"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| {
                // A route the title gives is left out of the entry.
                let href = p
                    .get(2)
                    .cloned()
                    .unwrap_or_else(|| format!("p-{}.html", digest(p[0].as_str().unwrap())).into());
                let aliases = p.get(1).cloned().unwrap_or_else(|| serde_json::json!([]));
                serde_json::json!({"id": href, "title": p[0], "href": href, "aliases": aliases})
            })
            .collect();
        let mut content = Vec::new();
        for digest in manifest["chunks"].as_array().unwrap() {
            let (chunk, text) = read("chunk", digest);
            raw.push_str(&text);
            for b in chunk["blocks"].as_array().unwrap() {
                let href = format!("{}#{}", b[0].as_str().unwrap(), b[1].as_str().unwrap());
                content.push(serde_json::json!({"page": b[0], "href": href, "text": b[2], "ancestors": b[3]}));
            }
        }
        assert_eq!(manifest["blocks"].as_u64().unwrap() as usize, content.len());
        for digest in manifest["shards"].as_object().unwrap().values() {
            raw.push_str(&read("shard", digest).1);
        }
        for record in &content {
            raw.push_str(record["href"].as_str().unwrap());
        }
        let corpus = serde_json::json!({"revision": manifest["revision"], "pages": pages, "content": content});
        (corpus, raw)
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
#[test]
fn selected_originals_safe_projection_aliases_and_anchor_destinations() {
    let f = Fixture::new();
    let source = "alias:: Welcome, Shared\nsecret:: PROPERTY_SENTINEL\n- Parent **safe** [[Public/Target]] [[PrivateRefSentinel]] #PrivateTagSentinel\n  collapsed:: true\n\t- child needle `codeword`\n\t  id:: 12345678-1234-1234-1234-123456789abc\n- before {{embed [[Public/Target]]}} after\n- ((87654321-4321-4321-4321-cba987654321))\n- {{query PRIVATE_QUERY_SENTINEL}}\n- ![ASSET_LABEL_SENTINEL](../assets/ASSET_PATH_SENTINEL.png)\n- <b>INLINE_HTML_SENTINEL</b>\n- hidden placeholder\n  search-exclude:: true\n\t- EXCLUDED_SUBTREE_SENTINEL\n- visible sibling\n";
    f.write("Public.md", source);
    f.write(
        "Public___Target.md",
        "alias:: Shared\n- TARGET_ORIGINAL_SENTINEL\n  id:: 87654321-4321-4321-4321-cba987654321\n",
    );
    f.write("Public___Excluded.md", "- EXCLUDED_PAGE_SENTINEL\n");
    f.write(
        "Public___Private.md",
        "public:: false\n- PRIVATE_PAGE_SENTINEL\n",
    );
    f.write("PrivateRefSentinel.md", "- OUTSIDE_SELECTION_SENTINEL\n");
    let output = f.publish("site", Some(SearchOptions::default()));
    let (corpus, raw) = Fixture::corpus(&output);
    for sentinel in [
        "PROPERTY_SENTINEL",
        "PrivateRefSentinel",
        "PRIVATE_QUERY_SENTINEL",
        "ASSET_LABEL_SENTINEL",
        "ASSET_PATH_SENTINEL",
        "INLINE_HTML_SENTINEL",
        "hidden placeholder",
        "EXCLUDED_SUBTREE_SENTINEL",
        "EXCLUDED_PAGE_SENTINEL",
        "PRIVATE_PAGE_SENTINEL",
        "OUTSIDE_SELECTION_SENTINEL",
    ] {
        assert!(!raw.contains(sentinel), "leaked {sentinel}: {raw}");
    }
    assert_eq!(raw.matches("TARGET_ORIGINAL_SENTINEL").count(), 1);
    // A tag with no file names a reference-only page, which search finds by title, and the
    // tag's words stay in the referring block.
    let titles: Vec<&str> = corpus["pages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|page| page["title"].as_str().unwrap())
        .collect();
    assert_eq!(titles, ["PrivateTagSentinel", "Public", "Public/Target"]);
    assert!(
        corpus["content"]
            .as_array()
            .unwrap()
            .iter()
            .any(|r| r["text"].as_str().unwrap().contains("PrivateTagSentinel"))
    );
    assert!(raw.contains("Welcome") && !raw.contains("Shared"));
    let records = corpus["content"].as_array().unwrap();
    assert!(records.iter().any(|r| r["text"] == "before after"));
    let child = records
        .iter()
        .find(|r| r["text"] == "child needle codeword")
        .unwrap();
    assert!(
        child["href"]
            .as_str()
            .unwrap()
            .ends_with("#block-12345678-1234-1234-1234-123456789abc")
    );
    assert_eq!(
        child["ancestors"][0],
        "Parent safe Public/Target PrivateTagSentinel"
    );
    for record in records {
        let (route, fragment) = record["href"].as_str().unwrap().split_once('#').unwrap();
        let html = fs::read_to_string(output.join(route)).unwrap();
        assert!(
            html.contains(&format!("id=\"{fragment}\"")),
            "missing {fragment}"
        );
        assert!(html.contains("data-lsg-fragments"));
    }
    for page in corpus["pages"].as_array().unwrap() {
        let html = fs::read_to_string(output.join(page["href"].as_str().unwrap())).unwrap();
        assert!(
            html.contains("<link rel=\"lsg-search\" href=\"lsg-search-manifest.json\">")
                && !html.contains(corpus["revision"].as_str().unwrap())
        );
        assert!(!html.contains("src=\"lsg-search.mjs\""));
    }
    assert_eq!(
        fs::read_to_string(f.0.join("garden/pages/Public.md")).unwrap(),
        source
    );
    let repeated = f.publish("repeat", Some(SearchOptions::default()));
    assert_eq!(Fixture::corpus(&repeated).1, raw);
}
#[test]
fn body_exclusion_and_default_opt_out() {
    let f = Fixture::new();
    f.write("Public.md", "- BODY_SENTINEL\n");
    let output = f.publish("disabled", None);
    assert!(
        !fs::read_to_string(output.join("index.html"))
            .unwrap()
            .contains("rel=\"lsg-search\"")
    );
    let output = f.publish(
        "excluded",
        Some(SearchOptions {
            excluded_page_bodies: BTreeSet::from(["public".to_owned()]),
        }),
    );
    let (corpus, raw) = Fixture::corpus(&output);
    assert!(!raw.contains("BODY_SENTINEL"));
    assert_eq!(corpus["pages"].as_array().unwrap().len(), 1);
    assert_eq!(corpus["content"], serde_json::json!([]));
}
#[test]
fn duplicate_uuid_uses_offsets_and_large_original_outlines_have_real_destinations() {
    let f = Fixture::new();
    let mut source = "- first\n  id:: 12345678-1234-1234-1234-123456789abc\n- second\n  id:: 12345678-1234-1234-1234-123456789abc\n".to_owned();
    for n in 0..10_050 {
        source.push_str(&format!("- line {n}\n"));
    }
    f.write("Public.md", &source);
    let output = f.publish("site", Some(SearchOptions::default()));
    let (corpus, raw) = Fixture::corpus(&output);
    // The first block keeps the id, as in Logseq; the second is addressed by its offset.
    assert_eq!(raw.matches("#block-12345678").count(), 1);
    assert!(raw.contains("line 10049"));
    let records = corpus["content"].as_array().unwrap();
    let html =
        fs::read_to_string(output.join(corpus["pages"][0]["href"].as_str().unwrap())).unwrap();
    let ids: BTreeSet<_> = html
        .split("id=\"")
        .skip(1)
        .filter_map(|part| part.split('"').next())
        .collect();
    for record in records {
        let fragment = record["href"].as_str().unwrap().split_once('#').unwrap().1;
        assert!(ids.contains(fragment));
    }
}
#[test]
fn shared_files_and_their_links_survive_a_garden_edit() {
    let f = Fixture::new();
    f.write("Public.md", "- Welcome [[Public/Target]]\n");
    f.write("Public___Target.md", "- Target\n");
    let before = f.publish("before", Some(SearchOptions::default()));
    f.write("Public___Target.md", "- Target\n- One more block\n");
    let after = f.publish("after", Some(SearchOptions::default()));
    for name in [
        "lsg-site.css",
        "lsg-search.css",
        "lsg-theme.js",
        "lsg-search-boot.js",
        "lsg-shell.js",
    ] {
        assert_eq!(
            fs::read(before.join(name)).unwrap(),
            fs::read(after.join(name)).unwrap(),
            "{name}"
        );
    }
    // Every page names the shared files with the same tags; the search manifest's digest is
    // the only part of the head that follows the garden.
    let tags = |site: &std::path::Path| {
        let page = fs::read_to_string(site.join("index.html")).unwrap();
        let head = page[..page.find("</head>").unwrap()].to_owned();
        let manifest = head.find("<link rel=\"lsg-search\"").unwrap();
        let end = manifest + head[manifest..].find('>').unwrap() + 1;
        head[..manifest].to_owned() + &head[end..]
    };
    assert_eq!(tags(&before), tags(&after));
    let head = tags(&after);
    for tag in [
        "<link rel=\"stylesheet\" href=\"lsg-site.css\">",
        "<link rel=\"stylesheet\" href=\"lsg-search.css\">",
        "<script defer src=\"lsg-search-boot.js\"></script>",
        "<script defer src=\"lsg-theme.js\"></script>",
    ] {
        assert!(head.contains(tag), "{tag}");
    }
    assert!(!head.contains("<style") && !head.contains("data-lsg-search-start"));
}
// Every file of a published site by its path inside the site.
fn site_files(site: &std::path::Path) -> std::collections::BTreeMap<String, Vec<u8>> {
    let mut out = std::collections::BTreeMap::new();
    let mut stack = vec![site.to_path_buf()];
    while let Some(folder) = stack.pop() {
        for entry in fs::read_dir(folder).unwrap().flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else {
                let name = path
                    .strip_prefix(site)
                    .unwrap()
                    .to_string_lossy()
                    .into_owned();
                out.insert(name, fs::read(&path).unwrap());
            }
        }
    }
    out
}

// The shard key each shard file name holds, from a manifest.
fn shard_keys(manifest: &Value) -> std::collections::BTreeMap<String, String> {
    manifest["shards"]
        .as_object()
        .unwrap()
        .iter()
        .map(|(key, digest)| {
            (
                format!("lsg-search-shard-{}.json", digest.as_str().unwrap()),
                key.clone(),
            )
        })
        .collect()
}

#[test]
fn one_added_block_rewrites_only_its_page_and_the_search_files_that_hold_it() {
    let words = [
        "river", "mountain", "lantern", "harbor", "meadow", "copper", "willow", "ember", "glacier",
        "orchard", "falcon", "quartz", "thistle", "velvet", "canyon", "saffron",
    ];
    let f = Fixture::new();
    f.write("Public.md", "- Home of the public pages\n");
    for page in 0..40 {
        let mut source = String::new();
        for block in 0..12 {
            let a = words[(page + block) % words.len()];
            let b = words[(page * 3 + block * 5) % words.len()];
            source.push_str(&format!("- Note {page} {block} about {a} and {b}\n"));
        }
        f.write(&format!("Public___P{page:02}.md"), &source);
    }
    let before = f.publish("before", Some(SearchOptions::default()));
    let edited = fs::read_to_string(f.0.join("garden/pages/Public___P05.md")).unwrap();
    f.write(
        "Public___P05.md",
        &(edited + "- An added block for a churn test.\n"),
    );
    let after = f.publish("after", Some(SearchOptions::default()));
    let (old, new) = (site_files(&before), site_files(&after));
    let removed: Vec<_> = old.keys().filter(|name| !new.contains_key(*name)).collect();
    let written: Vec<_> = new
        .iter()
        .filter(|(name, contents)| old.get(*name) != Some(contents))
        .map(|(name, _)| name)
        .collect();
    let search =
        |name: &str| name.starts_with("lsg-search-shard-") || name.starts_with("lsg-search-chunk-");
    // Outside search data, only the edited page, the search manifest and the site manifest
    // (whose revision follows every file) change.
    let others: Vec<_> = written.iter().filter(|name| !search(name)).collect();
    assert_eq!(others.len(), 3, "{others:?}");
    assert!(
        others.iter().any(|name| name.as_str() == "lsg-site.json"),
        "{others:?}"
    );
    assert!(
        others
            .iter()
            .any(|name| name.as_str() == "lsg-search-manifest.json"),
        "{others:?}"
    );
    let page = others
        .iter()
        .find(|name| name.ends_with(".html"))
        .expect("the edited page");
    assert!(String::from_utf8_lossy(&new[page.as_str()]).contains("churn test"));
    // Every replaced search file held the edited page's records or one of the added words.
    assert!(removed.iter().all(|name| search(name)), "{removed:?}");
    assert!(!removed.is_empty() && removed.len() < 12, "{removed:?}");
    let added = ["an", "added", "block", "for", "a", "churn", "test"];
    let old_manifest: Value = serde_json::from_slice(&old["lsg-search-manifest.json"]).unwrap();
    for name in removed {
        let json: Value = serde_json::from_slice(&old[name]).unwrap();
        if name.starts_with("lsg-search-chunk-") {
            let holds_page = json["blocks"]
                .as_array()
                .unwrap()
                .iter()
                .any(|record| record[0].as_str() == Some(page.as_str()));
            assert!(holds_page, "{name} does not hold the edited page");
        } else {
            // The shard's key is the start of an added word: the shard holds edited text.
            let key = old_manifest["shards"]
                .as_object()
                .unwrap()
                .iter()
                .find(|(_, digest)| name.contains(digest.as_str().unwrap()))
                .map(|(key, _)| key.clone())
                .unwrap();
            assert!(
                added.iter().any(|word| word.starts_with(&key)),
                "{name} ({key}) holds none of the added words"
            );
        }
    }
}

#[test]
fn one_added_page_changes_only_the_shards_of_its_words() {
    let words = [
        "river", "mountain", "lantern", "harbor", "meadow", "copper", "willow", "ember", "glacier",
        "orchard", "falcon", "quartz", "thistle", "velvet", "canyon", "saffron",
    ];
    let f = Fixture::new();
    f.write("Public.md", "- Home of the public pages\n");
    for page in 0..40 {
        let mut source = String::new();
        for block in 0..12 {
            let a = words[(page + block) % words.len()];
            let b = words[(page * 3 + block * 5) % words.len()];
            source.push_str(&format!("- Note {page} {block} about {a} and {b}\n"));
        }
        f.write(&format!("Public___P{page:02}.md"), &source);
    }
    let before = f.publish("before", Some(SearchOptions::default()));
    // The new page sorts between existing pages by title, so every later page's number in
    // title order moves; the shards name pages by slot instead.
    f.write("Public___P05a.md", "- An added page about zymurgy.\n");
    let after = f.publish("after", Some(SearchOptions::default()));
    let (old, new) = (site_files(&before), site_files(&after));
    let keys = shard_keys(&serde_json::from_slice(&old["lsg-search-manifest.json"]).unwrap());
    let added = ["an", "added", "page", "about", "zymurgy"];
    let removed: Vec<_> = keys
        .iter()
        .filter(|(name, _)| !new.contains_key(*name))
        .map(|(_, key)| key)
        .collect();
    assert!(!removed.is_empty(), "the added words change their shards");
    for key in removed {
        assert!(
            added.iter().any(|word| word.starts_with(key.as_str())),
            "shard {key} holds none of the added words"
        );
    }
    // Every shard that holds none of the added words keeps its name and bytes.
    for (name, key) in &keys {
        if !added.iter().any(|word| word.starts_with(key.as_str())) {
            assert_eq!(old.get(name), new.get(name), "shard {key} changed");
        }
    }
}
