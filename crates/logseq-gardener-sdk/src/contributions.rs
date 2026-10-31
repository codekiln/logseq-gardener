//! Build plugins and their contributions: the types, checking them against the view and
//! resolving their links to routes and anchors.
//!
//! A build plugin receives a [`PluginView`] and returns [`Contribution`]s. [`resolve`] checks
//! every contribution against the [`PublicationView`] and turns page and block ids into URLs
//! and fragments, so any caller (the publisher, an editor integration) accepts and rejects the
//! same contributions. The publisher only renders what [`resolve`] returns.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::publication::{BlockAnchor, PluginView, PublicationView};

/// The contribution format version.
pub const CONTRIBUTIONS_API_VERSION: u64 = 1;

/// The largest per-page metadata entry of one plugin, as compact JSON, in bytes.
pub const METADATA_LIMIT: usize = 64_000;

/// Code that adds derived rows and page metadata to a publication at build time.
///
/// The publisher calls [`contribute`](Self::contribute) once per build, on its own thread,
/// while it builds the reference index and search records. Every plugin sees the same view and
/// none of the others' contributions.
pub trait BuildPlugin: Send + Sync {
    /// Lower-case letters, digits and single hyphens; unique among a build's plugins,
    /// including plugin manifests.
    fn id(&self) -> &str;
    /// Page property names, lower-cased, whose visible values the view carries.
    fn page_properties(&self) -> &[String] {
        &[]
    }
    /// Whether the view carries blocks.
    fn wants_blocks(&self) -> bool {
        false
    }
    /// The plugin's contributions for this view, or a message for the person running the build.
    ///
    /// # Errors
    ///
    /// Returns why the plugin could not contribute; the build stops with it.
    fn contribute(&self, view: &PluginView<'_>) -> Result<Vec<Contribution>, String>;
}

/// In-process build plugins for [`crate::site::SiteOptions::build_plugins`].
#[derive(Clone, Default)]
pub struct BuildPlugins(pub Vec<Arc<dyn BuildPlugin>>);

impl BuildPlugins {
    /// Add a plugin.
    pub fn push(&mut self, plugin: impl BuildPlugin + 'static) {
        self.0.push(Arc::new(plugin));
    }

    /// Whether there are no plugins.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl fmt::Debug for BuildPlugins {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list()
            .entries(self.0.iter().map(|plugin| plugin.id().to_owned()))
            .finish()
    }
}

impl PartialEq for BuildPlugins {
    fn eq(&self, other: &Self) -> bool {
        self.0.len() == other.0.len()
            && self
                .0
                .iter()
                .zip(&other.0)
                .all(|(left, right)| Arc::ptr_eq(left, right))
    }
}

impl Eq for BuildPlugins {}

/// Something a build plugin adds to the publication.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase", deny_unknown_fields)]
pub enum Contribution {
    /// Derived rows under a block, or at the end of a page's outline when `block_id` is `None`.
    Rows {
        #[serde(rename = "pageId")]
        page_id: String,
        #[serde(rename = "blockId", default, skip_serializing_if = "Option::is_none")]
        block_id: Option<String>,
        rows: Vec<Row>,
    },
    /// Values added to the plugin's entry in a page's metadata, for its browser module.
    Metadata {
        #[serde(rename = "pageId")]
        page_id: String,
        values: Map<String, Value>,
    },
}

/// One derived row: inline content and child rows.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Row {
    pub content: Vec<InlineNode>,
    #[serde(default)]
    pub children: Vec<Row>,
}

/// A piece of a row's content. Text is shown as written, never parsed as Markdown or HTML.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum InlineNode {
    /// A link to a block of a view page.
    Block(BlockLink),
    /// A link to a view page.
    Page(PageLink),
    /// A link to an `http`, `https` or `mailto` URL.
    Url(UrlLink),
    /// Plain text.
    Text(Text),
}

/// `{"text"}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Text {
    pub text: String,
}

/// `{"page", "label"?}`: shows the page's title unless a label is given.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PageLink {
    pub page: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

/// `{"page", "block", "label"?}`: shows the page's title unless a label is given.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlockLink {
    pub page: String,
    pub block: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
}

