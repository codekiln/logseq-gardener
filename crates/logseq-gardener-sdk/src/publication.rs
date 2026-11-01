//! The pages a publication contains, and the read-only view build plugins receive.
//!
//! [`select`] applies namespace selection, `public:: false` withholding and Logseq's
//! all-pages-public rule, assigns each published page its route and collects aliases. The
//! publisher renders exactly the pages it returns, so any other caller that uses it (an editor
//! integration, a build tool) agrees with the generated site about what is published.
//!
//! [`PublicationView`] is the plugin-facing projection of that selection: page ids, titles,
//! URLs and kinds, and optionally every block's text, with no source paths, withheld pages or
//! hidden properties. [`PluginView`] narrows it to what one plugin asked for.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::io::{self, Write};
use std::path::PathBuf;

use serde::Serialize;
use serde::ser::{SerializeMap, SerializeSeq, SerializeStruct, Serializer};

use crate::ast::{Block, ListItem};
use crate::garden::{DocumentKind, Garden, GardenDocument};
use crate::garden_settings::GardenSettings;
use crate::journals::{JournalDate, JournalFormats, journal_identity};
use crate::outline::Outline;
use crate::page_names::{PageNameIndex, lookup_key, page_alias_spellings, page_aliases};
use crate::page_titles::{FilenameFormat, PageTitleError, document_title, page_title};
use crate::page_urls::{
    PageUrlError, PageUrls, check_collisions, is_readable, readable_url, same_path,
};
use crate::publishing::{NamespaceSelection, PublicPages};
use crate::search::digest;

/// Nesting deeper than this withholds a page rather than assuming it is visible.
const MAX_DEPTH: usize = 64;

/// A selected-source file that was skipped or withheld, for the caller's diagnostics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notice {
    /// Source file relative to the garden root.
    pub source: PathBuf,
    /// Why the file was skipped or withheld.
    pub message: String,
}

/// Selection that cannot produce a publication.
#[derive(Debug)]
pub enum SelectError {
    /// Deriving a page title failed.
    Title(PageTitleError),
    /// Two selected pages, or a page and a namespace destination, share a name or a route.
    Collision { title: String },
    /// With readable URLs, two pages' readable paths collide.
    PageUrl(PageUrlError),
}

impl fmt::Display for SelectError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Title(error) => error.fmt(f),
            Self::Collision { title } => write!(f, "ambiguous selected title or route: {title}"),
            Self::PageUrl(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for SelectError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Title(error) => Some(error),
            Self::Collision { .. } => None,
            Self::PageUrl(error) => Some(error),
        }
    }
}

/// The published pages of a garden and the names that resolve to them.
pub struct Publication<'a> {
    /// `(title, route, document)` of every published page and journal, in garden order.
    pub pages: Vec<(String, String, &'a GardenDocument)>,
    /// Route to title, for the published pages.
    pub routes: BTreeMap<String, String>,
    /// Lower-cased title to route, for the published pages.
    pub keys: BTreeMap<String, String>,
    /// Namespace destinations with no source file: lower-cased title to `(title, route)`.
    pub namespaces: BTreeMap<String, (String, String)>,
    /// Lower-cased titles of every page in the garden that is withheld, selected or not.
    pub withheld: BTreeSet<String>,
    /// Lower-cased titles of every page and journal in the garden.
    pub garden_titles: BTreeSet<String>,
    /// Route to date, for the published journals.
    pub journal_dates: BTreeMap<String, JournalDate>,
    /// The journal file-name and title formats the titles were derived with.
    pub journal_formats: JournalFormats,
    /// Titles, aliases and journal dates of the published pages.
    pub names: PageNameIndex<String>,
    /// Lower-cased aliases of the published pages.
    pub aliases: BTreeSet<String>,
    /// Lower-cased title or alias to its first authored spelling.
    pub spellings: BTreeMap<String, String>,
    /// Lower-cased title to its authored alias spellings.
    pub alias_spellings: BTreeMap<String, Vec<String>>,
    /// Journals left out because journals were not requested or their names are not dates.
    pub skipped_journals: usize,
    /// Selected pages withheld by `public:: false` or the all-pages-public rule.
    pub withheld_pages: usize,
    /// Skipped and withheld files.
    pub notices: Vec<Notice>,
}

