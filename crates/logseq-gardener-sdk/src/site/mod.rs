//! Generate a local HTML garden from explicitly selected Markdown pages.

mod assets;
mod backlinks;
mod config;
mod hierarchy;
mod page_status;
mod properties;
mod render;
mod sequence;

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::garden::{DocumentKind, GardenLoadError, load_garden};
use crate::page_names::{PageNameIndex, lookup_key, page_aliases};
use crate::page_titles::{FilenameFormat, PageTitleError, page_title};
use crate::publishing::NamespaceSelection;

/// Optional presentation settings for local publication.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SiteOptions {
    /// Local regular `.css` file copied to `custom.css` and loaded after built-in styles.
    /// Paths must have no symlink components. CSS resources resolve from the output root;
    /// only this file is copied. Choose CSS whose browser resource requests you trust.
    pub stylesheet: Option<PathBuf>,
    /// Unique selected page title or alias shown at index.html; page list moves to pages.html.
    pub homepage: Option<String>,
}

/// A selected-source construct that could not be published faithfully.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SiteDiagnostic {
    /// Source file relative to the garden root.
    pub source: PathBuf,
    /// Explanation of the fallback rendered in the page.
    pub message: String,
}

/// Successful local publication, with diagnostics kept outside generated files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SiteReport {
    /// Selected source documents, excluding generated namespace pages and the index.
    pub pages: usize,
    /// Number of distinct copied asset files.
    pub assets: usize,
    /// Journals skipped pending configured date naming.
    pub skipped_journals: usize,
    /// Included pages withheld by the private-property or nesting visibility check.
    pub withheld_pages: usize,
    /// Unsupported or unresolved constructs in selected source.
    pub diagnostics: Vec<SiteDiagnostic>,
}

/// A publication that could not complete.
#[derive(Debug)]
pub enum SiteError {
    /// Reading or parsing the garden failed.
    Garden(GardenLoadError),
    /// Deriving a page title failed.
    Title(PageTitleError),
    /// The output destination is existing or unsafe.
    InvalidOutput { path: PathBuf, reason: &'static str },
    /// A custom stylesheet has an invalid path or file type.
    InvalidStylesheet { path: PathBuf, reason: &'static str },
    /// The homepage does not resolve to one selected visible page.
    InvalidHomepage,
    /// Selected pages have ambiguous names or colliding output routes.
    Collision { title: String },
    /// A filesystem operation failed; a write failure can leave partial new output.
    Io { path: PathBuf, source: io::Error },
}

impl fmt::Display for SiteError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Garden(error) => error.fmt(f),
            Self::Title(error) => error.fmt(f),
            Self::InvalidOutput { path, reason } | Self::InvalidStylesheet { path, reason } => {
                write!(f, "{}: {reason}", path.display())
            }
            Self::InvalidHomepage => write!(f, "homepage must identify one selected visible page"),
            Self::Collision { title } => write!(f, "ambiguous selected title or route: {title}"),
            Self::Io { path, source } => write!(f, "{}: {source}", path.display()),
        }
    }
}

impl Error for SiteError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Garden(error) => Some(error),
            Self::Title(error) => Some(error),
            Self::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}

fn io_error(path: &Path, source: io::Error) -> SiteError {
    SiteError::Io {
        path: path.to_owned(),
        source,
    }
}

fn invalid_output(path: &Path, reason: &'static str) -> SiteError {
    SiteError::InvalidOutput {
        path: path.to_owned(),
        reason,
    }
}

