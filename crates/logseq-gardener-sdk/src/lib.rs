//! Rust API for Logseq Gardener.
//!
//! Load Logseq Markdown source through [`garden::load_garden`] without installing
//! the CLI. The initial reader retains syntax and references; page identity and
//! graph resolution are separate capabilities.
//!
//! ```
//! assert!(!logseq_gardener_sdk::VERSION.is_empty());
//! ```

/// The version of the SDK linked into the calling application.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Select logical page namespaces and public pages for publication.
pub mod publishing;

/// Read-only Markdown garden loading.
pub mod garden;

/// Syntax types from the pinned lsdoc parser used by the garden reader.
pub use lsdoc::ast;

/// Display titles for loaded Markdown page documents.
pub mod page_titles;

/// Video macros, YouTube timestamps, and which video each timestamp seeks.
pub mod media;

/// Generate a selected garden as local HTML.
pub mod site;

/// Fenced code text without outline indentation, and the fence language's mode.
pub mod code_blocks;

/// Block depth, ancestry, task markers, headings and stable block identifiers.
pub mod outline;

/// Journal dates, titles and date references.
pub mod journals;

/// Favorites, feature flags and publishing settings from the garden's `logseq/config.edn`.
pub mod garden_settings;

/// Bounded EDN reading for `logseq/config.edn`.
mod garden_config;

/// Which block keeps an `id::` that more than one block gives.
mod block_ids;

/// The spelling Logseq gives a page that refers to itself in another spelling.
mod title_spellings;

/// Block references and embeds: what they show and Logseq's nesting limit.
pub mod transclusion;

/// The page a tag points at.
pub mod inline_references;

/// Stored page names and alias candidate lookup.
pub mod page_names;

/// Find a page's source file by title, alias or journal date.
pub mod page_lookup;

/// Which page properties Logseq shows in a page's properties box.
pub mod page_properties;

/// Which property names Logseq's file parser keeps.
pub mod property_names;

/// Logseq's global graph and page graph from the reference index.
pub mod graph;

/// Linked references, filters, tagged pages and unlinked mentions from one index.
pub mod references;

/// The pages that exist only because published blocks name them.
pub mod reference_only;

/// Search records, the word index, the query and the published search files.
pub mod search;

/// First and last commit times of page and journal files.
pub mod git_dates;

/// All pages: every published page with its kind, icon, backlinks and dates.
pub mod page_list;

/// Logseq's simple query language and advanced query maps, evaluated at build time.
pub mod query;
pub mod query_advanced;
pub mod query_datalog;
pub mod query_form;
pub mod query_hiccup;
pub mod query_page;
pub mod query_script;

/// Namespace hierarchy rows from page titles and aliases, shared by every caller.
pub mod namespaces;

/// Flashcards: card blocks, `{{cards}}` decks and which cards are due.
pub mod cards;

/// Site plugin manifests: reading, checking and the page values a plugin may read.
pub mod plugins;

/// The plugins that ship inside `lsg`, turned on with `:lsg/plugins` in `logseq/config.edn`.
pub mod first_party;

/// Installable gardens: the `:lsg/install` settings, the icon check, the manifest and the page tags.
pub mod install;

/// The entity types a page declares with its `logseq-entity::` property.
pub mod entity_types;

/// A podcast episode page's recording and title.
pub mod podcast;

/// The previous and next pages a page names with `prev::` and `next::`.
pub mod page_sequence;

/// The pages a publication contains, and the read-only view build plugins receive.
pub mod publication;

/// How a publication names page files and links: digest file names or readable namespace URLs.
pub mod page_urls;

/// Build plugins and their contributions: checking them and resolving their links.
pub mod contributions;

/// Wall-clock time per phase of an operation, shared by every SDK caller.
pub mod timing;