/// Remove documents whose titles repeat a page in `pages/` or `journals/`, as Logseq does, and
/// return a notice for each. Call this before [`select`].
pub fn remove_shadowed(
    garden: &mut Garden,
    format: FilenameFormat,
    settings: &GardenSettings,
) -> Vec<Notice> {
    let journal_formats = JournalFormats::from_settings(settings);
    garden
        .remove_shadowed_documents(|document| {
            document_title(document, format, &journal_formats).map(|title| lookup_key(&title))
        })
        .into_iter()
        .map(|source| Notice {
            source,
            message: "Page skipped: a page in pages/ or journals/ has the same title".to_owned(),
        })
        .collect()
}

/// Choose the published pages of a garden and give each its `p-<digest>.html` route.
///
/// A page is published when the namespace selection includes it (journals are not subject to
/// it), it contains no `public:: false`, and the garden's all-pages-public setting or its own
/// `public:: true` makes it public (see [`PublicPages`]). Journals are published only when
/// `journals` is true and their file name is a date.
///
/// # Errors
///
/// Returns a page whose title cannot be derived, or a title or route shared by two pages.
pub fn select<'a>(
    garden: &'a Garden,
    format: FilenameFormat,
    selection: &NamespaceSelection,
    settings: &GardenSettings,
    journals: bool,
) -> Result<Publication<'a>, SelectError> {
    select_with_urls(
        garden,
        format,
        selection,
        settings,
        journals,
        PageUrls::Digest,
    )
}