fn fresh_destination(root: &Path, output: &Path) -> Result<PathBuf, SiteError> {
    match fs::symlink_metadata(output) {
        Ok(_) => {
            return Err(invalid_output(
                output,
                "output already exists; choose a fresh directory",
            ));
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(error) => return Err(io_error(output, error)),
    }
    let name = output
        .file_name()
        .ok_or_else(|| invalid_output(output, "expected a new directory name"))?;
    let parent = output
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let parent = fs::canonicalize(parent).map_err(|error| io_error(parent, error))?;
    if parent.starts_with(root) {
        return Err(invalid_output(
            output,
            "output must be outside the source garden",
        ));
    }
    Ok(parent.join(name))
}

// Fixed routes avoid filename limits and title-derived path components. Check
// collisions separately; this digest is a deterministic identifier, not a secret.
fn digest(value: &str) -> String {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

fn stylesheet_contents(path: &Path) -> Result<Vec<u8>, SiteError> {
    let invalid = |reason| SiteError::InvalidStylesheet {
        path: path.to_owned(),
        reason,
    };
    if !path
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("css"))
    {
        return Err(invalid("stylesheet must have a .css extension"));
    }
    let absolute = std::path::absolute(path).map_err(|error| io_error(path, error))?;
    let mut current = PathBuf::new();
    for component in absolute.components() {
        current.push(component);
        let metadata = fs::symlink_metadata(&current).map_err(|error| io_error(path, error))?;
        if metadata.file_type().is_symlink() {
            return Err(invalid("stylesheet path must have no symlink components"));
        }
    }
    if !fs::symlink_metadata(path)
        .map_err(|error| io_error(path, error))?
        .is_file()
    {
        return Err(invalid("stylesheet must be a regular file"));
    }
    fs::read(path).map_err(|error| io_error(path, error))
}

fn html_document(title: &str, body: &str, stylesheet: Option<&str>, homepage: bool) -> String {
    let navigation = if homepage {
        "<a href=\"index.html\">Home</a> · <a href=\"pages.html\">All pages</a>"
    } else {
        "<a href=\"index.html\">Garden index</a>"
    };
    let title = render::escape(title);
    let stylesheet = stylesheet
        .map(|route| {
            format!(
                "<link rel=\"stylesheet\" href=\"{}\">",
                render::escape(route)
            )
        })
        .unwrap_or_default();
    let fragments = include_str!("fragments.js");
    let navigation_styles = page_status::STYLES;
    let backlinks = include_str!("backlinks.js");
    let theme_script = include_str!("theme.js");
    let theme_styles = include_str!("theme.css");
    let theme_control = "<label class=\"theme-control\" data-lsg-theme-control hidden>Appearance <select data-lsg-theme-select aria-label=\"Appearance\"><option value=\"system\">System</option><option value=\"light\">Light</option><option value=\"dark\">Dark</option></select></label>";
    format!(
        "<!doctype html>\n<html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>{title}</title><script data-lsg-theme-script>{theme_script}</script><style>{theme_styles}body{{font:18px/1.6 system-ui,sans-serif;max-width:55rem;margin:2rem auto;padding:0 1rem;color:var(--lsg-text);background:var(--lsg-background)}}a{{color:var(--lsg-link)}}a:focus-visible,summary:focus-visible{{outline:2px solid var(--lsg-focus);outline-offset:3px}}nav{{margin-bottom:1.5rem}}h1,h2,h3,h4,h5,h6{{line-height:1.25;overflow-wrap:anywhere}}li{{margin:.25rem 0}}pre{{overflow:auto;background:var(--lsg-surface);padding:1rem}}code{{font-size:.9em}}img,video{{max-width:100%;height:auto}}table{{border-collapse:collapse}}td,th{{border:1px solid var(--lsg-border);padding:.4rem}}.unsupported{{color:var(--lsg-warning)}}blockquote{{border-left:3px solid var(--lsg-quote-border);margin-left:0;padding-left:1rem}}.outline-collapsed{{list-style:none}}.outline-disclosure>summary{{cursor:pointer;overflow-wrap:anywhere}}.outline-disclosure>summary>:is(h1,h2,h3,h4,h5,h6){{display:inline}}.page-properties{{margin-bottom:1.5rem}}.page-properties summary{{cursor:pointer}}.page-properties dl{{padding:.75rem;background:var(--lsg-surface)}}.page-property{{display:flex;flex-wrap:wrap;gap:.25rem .75rem;margin:.25rem 0;overflow-wrap:anywhere}}.page-property dt{{font-weight:600}}.page-property dd{{margin:0;min-width:0;overflow-wrap:anywhere}}.namespace-hierarchy{{margin-top:2rem}}.namespace-hierarchy summary{{cursor:pointer;font-size:1.25rem;font-weight:600}}.namespace-hierarchy ul{{list-style:none;padding:0;margin:.75rem 0 0}}.namespace-separator{{margin-inline:.2em}}.namespace-path{{display:block;overflow-wrap:anywhere}}.namespace-path a,.namespace-unavailable{{overflow-wrap:anywhere}}.linked-references{{margin-top:2rem}}.linked-references summary{{cursor:pointer;font-size:1.1rem;font-weight:600}}.backlinks-controls fieldset{{min-width:0;margin:1rem 0}}.backlinks-options{{display:flex;flex-wrap:wrap;gap:.5rem}}.backlinks-options label{{max-width:100%;overflow-wrap:anywhere}}.backlinks-controls input{{max-width:100%;box-sizing:border-box}}.backlinks-group article{{border-left:2px solid var(--lsg-border);margin:.75rem 0;padding-left:.75rem;overflow-wrap:anywhere}}.backlinks-group h3{{font-size:1rem}}.backlink-context,.backlink-source{{font-size:.85rem}}.linked-references [hidden]{{display:none!important}}{navigation_styles}</style>{stylesheet}</head><body><nav>{navigation}{theme_control}</nav><main><h1>{title}</h1>{body}</main><script data-lsg-fragments>{fragments}</script><script data-lsg-backlinks-script>{backlinks}</script></body></html>\n"
    )
}

/// Publish selected Markdown pages into an absent directory outside the garden.
///
/// Each selected title has a deterministic route. Unique selected titles and aliases
/// resolve to local page links.
/// Files containing parsed `public:: false` are withheld in full. Journals are
/// counted and skipped. Unique explicit outline UUID references link to selected
/// targets. Selected page and block embeds expand within fixed limits. Ambiguous or unavailable references, embeds, and queries receive fallbacks
/// and diagnostics. Supported local assets are copied
/// only when requested by rendered content. Existing output is never replaced.
///
/// The caller chooses the graph's filename format and namespace policy. Input
/// planning precedes output creation; write failures can leave partial new output.
/// Source and destination parents must remain unchanged during generation.
pub fn publish_site(
    root: impl AsRef<Path>,
    output: impl AsRef<Path>,
    format: FilenameFormat,
    selection: &NamespaceSelection,
) -> Result<SiteReport, SiteError> {
    publish_site_with_options(root, output, format, selection, &SiteOptions::default())
}

/// Publish with optional custom CSS, preserving selection and destination protections.
///
/// Stylesheet validation and reading precede output creation. Existing callers may
/// continue using [`publish_site`] for built-in presentation. Filesystem paths must
/// remain stable during generation; write failures can leave partial new output.
pub fn publish_site_with_options(
    root: impl AsRef<Path>,
    output: impl AsRef<Path>,
    format: FilenameFormat,
    selection: &NamespaceSelection,
    options: &SiteOptions,
) -> Result<SiteReport, SiteError> {
    let stylesheet = options
        .stylesheet
        .as_deref()
        .map(stylesheet_contents)
        .transpose()?;
    let stylesheet_route = stylesheet.as_ref().map(|_| "custom.css");
    let source_root = root.as_ref();
    let root = fs::canonicalize(source_root).map_err(|error| io_error(source_root, error))?;
    let output = fresh_destination(&root, output.as_ref())?;
    let garden = load_garden(source_root).map_err(SiteError::Garden)?;
    let mut report = SiteReport {
        pages: 0,
        assets: 0,
        skipped_journals: 0,
        withheld_pages: 0,
        diagnostics: Vec::new(),
    };
    let mut pages = BTreeMap::new();
    let mut routes = BTreeMap::new();
    let mut selected = Vec::new();
    let mut withheld_titles = BTreeSet::new();
    for document in &garden.documents {
        if document.kind == DocumentKind::Journal {
            report.skipped_journals += 1;
            continue;
        }
        let title = page_title(document, format).map_err(SiteError::Title)?;
        let withheld = render::has_private_properties(&document.parsed.blocks);
        if withheld {
            withheld_titles.insert(lookup_key(&title));
        }
        if !selection.includes(&title) {
            continue;
        }
        if withheld {
            report.withheld_pages += 1;
            report.diagnostics.push(SiteDiagnostic {
                source: document.relative_path.clone(),
                message: "Page withheld by private-property or nesting visibility check".to_owned(),
            });
            continue;
        }
        let route = format!("p-{}.html", digest(&title));
        if pages.insert(lookup_key(&title), route.clone()).is_some()
            || routes.insert(route.clone(), title.clone()).is_some()
        {
            return Err(SiteError::Collision { title });
        }
        selected.push((title, route, document));
    }
    let mut names = PageNameIndex::default();
    let mut aliases = BTreeSet::new();
    for (title, route, document) in &selected {
        names.insert(route.clone(), title, document);
        aliases.extend(page_aliases(document, title));
    }
    let hierarchy = hierarchy::Hierarchy::new(&routes, &withheld_titles, &aliases)?;
    let homepage_route = options
        .homepage
        .as_deref()
        .map(|name| match names.candidates(name) {
            [route] => Ok(route.clone()),
            _ => Err(SiteError::InvalidHomepage),
        })
        .transpose()?;
    let has_homepage = homepage_route.is_some();
    let (reference_config, config_diagnostics) = config::read(&root);
    for message in config_diagnostics {
        report.diagnostics.push(SiteDiagnostic {
            source: "logseq/config.edn".into(),
            message,
        });
    }
    let property_views: BTreeMap<_, _> = selected
        .iter()
        .map(|(_, route, document)| {
            (
                route.clone(),
                properties::view(document, route, &names, &routes, &reference_config),
            )
        })
        .collect();
    let reference_index = backlinks::Backlinks::new(
        &selected,
        &names,
        reference_config.properties.then_some(&property_views),
        hierarchy
            .entries()
            .map(|node| (node.route.clone(), node.title.clone()))
            .collect(),
    );
    let selected_documents = selected
        .iter()
        .map(|(_, route, document)| (route.clone(), *document))
        .collect();
    let mut targets = BTreeMap::new();
    for (_, route, document) in &selected {
        for (id, start, label) in render::outline_targets(&document.parsed.blocks) {
            let target = render::BlockTarget {
                route: route.clone(),
                start,
                label,
            };
            targets
                .entry(id)
                .and_modify(|value| *value = None)
                .or_insert(Some(target));
        }
    }
    // Only selected source reaches the renderer and may request asset contents.
    let mut copied: BTreeMap<String, (PathBuf, Vec<u8>)> = BTreeMap::new();
    let mut documents = BTreeMap::new();
    let mut index = String::from("<ul>");
    selected.sort_by(|left, right| left.0.cmp(&right.0));
    let mut asset = |source: &Path, url: &str| {
        let resolved = assets::resolve_asset(&root, source, url)?;
        let path = resolved
            .relative_path
            .to_str()
            .ok_or("asset path is not UTF-8")?
            .replace('\\', "/");
        let route = format!("assets/a-{}.{}", digest(&path), resolved.extension);
        if let Some((previous, _)) = copied.get(&route) {
            if previous != &resolved.relative_path {
                return Err("asset route collision".to_owned());
            }
        } else {
            copied.insert(route.clone(), (resolved.relative_path, resolved.contents));
        }
        Ok(route)
    };
    for (title, route, document) in selected {
        let properties = &property_views[&route];
        let mut omitted = properties.sequence_keys.clone();
        omitted.extend(reference_config.hidden_properties.iter().cloned());
        let sequence = if reference_config.sequence {
            sequence::navigation(document, &route, &names, &routes, &omitted)
        } else {
            String::new()
        };
        let rendered = render::render_document_with_source_anchors(
            document,
            &names,
            &targets,
            &route,
            &selected_documents,
            reference_index.source_anchors(&route),
            &mut asset,
        );
        let (reference_html, reference_diagnostics) = reference_index.render(
            &route,
            &reference_config,
            &names,
            &targets,
            &selected_documents,
            &mut asset,
        );
        for message in rendered
            .diagnostics
            .into_iter()
            .chain(reference_diagnostics)
        {
            report.diagnostics.push(SiteDiagnostic {
                source: document.relative_path.clone(),
                message,
            });
        }
        documents.insert(
            route.clone(),
            html_document(
                &title,
                &(properties.html.clone()
                    + &rendered.html
                    + &sequence
                    + &hierarchy.bottom_view(&title)
                    + &reference_html),
                stylesheet_route,
                has_homepage,
            ),
        );
        report.pages += 1;
    }
    let mut entries = hierarchy.entries().collect::<Vec<_>>();
    entries.sort_by(|left, right| left.title.cmp(&right.title));
    for node in entries {
        if !documents.contains_key(&node.route) {
            let (reference_html, _) = reference_index.render(
                &node.route,
                &reference_config,
                &names,
                &targets,
                &selected_documents,
                &mut asset,
            );
            documents.insert(
                node.route.clone(),
                html_document(
                    &node.title,
                    &(page_status::notice().to_owned()
                        + &hierarchy.bottom_view(&node.title)
                        + &reference_html),
                    stylesheet_route,
                    has_homepage,
                ),
            );
        }
        index.push_str(&format!(
            "<li><a{} href=\"{}\">{}</a>{}</li>",
            page_status::link_attributes(node.generated, &node.title),
            render::escape(&node.route),
            render::escape(&node.title),
            page_status::badge(node.generated)
        ));
    }
    index.push_str("</ul>");
    if report.pages == 0 {
        index.push_str("<p>No pages matched the publication policy.</p>");
    }
    let index_route = if has_homepage {
        "pages.html"
    } else {
        "index.html"
    };
    documents.insert(
        index_route.to_owned(),
        html_document("Garden", &index, stylesheet_route, has_homepage),
    );
    if let Some(route) = homepage_route {
        documents.insert("index.html".to_owned(), documents[&route].clone());
    }
    fs::create_dir(&output).map_err(|error| io_error(&output, error))?;
    for (route, html) in documents {
        let path = output.join(route);
        fs::write(&path, html).map_err(|error| io_error(&path, error))?;
    }
    if let Some(contents) = stylesheet {
        let path = output.join("custom.css");
        fs::write(&path, contents).map_err(|error| io_error(&path, error))?;
    }
    if !copied.is_empty() {
        let directory = output.join("assets");
        fs::create_dir(&directory).map_err(|error| io_error(&directory, error))?;
    }
    report.assets = copied.len();
    for (route, (_, contents)) in copied {
        let path = output.join(route);
        fs::write(&path, contents).map_err(|error| io_error(&path, error))?;
    }
    Ok(report)
}
