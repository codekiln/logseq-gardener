//! Text document synchronization: an open buffer's text takes the place of its file.

use logseq_gardener_sdk::index::LineIndex;
use lsp_types::{
    DidChangeTextDocumentParams, DidCloseTextDocumentParams, DidOpenTextDocumentParams,
};

use crate::features::diagnostics;
use crate::server::Server;

pub(crate) fn did_open(server: &mut Server, params: DidOpenTextDocumentParams) {
    let uri = params.text_document.uri.as_str().to_owned();
    server
        .buffers
        .insert(uri.clone(), params.text_document.text);
    diagnostics::opened(server, &uri);
    index_buffer(server, &uri);
}

pub(crate) fn did_change(server: &mut Server, params: DidChangeTextDocumentParams) {
    let uri = params.text_document.uri.as_str().to_owned();
    let encoding = server.encoding;
    let Some(text) = server.buffers.get_mut(&uri) else {
        server.log(&format!("ignored a change to {uri}, which is not open"));
        return;
    };
    for change in params.content_changes {
        match change.range {
            Some(range) => {
                let lines = LineIndex::new(text);
                let start = encoding.offset(text, &lines, range.start);
                let end = encoding.offset(text, &lines, range.end);
                text.replace_range(start.min(end)..end.max(start), &change.text);
            }
            None => *text = change.text,
        }
    }
    index_buffer(server, &uri);
}

pub(crate) fn did_close(server: &mut Server, params: DidCloseTextDocumentParams) {
    let uri = params.text_document.uri.as_str();
    server.buffers.remove(uri);
    let Some(path) = server.paths.relative_uri(uri) else {
        return;
    };
    if let Err(error) = server.index.close_buffer(&path) {
        server.log(&format!("cannot read {error}"));
    }
    diagnostics::closed(server, uri, &path);
}

/// Give the index the buffer's current text when the document is one the garden holds, and
/// publish the diagnostics the text changed.
fn index_buffer(server: &mut Server, uri: &str) {
    if let (Some(path), Some(text)) = (server.paths.relative_uri(uri), server.buffers.get(uri)) {
        server.index.set_buffer(&path, text.clone());
        diagnostics::refresh(server, &[path]);
    }
}