/// Choose the published pages of a garden, as [`select`] does, and give each its route in the
/// `urls` layout. With [`PageUrls::Readable`], every page and namespace destination gets its
/// [`readable_url`]. A title that cannot form a safe path keeps its `p-<digest>.html` route,
/// and its page gets a notice that names the title, the problem and the route.
///
/// # Errors
///
/// Returns a page whose title cannot be derived, a title or route shared by two pages, and with
/// readable URLs pages whose readable paths collide (see [`check_collisions`]).
pub fn select_with_urls<'a>(
    garden: &'a Garden,
    format: FilenameFormat,
    selection: &NamespaceSelection,
    settings: &GardenSettings,
    journals: bool,
    urls: PageUrls,
) -> Result<Publication<'a>, SelectError> {
    let journal_formats = JournalFormats::from_settings(settings);
    let mut skipped_journals = 0;
    let mut withheld_pages = 0;
    let mut notices = Vec::new();
    let mut keys = BTreeMap::new();
    let mut routes: BTreeMap<String, String> = BTreeMap::new();
    let mut pages = Vec::new();
    let mut withheld = BTreeSet::new();
    let mut garden_titles = BTreeSet::new();
    let public = if settings.all_pages_public {
        PublicPages::all()
    } else {
        PublicPages::marked(garden.documents.iter().filter_map(|document| {
            document_title(document, format, &journal_formats).map(|title| (title, document))
        }))
    };
    let mut journal_dates = BTreeMap::new();
    for document in &garden.documents {
        let journal = if document.kind == DocumentKind::Journal {
            if !journals {
                skipped_journals += 1;
                continue;
            }
            match journal_identity(document, &journal_formats) {
                Ok(identity) => Some(identity),
                Err(_) => {
                    skipped_journals += 1;
                    notices.push(Notice {
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
            None => page_title(document, format).map_err(SelectError::Title)?,
        };
        garden_titles.insert(lookup_key(&title));
        let private = has_private_properties(&document.parsed.blocks);
        let unmarked = !public.includes(&title);
        if private || unmarked {
            withheld.insert(lookup_key(&title));
        }
        if journal.is_none() && !selection.includes(&title) {
            continue;
        }
        if private || unmarked {
            withheld_pages += 1;
            notices.push(Notice {
                source: document.relative_path.clone(),
                message: if private {
                    "Page withheld by private-property or nesting visibility check"
                } else {
                    "Page withheld: it is not marked public:: true and the garden does not set :publishing/all-pages-public? true"
                }
                .to_owned(),
            });
            continue;
        }
        let (route, unsafe_title) = route_for(&title, urls);
        if let Some(error) = unsafe_title {
            notices.push(Notice {
                source: document.relative_path.clone(),
                message: format!("{error}; it is published at {route}"),
            });
        }
        if let Some(previous) = keys.insert(lookup_key(&title), route.clone()) {
            // Readable URLs name both pages, since their folders would be one on disk.
            return Err(match routes.get(&previous) {
                Some(other) if urls == PageUrls::Readable => {
                    SelectError::PageUrl(same_path(other, &title))
                }
                _ => SelectError::Collision { title },
            });
        }
        if routes.insert(route.clone(), title.clone()).is_some() {
            return Err(SelectError::Collision { title });
        }
        if let Some(identity) = journal {
            journal_dates.insert(route.clone(), identity.date);
        }
        pages.push((title, route, document));
    }
    let mut names = PageNameIndex::with_journal_formats(journal_formats.clone());
    let mut aliases = BTreeSet::new();
    let mut spellings = BTreeMap::new();
    let mut alias_spellings = BTreeMap::new();
    for (title, route, document) in &pages {
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
    let namespaces = namespace_destinations(&routes, &keys, &withheld, &aliases, urls)?;
    if urls == PageUrls::Readable {
        check_collisions(
            routes
                .iter()
                .chain(namespaces.values().map(|(title, route)| (route, title)))
                .filter(|(route, _)| is_readable(route))
                .map(|(_, title)| title.as_str()),
        )
        .map_err(SelectError::PageUrl)?;
    }
    Ok(Publication {
        pages,
        routes,
        keys,
        namespaces,
        withheld,
        garden_titles,
        journal_dates,
        journal_formats,
        names,
        aliases,
        spellings,
        alias_spellings,
        skipped_journals,
        withheld_pages,
        notices,
    })
}

/// A page's route in the `urls` layout. With readable URLs, a title that cannot form a safe
/// path keeps its digest route, returned with the reason.
fn route_for(title: &str, urls: PageUrls) -> (String, Option<PageUrlError>) {
    let digest_route = || format!("p-{}.html", digest(title));
    match urls {
        PageUrls::Digest => (digest_route(), None),
        PageUrls::Readable => match readable_url(title) {
            Ok(route) => (route, None),
            Err(error) => (digest_route(), Some(error)),
        },
    }
}

/// Every namespace prefix of a published title that is not itself a published, withheld or
/// aliased page, with its earliest spelling and its route. A prefix that cannot form a safe
/// readable path keeps its digest route without a notice of its own: every segment of the prefix
/// belongs to a published title, whose page already has one.
fn namespace_destinations(
    routes: &BTreeMap<String, String>,
    keys: &BTreeMap<String, String>,
    withheld: &BTreeSet<String>,
    aliases: &BTreeSet<String>,
    urls: PageUrls,
) -> Result<BTreeMap<String, (String, String)>, SelectError> {
    let mut prefixes: BTreeMap<String, String> = BTreeMap::new();
    for title in routes.values() {
        for (end, _) in title.match_indices('/') {
            let prefix = &title[..end];
            let key = lookup_key(prefix);
            if keys.contains_key(&key) || withheld.contains(&key) || aliases.contains(&key) {
                continue;
            }
            prefixes
                .entry(key)
                .and_modify(|previous| {
                    if prefix < previous.as_str() {
                        prefix.clone_into(previous);
                    }
                })
                .or_insert_with(|| prefix.to_owned());
        }
    }
    let mut taken: BTreeSet<String> = routes.keys().cloned().collect();
    let mut destinations = BTreeMap::new();
    for (key, title) in prefixes {
        let (route, _) = route_for(&title, urls);
        if !taken.insert(route.clone()) {
            return Err(SelectError::Collision { title });
        }
        destinations.insert(key, (title, route));
    }
    Ok(destinations)
}

/// Whether a page contains `public:: false` anywhere, or nests too deeply to check.
#[must_use]
pub fn has_private_properties(blocks: &[Block]) -> bool {
    fn items_private(items: &[ListItem], depth: usize) -> bool {
        depth > MAX_DEPTH
            || items.iter().any(|item| {
                private(&item.content, depth + 1) || items_private(&item.items, depth + 1)
            })
    }
    fn private(blocks: &[Block], depth: usize) -> bool {
        depth > MAX_DEPTH
            || blocks.iter().any(|block| match block {
                Block::Properties { props, .. } => props.iter().any(|p| {
                    p.0.trim().eq_ignore_ascii_case("public")
                        && p.1.trim().eq_ignore_ascii_case("false")
                }),
                Block::Quote { children, .. } | Block::Custom { children, .. } => {
                    private(children, depth + 1)
                }
                Block::List { items, .. } => items_private(items, depth + 1),
                _ => false,
            })
    }
    private(blocks, 0)
}

/// The view format version.
pub const VIEW_API_VERSION: u64 = 1;

/// What a view page is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ViewPageKind {
    /// A page with a source file.
    Page,
    /// A journal with a source file.
    Journal,
    /// A namespace destination generated for a prefix with no page of its own.
    Namespace,
}

/// One published page of a view.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewPage {
    /// Logseq's lower-cased page name.
    pub id: String,
    /// The title the page shows.
    pub title: String,
    /// The generated file, relative to the output root.
    pub url: String,
    pub kind: ViewPageKind,
    /// Page properties the page shows readers, as `(lower-cased name, authored text)`.
    visible: Vec<(String, String)>,
}

/// Where a block link points inside its page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockAnchor {
    /// `#block-<uuid>`, which the page always carries for a block with a unique UUID.
    Uuid(String),
    /// `#source-block-<offset>`, which the page carries only when asked for this offset.
    Source(usize),
    /// The block has no position in the source; links go to the page.
    None,
}

/// One block of a view page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewBlock {
    /// The block's unique `id::` UUID, otherwise its position path such as `3.1.2`.
    pub id: String,
    /// The id of the page that holds the block.
    pub page_id: String,
    /// The block's authored Markdown without its property lines and without its children.
    pub text: String,
    /// The ids of the block's children, in order.
    pub children: Vec<String>,
    /// Byte offset of the block's bullet in its page's source, which the renderer keys on.
    pub start: Option<usize>,
    anchor: BlockAnchor,
}

