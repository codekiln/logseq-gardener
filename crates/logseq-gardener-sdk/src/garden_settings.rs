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
use crate::graph::{ForceSettings, GraphSettings};
use crate::journals::{DEFAULT_FILE_NAME_FORMAT, DEFAULT_TITLE_FORMAT, DateFormat};
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

/// Favorites, feature flags and journal formats, with the defaults Logseq uses.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GardenSettings {
    /// Raw `:favorites` entries in config order.
    pub favorites: Vec<String>,
    /// Features the garden enables.
    pub features: FeatureFlags,
    /// `:journal/page-title-format`, a supported [`DateFormat`] pattern.
    pub journal_title_format: String,
    /// `:journal/file-name-format`, a supported [`DateFormat`] pattern.
    pub journal_file_name_format: String,
    /// `:ui/show-brackets?`: page references show `[[` and `]]` unless this is `false`.
    pub show_brackets: bool,
    /// `:default-home {:sidebar …}`: the right sidebar's cards on a reader's first visit.
    pub sidebar: Vec<String>,
    /// `:graph/settings`: which pages the graph view draws at first.
    pub graph: GraphSettings,
    /// `:graph/forcesettings`: the graph layout's forces.
    pub forces: ForceSettings,
    /// `g g` opens the graph view unless `:shortcuts {:go/graph-view false}`.
    pub graph_shortcut: bool,
}

impl Default for GardenSettings {
    fn default() -> Self {
        Self {
            favorites: Vec::new(),
            features: FeatureFlags::default(),
            journal_title_format: DEFAULT_TITLE_FORMAT.to_owned(),
            journal_file_name_format: DEFAULT_FILE_NAME_FORMAT.to_owned(),
            show_brackets: true,
            sidebar: Vec::new(),
            graph: GraphSettings::default(),
            forces: ForceSettings::default(),
            graph_shortcut: true,
        }
    }
}

/// A right sidebar card named by `:default-home {:sidebar …}`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SidebarCard<T> {
    /// `"Contents"`: the Contents card.
    Contents,
    /// A page card, by the page's identifier in the lookup.
    Page(T),
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
                ":ui/show-brackets?" => {
                    match value {
                        Form::Atom(ref v) if v == "true" => settings.show_brackets = true,
                        Form::Atom(ref v) if v == "false" => settings.show_brackets = false,
                        _ => {
                            settings.show_brackets = true;
                            diagnostics.push(format!(
                                "UI setting {key} is not a boolean; showing brackets"
                            ));
                        }
                    }
                    continue;
                }
                ":journal/page-title-format" | ":journal/file-name-format" => {
                    let (target, default) = if key == ":journal/page-title-format" {
                        (&mut settings.journal_title_format, DEFAULT_TITLE_FORMAT)
                    } else {
                        (
                            &mut settings.journal_file_name_format,
                            DEFAULT_FILE_NAME_FORMAT,
                        )
                    };
                    match value {
                        Form::Str(pattern) => match DateFormat::parse(&pattern) {
                            Ok(_) => *target = pattern,
                            Err(error) => diagnostics.push(format!(
                                "Journal setting {key} has an {error}; using {default:?}"
                            )),
                        },
                        _ => diagnostics.push(format!(
                            "Journal setting {key} is not a string; using {default:?}"
                        )),
                    }
                    continue;
                }
                ":default-home" => {
                    let Form::Map(entries) = value else {
                        continue;
                    };
                    for (key, value) in entries {
                        if !matches!(key, Form::Atom(ref k) if k == ":sidebar") {
                            continue;
                        }
                        let names = match value {
                            Form::Str(name) => Some(vec![name]),
                            Form::Collection(items) => items
                                .into_iter()
                                .map(|item| match item {
                                    Form::Str(name) => Some(name),
                                    _ => None,
                                })
                                .collect(),
                            _ => None,
                        };
                        match names {
                            Some(names) => settings.sidebar = names,
                            None => diagnostics.push(
                                "Default sidebar must be a page name or a vector of page names; showing none"
                                    .into(),
                            ),
                        }
                    }
                    continue;
                }
                ":graph/settings" => {
                    read_graph_settings(value, &mut settings.graph, &mut diagnostics);
                    continue;
                }
                ":graph/forcesettings" => {
                    read_force_settings(value, &mut settings.forces, &mut diagnostics);
                    continue;
                }
                ":shortcuts" => {
                    if let Form::Map(entries) = value {
                        for (key, value) in entries {
                            if matches!(key, Form::Atom(ref k) if k == ":go/graph-view") {
                                // Logseq turns a shortcut off with `false`; a key string keeps it.
                                settings.graph_shortcut = !matches!(value, Form::Atom(ref v) if v == "false" || v == "nil");
                            }
                        }
                    }
                    continue;
                }
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

