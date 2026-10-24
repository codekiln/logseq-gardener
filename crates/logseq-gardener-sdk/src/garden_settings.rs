//! Garden-wide settings from `logseq/config.edn` that shape navigation.
//!
//! ```
//! use logseq_gardener_sdk::garden_settings::GardenSettings;
//!
//! let (settings, diagnostics) = GardenSettings::parse(
//!     r#"{:favorites ["tmux" "Lazygit"] :feature/enable-flashcards? false}"#,
//! );
//! assert_eq!(settings.favorites, ["tmux", "Lazygit"]);
//! assert!(!settings.features.flashcards && settings.features.journals);
//! assert!(diagnostics.is_empty());
//! ```

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::garden_config::{self, Form};
use crate::page_names::{PageNameIndex, lookup_key};

/// Logseq features a garden can turn off with `:feature/enable-*?` keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FeatureFlags {
    /// `:feature/enable-journals?`
    pub journals: bool,
    /// `:feature/enable-flashcards?`
    pub flashcards: bool,
    /// `:feature/enable-whiteboards?`
    pub whiteboards: bool,
}

impl Default for FeatureFlags {
    fn default() -> Self {
        Self {
            journals: true,
            flashcards: true,
            whiteboards: true,
        }
    }
}

/// Favorites and feature flags, in the order and with the defaults Logseq uses.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GardenSettings {
    /// Raw `:favorites` entries in config order.
    pub favorites: Vec<String>,
    /// Features the garden enables.
    pub features: FeatureFlags,
}

/// A favorite that names a published page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Favorite<T> {
    /// The page's identifier in the lookup, such as its output route.
    pub page: T,
    /// The favorite's label: the page title or alias as the author spelled it.
    pub title: String,
}

impl GardenSettings {
    /// Read settings from a garden root. A missing config gives the defaults.
    ///
    /// Unreadable or malformed settings fall back to defaults with diagnostics.
    pub fn read(root: impl AsRef<Path>) -> (Self, Vec<String>) {
        match garden_config::read_text(root.as_ref()) {
            Ok(None) => (Self::default(), Vec::new()),
            Ok(Some(text)) => Self::parse(&text),
            Err(reason) => (
                Self::default(),
                vec![format!("Navigation settings use defaults: {reason}")],
            ),
        }
    }

    /// Parse settings from `config.edn` text without evaluating it.
    pub fn parse(text: &str) -> (Self, Vec<String>) {
        let mut settings = Self::default();
        let pairs = match garden_config::top_level(text) {
            Ok(pairs) => pairs,
            Err(reason) => {
                return (
                    settings,
                    vec![format!("Navigation settings use defaults: {reason}")],
                );
            }
        };
        let mut diagnostics = Vec::new();
        for (key, value) in pairs {
            let Form::Atom(key) = key else {
                continue;
            };
            let flag = match key.as_str() {
                ":feature/enable-journals?" => &mut settings.features.journals,
                ":feature/enable-flashcards?" => &mut settings.features.flashcards,
                ":feature/enable-whiteboards?" => &mut settings.features.whiteboards,
                ":favorites" => {
                    let entries = match value {
                        Form::Collection(items) => items
                            .into_iter()
                            .map(|item| match item {
                                Form::Str(name) => Some(name),
                                _ => None,
                            })
                            .collect::<Option<Vec<_>>>(),
                        _ => None,
                    };
                    match entries {
                        Some(entries) => settings.favorites = entries,
                        None => diagnostics
                            .push("Favorites must be a vector of page names; showing none".into()),
                    }
                    continue;
                }
                _ => continue,
            };
            match value {
                Form::Atom(ref v) if v == "true" => *flag = true,
                Form::Atom(ref v) if v == "false" => *flag = false,
                _ => {
                    *flag = true;
                    diagnostics.push(format!(
                        "Feature setting {key} is not a boolean; using true"
                    ));
                }
            }
        }
        (settings, diagnostics)
    }
}

/// Resolve favorites through page titles and aliases, keeping config order.
///
/// `spellings` maps each title's and alias's [`lookup_key`] to the author's spelling,
/// which labels the favorite as Logseq does. A name listed twice keeps its first
/// position. Entries that name no page in the lookup return a diagnostic.
pub fn resolve_favorites<T: Clone + Ord>(
    favorites: &[String],
    lookup: &PageNameIndex<T>,
    spellings: &BTreeMap<String, String>,
) -> (Vec<Favorite<T>>, Vec<String>) {
    let mut seen = BTreeSet::new();
    let mut resolved = Vec::new();
    let mut diagnostics = Vec::new();
    for name in favorites {
        let key = lookup_key(name);
        if !seen.insert(key.clone()) {
            continue;
        }
        match lookup.candidates(name) {
            [page] => resolved.push(Favorite {
                page: page.clone(),
                title: spellings.get(&key).cloned().unwrap_or_else(|| name.clone()),
            }),
            [] => diagnostics.push(format!(
                "Favorite {name:?} names no published page; it is left out of the sidebar"
            )),
            _ => diagnostics.push(format!(
                "Favorite {name:?} names more than one published page; it is left out of the sidebar"
            )),
        }
    }
    (resolved, diagnostics)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_favorites_in_order_and_feature_flags() {
        let (settings, diagnostics) = GardenSettings::parse(
            r#"{:favorites ["tmux" "say \"hi\"" "Lazygit"]
               :feature/enable-journals? false
               ;; :feature/enable-flashcards? false
               :feature/enable-whiteboards? false}"#,
        );
        assert!(diagnostics.is_empty());
        assert_eq!(settings.favorites, ["tmux", "say \"hi\"", "Lazygit"]);
        assert_eq!(
            settings.features,
            FeatureFlags {
                journals: false,
                flashcards: true,
                whiteboards: false
            }
        );
    }

    #[test]
    fn malformed_values_use_defaults_with_diagnostics() {
        let (settings, diagnostics) =
            GardenSettings::parse(r#"{:favorites ["ok" :keyword] :feature/enable-journals? 1}"#);
        assert_eq!(settings, GardenSettings::default());
        assert_eq!(diagnostics.len(), 2);
        let (settings, diagnostics) = GardenSettings::parse("[1 2]");
        assert_eq!(settings, GardenSettings::default());
        assert!(diagnostics[0].contains("EDN map"));
        assert_eq!(GardenSettings::parse("{}").0, GardenSettings::default());
    }

    #[test]
    fn missing_config_gives_defaults() {
        let root = std::env::temp_dir().join(format!("lsg-settings-{}", std::process::id()));
        assert_eq!(
            GardenSettings::read(&root),
            (GardenSettings::default(), Vec::new())
        );
    }
}