impl ViewBlock {
    /// The fragment a link to this block uses, without `#`; `None` links to the page.
    #[must_use]
    pub fn fragment(&self) -> Option<String> {
        match &self.anchor {
            BlockAnchor::Uuid(id) => Some(format!("block-{id}")),
            BlockAnchor::Source(start) => Some(format!("source-block-{start}")),
            BlockAnchor::None => None,
        }
    }

    /// Where a link to this block points inside its page.
    #[must_use]
    pub fn anchor(&self) -> &BlockAnchor {
        &self.anchor
    }
}

/// The published pages, and optionally blocks, of one build, shared by every build plugin.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PublicationView {
    pages: Vec<ViewPage>,
    blocks: Vec<ViewBlock>,
    has_blocks: bool,
    pages_by_id: BTreeMap<String, usize>,
    blocks_by_id: BTreeMap<(String, String), usize>,
}

impl PublicationView {
    /// Build the view of a publication.
    ///
    /// `visible` gives the page properties a published page shows readers, by route, as
    /// `(lower-cased name, authored text)`; the publisher passes the rows it renders, so a
    /// plugin sees no property that readers cannot. Blocks are read only when `blocks` is true.
    pub fn build<'v>(
        publication: &Publication<'_>,
        visible: impl FnMut(&str) -> &'v [(String, String)],
        blocks: bool,
    ) -> Self {
        Self::from_parts(
            &publication.pages,
            &publication.journal_dates,
            &publication.namespaces,
            visible,
            blocks,
        )
    }

    /// [`build`](Self::build) from the fields of a [`Publication`].
    pub(crate) fn from_parts<'v>(
        published: &[(String, String, &GardenDocument)],
        journal_dates: &BTreeMap<String, JournalDate>,
        namespaces: &BTreeMap<String, (String, String)>,
        mut visible: impl FnMut(&str) -> &'v [(String, String)],
        blocks: bool,
    ) -> Self {
        let mut pages: Vec<(ViewPage, Option<&GardenDocument>)> = published
            .iter()
            .map(|(title, route, document)| {
                (
                    ViewPage {
                        id: lookup_key(title),
                        title: title.clone(),
                        url: route.clone(),
                        kind: if journal_dates.contains_key(route) {
                            ViewPageKind::Journal
                        } else {
                            ViewPageKind::Page
                        },
                        visible: visible(route).to_vec(),
                    },
                    Some(*document),
                )
            })
            .collect();
        pages.extend(namespaces.values().map(|(title, route)| {
            (
                ViewPage {
                    id: lookup_key(title),
                    title: title.clone(),
                    url: route.clone(),
                    kind: ViewPageKind::Namespace,
                    visible: Vec::new(),
                },
                None,
            )
        }));
        pages.sort_by(|left, right| left.0.id.cmp(&right.0.id));
        let mut view_blocks = Vec::new();
        if blocks {
            let mut counts: BTreeMap<String, usize> = BTreeMap::new();
            for (_, document) in &pages {
                if let Some(document) = document {
                    for (id, _, _) in crate::transclusion::page_targets(&document.parsed.blocks) {
                        *counts.entry(id).or_default() += 1;
                    }
                }
            }
            for (page, document) in &pages {
                if let Some(document) = document {
                    page_blocks(&page.id, document, &counts, &mut view_blocks);
                }
            }
        }
        let pages: Vec<ViewPage> = pages.into_iter().map(|(page, _)| page).collect();
        let pages_by_id = pages
            .iter()
            .enumerate()
            .map(|(index, page)| (page.id.clone(), index))
            .collect();
        let blocks_by_id = view_blocks
            .iter()
            .enumerate()
            .map(|(index, block)| ((block.page_id.clone(), block.id.clone()), index))
            .collect();
        Self {
            pages,
            blocks: view_blocks,
            has_blocks: blocks,
            pages_by_id,
            blocks_by_id,
        }
    }

    /// Every published page, sorted by id.
    #[must_use]
    pub fn pages(&self) -> &[ViewPage] {
        &self.pages
    }

    /// Every block of every page with a source file, page by page in document order; empty
    /// when the view was built without blocks.
    #[must_use]
    pub fn blocks(&self) -> &[ViewBlock] {
        &self.blocks
    }

    /// Whether the view was built with blocks.
    #[must_use]
    pub fn has_blocks(&self) -> bool {
        self.has_blocks
    }

    /// The page with this id.
    #[must_use]
    pub fn page(&self, id: &str) -> Option<&ViewPage> {
        self.pages_by_id.get(id).map(|index| &self.pages[*index])
    }

    /// The block with this id on the page with this id.
    #[must_use]
    pub fn block(&self, page_id: &str, block_id: &str) -> Option<&ViewBlock> {
        self.blocks_by_id
            .get(&(page_id.to_owned(), block_id.to_owned()))
            .map(|index| &self.blocks[*index])
    }

    /// The values of `names` among a page's visible properties, lower-cased names to trimmed
    /// authored text. Empty when the page shows none of them.
    #[must_use]
    pub fn page_values(&self, page: &ViewPage, names: &[String]) -> BTreeMap<String, String> {
        crate::plugins::page_values(
            names,
            page.visible
                .iter()
                .map(|(name, text)| (name.as_str(), text.as_str())),
        )
    }
}