/// Read `:graph/settings`; an entry that is not a boolean keeps its default.
fn read_graph_settings(value: Form, graph: &mut GraphSettings, diagnostics: &mut Vec<String>) {
    let Form::Map(entries) = value else {
        diagnostics.push("Graph setting :graph/settings is not a map; using defaults".into());
        return;
    };
    for (key, value) in entries {
        let Form::Atom(key) = key else { continue };
        let flag = match key.as_str() {
            ":journal?" => &mut graph.journals,
            ":orphan-pages?" => &mut graph.orphan_pages,
            ":builtin-pages?" => &mut graph.builtin_pages,
            ":excluded-pages?" => &mut graph.excluded_pages,
            _ => continue,
        };
        match value {
            Form::Atom(ref v) if v == "true" => *flag = true,
            Form::Atom(ref v) if v == "false" || v == "nil" => *flag = false,
            _ => diagnostics.push(format!(
                "Graph setting {key} is not a boolean; using {}",
                *flag
            )),
        }
    }
}

/// Read `:graph/forcesettings`; an entry that is not a whole number keeps its default.
fn read_force_settings(value: Form, forces: &mut ForceSettings, diagnostics: &mut Vec<String>) {
    let Form::Map(entries) = value else {
        diagnostics.push("Graph setting :graph/forcesettings is not a map; using defaults".into());
        return;
    };
    for (key, value) in entries {
        let Form::Atom(key) = key else { continue };
        let number = match key.as_str() {
            ":link-dist" => &mut forces.link_distance,
            ":charge-strength" => &mut forces.charge_strength,
            ":charge-range" => &mut forces.charge_range,
            _ => continue,
        };
        match value {
            Form::Atom(ref v) if v.parse::<i32>().is_ok_and(|n| n.abs() <= 100_000) => {
                *number = v.parse().expect("checked above");
            }
            _ => diagnostics.push(format!(
                "Graph setting {key} is not a whole number; using {}",
                *number
            )),
        }
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

/// Resolve `:default-home {:sidebar …}` names, keeping config order.
///
/// `"Contents"`, in any case, is the Contents card; other names resolve through page
/// titles and aliases like favorites. A name listed twice keeps its first position.
/// Entries that name no page in the lookup return a diagnostic.
pub fn resolve_sidebar<T: Clone + Ord>(
    names: &[String],
    lookup: &PageNameIndex<T>,
) -> (Vec<SidebarCard<T>>, Vec<String>) {
    let mut seen = BTreeSet::new();
    let mut resolved = Vec::new();
    let mut diagnostics = Vec::new();
    for name in names {
        let key = lookup_key(name);
        if !seen.insert(key.clone()) {
            continue;
        }
        if key == "contents" {
            resolved.push(SidebarCard::Contents);
            continue;
        }
        match lookup.candidates(name) {
            [page] => resolved.push(SidebarCard::Page(page.clone())),
            [] => diagnostics.push(format!(
                "Default sidebar page {name:?} names no published page; it is left out of the right sidebar"
            )),
            _ => diagnostics.push(format!(
                "Default sidebar page {name:?} names more than one published page; it is left out of the right sidebar"
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
    fn reads_journal_formats_with_fallbacks() {
        let (settings, diagnostics) = GardenSettings::parse(
            r#"{:journal/page-title-format "yyyy-MM-dd EEE" :journal/file-name-format "yyyy_MM_dd"}"#,
        );
        assert!(diagnostics.is_empty());
        assert_eq!(settings.journal_title_format, "yyyy-MM-dd EEE");
        let (settings, diagnostics) = GardenSettings::parse(
            r#"{:journal/page-title-format "yyyy QQQ" :journal/file-name-format :yyyy}"#,
        );
        assert_eq!(settings, GardenSettings::default());
        assert!(diagnostics[0].contains(":journal/page-title-format"));
        assert!(diagnostics[1].contains(":journal/file-name-format"));
    }

    #[test]
    fn reads_bracket_setting() {
        assert!(GardenSettings::parse("{}").0.show_brackets);
        assert!(
            GardenSettings::parse("{;; :ui/show-brackets? false\n}")
                .0
                .show_brackets
        );
        let (settings, diagnostics) = GardenSettings::parse("{:ui/show-brackets? false}");
        assert!(!settings.show_brackets && diagnostics.is_empty());
        assert!(
            GardenSettings::parse("{:ui/show-brackets? true}")
                .0
                .show_brackets
        );
        let (settings, diagnostics) = GardenSettings::parse(r#"{:ui/show-brackets? "no"}"#);
        assert!(settings.show_brackets);
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].contains(":ui/show-brackets?"));
    }

    #[test]
    fn reads_default_sidebar_as_string_or_vector() {
        let (settings, diagnostics) =
            GardenSettings::parse(r#"{:default-home {:page "x" :sidebar ["Contents" "tmux"]}}"#);
        assert!(diagnostics.is_empty());
        assert_eq!(settings.sidebar, ["Contents", "tmux"]);
        let (settings, _) = GardenSettings::parse(r#"{:default-home {:sidebar "Contents"}}"#);
        assert_eq!(settings.sidebar, ["Contents"]);
        assert!(
            GardenSettings::parse(
                r#"{;; :default-home {:sidebar "Contents"}
            }"#
            )
            .0
            .sidebar
            .is_empty()
        );
        let (settings, diagnostics) = GardenSettings::parse(r#"{:default-home {:sidebar 1}}"#);
        assert!(settings.sidebar.is_empty());
        assert_eq!(diagnostics.len(), 1);
    }

    #[test]
    fn reads_graph_settings_forces_and_the_graph_shortcut() {
        let (settings, diagnostics) = GardenSettings::parse("{}");
        assert_eq!(settings.graph, GraphSettings::default());
        assert!(!settings.graph.journals && !settings.graph.orphan_pages);
        assert_eq!(
            (
                settings.forces.link_distance,
                settings.forces.charge_strength,
                settings.forces.charge_range
            ),
            (70, -600, 600)
        );
        assert!(settings.graph_shortcut && diagnostics.is_empty());
        let (settings, diagnostics) = GardenSettings::parse(
            "{:graph/settings {:journal? true :orphan-pages? true :builtin-pages? false :excluded-pages? true}
              :graph/forcesettings {:link-dist 120 :charge-strength -300 :charge-range 900}
              :shortcuts {:go/graph-view false :go/home \"g h\"}}",
        );
        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        assert!(settings.graph.journals && settings.graph.orphan_pages);
        assert!(!settings.graph.builtin_pages && settings.graph.excluded_pages);
        assert_eq!(
            (
                settings.forces.link_distance,
                settings.forces.charge_strength,
                settings.forces.charge_range
            ),
            (120, -300, 900)
        );
        assert!(!settings.graph_shortcut);
    }

    #[test]
    fn malformed_graph_settings_keep_defaults_with_diagnostics() {
        let (settings, diagnostics) = GardenSettings::parse(
            "{:graph/settings {:journal? \"yes\"} :graph/forcesettings {:link-dist 1.5} :shortcuts {:go/graph-view \"g g\"}}",
        );
        assert!(!settings.graph.journals);
        assert_eq!(settings.forces.link_distance, 70);
        assert!(settings.graph_shortcut);
        assert_eq!(
            diagnostics,
            [
                "Graph setting :journal? is not a boolean; using false",
                "Graph setting :link-dist is not a whole number; using 70",
            ]
        );
        let (_, diagnostics) = GardenSettings::parse("{:graph/settings [true]}");
        assert_eq!(
            diagnostics,
            ["Graph setting :graph/settings is not a map; using defaults"]
        );
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
