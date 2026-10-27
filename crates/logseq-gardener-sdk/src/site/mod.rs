//! Generate a local HTML garden from explicitly selected Markdown pages.

mod all_pages;
mod assets;
mod code;
mod dialogue;
mod hierarchy;
mod journals;
mod outline;
mod page_status;
mod properties;
mod references;
mod render;
mod search;
mod sequence;
pub use crate::search::SearchOptions;
mod shell;

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::garden::{DocumentKind, GardenLoadError, load_garden};
use crate::garden_config as config;
use crate::garden_settings::{GardenSettings, SidebarCard, resolve_favorites, resolve_sidebar};
use crate::git_dates::GitDates;
use crate::journals::{JournalDate, JournalFormats, journal_identity};
use crate::page_list::{PageKind, PageSource, page_list};
use crate::page_names::{PageNameIndex, lookup_key, page_alias_spellings, page_aliases};
use crate::page_titles::{FilenameFormat, PageTitleError, page_title};
use crate::publishing::NamespaceSelection;

/// Logseq's page references, tags, outside links, emphasis, inline code, images and
/// macro warnings, added to every page.
const INLINE_STYLES: &str = include_str!("inline.css");
/// Block references, embed wells, page embed headers and nested shades.
const TRANSCLUSION_STYLES: &str = include_str!("transclusion.css");

/// Optional presentation settings for local publication.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SiteOptions {
    /// Local regular `.css` file copied to `custom.css` and loaded after built-in styles.
    /// Paths must have no symlink components. CSS resources resolve from the output root;
    /// only this file is copied. Choose CSS whose browser resource requests you trust.
    pub stylesheet: Option<PathBuf>,
    /// Unique selected page title or alias shown at index.html; page list moves to pages.html.
    pub homepage: Option<String>,
    /// Generate external search artifacts without loading a browser UI.
    pub search: Option<SearchOptions>,
    /// Publish journals as pages and, unless the garden disables journals, a journals feed.
    pub journals: bool,
    /// Date that decides which journals the feed lists; defaults to the UTC date of
    /// `SOURCE_DATE_EPOCH`, or of the system clock when that is unset.
    pub build_date: Option<JournalDate>,
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
    /// Selected page files, excluding journals, generated namespace pages and the index.
    pub pages: usize,
    /// Published journal files.
    pub journals: usize,
    /// Number of distinct copied asset files.
    pub assets: usize,
    /// Journals skipped because journals were not requested or their names are not dates.
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
// The same digest names search files.
use crate::search::digest;

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

fn html_document(
    title: &str,
    body: &str,
    stylesheet: Option<&str>,
    shell: &shell::Shell,
    route: &str,
    home: bool,
) -> String {
    let main = format!("<h1>{}</h1>{body}", render::escape(title));
    html_page(title, &main, stylesheet, shell, route, home, "")
}