/// Append the outline blocks of one page to `out`.
fn page_blocks(
    page_id: &str,
    document: &GardenDocument,
    uuid_counts: &BTreeMap<String, usize>,
    out: &mut Vec<ViewBlock>,
) {
    let source = &document.parsed.blocks;
    let outline = Outline::from_blocks(source);
    let ids: Vec<String> = outline
        .blocks
        .iter()
        .map(|block| match &block.id {
            crate::outline::BlockId::Uuid(id) if uuid_counts.get(id) == Some(&1) => id.clone(),
            _ => crate::outline::BlockId::Path(block.path.clone()).to_string(),
        })
        .collect();
    let mut children: Vec<Vec<String>> = vec![Vec::new(); outline.blocks.len()];
    for (index, block) in outline.blocks.iter().enumerate() {
        if let Some(parent) = block.parent {
            children[parent].push(ids[index].clone());
        }
    }
    for (index, block) in outline.blocks.iter().enumerate() {
        let next = outline
            .blocks
            .get(index + 1)
            .map_or(source.len(), |next| next.index);
        let anchor = match (&block.id, block.start) {
            (crate::outline::BlockId::Uuid(id), _) if uuid_counts.get(id) == Some(&1) => {
                BlockAnchor::Uuid(id.clone())
            }
            (_, Some(start)) => BlockAnchor::Source(start),
            (_, None) => BlockAnchor::None,
        };
        out.push(ViewBlock {
            id: ids[index].clone(),
            page_id: page_id.to_owned(),
            text: block_text(&document.source, &source[block.index..next]),
            children: std::mem::take(&mut children[index]),
            start: block.start,
            anchor,
        });
    }
}

