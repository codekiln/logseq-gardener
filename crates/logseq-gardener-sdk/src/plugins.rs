//! Site plugin manifests: reading, checking and the page values a plugin may read.
//!
//! A plugin manifest is a JSON file that names an ES module, optional stylesheets, when the
//! browser host loads the module and which page properties the plugin reads. Every caller
//! (the CLI, the publisher, an editor integration) checks manifests through
//! [`read_manifests`], so each one accepts and rejects the same files.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::path::{Component, Path, PathBuf};

use serde_json::Value;

/// The host API major version this build supports.
pub const API_VERSION: u64 = 1;

/// When the browser host imports a plugin's module.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Activation {
    /// On every page, after first paint.
    AfterLoad,
    /// On the first page whose metadata has an entry for the plugin.
    OnDemand,
}

impl Activation {
    /// The manifest spelling: `after-load` or `on-demand`.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AfterLoad => "after-load",
            Self::OnDemand => "on-demand",
        }
    }
}

/// A module or stylesheet named by a manifest, read when the manifest was checked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginFile {
    /// The path as written in the manifest, relative to the manifest's folder, `/`-separated.
    pub path: String,
    /// The file's bytes.
    pub contents: Vec<u8>,
}

/// A checked plugin manifest with its files read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginManifest {
    /// The manifest file the caller named.
    pub source: PathBuf,
    /// Lower-case letters, digits and single hyphens; unique within a publication.
    pub id: String,
    /// The plugin's own version string.
    pub version: String,
    /// The ES module the host imports.
    pub module: PluginFile,
    /// Stylesheets the host adds once, in manifest order.
    pub styles: Vec<PluginFile>,
    /// When the host imports the module.
    pub activation: Activation,
    /// Page property names the plugin reads, lower-cased, in manifest order without repeats.
    pub page_properties: Vec<String>,
}

impl PluginManifest {
    /// The output folder for this plugin's files, relative to the site root.
    #[must_use]
    pub fn folder(&self) -> String {
        format!("plugins/{}", self.id)
    }

    /// The output path of a module or stylesheet, relative to the site root.
    #[must_use]
    pub fn output_path(&self, file: &PluginFile) -> String {
        format!("{}/{}", self.folder(), file.path)
    }

    /// The values of the named page properties among `visible`, the properties a page shows
    /// readers as `(lower-cased name, authored text)`. The result is empty when the page shows
    /// none of them, so a caller can leave the plugin out of that page's metadata.
    #[must_use]
    pub fn page_values<'a>(
        &self,
        visible: impl IntoIterator<Item = (&'a str, &'a str)>,
    ) -> BTreeMap<String, String> {
        let wanted: BTreeSet<&str> = self.page_properties.iter().map(String::as_str).collect();
        let mut values = BTreeMap::new();
        for (name, text) in visible {
            if wanted.contains(name) {
                values
                    .entry(name.to_owned())
                    .or_insert_with(|| text.trim().to_owned());
            }
        }
        values
    }
}

/// A manifest that cannot be used, naming the manifest and the problem.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PluginError {
    /// The manifest file, as the caller named it.
    pub source: PathBuf,
    /// What is wrong, for a person to fix.
    pub problem: String,
}

impl fmt::Display for PluginError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "plugin manifest {}: {}",
            self.source.display(),
            self.problem
        )
    }
}

impl std::error::Error for PluginError {}

/// Read and check every manifest, then order them by id.
///
/// # Errors
///
/// Returns the first manifest that fails [`read_manifest`], or a duplicate id naming both
/// manifest paths. Nothing is written.
pub fn read_manifests(paths: &[PathBuf]) -> Result<Vec<PluginManifest>, PluginError> {
    let mut manifests = paths
        .iter()
        .map(|path| read_manifest(path))
        .collect::<Result<Vec<_>, _>>()?;
    manifests.sort_by(|left, right| left.id.cmp(&right.id).then(left.source.cmp(&right.source)));
    for pair in manifests.windows(2) {
        if pair[0].id == pair[1].id {
            return Err(PluginError {
                source: pair[1].source.clone(),
                problem: format!(
                    "id {:?} is also declared by {}",
                    pair[1].id,
                    pair[0].source.display()
                ),
            });
        }
    }
    Ok(manifests)
}

const FIELDS: [&str; 7] = [
    "id",
    "apiVersion",
    "version",
    "module",
    "styles",
    "activation",
    "pageProperties",
];

