//! `lsg-site.json`, plugin files, per-page plugin metadata and the browser host.
//!
//! Manifests are read and checked by [`crate::plugins`]; this module only writes output.
use std::collections::BTreeMap;

use serde_json::{Map, Value, json};

use crate::plugins::PluginManifest;

/// The site manifest, written by every build at the output root.
pub(super) const SITE_MANIFEST: &str = "lsg-site.json";
/// The site manifest's format version.
const SITE_API_VERSION: u64 = 1;

/// One generated page as `lsg-site.json` and the page metadata describe it.
pub(super) struct SitePage {
    /// Logseq's lower-cased page name; `None` for generated lists that are not pages.
    pub id: Option<String>,
    pub title: String,
    pub url: String,
    /// `page`, `journal`, `namespace`, `home` or `list`.
    pub kind: &'static str,
    /// The page route whose visible properties supply plugin values.
    pub properties: Option<String>,
}

/// A digest of every generated file, so a host can tell builds apart. `files` must be in a
/// fixed order.
pub(super) fn revision<'a>(files: impl IntoIterator<Item = (&'a str, &'a [u8])>) -> String {
    let mut hash = 0xcbf2_9ce4_8422_2325_u64;
    let mut feed = |bytes: &[u8]| {
        for byte in bytes {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x0100_0000_01b3);
        }
    };
    for (path, contents) in files {
        feed(path.as_bytes());
        feed(&[0]);
        feed(&(contents.len() as u64).to_le_bytes());
        feed(contents);
    }
    format!("{hash:016x}")
}

/// `lsg-site.json`: pages and assets sorted by URL, plugins by id.
pub(super) fn site_manifest(
    pages: &[SitePage],
    assets: impl IntoIterator<Item = String>,
    plugins: &[PluginManifest],
    revision: &str,
) -> String {
    let mut pages: Vec<&SitePage> = pages.iter().collect();
    pages.sort_by(|left, right| left.url.cmp(&right.url));
    let mut assets: Vec<String> = assets.into_iter().collect();
    assets.sort();
    let mut manifest = json!({
        "apiVersion": SITE_API_VERSION,
        "revision": revision,
        "pages": pages.iter().map(|page| json!({
            "id": page.id,
            "title": page.title,
            "url": page.url,
            "kind": page.kind,
        })).collect::<Vec<_>>(),
        "assets": assets,
    });
    if plugins.iter().any(|plugin| plugin.module.is_some()) {
        manifest["plugins"] = plugin_list(plugins);
    }
    // Escaped like the page metadata, so no title reads as markup if the file is shown as HTML.
    script_json(&serde_json::to_string(&manifest).expect("JSON values serialize")) + "\n"
}

/// Entries for the plugins with a browser module, with output-relative URLs, shared by
/// `lsg-site.json` and the host.
fn plugin_list(plugins: &[PluginManifest]) -> Value {
    Value::Array(
        plugins
            .iter()
            .filter_map(|plugin| {
                Some(json!({
                    "id": plugin.id,
                    "version": plugin.version,
                    "module": plugin.output_path(plugin.module.as_ref()?),
                    "styles": plugin.styles.iter().map(|style| plugin.output_path(style)).collect::<Vec<_>>(),
                    "activation": plugin.activation?.as_str(),
                }))
            })
            .collect(),
    )
}

/// The browser host with the plugin list built in, and its content-addressed route. `readable`
/// selects the variant for readable page folders.
pub(super) fn host(plugins: &[PluginManifest], readable: bool) -> (String, String) {
    let list = serde_json::to_string(&plugin_list(plugins)).expect("JSON values serialize");
    let source = if readable {
        super::readable::host(include_str!("host.mjs"))
    } else {
        include_str!("host.mjs").to_owned()
    };
    let script = super::references::minify(&source).replace("__LSG_PLUGINS__", &script_json(&list));
    let route = format!("lsg-host-{}.mjs", crate::search::digest(&script));
    (route, script)
}

/// The tags added before `</head>` of a page when plugins with a browser module are configured:
/// the page metadata and a module that imports the host after first paint, as the search loader
/// does. `contributed` holds the values build plugins added for this page, by plugin id.
pub(super) fn page_head(
    page: &SitePage,
    revision: &str,
    plugins: &[PluginManifest],
    visible: &[(String, String)],
    contributed: Option<&BTreeMap<String, Map<String, Value>>>,
    host_route: &str,
) -> String {
    let mut entries = Map::new();
    for plugin in plugins.iter().filter(|plugin| plugin.module.is_some()) {
        let mut values: Map<String, Value> = plugin
            .page_values(
                visible
                    .iter()
                    .map(|(name, text)| (name.as_str(), text.as_str())),
            )
            .into_iter()
            .map(|(name, text)| (name, Value::String(text)))
            .collect();
        if let Some(added) = contributed.and_then(|contributed| contributed.get(&plugin.id)) {
            values.extend(added.clone());
        }
        if !values.is_empty() {
            entries.insert(plugin.id.clone(), Value::Object(values));
        }
    }
    let metadata = json!({
        "id": page.id,
        "title": page.title,
        "url": page.url,
        "revision": revision,
        "plugins": entries,
    });
    let metadata = script_json(&serde_json::to_string(&metadata).expect("JSON values serialize"));
    format!(
        "<script type=\"application/json\" data-lsg-page>{metadata}</script><script type=\"module\" data-lsg-host>requestAnimationFrame(()=>requestAnimationFrame(()=>import('./{host_route}')))</script>"
    )
}

/// JSON that is safe inside a `<script>` element: `<` cannot start `</script>` or `<!--`.
fn script_json(json: &str) -> String {
    json.replace('<', "\\u003c")
}

/// Every plugin file with its output path.
pub(super) fn files(plugins: &[PluginManifest]) -> BTreeMap<String, &[u8]> {
    let mut files = BTreeMap::new();
    for plugin in plugins {
        for file in plugin.module.iter().chain(&plugin.styles) {
            files.insert(plugin.output_path(file), file.contents.as_slice());
        }
    }
    files
}