/// The authored text of a bullet and the non-bullet blocks after it, without the bullet
/// marker, the property lines or the continuation indentation.
fn block_text(source: &str, blocks: &[Block]) -> String {
    let Some(Block::Bullet {
        span: Some(first), ..
    }) = blocks.first()
    else {
        return String::new();
    };
    let end = blocks
        .iter()
        .rev()
        .find_map(block_span)
        .map_or(first.1, |span| span.1.max(first.1));
    let mut kept = String::new();
    let mut position = first.0;
    for block in blocks {
        if let (Block::Properties { .. }, Some(span)) = (block, block_span(block))
            && span.0 >= position
        {
            kept.push_str(source.get(position..span.0).unwrap_or_default());
            position = span.1;
        }
    }
    kept.push_str(
        source
            .get(position..end.min(source.len()))
            .unwrap_or_default(),
    );
    let mut lines = kept.lines();
    let Some(head) = lines.next() else {
        return String::new();
    };
    let indent = &head[..head.len() - head.trim_start().len()];
    let head = head.trim_start();
    let head = head.strip_prefix('-').unwrap_or(head);
    let mut text = head.strip_prefix(' ').unwrap_or(head).to_owned();
    for line in lines {
        let line = line.strip_prefix(indent).unwrap_or(line);
        let line = line
            .strip_prefix("  ")
            .or_else(|| line.strip_prefix('\t'))
            .unwrap_or(line);
        text.push('\n');
        text.push_str(line);
    }
    text.trim_end().to_owned()
}

fn block_span(block: &Block) -> Option<(usize, usize)> {
    let span = match block {
        Block::Bullet { span, .. }
        | Block::Paragraph { span, .. }
        | Block::Properties { span, .. }
        | Block::Heading { span, .. }
        | Block::List { span, .. }
        | Block::Quote { span, .. }
        | Block::Custom { span, .. }
        | Block::Src { span, .. }
        | Block::Example { span, .. }
        | Block::Table { span, .. }
        | Block::Hr { span, .. }
        | Block::FootnoteDef { span, .. }
        | Block::RawHtml { span, .. }
        | Block::Hiccup { span, .. }
        | Block::DisplayedMath { span, .. }
        | Block::Export { span, .. }
        | Block::LatexEnv { span, .. }
        | Block::Drawer { span, .. }
        | Block::Directive { span, .. }
        | Block::Results { span, .. }
        | Block::Comment { span, .. }
        | Block::CommentBlock { span, .. } => span,
    };
    span.as_ref().map(|span| (span.0, span.1))
}

/// The view one plugin receives: every page with only the properties it named, and blocks only
/// when it asked for them. Serializes to the process protocol's `view` object.
pub struct PluginView<'a> {
    view: &'a PublicationView,
    properties: Vec<BTreeMap<String, String>>,
    blocks: bool,
    revision: String,
}

impl<'a> PluginView<'a> {
    /// Narrow `view` for a plugin that reads `page_properties` and, when `blocks` is true,
    /// blocks. Asking for blocks of a view built without them gives no blocks.
    #[must_use]
    pub fn new(view: &'a PublicationView, page_properties: &[String], blocks: bool) -> Self {
        let properties = view
            .pages
            .iter()
            .map(|page| view.page_values(page, page_properties))
            .collect();
        let mut plugin_view = Self {
            view,
            properties,
            blocks: blocks && view.has_blocks,
            revision: String::new(),
        };
        let mut hash = Fnv::default();
        hash.write_all(crate::VERSION.as_bytes())
            .and_then(|()| hash.write_all(&[0]))
            .expect("hashing does not fail");
        serde_json::to_writer(&mut hash, &plugin_view).expect("the view serializes");
        plugin_view.revision = format!("{:016x}", hash.0);
        plugin_view
    }

    /// A digest of this view's content and the `lsg` version.
    #[must_use]
    pub fn revision(&self) -> &str {
        &self.revision
    }