/// Read one manifest and the files it names.
///
/// The manifest must be a JSON object with exactly the fields `id`, `apiVersion`, `version`,
/// `module`, `activation` and the optional `styles` and `pageProperties`. Module and stylesheet
/// paths are relative to the manifest's folder, use `/`, contain no `..`, pass through no
/// symlink and name regular files; the module ends in `.js` or `.mjs` and stylesheets in `.css`.
///
/// # Errors
///
/// Returns the manifest path and the first problem found.
pub fn read_manifest(path: &Path) -> Result<PluginManifest, PluginError> {
    let fail = |problem: String| PluginError {
        source: path.to_owned(),
        problem,
    };
    let text = fs::read_to_string(path).map_err(|error| fail(error.to_string()))?;
    let value: Value =
        serde_json::from_str(&text).map_err(|error| fail(format!("invalid JSON: {error}")))?;
    let Value::Object(fields) = value else {
        return Err(fail("expected a JSON object".to_owned()));
    };
    if let Some(unknown) = fields.keys().find(|key| !FIELDS.contains(&key.as_str())) {
        return Err(fail(format!(
            "unknown field {unknown:?}; expected {}",
            FIELDS.join(", ")
        )));
    }
    let string = |name: &str| match fields.get(name) {
        Some(Value::String(text)) => Ok(text.clone()),
        Some(_) => Err(fail(format!("{name} must be a string"))),
        None => Err(fail(format!("missing field {name}"))),
    };
    let strings = |name: &str| match fields.get(name) {
        None => Ok(Vec::new()),
        Some(Value::Array(items)) => items
            .iter()
            .map(|item| match item {
                Value::String(text) => Ok(text.clone()),
                _ => Err(fail(format!("{name} must be a list of strings"))),
            })
            .collect(),
        Some(_) => Err(fail(format!("{name} must be a list of strings"))),
    };
    match fields.get("apiVersion") {
        None => return Err(fail("missing field apiVersion".to_owned())),
        Some(Value::Number(number)) if number.as_u64() == Some(API_VERSION) => {}
        Some(other) => {
            return Err(fail(format!(
                "unsupported apiVersion {other}; this lsg supports apiVersion {API_VERSION}"
            )));
        }
    }
    let id = string("id")?;
    if !valid_id(&id) {
        return Err(fail(format!(
            "id {id:?} must be 1 to 64 lower-case letters, digits and single hyphens, starting and ending with a letter or digit"
        )));
    }
    let version = string("version")?;
    if version.trim().is_empty() || version.chars().any(char::is_control) {
        return Err(fail(
            "version must be a nonempty string without control characters".to_owned(),
        ));
    }
    let activation = match string("activation")?.as_str() {
        "after-load" => Activation::AfterLoad,
        "on-demand" => Activation::OnDemand,
        other => {
            return Err(fail(format!(
                "unknown activation {other:?}; expected \"after-load\" or \"on-demand\""
            )));
        }
    };
    let mut page_properties = Vec::new();
    for name in strings("pageProperties")? {
        let normalized = name.to_lowercase();
        if normalized.is_empty()
            || normalized
                .chars()
                .any(|c| c.is_whitespace() || c.is_control() || c == ':')
        {
            return Err(fail(format!(
                "page property {name:?} must be a property name without spaces or colons"
            )));
        }
        if !page_properties.contains(&normalized) {
            page_properties.push(normalized);
        }
    }
    let folder = path.parent().unwrap_or(Path::new(""));
    let module = string("module")?;
    let module = plugin_file(folder, &module, &[".js", ".mjs"]).map_err(fail)?;
    let mut styles = Vec::new();
    let mut seen = BTreeSet::from([module.path.to_lowercase()]);
    for style in strings("styles")? {
        let file = plugin_file(folder, &style, &[".css"]).map_err(fail)?;
        // Compare without case: two names that differ only in case are one file on macOS.
        if !seen.insert(file.path.to_lowercase()) {
            return Err(fail(format!(
                "{style:?} collides with another file of this plugin in plugins/{id}/"
            )));
        }
        styles.push(file);
    }
    Ok(PluginManifest {
        source: path.to_owned(),
        id,
        version,
        module,
        styles,
        activation,
        page_properties,
    })
}

fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
        && !id.starts_with('-')
        && !id.ends_with('-')
        && !id.contains("--")
}

