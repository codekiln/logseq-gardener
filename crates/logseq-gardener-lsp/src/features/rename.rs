//! `textDocument/prepareRename` and `textDocument/rename`: rename the page or the property
//! key under the cursor across the garden.
//!
//! The index plans the rename; [`workspace_edit`] turns any [`RenamePlan`] into the edit the
//! client applies, so a rename of another kind of name reuses it.

use std::collections::HashMap;
use std::path::PathBuf;

use logseq_gardener_sdk::index::{
    CONFIG_PATH, LineIndex, RenamePlan, RenameRefusal, RenameTarget, Subject,
};
use lsp_types::{
    DocumentChangeOperation, DocumentChanges, OneOf, OptionalVersionedTextDocumentIdentifier,
    PrepareRenameResponse, Range, RenameFile, RenameParams, ResourceOp, ResourceOperationKind,
    TextDocumentEdit, TextDocumentPositionParams, TextEdit, Uri, WorkspaceEdit,
};

use crate::dispatch::{Answer, failed};
use crate::server::Server;

fn refused(refusal: RenameRefusal) -> lsp_server::ResponseError {
    failed(format!("cannot rename: {refusal}"))
}

/// What a rename at a position is about, or `None` for a document the garden does not hold.
fn target(
    server: &Server,
    at: &TextDocumentPositionParams,
) -> Answer<Option<(PathBuf, RenameTarget)>> {
    let Some(path) = server.paths.relative_uri(at.text_document.uri.as_str()) else {
        return Ok(None);
    };
    let Some(document) = server.index.document(&path) else {
        return Ok(None);
    };
    let offset = server.encoding.document_offset(document, at.position);
    let target = server
        .index
        .rename_target_at(&path, offset)
        .map_err(refused)?;
    Ok(Some((path, target)))
}

pub(crate) fn prepare_rename(
    server: &mut Server,
    params: TextDocumentPositionParams,
) -> Answer<Option<PrepareRenameResponse>> {
    let Some((path, target)) = target(server, &params)? else {
        return Ok(None);
    };
    let range = match server.index.document(&path) {
        Some(document) => server.encoding.range(document, target.range),
        None => Range::new(params.position, params.position),
    };
    Ok(Some(PrepareRenameResponse::RangeWithPlaceholder {
        range,
        placeholder: target.name,
    }))
}

pub(crate) fn rename(server: &mut Server, params: RenameParams) -> Answer<Option<WorkspaceEdit>> {
    let Some((_, target)) = target(server, &params.text_document_position)? else {
        return Ok(None);
    };
    let plan = match &target.subject {
        Subject::Page(page) => {
            // An open `logseq/config.edn` counts as the buffer's text.
            let config = server
                .buffers
                .iter()
                .find(|(uri, _)| {
                    server.paths.relative_uri(uri).as_deref() == Some(CONFIG_PATH.as_ref())
                })
                .map(|(_, text)| text.clone());
            match config {
                Some(text) => {
                    server
                        .index
                        .rename_page_with_config(page, &params.new_name, Some(text))
                }
                None => server.index.rename_page(page, &params.new_name),
            }
            .map_err(refused)?
        }
        Subject::Block(_) => return Err(refused(RenameRefusal::Block)),
        Subject::Property(key) => server
            .index
            .rename_property(key, &params.new_name)
            .map_err(refused)?,
    };
    workspace_edit(server, &plan).map(Some)
}

/// The workspace edit that carries out a plan: for each file its text edits, then its file
/// renames, so a client that applies the changes in order edits a file under the name the
/// server knows it by and then renames it. An edit to an open buffer carries the version
/// the plan was computed for.
///
/// A plan that renames a file needs a client that advertises the `rename` resource
/// operation; the answer is an error that says so otherwise. A client that advertises
/// neither `documentChanges` nor a resource operation gets the text edits as `changes`.
// `WorkspaceEdit::changes` is keyed by `Uri`, whose cache of parsed parts takes no part in
// its hash or equality.
#[allow(clippy::mutable_key_type)]
pub(crate) fn workspace_edit(server: &Server, plan: &RenamePlan) -> Answer<WorkspaceEdit> {
    let supported = server
        .client
        .workspace
        .as_ref()
        .and_then(|workspace| workspace.workspace_edit.as_ref());
    let operations = supported.and_then(|edit| edit.resource_operations.as_deref());
    // Resource operations exist only in `documentChanges`, so a client that lists any reads
    // them. Neovim lists the operations and leaves `documentChanges` out.
    let document_changes =
        supported.and_then(|edit| edit.document_changes) == Some(true) || operations.is_some();
    let renames = operations.is_some_and(|kinds| kinds.contains(&ResourceOperationKind::Rename));
    if plan.renamed_file_count() > 0 && !renames {
        return Err(failed(
            "cannot rename: the rename gives a file a new name, and the editor does not advertise workspace.workspaceEdit.resourceOperations with \"rename\"",
        ));
    }
    // Each open document by its path, with the URI the client opened it under.
    let open: HashMap<PathBuf, (&str, i32)> = server
        .versions
        .iter()
        .filter_map(|(uri, version)| {
            Some((server.paths.relative_uri(uri)?, (uri.as_str(), *version)))
        })
        .collect();
    let parse = |uri: &str| -> Answer<Uri> {
        uri.parse()
            .map_err(|_| failed(format!("cannot rename: {uri} is not a URI")))
    };
    let mut changed = Vec::new();
    let mut changes = HashMap::new();
    for document in &plan.documents {
        let (uri, version) = match open.get(&document.path) {
            Some((uri, version)) => (parse(uri)?, Some(*version)),
            None => (parse(&server.paths.uri(&document.path))?, None),
        };
        let lines = LineIndex::new(&document.source);
        let at = |offset| server.encoding.position(&document.source, &lines, offset);
        let edits: Vec<TextEdit> = document
            .replacements
            .iter()
            .map(|replacement| TextEdit {
                range: Range::new(at(replacement.range.start), at(replacement.range.end)),
                new_text: replacement.text.clone(),
            })
            .collect();
        if edits.is_empty() {
            // Nothing to edit in this file.
        } else if document_changes {
            changed.push(DocumentChangeOperation::Edit(TextDocumentEdit {
                text_document: OptionalVersionedTextDocumentIdentifier { uri, version },
                edits: edits.into_iter().map(OneOf::Left).collect(),
            }));
        } else {
            changes.insert(uri, edits);
        }
        for file in &document.moves {
            changed.push(DocumentChangeOperation::Op(ResourceOp::Rename(
                RenameFile {
                    old_uri: parse(&server.paths.uri(&file.from))?,
                    new_uri: parse(&server.paths.uri(&file.to))?,
                    options: None,
                    annotation_id: None,
                },
            )));
        }
    }
    Ok(if document_changes {
        WorkspaceEdit {
            document_changes: Some(DocumentChanges::Operations(changed)),
            ..WorkspaceEdit::default()
        }
    } else {
        WorkspaceEdit {
            changes: Some(changes),
            ..WorkspaceEdit::default()
        }
    })
}
