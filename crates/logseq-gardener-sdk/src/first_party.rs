//! The plugins that ship inside `lsg`, which a garden turns on with `:lsg/plugins` in its
//! `logseq/config.edn`.
//!
//! Each plugin is one [`FirstParty`] variant, so no code past reading the key can hold an
//! unknown plugin name. A variant gives the plugin's manifest, whose module and stylesheets are
//! compiled into the SDK from `plugins/<id>/`, and its build step. The manifest passes the same
//! checks as a `--plugin` manifest ([`crate::plugins`]) and reports the `lsg` version as its
//! own. A build step chooses its pages by their authored properties and `logseq-entity::` types
//! and contributes page metadata for its browser module; nothing names a garden's pages.
//!
//! ```
//! use logseq_gardener_sdk::first_party::FirstParty;
//!
//! let plugin = FirstParty::from_id("podcast-episode").unwrap();
//! let manifest = plugin.manifest();
//! assert_eq!(manifest.version, logseq_gardener_sdk::VERSION);
//! assert!(manifest.module.is_some());
//! ```
use std::path::{Path, PathBuf};
use std::sync::Arc;

use serde_json::{Map, Value};

use crate::contributions::{BuildPlugin, Contribution};
use crate::entity_types::has_entity_type;
use crate::garden::GardenDocument;
use crate::page_names::lookup_key;
use crate::plugins::{PluginFile, PluginManifest, PluginSource, check_manifest, check_relative};
use crate::publication::PluginView;

/// A plugin that ships inside `lsg`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FirstParty {
    /// `podcast-episode`: a player that keeps an episode's recording playing across pages, and
    /// dialogue lines that seek it.
    PodcastEpisode,
}

/// One file of a plugin's folder, compiled in.
struct Embedded {
    path: &'static str,
    contents: &'static [u8],
}

macro_rules! embedded {
    ($id:literal, $($file:literal),+) => {
        &[$(Embedded {
            path: $file,
            contents: include_bytes!(concat!("../plugins/", $id, "/", $file)),
        }),+]
    };
}

const PODCAST_EPISODE: &[Embedded] = embedded!(
    "podcast-episode",
    "lsg-plugin.json",
    "podcast-episode.mjs",
    "podcast-episode.css"
);

impl FirstParty {
    /// Every first-party plugin, in id order.
    pub const ALL: [Self; 1] = [Self::PodcastEpisode];

    /// The plugin's id, which is also its `:lsg/plugins` keyword without the colon.
    #[must_use]
    pub fn id(self) -> &'static str {
        match self {
            Self::PodcastEpisode => "podcast-episode",
        }
    }

    /// The plugin with this id.
    #[must_use]
    pub fn from_id(id: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|plugin| plugin.id() == id)
    }

    /// What the plugin does, in one line for help text.
    #[must_use]
    pub fn summary(self) -> &'static str {
        match self {
            Self::PodcastEpisode => {
                "keeps a podcast episode's recording playing across pages; dialogue timestamps seek it"
            }
        }
    }

    fn files(self) -> &'static [Embedded] {
        match self {
            Self::PodcastEpisode => PODCAST_EPISODE,
        }
    }

    /// The plugin's checked manifest with its module and stylesheets, and the `lsg` version as
    /// its version.
    ///
    /// # Panics
    ///
    /// Panics when an embedded manifest fails the manifest checks; a unit test runs every
    /// embedded manifest through them.
    #[must_use]
    pub fn manifest(self) -> PluginManifest {
        let files = self.files();
        let text = files
            .iter()
            .find(|file| file.path == "lsg-plugin.json")
            .map(|file| file.contents)
            .expect("every first-party plugin has a manifest");
        let mut value: Value =
            serde_json::from_slice(text).expect("embedded manifests are JSON objects");
        value["version"] = Value::String(crate::VERSION.to_owned());
        let read = |relative: &str, extensions: &[&str]| {
            check_relative(relative, extensions)?;
            files
                .iter()
                .find(|file| file.path == relative)
                .map(|file| PluginFile {
                    path: relative.to_owned(),
                    contents: file.contents.to_vec(),
                })
                .ok_or_else(|| format!("path {relative:?} is not a file of this plugin"))
        };
        let manifest = check_manifest(&value.to_string(), PluginSource::FirstParty(self), &read)
            .unwrap_or_else(|problem| panic!("first-party plugin {}: {problem}", self.id()));
        assert_eq!(
            manifest.id,
            self.id(),
            "a first-party manifest has its own id"
        );
        PluginManifest {
            build: Some(crate::plugins::BuildStep::InProcess(self)),
            ..manifest
        }
    }

    /// The plugin's build step for one publication, with the diagnostics it found while
    /// choosing its pages.
    #[must_use]
    pub fn build_step(self, inputs: &FirstPartyInputs<'_>) -> FirstPartyStep {
        match self {
            Self::PodcastEpisode => podcast_episode(inputs),
        }
    }
}