fn plugin_file(folder: &Path, relative: &str, extensions: &[&str]) -> Result<PluginFile, String> {
    let shown = format!("{relative:?}");
    if relative.is_empty() || relative.contains('\\') || relative.chars().any(char::is_control) {
        return Err(format!(
            "path {shown} must be a nonempty /-separated path without backslashes"
        ));
    }
    let candidate = Path::new(relative);
    if candidate.is_absolute() || relative.starts_with('/') {
        return Err(format!(
            "path {shown} must be relative to the manifest's folder"
        ));
    }
    if relative
        .split('/')
        .any(|segment| segment.is_empty() || segment == "." || segment == "..")
        || candidate
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err(format!(
            "path {shown} must stay inside the manifest's folder: no empty, . or .. segments"
        ));
    }
    if !extensions
        .iter()
        .any(|extension| relative.to_lowercase().ends_with(extension))
    {
        return Err(format!(
            "path {shown} must end in {}",
            extensions.join(" or ")
        ));
    }
    let mut current = folder.to_path_buf();
    for segment in relative.split('/') {
        current.push(segment);
        let metadata =
            fs::symlink_metadata(&current).map_err(|error| format!("path {shown}: {error}"))?;
        if metadata.file_type().is_symlink() {
            return Err(format!("path {shown} passes through a symlink"));
        }
    }
    if !fs::symlink_metadata(&current)
        .map_err(|error| format!("path {shown}: {error}"))?
        .is_file()
    {
        return Err(format!("path {shown} is not a regular file"));
    }
    let contents = fs::read(&current).map_err(|error| format!("path {shown}: {error}"))?;
    Ok(PluginFile {
        path: relative.to_owned(),
        contents,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT: AtomicUsize = AtomicUsize::new(0);

    struct Folder(PathBuf);
    impl Folder {
        fn new() -> Self {
            let root = std::env::temp_dir().join(format!(
                "lsg-plugins-{}-{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir_all(root.join("player")).unwrap();
            fs::write(root.join("player/player.mjs"), "export default {};\n").unwrap();
            fs::write(root.join("player/player.css"), "audio{}\n").unwrap();
            Self(root)
        }
        fn manifest(&self, name: &str, json: &str) -> PathBuf {
            let path = self.0.join("player").join(name);
            fs::write(&path, json).unwrap();
            path
        }
    }
    impl Drop for Folder {
        fn drop(&mut self) {
            fs::remove_dir_all(&self.0).unwrap();
        }
    }

    const VALID: &str = r#"{"id":"player","apiVersion":1,"version":"1.2.0","module":"player.mjs","styles":["player.css"],"activation":"after-load","pageProperties":["Recording","recording"]}"#;

    fn error(folder: &Folder, json: &str) -> String {
        let path = folder.manifest("lsg-plugin.json", json);
        let error = read_manifest(&path).unwrap_err();
        assert_eq!(error.source, path);
        assert!(error.to_string().contains("lsg-plugin.json"), "{error}");
        error.problem
    }

    #[test]
    fn valid_manifest_reads_files_and_normalizes_properties() {
        let folder = Folder::new();
        let manifest = read_manifest(&folder.manifest("lsg-plugin.json", VALID)).unwrap();
        assert_eq!(manifest.id, "player");
        assert_eq!(manifest.version, "1.2.0");
        assert_eq!(manifest.activation, Activation::AfterLoad);
        assert_eq!(manifest.module.contents, b"export default {};\n");
        assert_eq!(
            manifest.output_path(&manifest.styles[0]),
            "plugins/player/player.css"
        );
        assert_eq!(manifest.page_properties, ["recording"]);
    }

    #[test]
    fn rejects_unsupported_api_version_naming_the_supported_one() {
        let folder = Folder::new();
        let problem = error(
            &folder,
            &VALID.replace("\"apiVersion\":1", "\"apiVersion\":2"),
        );
        assert!(problem.contains("unsupported apiVersion 2"), "{problem}");
        assert!(problem.contains("supports apiVersion 1"), "{problem}");
        let problem = error(
            &folder,
            &VALID.replace("\"apiVersion\":1", "\"apiVersion\":\"1\""),
        );
        assert!(problem.contains("unsupported apiVersion"), "{problem}");
    }

    #[test]
    fn rejects_malformed_ids_and_missing_or_unknown_fields() {
        let folder = Folder::new();
        for id in [
            "", "Player", "-player", "player-", "pl--ayer", "pl_ayer", "pl ayer",
        ] {
            let problem = error(&folder, &VALID.replace("\"player\"", &format!("{id:?}")));
            assert!(problem.contains("must be 1 to 64"), "{id}: {problem}");
        }
        assert!(
            error(&folder, &VALID.replace("\"version\":\"1.2.0\",", ""))
                .contains("missing field version")
        );
        assert!(
            error(&folder, &VALID.replace("\"id\":\"player\",", "")).contains("missing field id")
        );
        assert!(
            error(&folder, &VALID.replace("\"apiVersion\":1,", ""))
                .contains("missing field apiVersion")
        );
        assert!(
            error(&folder, &VALID.replace("{", "{\"extra\":1,"))
                .contains("unknown field \"extra\"")
        );
        assert!(error(&folder, "[]").contains("expected a JSON object"));
        assert!(error(&folder, "{").contains("invalid JSON"));
    }

    #[test]
    fn rejects_unknown_activation() {
        let folder = Folder::new();
        let problem = error(&folder, &VALID.replace("after-load", "eager"));
        assert!(
            problem.contains("unknown activation \"eager\""),
            "{problem}"
        );
    }

    #[test]
    fn rejects_escaping_absolute_and_odd_paths() {
        let folder = Folder::new();
        fs::write(folder.0.join("outside.mjs"), "").unwrap();
        for (path, expected) in [
            ("../outside.mjs", "stay inside"),
            ("./player.mjs", "stay inside"),
            ("a//player.mjs", "stay inside"),
            ("/tmp/player.mjs", "relative"),
            ("sub\\player.mjs", "backslashes"),
            ("", "nonempty"),
            ("player.css", "must end in .js or .mjs"),
            ("missing.mjs", "missing.mjs"),
        ] {
            let problem = error(
                &folder,
                &VALID.replace("\"player.mjs\"", &format!("{path:?}")),
            );
            assert!(problem.contains(expected), "{path}: {problem}");
            if !path.is_empty() {
                assert!(problem.contains(&format!("{path:?}")), "{path}: {problem}");
            }
        }
    }

    #[test]
    fn rejects_directories_and_symlinks() {
        let folder = Folder::new();
        fs::create_dir(folder.0.join("player/dir.css")).unwrap();
        let problem = error(&folder, &VALID.replace("\"player.css\"", "\"dir.css\""));
        assert!(problem.contains("not a regular file"), "{problem}");
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(
                folder.0.join("player/player.mjs"),
                folder.0.join("player/link.mjs"),
            )
            .unwrap();
            let problem = error(&folder, &VALID.replace("\"player.mjs\"", "\"link.mjs\""));
            assert!(problem.contains("symlink"), "{problem}");
            std::os::unix::fs::symlink(folder.0.join("player"), folder.0.join("player/linked"))
                .unwrap();
            let problem = error(
                &folder,
                &VALID.replace("\"player.mjs\"", "\"linked/player.mjs\""),
            );
            assert!(problem.contains("symlink"), "{problem}");
        }
    }

    #[test]
    fn rejects_colliding_files_within_a_plugin() {
        let folder = Folder::new();
        let problem = error(
            &folder,
            &VALID.replace("[\"player.css\"]", "[\"player.css\",\"PLAYER.css\"]"),
        );
        assert!(problem.contains("collides"), "{problem}");
    }

    #[test]
    fn orders_by_id_and_rejects_duplicates_naming_both_manifests() {
        let folder = Folder::new();
        let zeta = folder.manifest("zeta.json", &VALID.replace("\"player\"", "\"zeta\""));
        let alpha = folder.manifest("alpha.json", &VALID.replace("\"player\"", "\"alpha\""));
        let ids = |paths: &[PathBuf]| {
            read_manifests(paths)
                .unwrap()
                .into_iter()
                .map(|manifest| manifest.id)
                .collect::<Vec<_>>()
        };
        assert_eq!(ids(&[zeta.clone(), alpha.clone()]), ["alpha", "zeta"]);
        assert_eq!(ids(&[alpha.clone(), zeta.clone()]), ["alpha", "zeta"]);
        let copy = folder.manifest("copy.json", &VALID.replace("\"player\"", "\"zeta\""));
        let error = read_manifests(&[zeta.clone(), alpha, copy.clone()]).unwrap_err();
        let message = error.to_string();
        assert!(
            message.contains("copy.json") && message.contains("zeta.json"),
            "{message}"
        );
    }

    #[test]
    fn page_values_keep_only_named_visible_properties() {
        let folder = Folder::new();
        let manifest = read_manifest(&folder.manifest("lsg-plugin.json", VALID)).unwrap();
        let values = manifest.page_values([
            ("type", "episode"),
            ("recording", " https://example.com/a.mp3 "),
        ]);
        assert_eq!(
            values,
            BTreeMap::from([(
                "recording".to_owned(),
                "https://example.com/a.mp3".to_owned()
            )])
        );
        assert!(manifest.page_values([("type", "episode")]).is_empty());
    }
}