/// `{"url", "label"}`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UrlLink {
    pub url: String,
    pub label: String,
}

/// A rejected contribution or a failed plugin, naming the plugin.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContributionError {
    /// The plugin's id.
    pub plugin: String,
    /// The contribution's index in the plugin's list, when one contribution is at fault.
    pub position: Option<usize>,
    /// What is wrong, for a person to fix.
    pub problem: String,
}

impl fmt::Display for ContributionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "build plugin {}: ", self.plugin)?;
        if let Some(position) = self.position {
            write!(f, "contributions[{position}]: ")?;
        }
        f.write_str(&self.problem)
    }
}

impl std::error::Error for ContributionError {}

/// One plugin's contributions with what checking them needs to know about the plugin.
pub struct PluginOutput<'a> {
    pub id: &'a str,
    /// Whether the plugin has a browser module; metadata needs one.
    pub has_module: bool,
    pub page_properties: &'a [String],
    /// Whether the plugin's view carried blocks.
    pub blocks: bool,
    pub contributions: Vec<Contribution>,
}

/// Derived rows placed on a page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlacedRows {
    pub plugin: String,
    /// Byte offset of the target block's bullet in the page's source; `None` for the page end.
    pub block: Option<usize>,
    pub rows: Vec<ResolvedRow>,
}

/// A row whose links are resolved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedRow {
    pub content: Vec<ResolvedNode>,
    pub children: Vec<ResolvedRow>,
}

/// Row content ready to render.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResolvedNode {
    Text(String),
    /// A link to a page of the site, relative to the output root, with an optional fragment.
    Page {
        href: String,
        label: String,
    },
    /// A link outside the site.
    External {
        href: String,
        label: String,
    },
}

/// Every contribution of a build, checked and resolved.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Resolved {
    /// Derived rows by page URL, in plugin id order, then in each plugin's order.
    pub rows: BTreeMap<String, Vec<PlacedRows>>,
    /// Source anchors that pages must emit for block links, by page URL.
    pub anchors: BTreeMap<String, BTreeSet<usize>>,
    /// Contributed metadata by page URL, then plugin id.
    pub metadata: BTreeMap<String, BTreeMap<String, Map<String, Value>>>,
}

impl Resolved {
    /// Whether no plugin contributed anything.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty() && self.metadata.is_empty()
    }
}

