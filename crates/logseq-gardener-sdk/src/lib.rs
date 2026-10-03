//! Rust API for Logseq Gardener.
//!
//! This initial package exposes version information. Garden operations will be
//! added here after the parser comparison, so Rust applications can use them
//! without installing the CLI.
//!
//! ```
//! assert!(!logseq_gardener_sdk::VERSION.is_empty());
//! ```

/// The version of the SDK linked into the calling application.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
