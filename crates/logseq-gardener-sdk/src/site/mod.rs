//! Generate a local HTML garden from explicitly selected Markdown pages.

mod all_pages;
mod assets;
mod code;
mod derived;
mod dialogue;
mod flashcards;
mod graph;
mod graph_layout;
mod hierarchy;
mod journals;
mod media;
mod outline;
mod page_status;
mod plugins;
mod properties;
mod queries;
mod readable;
mod references;
mod render;
mod search;
mod sequence;
pub use crate::page_urls::PageUrls;
pub use crate::search::SearchOptions;
mod shell;

use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::garden::{GardenLoadError, load_garden_timed};
use crate::garden_config as config;
use crate::garden_settings::{GardenSettings, SidebarCard, resolve_favorites, resolve_sidebar};
use crate::git_dates::GitDates;
use crate::journals::{JournalDate, parse_journal_reference};
use crate::page_list::{PageKind, PageSource, page_list};
use crate::page_names::lookup_key;
use crate::page_titles::{FilenameFormat, PageTitleError};
use crate::page_urls::{PageUrlError, check_reserved, is_readable, output_path, readable_title};
use crate::publication::{Publication, SelectError};
use crate::publishing::NamespaceSelection;
use crate::timing::{Stopwatch, Timings};

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
    /// Directory that keeps the graph view's layout between builds, keyed by the graph's
    /// content; `None` lays the graph out on every build. Created when missing.
    pub layout_cache: Option<PathBuf>,
    /// Plugin manifest files, checked by [`crate::plugins::read_manifests`] before output is
    /// created. With at least one plugin, every page carries plugin metadata and loads the
    /// browser host after first paint; with none, pages are unchanged. A manifest with a
    /// `build` section also runs its command once per build as a build plugin.
    pub plugins: Vec<PathBuf>,
    /// In-process build plugins. Their ids share one namespace with the manifests' ids. With
    /// none here and no manifest `build` section, the build does no plugin work.
    pub build_plugins: crate::contributions::BuildPlugins,
    /// How page files and links are named. [`PageUrls::Readable`] writes each page, journal and
    /// namespace page to `<title path>/index.html` and links to it as its
    /// [`readable_url`](crate::page_urls::readable_url); home, All pages, the journals feed and
    /// the graph view stay at the output root. A page whose title cannot form a safe path keeps
    /// its `p-<digest>.html` file and gets a diagnostic; colliding paths fail before output is
    /// created.
    pub page_urls: PageUrls,
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
    /// Included pages withheld by the private-property or nesting visibility check, or
    /// because they are not marked `public:: true` in a garden that publishes only those.
    pub withheld_pages: usize,
    /// Unsupported or unresolved constructs in selected source.
    pub diagnostics: Vec<SiteDiagnostic>,
    /// Wall-clock time of each publication phase, in this order: `read` (finding and reading
    /// source files and checking options), `parse`, `index` (selection, names, references,
    /// queries and search records), `plugins` (waiting for build plugins), `render` (page and
    /// list documents), `history` (waiting for the git history read), `search` (search files),
    /// `graph` (waiting for graph layouts and neighbourhoods) and `write`. Waiting phases
    /// measure only the time the publisher waited for work that ran alongside other phases.
    /// Timings never affect the published files.
    pub timings: Timings,
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
    /// A plugin manifest is invalid, names unusable files or repeats another plugin's id.
    InvalidPlugin(crate::plugins::PluginError),
    /// A build plugin failed, repeated another plugin's id or returned a rejected contribution.
    BuildPlugin(crate::contributions::ContributionError),
    /// With readable URLs, a page's path collides with another page's or with a file the build
    /// writes at the output root.
    PageUrl(PageUrlError),
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
            Self::InvalidPlugin(error) => error.fmt(f),
            Self::BuildPlugin(error) => error.fmt(f),
            Self::PageUrl(error) => error.fmt(f),
            Self::Io { path, source } => write!(f, "{}: {source}", path.display()),
        }
    }
}

