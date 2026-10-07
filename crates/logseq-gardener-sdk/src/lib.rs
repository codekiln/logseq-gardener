//! Rust API for Logseq Gardener.
//!
//! This package exposes version information and a publishing namespace policy.
//! Garden loading will follow the parser comparison, so Rust applications can
//! use garden operations without installing the CLI.
//!
//! ```
//! assert!(!logseq_gardener_sdk::VERSION.is_empty());
//! ```

/// The version of the SDK linked into the calling application.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Select logical page namespaces for publication.
pub mod publishing;