    /// Every published page, sorted by id, with this plugin's property values.
    pub fn pages(&self) -> impl Iterator<Item = (&'a ViewPage, &BTreeMap<String, String>)> {
        self.view.pages.iter().zip(&self.properties)
    }

    /// Every block, or none when the plugin did not ask for blocks.
    #[must_use]
    pub fn blocks(&self) -> &'a [ViewBlock] {
        if self.blocks { &self.view.blocks } else { &[] }
    }

    /// Whether this plugin receives blocks.
    #[must_use]
    pub fn has_blocks(&self) -> bool {
        self.blocks
    }

    /// The shared view this one narrows.
    #[must_use]
    pub fn publication(&self) -> &'a PublicationView {
        self.view
    }
}

impl Serialize for PluginView<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        struct Pages<'v>(&'v PluginView<'v>);
        impl Serialize for Pages<'_> {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                let mut seq = serializer.serialize_seq(Some(self.0.view.pages.len()))?;
                for (page, properties) in self.0.pages() {
                    seq.serialize_element(&PageEntry { page, properties })?;
                }
                seq.end()
            }
        }
        struct PageEntry<'v> {
            page: &'v ViewPage,
            properties: &'v BTreeMap<String, String>,
        }
        impl Serialize for PageEntry<'_> {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                let mut map = serializer.serialize_map(Some(5))?;
                map.serialize_entry("id", &self.page.id)?;
                map.serialize_entry("title", &self.page.title)?;
                map.serialize_entry("url", &self.page.url)?;
                map.serialize_entry("kind", &self.page.kind)?;
                map.serialize_entry("properties", self.properties)?;
                map.end()
            }
        }
        struct Blocks<'v>(&'v [ViewBlock]);
        impl Serialize for Blocks<'_> {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
                for block in self.0 {
                    seq.serialize_element(&BlockEntry(block))?;
                }
                seq.end()
            }
        }
        struct BlockEntry<'v>(&'v ViewBlock);
        impl Serialize for BlockEntry<'_> {
            fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                let mut map = serializer.serialize_map(Some(4))?;
                map.serialize_entry("id", &self.0.id)?;
                map.serialize_entry("pageId", &self.0.page_id)?;
                map.serialize_entry("text", &self.0.text)?;
                map.serialize_entry("children", &self.0.children)?;
                map.end()
            }
        }
        let mut state = serializer.serialize_struct("PublicationView", 4)?;
        state.serialize_field("apiVersion", &VIEW_API_VERSION)?;
        // The revision is computed from the serialized view with an empty revision.
        state.serialize_field("revision", &self.revision)?;
        state.serialize_field("pages", &Pages(self))?;
        state.serialize_field("blocks", &Blocks(self.blocks()))?;
        state.end()
    }
}

/// FNV-1a over everything written to it.
struct Fnv(u64);

impl Default for Fnv {
    fn default() -> Self {
        Self(0xcbf2_9ce4_8422_2325)
    }
}

impl Write for Fnv {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        for byte in bytes {
            self.0 ^= u64::from(*byte);
            self.0 = self.0.wrapping_mul(0x0100_0000_01b3);
        }
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn text(source: &str) -> Vec<String> {
        let parsed = lsdoc::parse_to_projection(source);
        let outline = Outline::from_blocks(&parsed.blocks);
        (0..outline.blocks.len())
            .map(|index| {
                let next = outline
                    .blocks
                    .get(index + 1)
                    .map_or(parsed.blocks.len(), |next| next.index);
                block_text(source, &parsed.blocks[outline.blocks[index].index..next])
            })
            .collect()
    }

    #[test]
    fn block_text_drops_markers_properties_and_children() {
        assert_eq!(
            text(
                "title:: X\n\n- Intro **b**\n  id:: 6add3ffb-e52b-4e8f-908a-9958186e4dfb\n  collapsed:: true\n  second line\n\t- Detail\n\t  ```js\n\t  x\n\t  ```\n- last"
            ),
            ["Intro **b**\nsecond line", "Detail\n```js\nx\n```", "last"]
        );
        assert_eq!(
            text("- [[Secret]] <b>x</b>\n  heading:: true\n"),
            ["[[Secret]] <b>x</b>"]
        );
    }
}