/// What a first-party build step reads from a publication besides its view: the published
/// pages' authored properties and outlines, how names resolve, and where assets are published.
pub struct FirstPartyInputs<'a> {
    /// Every published page and journal as `(title, route, document)`.
    pub pages: &'a [(String, String, &'a GardenDocument)],
    /// The identity a written page name or alias selects, compared to match entity types.
    pub resolve: &'a dyn Fn(&str) -> String,
    /// The published URL, relative to the site root, of a local asset that a document links
    /// to, given the document's garden-relative path and the link's destination; `None` when
    /// the link names no publishable asset.
    pub asset_url: &'a dyn Fn(&Path, &str) -> Option<String>,
}

/// A first-party build step and the diagnostics found while preparing it.
pub struct FirstPartyStep {
    /// The step, run with the other build plugins.
    pub plugin: Arc<dyn BuildPlugin>,
    /// `(garden-relative source, message)` for each page the plugin could not serve.
    pub diagnostics: Vec<(PathBuf, String)>,
}

/// A build step whose metadata is decided while it is prepared: it returns one metadata
/// contribution for each page it chose that is in the view.
struct PreparedMetadata {
    id: &'static str,
    entries: Vec<(String, Map<String, Value>)>,
}

impl BuildPlugin for PreparedMetadata {
    fn id(&self) -> &str {
        self.id
    }
    fn contribute(&self, view: &PluginView<'_>) -> Result<Vec<Contribution>, String> {
        Ok(self
            .entries
            .iter()
            .filter(|(page_id, _)| view.publication().page(page_id).is_some())
            .map(|(page_id, values)| Contribution::Metadata {
                page_id: page_id.clone(),
                values: values.clone(),
            })
            .collect())
    }
}

/// `podcast-episode`: each published page typed [`crate::podcast::EPISODE_TYPE`] gets
/// `recording` (a URL relative to the site root, or an `https` URL) and `title`. A typed page
/// without a recording gets no entry and one diagnostic.
fn podcast_episode(inputs: &FirstPartyInputs<'_>) -> FirstPartyStep {
    let mut entries = Vec::new();
    let mut diagnostics = Vec::new();
    for (title, _, document) in inputs.pages {
        let properties = crate::query::leading_properties(document);
        if !has_entity_type(&properties, crate::podcast::EPISODE_TYPE, inputs.resolve) {
            continue;
        }
        let source = &document.relative_path;
        let Some(recording) = crate::podcast::recording(&document.parsed.blocks) else {
            diagnostics.push((
                source.clone(),
                format!(
                    "podcast-episode: the page is a {} and has no audio embed under its first heading, so it has no player",
                    crate::podcast::EPISODE_TYPE
                ),
            ));
            continue;
        };
        let destination = &recording.destination;
        let url = if is_https(destination) {
            Some(destination.clone())
        } else if destination.contains("://") {
            None
        } else {
            (inputs.asset_url)(source, destination)
        };
        let Some(url) = url else {
            diagnostics.push((
                source.clone(),
                format!(
                    "podcast-episode: the recording {destination} is neither an https URL nor a published asset, so the page has no player"
                ),
            ));
            continue;
        };
        let mut values = Map::new();
        values.insert("recording".to_owned(), Value::String(url));
        values.insert("title".to_owned(), Value::String(recording.title));
        entries.push((lookup_key(title), values));
    }
    FirstPartyStep {
        plugin: Arc::new(PreparedMetadata {
            id: FirstParty::PodcastEpisode.id(),
            entries,
        }),
        diagnostics,
    }
}

