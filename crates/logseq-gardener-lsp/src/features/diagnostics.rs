//! Diagnostics: the problems the index finds in a document, published to the client.
//!
//! The server publishes for a document when it opens, and for a document that is not open
//! when its file changes on disk. After every change to the index, [`refresh`] computes
//! again the problems of each open document and of each document with a published problem,
//! and sends the lists that changed. A fix in one file therefore clears the diagnostics it
//! caused in another. The server reads no other document for this, so the work after an
//! edit grows with the number of open and reported documents and stays the same as the
//! garden grows.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use logseq_gardener_sdk::index::{Problem, Severity};
use lsp_types::notification::PublishDiagnostics;
use lsp_types::{
    Diagnostic, DiagnosticRelatedInformation, DiagnosticSeverity, Location, NumberOrString,
    PublishDiagnosticsParams, Uri,
};

use crate::server::Server;

/// The name clients show beside each diagnostic.
const SOURCE: &str = "lsg";

/// Make the next [`refresh`] publish for the document the client just opened at `uri`,
/// whatever the server published for it before.
pub(crate) fn opened(server: &mut Server, uri: &str) {
    if let Some(path) = server.paths.relative_uri(uri) {
        server.published.remove(&path);
    }
}

/// Publish the diagnostics that changed. `changed` holds the paths, relative to the garden
/// root, of the documents whose text the index just read again.
pub(crate) fn refresh(server: &mut Server, changed: &[PathBuf]) {
    let open = open_documents(server);
    let mut candidates: BTreeSet<PathBuf> = open.keys().cloned().collect();
    candidates.extend(server.published.keys().cloned());
    candidates.extend(changed.iter().cloned());
    for path in candidates {
        let diagnostics = diagnostics(server, &open, &path);
        let is_open = open.contains_key(&path);
        let published = server.published.get(&path);
        let unchanged = match published {
            Some(published) => *published == diagnostics,
            // A document that is not open and has no problem needs no message.
            None => !is_open && diagnostics.is_empty(),
        };
        if unchanged {
            continue;
        }
        publish(server, &open, &path, diagnostics.clone());
        if is_open || !diagnostics.is_empty() {
            server.published.insert(path, diagnostics);
        } else {
            server.published.remove(&path);
        }
    }
}

/// Clear the diagnostics of a document the client just closed at `uri`, then publish what
/// its file's text changed in other documents.
pub(crate) fn closed(server: &mut Server, uri: &str, path: &Path) {
    if server
        .published
        .remove(path)
        .is_some_and(|published| !published.is_empty())
    {
        let open = BTreeMap::from([(path.to_path_buf(), uri.to_owned())]);
        publish(server, &open, path, Vec::new());
    }
    refresh(server, &[]);
}

/// The garden's open documents, each with the URI the client opened it under.
fn open_documents(server: &Server) -> BTreeMap<PathBuf, String> {
    server
        .buffers
        .keys()
        .filter_map(|uri| Some((server.paths.relative_uri(uri)?, uri.clone())))
        .filter(|(path, _)| server.index.holds(path))
        .collect()
}

/// The URI of a document: the one the client opened it under, or the one its path gives.
fn uri(server: &Server, open: &BTreeMap<PathBuf, String>, path: &Path) -> Option<Uri> {
    let uri = open
        .get(path)
        .cloned()
        .unwrap_or_else(|| server.paths.uri(path));
    uri.parse().ok()
}

fn publish(
    server: &Server,
    open: &BTreeMap<PathBuf, String>,
    path: &Path,
    diagnostics: Vec<Diagnostic>,
) {
    let Some(uri) = uri(server, open, path) else {
        return;
    };
    server.notify::<PublishDiagnostics>(PublishDiagnosticsParams {
        uri,
        diagnostics,
        version: None,
    });
}

/// The diagnostics of the document at `path`; none when the index holds no such document.
fn diagnostics(server: &Server, open: &BTreeMap<PathBuf, String>, path: &Path) -> Vec<Diagnostic> {
    let Some(document) = server.index.document(path) else {
        return Vec::new();
    };
    let related_information = server
        .client
        .text_document
        .as_ref()
        .and_then(|text_document| text_document.publish_diagnostics.as_ref())
        .and_then(|publish| publish.related_information)
        == Some(true);
    server
        .index
        .problems(path)
        .into_iter()
        .map(|problem| {
            let mut message = problem.message();
            let related = related(server, open, &problem);
            if !related_information && !related.is_empty() {
                let places: Vec<String> = related.iter().map(|(place, _)| place.clone()).collect();
                message = format!("{message}. Also at {}", places.join(", "));
            }
            Diagnostic {
                range: server.encoding.range(document, problem.range),
                severity: Some(match problem.severity() {
                    Severity::Error => DiagnosticSeverity::ERROR,
                    Severity::Warning => DiagnosticSeverity::WARNING,
                }),
                code: Some(NumberOrString::String(problem.kind.code().to_owned())),
                source: Some(SOURCE.to_owned()),
                message,
                related_information: (related_information && !related.is_empty()).then(|| {
                    related
                        .into_iter()
                        .map(|(_, location)| DiagnosticRelatedInformation {
                            location,
                            message: "Another block with this id".to_owned(),
                        })
                        .collect()
                }),
                ..Diagnostic::default()
            }
        })
        .collect()
}

/// Each related location of a problem as `path:line`, with the line counted from one, and
/// as a protocol location.
fn related(
    server: &Server,
    open: &BTreeMap<PathBuf, String>,
    problem: &Problem,
) -> Vec<(String, Location)> {
    problem
        .related
        .iter()
        .filter_map(|place| {
            let document = server.index.document(&place.path)?;
            let range = server.encoding.range(document, place.range);
            let name = place.path.to_string_lossy().replace('\\', "/");
            Some((
                format!("{name}:{}", range.start.line + 1),
                Location {
                    uri: uri(server, open, &place.path)?,
                    range,
                },
            ))
        })
        .collect()
}