impl Error for SiteError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Garden(error) => Some(error),
            Self::Title(error) => Some(error),
            Self::InvalidPlugin(error) => Some(error),
            Self::BuildPlugin(error) => Some(error),
            Self::PageUrl(error) => Some(error),
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
    let media_head = media::head(main);
    let properties_script = if shell.properties {
        let start = properties::STARTUP;
        let route = format!("lsg-properties-{}.mjs", digest(properties::SCRIPT));
        format!(
            "<link rel=\"lsg-properties\" href=\"{route}\"><script data-lsg-properties-start>{start}</script>"
        )
    } else {
        String::new()
    };
    let frame = shell.render(route, home);
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
    let theme_start = theme_start();
    let fragments = fragments_script();
    format!(
        "<!doctype html>\n<html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>{title}</title><script data-lsg-theme-script>{theme_start}</script><script defer src=\"{THEME_ROUTE}\"></script>{properties_script}<link rel=\"stylesheet\" href=\"{SITE_STYLES_ROUTE}\">{stylesheet}<script defer src=\"{shell_script}\"></script>{dialogue_script}{media_head}{head}</head><body>{frame}<main>{main}</main><script data-lsg-fragments>{fragments}</script></body></html>\n"
    )
}

/// The search styles, linked by every page when search is published.
pub(crate) const SEARCH_STYLES_ROUTE: &str = "lsg-search.css";
/// The search start-up, loaded with `defer` by every page when search is published.
pub(crate) const SEARCH_BOOT_ROUTE: &str = "lsg-search-boot.js";
/// The shared stylesheet every page links where its built-in styles were.
pub(crate) const SITE_STYLES_ROUTE: &str = "lsg-site.css";
/// The theme controller, loaded with `defer` after the inline part that runs before paint.
pub(crate) const THEME_ROUTE: &str = "lsg-theme.js";
/// Page layout rules that belong to no other stylesheet.
const BASE_STYLES: &str = "body{margin:0;font:16px/1.5 \"Source Sans 3\",\"Source Sans Pro\",system-ui,sans-serif;color:var(--lsg-text);background:var(--lsg-background)}main{max-width:810px;margin:0 auto;padding:8px 16px 48px}html[data-lsg-wide=true] main{max-width:1280px}@media(min-width:640px){main{padding-inline:32px}}a{color:var(--lsg-link)}a:focus-visible,summary:focus-visible{outline:2px solid var(--lsg-focus);outline-offset:3px}nav{margin-bottom:1.5rem}h1,h2,h3,h4,h5,h6{line-height:1.25;overflow-wrap:anywhere}li{margin:.25rem 0}pre{overflow:auto;background:var(--lsg-surface);padding:1rem}code{font-size:.9em}img,video{max-width:100%;height:auto}table{border-collapse:collapse}td,th{border:1px solid var(--lsg-border);padding:.4rem}.unsupported{color:var(--lsg-warning)}blockquote{border-left:3px solid var(--lsg-quote-border);margin-left:0;padding-left:1rem}.outline-collapsed{list-style:none}.outline-disclosure>summary{cursor:pointer;overflow-wrap:anywhere}.outline-disclosure>summary>:is(h1,h2,h3,h4,h5,h6){display:inline}.page-properties{margin-bottom:1.5rem}.page-properties summary{cursor:pointer}.page-properties dl{padding:.75rem;background:var(--lsg-surface)}.page-property{display:flex;flex-wrap:wrap;gap:.25rem .75rem;margin:.25rem 0;overflow-wrap:anywhere}.page-property dt{font-weight:600}.page-property dd{margin:0;min-width:0;overflow-wrap:anywhere}";

/// The built-in page styles, in the order pages carried them inline: theme, page layout,
/// navigation, outline, inline markup, code, transclusion, references and shell.
fn site_styles() -> String {
    [
        include_str!("theme.css"),
        BASE_STYLES,
        page_status::STYLES,
        outline::STYLES,
        INLINE_STYLES,
        code::STYLES,
        TRANSCLUSION_STYLES,
        references::STYLES,
        shell::STYLES,
    ]
    .concat()
}

/// The inline script that applies the saved layout and theme before the page paints.
fn theme_start() -> String {
    references::minify(include_str!("theme-start.js"))
        .trim_end()
        .to_owned()
}

/// The inline script at the end of the body that opens the block a link's fragment names.
fn fragments_script() -> String {
    references::minify(include_str!("fragments.js"))
        .trim_end()
        .to_owned()
}