/// A complete page whose `<main>` holds `main`; `head` is appended inside `<head>`.
fn html_page(
    title: &str,
    main: &str,
    stylesheet: Option<&str>,
    shell: &shell::Shell,
    route: &str,
    home: bool,
    head: &str,
) -> String {
    let dialogue_script = if main.contains(dialogue::CONTROL_ATTRIBUTE) {
        format!(
            "<link rel=\"lsg-dialogue\" href=\"{}\"><script data-lsg-dialogue-start>{}</script>",
            dialogue::SCRIPT_ROUTE,
            dialogue::STARTUP
        )
    } else {
        String::new()
    };
    let frame = shell.render(route, home);
    let shell_styles = shell::STYLES;
    let shell_script = shell::SCRIPT_ROUTE;
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
    let outline_styles = outline::STYLES;
    let inline_styles = INLINE_STYLES;
    let code_styles = code::STYLES;
    let transclusion_styles = TRANSCLUSION_STYLES;
    let references_styles = references::STYLES;
    let theme_script = include_str!("theme.js");
    let theme_styles = include_str!("theme.css");
    format!(
        "<!doctype html>\n<html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>{title}</title><script data-lsg-theme-script>{theme_script}</script><style>{theme_styles}body{{margin:0;font:16px/1.5 \"Source Sans 3\",\"Source Sans Pro\",system-ui,sans-serif;color:var(--lsg-text);background:var(--lsg-background)}}main{{max-width:810px;margin:0 auto;padding:8px 16px 48px}}html[data-lsg-wide=true] main{{max-width:1280px}}@media(min-width:640px){{main{{padding-inline:32px}}}}a{{color:var(--lsg-link)}}a:focus-visible,summary:focus-visible{{outline:2px solid var(--lsg-focus);outline-offset:3px}}nav{{margin-bottom:1.5rem}}h1,h2,h3,h4,h5,h6{{line-height:1.25;overflow-wrap:anywhere}}li{{margin:.25rem 0}}pre{{overflow:auto;background:var(--lsg-surface);padding:1rem}}code{{font-size:.9em}}img,video{{max-width:100%;height:auto}}table{{border-collapse:collapse}}td,th{{border:1px solid var(--lsg-border);padding:.4rem}}.unsupported{{color:var(--lsg-warning)}}blockquote{{border-left:3px solid var(--lsg-quote-border);margin-left:0;padding-left:1rem}}.outline-collapsed{{list-style:none}}.outline-disclosure>summary{{cursor:pointer;overflow-wrap:anywhere}}.outline-disclosure>summary>:is(h1,h2,h3,h4,h5,h6){{display:inline}}.page-properties{{margin-bottom:1.5rem}}.page-properties summary{{cursor:pointer}}.page-properties dl{{padding:.75rem;background:var(--lsg-surface)}}.page-property{{display:flex;flex-wrap:wrap;gap:.25rem .75rem;margin:.25rem 0;overflow-wrap:anywhere}}.page-property dt{{font-weight:600}}.page-property dd{{margin:0;min-width:0;overflow-wrap:anywhere}}{navigation_styles}{outline_styles}{inline_styles}{code_styles}{transclusion_styles}{references_styles}{shell_styles}</style>{stylesheet}<script defer src=\"{shell_script}\"></script>{dialogue_script}{head}</head><body>{frame}<main>{main}</main><script data-lsg-fragments>{fragments}</script></body></html>\n"
    )
}

