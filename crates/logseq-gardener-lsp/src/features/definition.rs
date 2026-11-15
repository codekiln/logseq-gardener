//! `textDocument/definition`: the file and range a link, tag, block reference, embed or
//! page name in a property value leads to.

use std::path::Path;

use logseq_gardener_sdk::index::SourceRange;
use lsp_types::{GotoDefinitionParams, Location, Position, Range};

use crate::dispatch::Answer;
use crate::server::Server;

/// The protocol location of a range of a file. A file the index holds no document for, such
/// as the file of a page that has none yet, gets the start of the file.
pub(crate) fn location(server: &Server, path: &Path, range: SourceRange) -> Option<Location> {
    let range = match server.index.document(path) {
        Some(document) => server.encoding.range(document, range),
        None => Range::new(Position::new(0, 0), Position::new(0, 0)),
    };
    let uri = server.paths.uri(path).parse().ok()?;
    Some(Location::new(uri, range))
}

pub(crate) fn definition(
    server: &mut Server,
    params: GotoDefinitionParams,
) -> Answer<Option<Vec<Location>>> {
    let at = params.text_document_position_params;
    let Some(path) = server.paths.relative_uri(at.text_document.uri.as_str()) else {
        return Ok(None);
    };
    let Some(document) = server.index.document(&path) else {
        return Ok(None);
    };
    let offset = server.encoding.document_offset(document, at.position);
    Ok(server
        .index
        .definition_at(&path, offset)
        .map(|(_, destinations)| {
            destinations
                .iter()
                .filter_map(|destination| location(server, &destination.path, destination.range))
                .collect()
        }))
}
