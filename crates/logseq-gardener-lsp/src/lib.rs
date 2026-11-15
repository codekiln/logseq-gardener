//! The Logseq Gardener language server.
//!
//! [`run_stdio`] serves the Language Server Protocol over standard input and output for one
//! garden. This package only translates: every garden query lives in
//! [`logseq_gardener_sdk::index`], and the modules here turn protocol messages into calls on
//! it and its answers back into protocol messages.
//!
//! To add an editor feature, add a module under `features/`, give it a line in
//! [`dispatch`] and advertise it in [`capabilities`]. Convert every position and path with
//! [`positions`].

mod capabilities;
mod dispatch;
mod features;
mod positions;
mod progress;
mod server;

pub use server::{Exit, serve};

/// The version of the language server package.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Serve one client over standard input and output until it ends the session.
///
/// Returns [`Exit::Clean`] after `shutdown` and `exit`, and [`Exit::Failed`] with a message
/// for standard error when the folder is not a garden or the client goes away without
/// `shutdown`.
pub fn run_stdio() -> Exit {
    let (connection, io_threads) = lsp_server::Connection::stdio();
    match serve(connection) {
        // `serve` dropped the connection, so the writer ends once it has written every
        // message, and the reader ended when it read `exit`.
        Exit::Clean => match io_threads.join() {
            Ok(()) => Exit::Clean,
            Err(error) => Exit::Failed(format!("cannot write to the client: {error}")),
        },
        // The client is gone or was told why; the reader may still wait on standard input.
        failed => failed,
    }
}
