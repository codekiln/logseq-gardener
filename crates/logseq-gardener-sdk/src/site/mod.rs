//! Generate a local HTML garden from explicitly selected Markdown pages.

mod assets;
mod render;

use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::garden::{DocumentKind, GardenLoadError, load_garden};
use crate::page_titles::{FilenameFormat, PageTitleError, page_title};
use crate::publishing::NamespaceSelection;

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
    /// Number of generated page documents, excluding the index.
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
            Self::InvalidOutput { path, reason } => write!(f, "{}: {reason}", path.display()),
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

fn html_document(title: &str, body: &str) -> String {
    let title = render::escape(title);
    format!(
        "<!doctype html>\n<html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>{title}</title><style>body{{font:18px/1.6 system-ui,sans-serif;max-width:55rem;margin:2rem auto;padding:0 1rem;color:#20252b;background:#fafafa}}a{{color:#1855a3}}nav{{margin-bottom:1.5rem}}h1,h2,h3,h4,h5,h6{{line-height:1.25;overflow-wrap:anywhere}}li{{margin:.25rem 0}}pre{{overflow:auto;background:#eceff2;padding:1rem}}code{{font-size:.9em}}img,video{{max-width:100%;height:auto}}table{{border-collapse:collapse}}td,th{{border:1px solid #c6ccd2;padding:.4rem}}.unsupported{{color:#695429}}blockquote{{border-left:3px solid #b3bbc4;margin-left:0;padding-left:1rem}}</style></head><body><nav><a href=\"index.html\">Garden index</a></nav><main><h1>{title}</h1>{body}</main></body></html>\n"
    )
}

/// Publish selected Markdown pages into an absent directory outside the garden.
///
/// Each selected title has a deterministic route and selected-title page links.
/// Files containing parsed `public:: false` are withheld in full. Journals are
/// counted and skipped. Unique explicit outline UUID references link to selected
/// targets. Aliases, unavailable references, embeds, and queries receive fallbacks
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
    for document in &garden.documents {
        if document.kind == DocumentKind::Journal {
            report.skipped_journals += 1;
            continue;
        }
        let title = page_title(document, format).map_err(SiteError::Title)?;
        if !selection.includes(&title) {
            continue;
        }
        if render::has_private_properties(&document.parsed.blocks) {
            report.withheld_pages += 1;
            report.diagnostics.push(SiteDiagnostic {
                source: document.relative_path.clone(),
                message: "Page withheld by private-property or nesting visibility check".to_owned(),
            });
            continue;
        }
        let route = format!("p-{}.html", digest(&title));
        if pages.insert(title.to_lowercase(), route.clone()).is_some()
            || routes.insert(route.clone(), title.clone()).is_some()
        {
            return Err(SiteError::Collision { title });
        }
        selected.push((title, route, document));
    }
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
    for (title, route, document) in selected {
        let mut asset = |url: &str| {
            let resolved = assets::resolve_asset(&root, &document.relative_path, url)?;
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
        let rendered = render::render_document(document, &pages, &targets, &route, &mut asset);
        for message in rendered.diagnostics {
            report.diagnostics.push(SiteDiagnostic {
                source: document.relative_path.clone(),
                message,
            });
        }
        documents.insert(route.clone(), html_document(&title, &rendered.html));
        index.push_str(&format!(
            "<li><a href=\"{}\">{}</a></li>",
            render::escape(&route),
            render::escape(&title)
        ));
        report.pages += 1;
    }
    index.push_str("</ul>");
    if report.pages == 0 {
        index.push_str("<p>No pages matched the publication policy.</p>");
    }
    documents.insert("index.html".to_owned(), html_document("Garden", &index));
    fs::create_dir(&output).map_err(|error| io_error(&output, error))?;
    for (route, html) in documents {
        let path = output.join(route);
        fs::write(&path, html).map_err(|error| io_error(&path, error))?;
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
