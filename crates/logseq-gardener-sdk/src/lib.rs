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

/// Select logical page namespaces for publication.
pub mod publishing;

/// Read-only Markdown garden loading.
pub mod garden;

/// Syntax types from the pinned lsdoc parser used by the garden reader.
pub use lsdoc::ast;

/// Display titles for loaded Markdown page documents.
pub mod page_titles;
