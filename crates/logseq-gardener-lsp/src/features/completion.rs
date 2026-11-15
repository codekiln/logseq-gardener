//! `textDocument/completion`: page names, blocks and properties for the text at the cursor.
//!
//! The SDK reads what is being typed and ranks the candidates. This module turns each
//! candidate into an item whose text edit replaces the typed text and a closing `]]` or `))`
//! after the cursor.

use logseq_gardener_sdk::index::{Candidate, CandidateKind, LineIndex, completion_context};
use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionItemLabelDetails, CompletionList,
    CompletionOptions, CompletionParams, CompletionTextEdit, Range, TextEdit,
};

use crate::dispatch::Answer;
use crate::server::Server;

/// The most items one answer holds. A longer list is marked incomplete, so the client asks
/// again as the typed text grows.
pub(crate) const LIMIT: usize = 100;

/// The characters that open a link, a tag, a block reference and a property value.
const TRIGGERS: [&str; 4] = ["[", "#", "(", ":"];

pub(crate) fn options() -> CompletionOptions {
    CompletionOptions {
        trigger_characters: Some(TRIGGERS.iter().map(|&trigger| trigger.to_owned()).collect()),
        ..CompletionOptions::default()
    }
}

pub(crate) fn completion(server: &mut Server, params: CompletionParams) -> Answer<CompletionList> {
    let at = params.text_document_position;
    let uri = at.text_document.uri.as_str();
    // An open buffer's text, also for a buffer the garden holds no file for.
    let text = match (server.buffers.get(uri), server.document(uri)) {
        (Some(text), _) => text.as_str(),
        (None, Some(document)) => document.source(),
        (None, None) => return Ok(CompletionList::default()),
    };
    let lines = LineIndex::new(text);
    let offset = server.encoding.offset(text, &lines, at.position);
    let Some(context) = completion_context(text, offset) else {
        return Ok(CompletionList::default());
    };
    let found = server.index.completions(&context, LIMIT);
    let position = |offset| server.encoding.position(text, &lines, offset);
    let range = Range::new(
        position(context.replace.start),
        position(context.replace.end),
    );
    let items = found
        .candidates
        .into_iter()
        .enumerate()
        .map(|(rank, candidate)| item(candidate, rank, range))
        .collect();
    Ok(CompletionList {
        is_incomplete: found.partial,
        items,
    })
}

fn item(candidate: Candidate, rank: usize, range: Range) -> CompletionItem {
    let detail = match (candidate.kind, &candidate.page, candidate.uses) {
        (CandidateKind::Alias, Some(page), _) => Some(format!("alias of {page}")),
        (_, Some(page), _) => Some(page.clone()),
        (CandidateKind::Journal, ..) => Some("journal".to_owned()),
        (CandidateKind::Namespace, ..) => Some("namespace".to_owned()),
        (_, _, Some(1)) => Some("1 use".to_owned()),
        (_, _, Some(uses)) => Some(format!("{uses} uses")),
        _ => None,
    };
    CompletionItem {
        label: candidate.label,
        kind: Some(match candidate.kind {
            CandidateKind::Page => CompletionItemKind::FILE,
            CandidateKind::Journal => CompletionItemKind::EVENT,
            CandidateKind::Namespace => CompletionItemKind::FOLDER,
            CandidateKind::Alias => CompletionItemKind::REFERENCE,
            CandidateKind::Block => CompletionItemKind::TEXT,
            CandidateKind::PropertyKey => CompletionItemKind::PROPERTY,
            CandidateKind::PropertyValue => CompletionItemKind::VALUE,
        }),
        label_details: candidate.page.map(|page| CompletionItemLabelDetails {
            detail: None,
            description: Some(page),
        }),
        detail,
        filter_text: Some(candidate.filter),
        // The rank in digits of equal length, so a client that sorts by this text keeps the
        // server's order.
        sort_text: Some(format!("{rank:04}")),
        text_edit: Some(CompletionTextEdit::Edit(TextEdit {
            range,
            new_text: candidate.insert,
        })),
        ..CompletionItem::default()
    }
}
