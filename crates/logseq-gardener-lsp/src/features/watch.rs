//! Changes made outside the editor, by the Logseq app, git or another program.
//!
//! A client that supports dynamic registration watches the files and sends
//! `workspace/didChangeWatchedFiles`. For any other client the server watches the garden's
//! folders itself and handles the events in the main loop. Both end in [`changed`].
//!
//! The server's watcher uses the operating system's file events on Linux and Windows. On
//! macOS it compares the files' modification times every [`POLL_INTERVAL`].

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crossbeam_channel::{Receiver, unbounded};
use logseq_gardener_sdk::index::{CONFIG_PATH, NOTE_DIRECTORIES};
use lsp_server::{Request, RequestId};
use lsp_types::request::{RegisterCapability, Request as _};
use lsp_types::{
    ClientCapabilities, DidChangeWatchedFilesParams, DidChangeWatchedFilesRegistrationOptions,
    FileSystemWatcher, GlobPattern, OneOf, Registration, RegistrationParams, RelativePattern, Uri,
};
use notify::{RecursiveMode, Watcher};

/// How often the server looks for changed files where it has no file events.
const POLL_INTERVAL: Duration = Duration::from_secs(2);

#[cfg(target_os = "macos")]
type GardenWatcher = notify::PollWatcher;
#[cfg(not(target_os = "macos"))]
type GardenWatcher = notify::RecommendedWatcher;

use crate::positions::{Paths, path_to_uri, uri_to_path};
use crate::server::Server;

/// Who watches the garden's files.
pub(crate) enum Watching {
    /// The client, which sends `workspace/didChangeWatchedFiles`.
    Client,
    /// The server. The watcher stops when this value drops.
    Server {
        _watcher: GardenWatcher,
        events: Receiver<notify::Result<notify::Event>>,
    },
    /// Nobody: the client cannot, and the server's watcher failed to start.
    Nobody,
}

impl Watching {
    /// The word `lsg/status` reports.
    pub(crate) fn label(&self) -> &'static str {
        match self {
            Self::Client => "client",
            Self::Server { .. } => "server",
            Self::Nobody => "none",
        }
    }
}

/// Ask the client to watch the garden when it can register that; otherwise start a watcher
/// in the server. Returns a message for the log when neither works.
pub(crate) fn start(
    capabilities: &ClientCapabilities,
    paths: &Paths,
    send: &mut dyn FnMut(Request),
    request: RequestId,
) -> (Watching, Option<String>) {
    let client = capabilities
        .workspace
        .as_ref()
        .and_then(|workspace| workspace.did_change_watched_files.as_ref());
    if client.is_some_and(|files| files.dynamic_registration == Some(true)) {
        let relative = client.is_some_and(|files| files.relative_pattern_support == Some(true));
        let root_uri = path_to_uri(paths.root());
        let watchers = NOTE_DIRECTORIES
            .iter()
            .map(|folder| format!("{folder}/**/*.md"))
            .chain([CONFIG_PATH.to_owned()])
            .map(|pattern| FileSystemWatcher {
                glob_pattern: match root_uri.parse::<Uri>() {
                    Ok(base) if relative => GlobPattern::Relative(RelativePattern {
                        base_uri: OneOf::Right(base),
                        pattern,
                    }),
                    _ => GlobPattern::String(format!(
                        "{}/{pattern}",
                        paths.root().to_string_lossy().replace('\\', "/")
                    )),
                },
                // Create, change and delete.
                kind: None,
            })
            .collect();
        let options = DidChangeWatchedFilesRegistrationOptions { watchers };
        let params = RegistrationParams {
            registrations: vec![Registration {
                id: "lsg-watch-garden".to_owned(),
                method: "workspace/didChangeWatchedFiles".to_owned(),
                register_options: serde_json::to_value(options).ok(),
            }],
        };
        send(Request::new(
            request,
            RegisterCapability::METHOD.to_owned(),
            params,
        ));
        return (Watching::Client, None);
    }
    match watch(paths.resolved_root()) {
        Ok(watching) => (watching, None),
        Err(error) => (
            Watching::Nobody,
            Some(format!(
                "cannot watch the garden for changes made outside the editor: {error}"
            )),
        ),
    }
}

/// Watch `pages/` and `journals/` and their subfolders, and `logseq/` for `config.edn`.
fn watch(root: &Path) -> notify::Result<Watching> {
    let (sender, events) = unbounded();
    let mut watcher = GardenWatcher::new(
        move |event| {
            let _ = sender.send(event);
        },
        notify::Config::default().with_poll_interval(POLL_INTERVAL),
    )?;
    for folder in NOTE_DIRECTORIES {
        let path = root.join(folder);
        if path.is_dir() {
            watcher.watch(&path, RecursiveMode::Recursive)?;
        }
    }
    if let Some(folder) = root.join(CONFIG_PATH).parent() {
        watcher.watch(folder, RecursiveMode::NonRecursive)?;
    }
    Ok(Watching::Server {
        _watcher: watcher,
        events,
    })
}

pub(crate) fn did_change_watched_files(server: &mut Server, params: DidChangeWatchedFilesParams) {
    let paths = params
        .changes
        .iter()
        .filter_map(|change| uri_to_path(change.uri.as_str()))
        .collect();
    changed(server, paths);
}

/// Handle one event from the server's own watcher, together with every event already
/// waiting, so a burst of events reads each path once.
pub(crate) fn server_event(server: &mut Server, first: notify::Result<notify::Event>) {
    let mut paths = BTreeSet::new();
    let mut add = |event: notify::Result<notify::Event>| match event {
        // Reading a file is an event on some systems; it changes nothing.
        Ok(event) if !event.kind.is_access() => paths.extend(event.paths),
        _ => {}
    };
    add(first);
    if let Watching::Server { events, .. } = &server.watching {
        events.try_iter().for_each(add);
    }
    changed(server, paths);
}

/// Bring the index up to date with absolute paths that were created, changed or deleted.
fn changed(server: &mut Server, paths: BTreeSet<PathBuf>) {
    let paths: BTreeSet<PathBuf> = paths
        .iter()
        .filter_map(|path| server.paths.relative(path))
        .collect();
    if paths.contains(Path::new(CONFIG_PATH)) {
        // New settings can change every title, so everything is read again.
        server.load("Reading the garden again");
        return;
    }
    for path in paths {
        let result = if server.index.holds(&path) {
            server.index.refresh_file(&path)
        } else if path.extension().is_none_or(|extension| extension != "md")
            && path.components().count() > 1
        {
            // A folder beneath `pages/` or `journals/` that was created, renamed or deleted,
            // or a file that is no page. An event for `pages/` or `journals/` itself says
            // only that an entry was added or removed, and that entry has its own event.
            server.index.refresh_folder(&path)
        } else {
            Ok(())
        };
        if let Err(error) = result {
            server.log(&format!("cannot read {error}"));
        }
    }
}
