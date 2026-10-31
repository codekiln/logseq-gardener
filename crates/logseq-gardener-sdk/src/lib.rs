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

/// Block references and embeds: what they show and Logseq's nesting limit.
pub mod transclusion;

/// The page a tag points at.
pub mod inline_references;

/// Stored page names and alias candidate lookup.
pub mod page_names;

/// Find a page's source file by title, alias or journal date.
pub mod page_lookup;

/// Logseq's global graph and page graph from the reference index.
pub mod graph;

/// Linked references, filters, tagged pages and unlinked mentions from one index.
pub mod references;

/// Search records, the word index, the query and the published search files.
pub mod search;

/// First and last commit times of page and journal files.
pub mod git_dates;

/// All pages: every published page with its kind, icon, backlinks and dates.
pub mod page_list;

/// Logseq's simple query language and advanced query maps, evaluated at build time.
pub mod query;

/// Namespace hierarchy rows from page titles and aliases, shared by every caller.
pub mod namespaces;

/// Flashcards: card blocks, `{{cards}}` decks and which cards are due.
pub mod cards;

/// Site plugin manifests: reading, checking and the page values a plugin may read.
pub mod plugins;