/// Publish selected Markdown pages into an absent directory outside the garden.
///
/// Each selected title has a deterministic route. Unique selected titles and aliases
/// resolve to local page links.
/// Files containing parsed `public:: false` are withheld in full. Journals are
/// counted and skipped unless [`SiteOptions::journals`] requests them. Unique explicit outline UUID references link to selected
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
    // Reading the git history takes seconds on a large garden; it runs while pages render.
    let git_root = root.clone();
    let git_dates = std::thread::spawn(move || GitDates::read(git_root));
    let mut report = SiteReport {
        pages: 0,
        journals: 0,
        assets: 0,
        skipped_journals: 0,
        withheld_pages: 0,
        diagnostics: Vec::new(),
    };
    let mut pages = BTreeMap::new();
    let mut routes = BTreeMap::new();
    let mut selected = Vec::new();
    let mut withheld_titles = BTreeSet::new();
    let (settings, settings_diagnostics) = GardenSettings::read(&root);
    let journal_formats = JournalFormats::from_settings(&settings);
    let mut journal_dates = BTreeMap::new();
    for document in &garden.documents {
        let journal = if document.kind == DocumentKind::Journal {
            if !options.journals {
                report.skipped_journals += 1;
                continue;
            }
            match journal_identity(document, &journal_formats) {
                Ok(identity) => Some(identity),
                Err(_) => {
                    report.skipped_journals += 1;
                    report.diagnostics.push(SiteDiagnostic {
                        source: document.relative_path.clone(),
                        message: "Journal skipped: its file name is not a date in the journal file-name format".to_owned(),
                    });
                    continue;
                }
            }
        } else {
            None
        };
        let title = match &journal {
            Some(identity) => identity.title.clone(),
            None => page_title(document, format).map_err(SiteError::Title)?,
        };
        let withheld = render::has_private_properties(&document.parsed.blocks);
        if withheld {
            withheld_titles.insert(lookup_key(&title));
        }
        if journal.is_none() && !selection.includes(&title) {
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
        if let Some(identity) = journal {
            journal_dates.insert(route.clone(), identity.date);
        }
        selected.push((title, route, document));
    }
    let mut names = PageNameIndex::with_journal_formats(journal_formats);
    let mut aliases = BTreeSet::new();
    let mut spellings = BTreeMap::new();
    let mut alias_spellings = BTreeMap::new();
    for (title, route, document) in &selected {
        names.insert(route.clone(), title, document);
        if let Some(date) = journal_dates.get(route) {
            names.insert_journal(route.clone(), *date);
        }
        aliases.extend(page_aliases(document, title));
        spellings
            .entry(lookup_key(title))
            .or_insert_with(|| title.clone());
        let authored = page_alias_spellings(document, title);
        for alias in &authored {
            spellings
                .entry(lookup_key(alias))
                .or_insert_with(|| alias.clone());
        }
        alias_spellings.insert(lookup_key(title), authored);
    }
    let hierarchy =
        hierarchy::Hierarchy::new(&routes, &withheld_titles, &aliases, &alias_spellings)?;
    let homepage_route = options
        .homepage
        .as_deref()
        .map(|name| match names.candidates(name) {
            [route] => Ok(route.clone()),
            _ => Err(SiteError::InvalidHomepage),
        })
        .transpose()?;
    let has_homepage = homepage_route.is_some();
    let build_date = options.build_date.unwrap_or_else(journals::build_date);
    let feed = journals::Feed::plan(
        &journal_dates,
        build_date,
        settings.features.journals,
        has_homepage,
    );
    let (favorites, favorite_diagnostics) =
        resolve_favorites(&settings.favorites, &names, &spellings);
    let (sidebar, sidebar_diagnostics) = resolve_sidebar(&settings.sidebar, &names);
    let sidebar = sidebar
        .into_iter()
        .map(|card| match card {
            SidebarCard::Contents => "contents".to_owned(),
            SidebarCard::Page(route) => route,
        })
        .collect();
    let contents = match names.candidates("contents") {
        [route] => Some(route.clone()),
        _ => None,
    };
    for message in settings_diagnostics
        .into_iter()
        .chain(favorite_diagnostics)
        .chain(sidebar_diagnostics)
    {
        report.diagnostics.push(SiteDiagnostic {
            source: "logseq/config.edn".into(),
            message,
        });
    }
    let index_route = if has_homepage || feed.home() {
        "pages.html"
    } else {
        "index.html"
    };
    let shell = shell::Shell {
        favorites,
        all_pages: index_route,
        journals: feed.first_route(),
        contents,
        sidebar,
    };
    let mut homepage_document = None;
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
    let reference_index = references::References::new(
        &selected,
        &names,
        reference_config.properties.then_some(&property_views),
        hierarchy
            .entries()
            .map(|node| (node.route.clone(), node.title.clone()))
            .collect(),
        journal_dates.clone(),
        settings.show_brackets,
    );
    // Embeds show page titles while `selected` is still sorted and consumed below.
    let embed_pages: Vec<(String, String, &crate::garden::GardenDocument)> = selected.clone();
    let selected_documents = embed_pages
        .iter()
        .map(|(title, route, document)| {
            (
                route.clone(),
                crate::transclusion::EmbedPage { title, document },
            )
        })
        .collect();
    let mut targets = crate::transclusion::ReferenceTargets::default();
    for (_, route, document) in &selected {
        targets.add_page(route.clone(), &document.parsed.blocks);
    }
    let mut search = options
        .search
        .as_ref()
        .map(|options| search::Search::new(&selected, &names, &targets, options));
    // Only selected source reaches the renderer and may request asset contents.
    let mut copied: BTreeMap<String, (PathBuf, Vec<u8>)> = BTreeMap::new();
    let mut documents = BTreeMap::new();
    let mut feed_bodies = BTreeMap::new();
    selected.sort_by(|left, right| left.0.cmp(&right.0));
    code::prepare(selected.iter().map(|(_, _, document)| *document));
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
        let mut source_anchors = reference_index.source_anchors(&route);
        if let Some(search) = &search {
            source_anchors.extend(search.anchors.get(&route).into_iter().flatten().copied());
        }
        let rendered = render::render_document_with_source_anchors(
            document,
            &names,
            &targets,
            &route,
            &selected_documents,
            render::PageRender {
                source_anchors,
                show_brackets: settings.show_brackets,
            },
            &mut asset,
        );
        if let Some(search) = &mut search {
            search.admit(&route, &rendered.source_anchors);
        }
        let sections = reference_index.render(
            &route,
            &references::Context {
                config: &reference_config,
                names: &names,
                targets: &targets,
                documents: &selected_documents,
            },
            &mut asset,
        );
        for message in rendered.diagnostics.into_iter().chain(sections.diagnostics) {
            report.diagnostics.push(SiteDiagnostic {
                source: document.relative_path.clone(),
                message,
            });
        }
        let is_journal = journal_dates.contains_key(&route);
        if feed.lists(&route) {
            feed_bodies.insert(
                route.clone(),
                (
                    title.clone(),
                    properties.html.clone() + &rendered.html,
                    reference_index.page_count(&route),
                ),
            );
        }
        let hierarchy_section = if is_journal {
            hierarchy::Section::default()
        } else {
            hierarchy.bottom_view(&title, &route, settings.show_brackets)
        };
        let body = properties.html.clone()
            + &rendered.html
            + &sequence
            + &sections.tagged
            + &sections.linked
            + &hierarchy_section.html
            + &sections.unlinked;
        documents.extend(sections.files);
        documents.extend(hierarchy_section.files);
        if homepage_route.as_ref() == Some(&route) {
            homepage_document = Some(html_document(
                &title,
                &body,
                stylesheet_route,
                &shell,
                &route,
                true,
            ));
        }
        documents.insert(
            route.clone(),
            html_document(&title, &body, stylesheet_route, &shell, &route, false),
        );
        if is_journal {
            report.journals += 1;
        } else {
            report.pages += 1;
        }
    }
    let mut entries = hierarchy.entries().collect::<Vec<_>>();
    entries.sort_by(|left, right| left.title.cmp(&right.title));
    for node in &entries {
        if !documents.contains_key(&node.route) {
            let sections = reference_index.render(
                &node.route,
                &references::Context {
                    config: &reference_config,
                    names: &names,
                    targets: &targets,
                    documents: &selected_documents,
                },
                &mut asset,
            );
            documents.extend(sections.files);
            let hierarchy_section =
                hierarchy.bottom_view(&node.title, &node.route, settings.show_brackets);
            documents.extend(hierarchy_section.files);
            documents.insert(
                node.route.clone(),
                html_document(
                    &node.title,
                    &(page_status::notice().to_owned()
                        + &sections.tagged
                        + &sections.linked
                        + &hierarchy_section.html
                        + &sections.unlinked),
                    stylesheet_route,
                    &shell,
                    &node.route,
                    false,
                ),
            );
        }
    }
    let (dates, dates_diagnostic) = git_dates.join().expect("git history reader panicked");
    if let Some(message) = dates_diagnostic {
        report.diagnostics.push(SiteDiagnostic {
            source: ".".into(),
            message,
        });
    }
    let list = page_list(
        entries.iter().map(|node| PageSource {
            id: node.route.clone(),
            title: node.title.clone(),
            kind: if node.generated {
                PageKind::Namespace
            } else if journal_dates.contains_key(&node.route) {
                PageKind::Journal
            } else {
                PageKind::Page
            },
            document: selected_documents
                .get(&node.route)
                .map(|page: &crate::transclusion::EmbedPage| page.document),
        }),
        reference_index.index(),
        dates.as_ref(),
    );
    for (route, main) in all_pages::screens(&list, index_route) {
        let home = route == index_route && !has_homepage;
        let html = html_page(
            "All pages",
            &main,
            stylesheet_route,
            &shell,
            &route,
            home,
            &all_pages::head(),
        );
        documents.insert(route, html);
    }
    documents.insert(
        all_pages::LIST_ROUTE.to_owned(),
        all_pages::list_json(&list),
    );
    documents.insert(all_pages::SCRIPT_ROUTE.to_owned(), all_pages::script());
    if let Some(document) = homepage_document {
        documents.insert("index.html".to_owned(), document);
    }
    for (route, body) in feed.render(&feed_bodies) {
        let home = route == "index.html";
        documents.insert(
            route.clone(),
            html_page(
                "Journals",
                &body,
                stylesheet_route,
                &shell,
                &route,
                home,
                journals::HEAD,
            ),
        );
    }
    if !feed.is_empty() {
        documents.insert(
            journals::SCRIPT_ROUTE.to_owned(),
            journals::SCRIPT.to_owned(),
        );
    }
    if documents
        .values()
        .any(|html| html.contains(dialogue::CONTROL_ATTRIBUTE))
    {
        documents.insert(
            dialogue::SCRIPT_ROUTE.to_owned(),
            dialogue::SCRIPT.to_owned(),
        );
    }
    documents.insert(shell::SCRIPT_ROUTE.to_owned(), shell::SCRIPT.to_owned());
    documents.insert(outline::SCRIPT_ROUTE.to_owned(), outline::SCRIPT.to_owned());
    documents.insert(shell::SIDEBAR_ROUTE.to_owned(), shell::sidebar_script());
    documents.insert(
        shell::SIDEBAR_STYLES_ROUTE.to_owned(),
        shell::SIDEBAR_STYLES.to_owned(),
    );
    documents.insert(references::SCRIPT_ROUTE.to_owned(), references::script());
    if let Some(search) = search {
        let (artifacts, manifest_route, revision) = search.artifacts();
        let controller = include_str!("search-ui.mjs");
        let controller_route = format!("lsg-search-ui-{}.mjs", digest(controller));
        let startup = include_str!("search-start.mjs")
            .lines()
            .map(str::trim)
            .collect::<Vec<_>>()
            .join("\n")
            .replace("__LSG_SEARCH_UI__", &controller_route);
        let startup_route = format!("lsg-search-start-{}.mjs", digest(&startup));
        let start_after_paint = r#"(()=>{const f=(_,o)=>Object.assign(f.options??={},o);window.logseqGardenerInitSearch=f;const u=new URL('__STARTUP__',document.querySelector('[rel=lsg-search]').href);document.addEventListener('DOMContentLoaded',()=>requestAnimationFrame(()=>requestAnimationFrame(()=>{const s=document.createElement('script');s.type='module';s.src=u;s.onerror=()=>{if(!document.querySelector('[rel=lsg-search]'))return;const t=document.createElement('span');t.id='lsg-search-load';t.setAttribute('role','status');t.textContent='Search unavailable. Reload this page to retry search.';document.body.append(t)};document.head.append(s)})),{once:true})})();"#.replace("__STARTUP__", &startup_route);
        let styles = include_str!("search-ui.css");
        let link = format!(
            "<link rel=\"lsg-search\" href=\"{manifest_route}\" data-revision=\"{revision}\"><style data-lsg-search-styles>{styles}</style><script data-lsg-search-start>{start_after_paint}</script>"
        );
        for (route, html) in &mut documents {
            if route.ends_with(".html") && !route.starts_with("_refs/") {
                *html = html.replacen("</head>", &(link.clone() + "</head>"), 1);
            }
        }
        documents.extend(artifacts);
        documents.insert(controller_route, controller.to_owned());
        documents.insert(startup_route, startup);
    }
    fs::create_dir(&output).map_err(|error| io_error(&output, error))?;
    if documents
        .keys()
        .any(|route| route.starts_with(&format!("{}/", references::REFS)))
    {
        let directory = output.join(references::REFS);
        fs::create_dir(&directory).map_err(|error| io_error(&directory, error))?;
    }
    let directory = output.join(all_pages::LIST_DIRECTORY);
    fs::create_dir(&directory).map_err(|error| io_error(&directory, error))?;
    write_files(&output, documents)?;
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

/// Write the generated files on several threads: creating files, not rendering them,
/// dominates the time to write a large garden.
fn write_files(output: &Path, documents: BTreeMap<String, String>) -> Result<(), SiteError> {
    let documents: Vec<_> = documents.into_iter().collect();
    let threads = std::thread::available_parallelism().map_or(1, |n| n.get().min(8));
    let chunk = documents.len().div_ceil(threads).max(1);
    std::thread::scope(|scope| {
        let workers: Vec<_> = documents
            .chunks(chunk)
            .map(|files| {
                scope.spawn(move || {
                    for (route, html) in files {
                        let path = output.join(route);
                        fs::write(&path, html).map_err(|error| io_error(&path, error))?;
                    }
                    Ok(())
                })
            })
            .collect();
        workers
            .into_iter()
            .try_for_each(|worker| worker.join().expect("file writer panicked"))
    })
}