/// The shared stylesheets and scripts every page names. Their names are fixed and their
/// contents depend only on the `lsg` version, so a garden edit never changes them.
fn shared_files() -> [(&'static str, String); 2] {
    [
        (SITE_STYLES_ROUTE, site_styles()),
        (THEME_ROUTE, references::minify(include_str!("theme.js"))),
    ]
}

/// Publish selected Markdown pages into an absent directory outside the garden.
///
/// Each selected title has a deterministic route. Unique selected titles and aliases
/// resolve to local page links.
/// Files containing parsed `public:: false` are withheld in full. Unless the garden sets
/// `:publishing/all-pages-public? true`, pages and journals that are not marked
/// `public:: true` are withheld too; see [`PublicPages`]. Journals are
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
    let mut stopwatch = Stopwatch::start();
    let stylesheet = options
        .stylesheet
        .as_deref()
        .map(stylesheet_contents)
        .transpose()?;
    let stylesheet_route = stylesheet.as_ref().map(|_| "custom.css");
    let plugin_manifests =
        crate::plugins::read_manifests(&options.plugins).map_err(SiteError::InvalidPlugin)?;
    let build_plugins = build_plugins(&plugin_manifests, &options.build_plugins)?;
    let source_root = root.as_ref();
    let root = fs::canonicalize(source_root).map_err(|error| io_error(source_root, error))?;
    let output = fresh_destination(&root, output.as_ref())?;
    let mut garden = load_garden_timed(source_root, &mut stopwatch).map_err(SiteError::Garden)?;
    // Reading the git history takes seconds on a large garden; it runs while pages render.
    let git_root = root.clone();
    let git_dates = queries::LazyDates::new(std::thread::spawn(move || GitDates::read(git_root)));
    let mut report = SiteReport {
        pages: 0,
        journals: 0,
        assets: 0,
        skipped_journals: 0,
        withheld_pages: 0,
        diagnostics: Vec::new(),
        timings: Timings::default(),
    };
    let (settings, settings_diagnostics) = GardenSettings::read(&root);
    let shadowed = crate::publication::remove_shadowed(&mut garden, format, &settings);
    let readable = options.page_urls == PageUrls::Readable;
    let publication = crate::publication::select_with_urls(
        &garden,
        format,
        selection,
        &settings,
        options.journals,
        options.page_urls,
    )
    .map_err(|error| match error {
        SelectError::Title(error) => SiteError::Title(error),
        SelectError::Collision { title } => SiteError::Collision { title },
        SelectError::PageUrl(error) => SiteError::PageUrl(error),
    })?;
    report.skipped_journals = publication.skipped_journals;
    report.withheld_pages = publication.withheld_pages;
    report.diagnostics.extend(
        shadowed
            .into_iter()
            .chain(publication.notices.iter().cloned())
            .map(|notice| SiteDiagnostic {
                source: notice.source,
                message: notice.message,
            }),
    );
    let Publication {
        pages: mut selected,
        routes,
        keys: pages,
        namespaces: generated,
        withheld: withheld_titles,
        garden_titles,
        journal_dates,
        journal_formats,
        names,
        aliases: _,
        spellings,
        alias_spellings,
        folders,
        ..
    } = publication;
    let hierarchy =
        hierarchy::Hierarchy::new(&routes, &withheld_titles, &generated, &alias_spellings);
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
    let (reference_config, config_diagnostics) = config::read(&root);
    let mut shell = shell::Shell {
        flashcards: None,
        graph_shortcut: settings.graph_shortcut,
        favorites,
        all_pages: index_route,
        journals: feed.first_route(),
        contents,
        sidebar,
        properties: reference_config.properties,
        readable,
    };
    let mut homepage_document = None;
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
    // Build plugins run on their own threads while the indexes below are built, and are
    // joined before the first page renders. Without build plugins nothing here runs.
    let pending_plugins = (!build_plugins.is_empty()).then(|| {
        let view = Arc::new(crate::publication::PublicationView::from_parts(
            &selected,
            &journal_dates,
            &generated,
            |route| {
                property_views
                    .get(route)
                    .map_or(&[][..], |view| view.visible.as_slice())
            },
            build_plugins
                .iter()
                .any(|entry| entry.plugin.wants_blocks()),
        ));
        let threads: Vec<_> = build_plugins
            .iter()
            .map(|entry| {
                let plugin = Arc::clone(&entry.plugin);
                let view = Arc::clone(&view);
                std::thread::spawn(move || {
                    let plugin_view = crate::publication::PluginView::new(
                        &view,
                        plugin.page_properties(),
                        plugin.wants_blocks(),
                    );
                    plugin.contribute(&plugin_view)
                })
            })
            .collect();
        (view, threads)
    });
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
    // The graph's layouts and neighbourhood files are computed while pages render.
    let mut graph_files = {
        let mut published: BTreeSet<String> = pages.keys().cloned().collect();
        published.extend(hierarchy.entries().map(|node| lookup_key(&node.title)));
        let hidden = graph::hidden_names(&garden_titles, &published);
        let journal_routes: BTreeSet<String> = journal_dates.keys().cloned().collect();
        let is_journal_name =
            |name: &str| parse_journal_reference(name, &journal_formats).is_some();
        graph::Pending::start(
            &crate::graph::GraphSource {
                index: reference_index.index(),
                journals: &journal_routes,
                hidden: &hidden,
                is_journal_name: &is_journal_name,
            },
            !journal_routes.is_empty(),
            settings.forces,
            options.layout_cache.clone(),
        )
    };
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
    let resolve = |name: &str| reference_index.resolve(&names, name);
    let queries = queries::Queries::new(
        &reference_index,
        &resolve,
        &journal_formats,
        build_date,
        references::Context {
            config: &reference_config,
            names: &names,
            targets: &targets,
            documents: &selected_documents,
        },
        &git_dates,
    );
    // Decks are chosen with the query evaluator, then fixed at the build time.
    let flashcards = settings.features.flashcards.then(|| {
        let now = match options.build_date {
            Some(date) => date.unix_days() * 86_400,
            None => journals::build_seconds(),
        };
        flashcards::Flashcards::new(
            &reference_index,
            queries.graph(),
            &resolve,
            build_date,
            &journal_formats,
            now,
            references::Context {
                config: &reference_config,
                names: &names,
                targets: &targets,
                documents: &selected_documents,
            },
        )
    });
    shell.flashcards = flashcards.as_ref().map(flashcards::Flashcards::due);
    let boxes = PageBoxes {
        queries: &queries,
        flashcards: flashcards.as_ref(),
    };
    // Default journal queries render under the build date's journal; Datalog ones cannot.
    let default_queries: Vec<String> = config::read_text(&root)
        .ok()
        .flatten()
        .map(|text| crate::query::default_journal_queries(&text))
        .unwrap_or_default()
        .into_iter()
        .filter(|text| {
            let datalog = matches!(
                crate::query::parse_advanced(text, build_date, &journal_formats),
                Ok(crate::query::AdvancedQuery {
                    query: crate::query::AdvancedBody::Datalog,
                    ..
                })
            );
            if datalog {
                report.diagnostics.push(SiteDiagnostic {
                    source: "logseq/config.edn".into(),
                    message: "Default journal query skipped: this published site evaluates queries in the simple query language only".to_owned(),
                });
            }
            !datalog
        })
        .collect();
    let mut search = options
        .search
        .as_ref()
        .map(|options| search::Search::new(&selected, &names, &targets, options));
    // Only selected source reaches the renderer and may request asset contents.
    let mut copied: BTreeMap<String, (PathBuf, Vec<u8>)> = BTreeMap::new();
    let mut documents = BTreeMap::new();
    let mut feed_bodies = BTreeMap::new();
    let mut site_pages = Vec::new();
    stopwatch.lap("index");
    let contributions = match pending_plugins {
        None => crate::contributions::Resolved::default(),
        Some((view, threads)) => collect_contributions(&build_plugins, &view, threads)?,
    };
    stopwatch.lap("plugins");
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
        let page_rows = contributions
            .rows
            .get(&route)
            .map(|placed| derived::page_rows(placed))
            .unwrap_or_default();
        source_anchors.extend(
            contributions
                .anchors
                .get(&route)
                .into_iter()
                .flatten()
                .copied(),
        );
        let rendered = render::render_document_with_source_anchors(
            document,
            &names,
            &targets,
            &route,
            &selected_documents,
            render::PageRender {
                source_anchors,
                show_brackets: settings.show_brackets,
                queries: Some(&boxes),
                derived: page_rows.blocks,
            },
            &mut asset,
        );
        let mut rendered = rendered;
        if journal_dates.get(&route) == Some(&build_date) && !default_queries.is_empty() {
            let mut boxes = String::new();
            for text in &default_queries {
                let query = queries.default_query(&route, text, &mut asset);
                boxes.push_str(&query.html);
                rendered.diagnostics.extend(query.diagnostics);
            }
            if !boxes.is_empty() {
                rendered
                    .html
                    .push_str(&format!("<div class=\"lsg-default-queries\">{boxes}</div>"));
            }
        }
        rendered.html.push_str(&page_rows.end);
        documents.extend(queries.take_files());
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
        let id = Some(lookup_key(&title));
        if homepage_route.as_ref() == Some(&route) {
            site_pages.push(plugins::SitePage {
                id: id.clone(),
                title: title.clone(),
                url: "index.html".to_owned(),
                kind: "home",
                properties: Some(route.clone()),
            });
            homepage_document = Some(html_document(
                &title,
                &body,
                stylesheet_route,
                &shell,
                &route,
                true,
            ));
        }
        site_pages.push(plugins::SitePage {
            id,
            title: title.clone(),
            url: route.clone(),
            kind: if is_journal { "journal" } else { "page" },
            properties: Some(route.clone()),
        });
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
            site_pages.push(plugins::SitePage {
                id: Some(lookup_key(&node.title)),
                title: node.title.clone(),
                url: node.route.clone(),
                kind: "namespace",
                properties: None,
            });
            documents.insert(
                node.route.clone(),
                html_document(
                    &node.title,
                    &(page_status::notice().to_owned()
                        + &contributions
                            .rows
                            .get(&node.route)
                            .map(|placed| derived::page_rows(placed).end)
                            .unwrap_or_default()
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
    stopwatch.lap("render");
    let (dates, dates_diagnostic) = git_dates.get();
    stopwatch.lap("history");
    if let Some(message) = dates_diagnostic {
        report.diagnostics.push(SiteDiagnostic {
            source: ".".into(),
            message: message.clone(),
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
        site_pages.push(plugins::SitePage {
            id: None,
            title: "All pages".to_owned(),
            url: route.clone(),
            kind: if route == "index.html" {
                "home"
            } else {
                "list"
            },
            properties: None,
        });
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
    documents.insert(
        all_pages::STYLES_ROUTE.to_owned(),
        all_pages::STYLES.to_owned(),
    );
    if let Some(document) = homepage_document {
        documents.insert("index.html".to_owned(), document);
    }
    let has_journals_graph = graph_files.journals;
    documents.insert(
        graph::PAGE_ROUTE.to_owned(),
        html_page(
            "Graph",
            &graph::main(
                index_route,
                settings.graph,
                settings.forces,
                has_journals_graph,
            ),
            stylesheet_route,
            &shell,
            graph::PAGE_ROUTE,
            false,
            &graph::head(),
        ),
    );
    documents.insert(
        graph::SCRIPT_ROUTE.to_owned(),
        graph::script(readable.then_some(&folders)),
    );
    documents.insert(graph::WORKER_ROUTE.to_owned(), graph::worker());
    documents.insert(graph::STYLES_ROUTE.to_owned(), graph::styles());
    let mut has_feed_page = false;
    for (route, body) in feed.render(&feed_bodies) {
        has_feed_page = true;
        let home = route == "index.html";
        site_pages.push(plugins::SitePage {
            id: None,
            title: "Journals".to_owned(),
            url: route.clone(),
            kind: if home { "home" } else { "list" },
            properties: None,
        });
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
    if has_feed_page {
        documents.insert(
            journals::STYLES_ROUTE.to_owned(),
            journals::STYLES.to_owned(),
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
    if reference_config.properties {
        documents.insert(
            format!("lsg-properties-{}.mjs", digest(properties::SCRIPT)),
            properties::SCRIPT.to_owned(),
        );
    }
    if documents.values().any(|html| html.contains(media::MARKER)) {
        documents.insert(media::SCRIPT_ROUTE.to_owned(), media::SCRIPT.to_owned());
        documents.insert(media::STYLES_ROUTE.to_owned(), media::STYLES.to_owned());
    }
    if let Some(flashcards) = &flashcards {
        documents.extend(flashcards.files(&mut asset));
        for message in flashcards.take_diagnostics() {
            report.diagnostics.push(SiteDiagnostic {
                source: "flashcards".into(),
                message,
            });
        }
        let widget_head = format!(
            "<link rel=\"stylesheet\" href=\"{}\">",
            flashcards::WIDGET_STYLES_ROUTE
        );
        let mut has_widget = false;
        for (route, html) in &mut documents {
            if is_html(route)
                && !route.starts_with("_refs/")
                && !route.starts_with("flashcards/")
                && html.contains(flashcards::MARKER)
            {
                *html = html.replacen("</head>", &(widget_head.clone() + "</head>"), 1);
                has_widget = true;
            }
        }
        if has_widget {
            documents.insert(
                flashcards::WIDGET_STYLES_ROUTE.to_owned(),
                flashcards::WIDGET_STYLES.to_owned(),
            );
        }
        documents.insert(flashcards::SCRIPT_ROUTE.to_owned(), flashcards::script());
        documents.insert(
            flashcards::STYLES_ROUTE.to_owned(),
            flashcards::STYLES.to_owned(),
        );
    }
    let query_head = queries::head();
    let mut has_queries = false;
    for (route, html) in &mut documents {
        if is_html(route) && !route.starts_with("_refs/") && html.contains(queries::MARKER) {
            *html = html.replacen("</head>", &(query_head.clone() + "</head>"), 1);
            has_queries = true;
        }
    }
    if has_queries {
        documents.insert(queries::SCRIPT_ROUTE.to_owned(), queries::script());
        documents.insert(queries::STYLES_ROUTE.to_owned(), queries::STYLES.to_owned());
    }
    documents.insert(shell::SCRIPT_ROUTE.to_owned(), shell::script());
    for (route, contents) in shared_files() {
        documents.insert(route.to_owned(), contents);
    }
    documents.insert(
        outline::SCRIPT_ROUTE.to_owned(),
        if readable {
            readable::outline(outline::SCRIPT)
        } else {
            outline::SCRIPT.to_owned()
        },
    );
    documents.insert(shell::SIDEBAR_ROUTE.to_owned(), shell::sidebar_script());
    documents.insert(shell::SLIDES_ROUTE.to_owned(), shell::slides_script());
    documents.insert(
        shell::SIDEBAR_STYLES_ROUTE.to_owned(),
        shell::SIDEBAR_STYLES.to_owned(),
    );
    documents.insert(references::SCRIPT_ROUTE.to_owned(), references::script());
    stopwatch.lap("render");
    if let Some(search) = search {
        let (artifacts, manifest_route, revision) = search.artifacts(readable);
        let controller = include_str!("search-ui.mjs");
        let controller_route = format!("lsg-search-ui-{}.mjs", digest(controller));
        let startup = include_str!("search-start.mjs")
            .lines()
            .map(str::trim)
            .collect::<Vec<_>>()
            .join("\n")
            .replace("__LSG_SEARCH_UI__", &controller_route);
        let startup_route = format!("lsg-search-start-{}.mjs", digest(&startup));
        let boot = include_str!("search-boot.js")
            .lines()
            .filter(|line| !line.starts_with("//"))
            .collect::<String>()
            .replace("__STARTUP__", &startup_route);
        let link = format!(
            "<link rel=\"lsg-search\" href=\"{manifest_route}\" data-revision=\"{revision}\"><link rel=\"stylesheet\" href=\"{SEARCH_STYLES_ROUTE}\"><script defer src=\"{SEARCH_BOOT_ROUTE}\"></script>"
        );
        for (route, html) in &mut documents {
            if is_html(route) && !route.starts_with("_refs/") && !route.starts_with("_graph/") {
                *html = html.replacen("</head>", &(link.clone() + "</head>"), 1);
            }
        }
        documents.insert(
            SEARCH_STYLES_ROUTE.to_owned(),
            include_str!("search-ui.css").to_owned(),
        );
        documents.insert(SEARCH_BOOT_ROUTE.to_owned(), boot);
        documents.extend(artifacts);
        documents.insert(controller_route, controller.to_owned());
        documents.insert(startup_route, startup);
    }
    stopwatch.lap("search");
    let neighbourhoods = graph_files.neighbourhoods();
    stopwatch.lap("graph");
    documents.extend(neighbourhoods);
    // The graph's layout data files are left out: they are still being computed, and they
    // follow from the pages and settings that the other files already reflect.
    if !contributions.rows.is_empty() {
        for (route, html) in &mut documents {
            if is_html(route) && !route.starts_with("_refs/") && html.contains(derived::MARKER) {
                *html = html.replacen("</head>", &(derived::head() + "</head>"), 1);
            }
        }
        documents.insert(derived::STYLES_ROUTE.to_owned(), derived::STYLES.to_owned());
    }
    if readable {
        let script = readable::script();
        for (route, html) in &mut documents {
            if is_readable(route) || route.ends_with(".html") && !route.contains('/') {
                *html = html.replacen(
                    "<meta charset=\"utf-8\">",
                    &format!("<meta charset=\"utf-8\">{}", readable::head(route, &script)),
                    1,
                );
            }
        }
    }
    let plugin_files = plugins::files(&plugin_manifests);
    let revision = plugins::revision(
        documents
            .iter()
            .map(|(route, text)| (route.as_str(), text.as_bytes()))
            .chain(
                copied
                    .iter()
                    .map(|(route, (_, contents))| (route.as_str(), contents.as_slice())),
            )
            .chain(
                stylesheet
                    .iter()
                    .map(|contents| ("custom.css", contents.as_slice())),
            )
            .chain(
                plugin_files
                    .iter()
                    .map(|(route, contents)| (route.as_str(), *contents)),
            ),
    );
    if plugin_manifests
        .iter()
        .any(|plugin| plugin.module.is_some())
    {
        let (host_route, host) = plugins::host(&plugin_manifests, readable);
        let none = Vec::new();
        for page in &site_pages {
            let visible = page
                .properties
                .as_ref()
                .map_or(&none, |route| &property_views[route].visible);
            let contributed = contributions
                .metadata
                .get(page.properties.as_ref().unwrap_or(&page.url));
            let head = plugins::page_head(
                page,
                &revision,
                &plugin_manifests,
                visible,
                contributed,
                &host_route,
            );
            if let Some(html) = documents.get_mut(&page.url) {
                *html = html.replacen("</head>", &(head + "</head>"), 1);
            }
        }
        documents.insert(host_route, host);
    }
    documents.insert(
        plugins::SITE_MANIFEST.to_owned(),
        plugins::site_manifest(
            &site_pages,
            copied.keys().cloned(),
            &plugin_manifests,
            &revision,
        ),
    );
    stopwatch.lap("render");
    let graph_data = graph_files.finish();
    stopwatch.lap("graph");
    if readable {
        let titles: Vec<String> = documents
            .keys()
            .filter(|route| is_readable(route))
            .map(|route| readable_title(route))
            .collect();
        let generated: BTreeSet<&str> = documents
            .keys()
            .filter(|route| !is_readable(route))
            .chain(copied.keys())
            .chain(plugin_files.keys())
            .map(String::as_str)
            .chain(graph_data.iter().map(|(route, _)| *route))
            .chain(stylesheet.as_ref().map(|_| "custom.css"))
            .map(|route| route.split('/').next().unwrap_or(route))
            .collect();
        check_reserved(titles.iter().map(String::as_str), generated).map_err(SiteError::PageUrl)?;
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
    if documents.contains_key(flashcards::DECK_LIST) {
        for directory in flashcards::DIRECTORIES {
            let directory = output.join(directory);
            fs::create_dir(&directory).map_err(|error| io_error(&directory, error))?;
        }
    }
    for directory in [graph::DIRECTORY, graph::NEIGHBOURHOODS] {
        let directory = output.join(directory);
        fs::create_dir(&directory).map_err(|error| io_error(&directory, error))?;
    }
    for route in documents.keys().filter(|route| is_readable(route)) {
        let path = output.join(output_path(route));
        let directory = path.parent().unwrap_or(&output);
        fs::create_dir_all(directory).map_err(|error| io_error(directory, error))?;
    }
    write_files(&output, documents)?;
    for (route, contents) in plugin_files {
        let path = output.join(&route);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| io_error(parent, error))?;
        }
        fs::write(&path, contents).map_err(|error| io_error(&path, error))?;
    }
    for (route, bytes) in graph_data {
        let path = output.join(route);
        fs::write(&path, bytes).map_err(|error| io_error(&path, error))?;
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
    stopwatch.lap("write");
    report.timings = stopwatch.finish();
    Ok(report)
}

/// A build plugin with whether its manifest names a browser module.
struct BuildEntry {
    plugin: Arc<dyn crate::contributions::BuildPlugin>,
    has_module: bool,
}

/// Every build plugin, from manifests with a `build` section and from the caller, in id order.
fn build_plugins(
    manifests: &[crate::plugins::PluginManifest],
    in_process: &crate::contributions::BuildPlugins,
) -> Result<Vec<BuildEntry>, SiteError> {
    let mut entries: Vec<BuildEntry> = manifests
        .iter()
        .filter_map(|manifest| {
            crate::plugins::ProcessPlugin::new(manifest).map(|plugin| BuildEntry {
                plugin: Arc::new(plugin),
                has_module: manifest.module.is_some(),
            })
        })
        .collect();
    for plugin in &in_process.0 {
        let id = plugin.id();
        let fail = |problem: String| {
            SiteError::BuildPlugin(crate::contributions::ContributionError {
                plugin: id.to_owned(),
                position: None,
                problem,
            })
        };
        if !crate::plugins::valid_id(id) {
            return Err(fail(
                "id must be 1 to 64 lower-case letters, digits and single hyphens".to_owned(),
            ));
        }
        if let Some(manifest) = manifests.iter().find(|manifest| manifest.id == id) {
            return Err(fail(format!(
                "id {id:?} is also declared by plugin manifest {}",
                manifest.source.display()
            )));
        }
        if entries.iter().any(|entry| entry.plugin.id() == id) {
            return Err(fail(format!("id {id:?} is declared by two build plugins")));
        }
        entries.push(BuildEntry {
            plugin: Arc::clone(plugin),
            has_module: false,
        });
    }
    entries.sort_by(|left, right| left.plugin.id().cmp(right.plugin.id()));
    Ok(entries)
}

type PluginThread =
    std::thread::JoinHandle<Result<Vec<crate::contributions::Contribution>, String>>;

/// Wait for every build plugin, then check and resolve their contributions together.
fn collect_contributions(
    entries: &[BuildEntry],
    view: &crate::publication::PublicationView,
    threads: Vec<PluginThread>,
) -> Result<crate::contributions::Resolved, SiteError> {
    // Join every thread before reporting a failure, so no plugin outlives the build.
    let results: Vec<_> = threads
        .into_iter()
        .map(|thread| {
            thread
                .join()
                .unwrap_or_else(|_| Err("the plugin panicked".to_owned()))
        })
        .collect();
    let mut outputs = Vec::with_capacity(entries.len());
    for (entry, result) in entries.iter().zip(results) {
        let contributions = result.map_err(|problem| {
            SiteError::BuildPlugin(crate::contributions::ContributionError {
                plugin: entry.plugin.id().to_owned(),
                position: None,
                problem,
            })
        })?;
        outputs.push(crate::contributions::PluginOutput {
            id: entry.plugin.id(),
            has_module: entry.has_module,
            page_properties: entry.plugin.page_properties(),
            blocks: entry.plugin.wants_blocks() && view.has_blocks(),
            contributions,
        });
    }
    crate::contributions::resolve(view, &mut outputs).map_err(SiteError::BuildPlugin)
}

/// Whether the document at `route` is an HTML page: a file ending in `.html` or a readable
/// page folder.
fn is_html(route: &str) -> bool {
    route.ends_with(".html") || is_readable(route)
}

/// Write the generated files on several threads: creating files, not rendering them,
/// dominates the time to write a large garden. A route ending in `.gz` is stored
/// gzip-compressed. Each thread takes every `threads`-th file, so the compressed files, which
/// sort together under `_refs/`, spread evenly across the threads.
fn write_files(output: &Path, documents: BTreeMap<String, String>) -> Result<(), SiteError> {
    let documents: Vec<_> = documents.into_iter().collect();
    let threads = std::thread::available_parallelism().map_or(1, |n| n.get().min(8));
    std::thread::scope(|scope| {
        let documents = &documents;
        let workers: Vec<_> = (0..threads)
            .map(|first| {
                scope.spawn(move || {
                    for (route, html) in documents.iter().skip(first).step_by(threads) {
                        let path = output.join(output_path(route));
                        let written = if route.ends_with(".gz") {
                            fs::write(&path, gzip(html.as_bytes()))
                        } else {
                            fs::write(&path, html)
                        };
                        written.map_err(|error| io_error(&path, error))?;
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

/// `bytes` gzip-compressed at level 6, with no file name and a zero time in the header, so the
/// same bytes always produce the same file.
fn gzip(bytes: &[u8]) -> Vec<u8> {
    use std::io::Write;
    let mut encoder = flate2::GzBuilder::new()
        .mtime(0)
        .write(Vec::new(), flate2::Compression::new(6));
    encoder
        .write_all(bytes)
        .expect("writing to memory cannot fail");
    encoder.finish().expect("writing to memory cannot fail")
}

/// What renders a page's query boxes and deck widgets.
struct PageBoxes<'x, 'r, 'a> {
    queries: &'x queries::Queries<'r, 'a>,
    flashcards: Option<&'x flashcards::Flashcards<'r, 'a>>,
}

impl render::QueryBoxes for PageBoxes<'_, '_, '_> {
    fn simple(
        &self,
        page: &str,
        block: Option<usize>,
        text: &str,
        asset: &mut dyn FnMut(&Path, &str) -> Result<String, String>,
    ) -> render::QueryBox {
        self.queries.simple(page, block, text, asset)
    }
    fn advanced(
        &self,
        page: &str,
        block: Option<usize>,
        text: &str,
        asset: &mut dyn FnMut(&Path, &str) -> Result<String, String>,
    ) -> render::QueryBox {
        self.queries.advanced(page, block, text, asset)
    }
    fn cards(
        &self,
        text: &str,
        asset: &mut dyn FnMut(&Path, &str) -> Result<String, String>,
    ) -> Option<render::QueryBox> {
        self.flashcards
            .map(|flashcards| flashcards.widget(text, asset))
    }
}