/// Check every plugin's contributions against `view` and resolve their links.
///
/// Plugins are applied in id order, then contributions in the order each plugin returned them.
///
/// # Errors
///
/// Returns the first plugin, contribution and problem found: a page or block outside the
/// plugin's view, a link to one, an unsupported URL scheme, an empty row list or row, metadata
/// from a plugin without a module, a metadata key that repeats one of the plugin's page
/// properties or another of its contributions on that page, or a page entry over
/// [`METADATA_LIMIT`].
pub fn resolve(
    view: &PublicationView,
    plugins: &mut [PluginOutput<'_>],
) -> Result<Resolved, ContributionError> {
    plugins.sort_by(|left, right| left.id.cmp(right.id));
    let mut resolved = Resolved::default();
    for plugin in plugins.iter() {
        let mut values: BTreeMap<String, Map<String, Value>> = BTreeMap::new();
        for (position, contribution) in plugin.contributions.iter().enumerate() {
            let fail = |problem: String| ContributionError {
                plugin: plugin.id.to_owned(),
                position: Some(position),
                problem,
            };
            match contribution {
                Contribution::Rows {
                    page_id,
                    block_id,
                    rows,
                } => {
                    let page = view
                        .page(page_id)
                        .ok_or_else(|| fail(format!("page {page_id:?} is not in the view")))?;
                    let block = match block_id {
                        None => None,
                        Some(block_id) => {
                            let block = plugin
                                .blocks
                                .then(|| view.block(page_id, block_id))
                                .flatten()
                                .ok_or_else(|| {
                                    fail(format!(
                                        "block {block_id:?} of page {page_id:?} is outside the plugin's view"
                                    ))
                                })?;
                            Some(block.start.ok_or_else(|| {
                                fail(format!(
                                    "block {block_id:?} of page {page_id:?} has no position to place rows at"
                                ))
                            })?)
                        }
                    };
                    if rows.is_empty() {
                        return Err(fail("rows must not be empty".to_owned()));
                    }
                    let rows = rows
                        .iter()
                        .map(|row| resolve_row(view, plugin.blocks, row, &mut resolved.anchors))
                        .collect::<Result<Vec<_>, _>>()
                        .map_err(fail)?;
                    resolved
                        .rows
                        .entry(page.url.clone())
                        .or_default()
                        .push(PlacedRows {
                            plugin: plugin.id.to_owned(),
                            block,
                            rows,
                        });
                }
                Contribution::Metadata {
                    page_id,
                    values: added,
                } => {
                    if !plugin.has_module {
                        return Err(fail(
                            "metadata needs a browser module, and this plugin has none".to_owned(),
                        ));
                    }
                    let page = view
                        .page(page_id)
                        .ok_or_else(|| fail(format!("page {page_id:?} is not in the view")))?;
                    let entry = values.entry(page.url.clone()).or_default();
                    for (key, value) in added {
                        if plugin.page_properties.contains(key) {
                            return Err(fail(format!(
                                "metadata key {key:?} repeats one of the plugin's pageProperties"
                            )));
                        }
                        if entry.insert(key.clone(), value.clone()).is_some() {
                            return Err(fail(format!(
                                "metadata key {key:?} is set twice for page {page_id:?}"
                            )));
                        }
                    }
                    let mut whole: Map<String, Value> = view
                        .page_values(page, plugin.page_properties)
                        .into_iter()
                        .map(|(name, text)| (name, Value::String(text)))
                        .collect();
                    whole.extend(entry.clone());
                    let size = serde_json::to_string(&whole)
                        .expect("JSON values serialize")
                        .len();
                    if size > METADATA_LIMIT {
                        return Err(fail(format!(
                            "metadata for page {page_id:?} is {size} bytes of JSON, over {METADATA_LIMIT}"
                        )));
                    }
                }
            }
        }
        for (url, entry) in values {
            resolved
                .metadata
                .entry(url)
                .or_default()
                .insert(plugin.id.to_owned(), entry);
        }
    }
    Ok(resolved)
}

fn resolve_row(
    view: &PublicationView,
    blocks: bool,
    row: &Row,
    anchors: &mut BTreeMap<String, BTreeSet<usize>>,
) -> Result<ResolvedRow, String> {
    if row.content.is_empty() {
        return Err("a row must have content".to_owned());
    }
    let mut content = Vec::with_capacity(row.content.len());
    for node in &row.content {
        content.push(match node {
            InlineNode::Text(Text { text }) => ResolvedNode::Text(text.clone()),
            InlineNode::Page(link) => {
                let page = view.page(&link.page).ok_or_else(|| {
                    format!("link to page {:?}, which is not in the view", link.page)
                })?;
                ResolvedNode::Page {
                    href: page.url.clone(),
                    label: link.label.clone().unwrap_or_else(|| page.title.clone()),
                }
            }
            InlineNode::Block(link) => {
                let page = view.page(&link.page).ok_or_else(|| {
                    format!("link to page {:?}, which is not in the view", link.page)
                })?;
                let block = blocks
                    .then(|| view.block(&link.page, &link.block))
                    .flatten()
                    .ok_or_else(|| {
                        format!(
                            "link to block {:?} of page {:?}, which is outside the plugin's view",
                            link.block, link.page
                        )
                    })?;
                if let BlockAnchor::Source(start) = block.anchor() {
                    anchors.entry(page.url.clone()).or_default().insert(*start);
                }
                ResolvedNode::Page {
                    href: match block.fragment() {
                        Some(fragment) => format!("{}#{fragment}", page.url),
                        None => page.url.clone(),
                    },
                    label: link.label.clone().unwrap_or_else(|| page.title.clone()),
                }
            }
            InlineNode::Url(UrlLink { url, label }) => {
                let scheme = url
                    .split_once(':')
                    .map(|(scheme, _)| scheme.to_ascii_lowercase());
                if !matches!(scheme.as_deref(), Some("http" | "https" | "mailto"))
                    || url.chars().any(char::is_control)
                {
                    return Err(format!(
                        "URL {url:?} must use the http, https or mailto scheme"
                    ));
                }
                ResolvedNode::External {
                    href: url.clone(),
                    label: label.clone(),
                }
            }
        });
    }
    let children = row
        .children
        .iter()
        .map(|child| resolve_row(view, blocks, child, anchors))
        .collect::<Result<_, _>>()?;
    Ok(ResolvedRow { content, children })
}

/// Read a process plugin's answer: `{"apiVersion": 1, "contributions": […]}`.
///
/// # Errors
///
/// Returns why the answer is not that object, naming a malformed contribution's index.
pub fn parse_response(bytes: &[u8]) -> Result<Vec<Contribution>, String> {
    let value: Value =
        serde_json::from_slice(bytes).map_err(|error| format!("output is not JSON: {error}"))?;
    let Value::Object(mut fields) = value else {
        return Err("output must be a JSON object".to_owned());
    };
    if let Some(unknown) = fields
        .keys()
        .find(|key| !matches!(key.as_str(), "apiVersion" | "contributions"))
    {
        return Err(format!(
            "output has unknown field {unknown:?}; expected apiVersion and contributions"
        ));
    }
    match fields.get("apiVersion") {
        Some(Value::Number(number)) if number.as_u64() == Some(CONTRIBUTIONS_API_VERSION) => {}
        Some(other) => {
            return Err(format!(
                "output has apiVersion {other}; this lsg supports apiVersion {CONTRIBUTIONS_API_VERSION}"
            ));
        }
        None => return Err("output is missing apiVersion".to_owned()),
    }
    let Some(Value::Array(items)) = fields.remove("contributions") else {
        return Err("output must have a contributions list".to_owned());
    };
    items
        .into_iter()
        .enumerate()
        .map(|(position, item)| {
            serde_json::from_value(item)
                .map_err(|error| format!("contributions[{position}]: malformed: {error}"))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_both_kinds_and_every_node() {
        let contributions = parse_response(
            br#"{"apiVersion":1,"contributions":[
                {"kind":"rows","pageId":"home","blockId":"4","rows":[{"content":[{"page":"episode/1"},{"text":" - 42 min"},{"page":"a","block":"1.2","label":"x"},{"url":"https://example.com","label":"site"}],"children":[]}]},
                {"kind":"metadata","pageId":"episode/1","values":{"chapters":[]}}]}"#,
        )
        .unwrap();
        let Contribution::Rows { rows, block_id, .. } = &contributions[0] else {
            panic!("rows first");
        };
        assert_eq!(block_id.as_deref(), Some("4"));
        assert!(matches!(rows[0].content[0], InlineNode::Page(_)));
        assert!(matches!(rows[0].content[1], InlineNode::Text(_)));
        assert!(matches!(rows[0].content[2], InlineNode::Block(_)));
        assert!(matches!(rows[0].content[3], InlineNode::Url(_)));
        assert!(matches!(contributions[1], Contribution::Metadata { .. }));
    }

    fn view() -> PublicationView {
        let source = "recording:: https://example.com/a.mp3\n\n- Intro\n";
        let document = crate::garden::GardenDocument {
            relative_path: "pages/Episode.md".into(),
            kind: crate::garden::DocumentKind::Page,
            source: source.to_owned(),
            parsed: lsdoc::parse_to_projection(source),
        };
        let visible = vec![(
            "recording".to_owned(),
            "https://example.com/a.mp3".to_owned(),
        )];
        PublicationView::from_parts(
            &[("Episode".to_owned(), "p-e.html".to_owned(), &document)],
            &BTreeMap::new(),
            &BTreeMap::new(),
            |_| visible.as_slice(),
            true,
        )
    }

    fn metadata_error(values: &[Value], has_module: bool) -> String {
        let properties = ["recording".to_owned()];
        let mut plugins = [PluginOutput {
            id: "player",
            has_module,
            page_properties: &properties,
            blocks: true,
            contributions: values
                .iter()
                .map(|values| Contribution::Metadata {
                    page_id: "episode".to_owned(),
                    values: values.as_object().unwrap().clone(),
                })
                .collect(),
        }];
        resolve(&view(), &mut plugins).unwrap_err().to_string()
    }

    #[test]
    fn metadata_rules() {
        let properties = ["recording".to_owned()];
        let mut plugins = [PluginOutput {
            id: "player",
            has_module: true,
            page_properties: &properties,
            blocks: true,
            contributions: vec![
                Contribution::Metadata {
                    page_id: "episode".to_owned(),
                    values: serde_json::json!({"chapters": [0]})
                        .as_object()
                        .unwrap()
                        .clone(),
                },
                Contribution::Metadata {
                    page_id: "episode".to_owned(),
                    values: serde_json::json!({"length": 3})
                        .as_object()
                        .unwrap()
                        .clone(),
                },
            ],
        }];
        let resolved = resolve(&view(), &mut plugins).unwrap();
        assert_eq!(
            Value::Object(resolved.metadata["p-e.html"]["player"].clone()),
            serde_json::json!({"chapters": [0], "length": 3})
        );
        let error = metadata_error(&[serde_json::json!({"a": 1})], false);
        assert!(
            error.contains("contributions[0]: metadata needs a browser module"),
            "{error}"
        );
        let error = metadata_error(&[serde_json::json!({"recording": "x"})], true);
        assert!(
            error.contains("repeats one of the plugin's pageProperties"),
            "{error}"
        );
        let error = metadata_error(
            &[serde_json::json!({"a": 1}), serde_json::json!({"a": 2})],
            true,
        );
        assert!(
            error.contains("contributions[1]: metadata key \"a\" is set twice"),
            "{error}"
        );
        let error = metadata_error(
            &[serde_json::json!({"big": "x".repeat(METADATA_LIMIT)})],
            true,
        );
        assert!(error.contains("over 64000"), "{error}");
    }

    #[test]
    fn rows_resolve_block_links_to_anchors() {
        let mut plugins = [PluginOutput {
            id: "list",
            has_module: false,
            page_properties: &[],
            blocks: true,
            contributions: vec![Contribution::Rows {
                page_id: "episode".to_owned(),
                block_id: Some("1".to_owned()),
                rows: vec![Row {
                    content: vec![InlineNode::Block(BlockLink {
                        page: "episode".to_owned(),
                        block: "1".to_owned(),
                        label: None,
                    })],
                    children: Vec::new(),
                }],
            }],
        }];
        let resolved = resolve(&view(), &mut plugins).unwrap();
        let placed = &resolved.rows["p-e.html"][0];
        let start = placed.block.unwrap();
        assert_eq!(
            placed.rows[0].content,
            [ResolvedNode::Page {
                href: format!("p-e.html#source-block-{start}"),
                label: "Episode".to_owned()
            }]
        );
        assert_eq!(resolved.anchors["p-e.html"], BTreeSet::from([start]));
        plugins[0].blocks = false;
        let error = resolve(&view(), &mut plugins).unwrap_err().to_string();
        assert!(error.contains("outside the plugin's view"), "{error}");
    }

    #[test]
    fn rejects_malformed_answers() {
        for (answer, expected) in [
            ("[]", "JSON object"),
            ("{", "not JSON"),
            (r#"{"apiVersion":2,"contributions":[]}"#, "apiVersion 2"),
            (r#"{"contributions":[]}"#, "missing apiVersion"),
            (r#"{"apiVersion":1}"#, "contributions list"),
            (
                r#"{"apiVersion":1,"contributions":[],"x":1}"#,
                "unknown field",
            ),
            (
                r#"{"apiVersion":1,"contributions":[{"kind":"files","pageId":"a"}]}"#,
                "contributions[0]: malformed",
            ),
            (
                r#"{"apiVersion":1,"contributions":[{"kind":"rows","pageId":"a","rows":[],"extra":1}]}"#,
                "contributions[0]: malformed",
            ),
            (
                r#"{"apiVersion":1,"contributions":[{"kind":"rows","pageId":"a","rows":[{"content":[{"html":"<b>"}]}]}]}"#,
                "contributions[0]: malformed",
            ),
        ] {
            let error = parse_response(answer.as_bytes()).unwrap_err();
            assert!(error.contains(expected), "{answer}: {error}");
        }
    }
}