fn is_https(destination: &str) -> bool {
    destination
        .get(..8)
        .is_some_and(|scheme| scheme.eq_ignore_ascii_case("https://"))
        && !destination
            .chars()
            .any(|c| c.is_control() || c.is_whitespace())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plugins::{Activation, BuildStep};

    #[test]
    fn every_embedded_manifest_passes_the_manifest_checks() {
        for plugin in FirstParty::ALL {
            let manifest = plugin.manifest();
            assert_eq!(manifest.id, plugin.id());
            assert_eq!(manifest.version, crate::VERSION);
            assert_eq!(manifest.activation, Some(Activation::OnDemand));
            assert!(manifest.page_properties.is_empty());
            assert_eq!(manifest.build, Some(BuildStep::InProcess(plugin)));
            assert_eq!(manifest.source, PluginSource::FirstParty(plugin));
            let module = manifest.module.as_ref().unwrap();
            assert_eq!(module.path, format!("{}.mjs", plugin.id()));
            assert!(!module.contents.is_empty());
            assert_eq!(
                manifest
                    .styles
                    .iter()
                    .map(|style| style.path.as_str())
                    .collect::<Vec<_>>(),
                [format!("{}.css", plugin.id())]
            );
        }
    }

    #[test]
    fn ids_round_trip_and_are_ordered() {
        let mut sorted = FirstParty::ALL;
        sorted.sort_by_key(|plugin| plugin.id());
        assert_eq!(sorted, FirstParty::ALL);
        for plugin in FirstParty::ALL {
            assert_eq!(FirstParty::from_id(plugin.id()), Some(plugin));
            assert!(crate::plugins::valid_id(plugin.id()));
        }
        assert_eq!(FirstParty::from_id("podcast-epsiode"), None);
    }

    /// `scripts/parity/budgets.toml` holds a `[plugins.<id>]` table with the four rows for every
    /// shipped plugin, so each new plugin is measured the same way.
    #[test]
    fn every_shipped_plugin_has_its_budget_rows() {
        const ROWS: [&str; 4] = [
            "script_styles_gzip_kb",
            "page_growth_gzip_kb",
            "phone_heap_growth_mb",
            "build_ms_per_page",
        ];
        let budgets = include_str!("../../../scripts/parity/budgets.toml");
        for plugin in FirstParty::ALL {
            let header = format!("[plugins.{}]", plugin.id());
            let table = budgets
                .split_once(&format!("\n{header}\n"))
                .map(|(_, rest)| rest.split("\n[").next().unwrap_or_default())
                .unwrap_or_else(|| {
                    panic!("scripts/parity/budgets.toml has no {header} table for first-party plugin {}", plugin.id())
                });
            for row in ROWS {
                let present = table.lines().any(|line| {
                    line.split_once('=').is_some_and(|(key, value)| {
                        key.trim() == row
                            && value
                                .split('#')
                                .next()
                                .unwrap_or_default()
                                .trim()
                                .parse::<f64>()
                                .is_ok()
                    })
                });
                assert!(
                    present,
                    "scripts/parity/budgets.toml {header} lacks a numeric {row} for first-party plugin {}",
                    plugin.id()
                );
            }
        }
    }

    #[test]
    fn https_recognition() {
        assert!(is_https("https://example.com/a.mp3"));
        assert!(is_https("HTTPS://example.com/a.mp3"));
        assert!(!is_https("http://example.com/a.mp3"));
        assert!(!is_https("../assets/a.mp3"));
        assert!(!is_https("https://example.com/a b.mp3"));
    }
}
