//! `textDocument/hover`: a Markdown summary of the page or block a link names.

use lsp_types::{Hover, HoverContents, HoverParams, MarkupContent, MarkupKind};

use crate::dispatch::Answer;
use crate::server::Server;

pub(crate) fn hover(server: &mut Server, params: HoverParams) -> Answer<Option<Hover>> {
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
        .summary_at(&path, offset)
        .map(|(range, summary)| Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: summary.markdown(),
            }),
            range: Some(server.encoding.range(document, range)),
        }))
}
