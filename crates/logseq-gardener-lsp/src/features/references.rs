//! `textDocument/references`: every place that names the page or block under the cursor,
//! or the current page when the cursor is in plain text.

use lsp_types::{Location, ReferenceParams};

use crate::dispatch::Answer;
use crate::features::definition::location;
use crate::server::Server;

pub(crate) fn references(
    server: &mut Server,
    params: ReferenceParams,
) -> Answer<Option<Vec<Location>>> {
    let at = params.text_document_position;
    let Some(path) = server.paths.relative_uri(at.text_document.uri.as_str()) else {
        return Ok(None);
    };
    let Some(document) = server.index.document(&path) else {
        return Ok(None);
    };
    let offset = server.encoding.document_offset(document, at.position);
    let Some(subject) = server.index.subject_at(&path, offset) else {
        return Ok(None);
    };
    let found = server
        .index
        .references(&subject, params.context.include_declaration);
    Ok(Some(
        found
            .into_iter()
            .filter_map(|reference| location(server, &reference.path, reference.range))
            .collect(),
    ))
}
