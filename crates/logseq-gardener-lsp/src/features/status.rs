//! The server's own requests: what it loaded, and what the index sees at a position.
//!
//! `lsg/status` and `lsg/occurrence` let a test or a person observe the index. They are the
//! smallest example of a request handler: convert the position, ask the index, convert the
//! answer.

use logseq_gardener_sdk::garden::DocumentKind;
use logseq_gardener_sdk::index::{
    BlockLocation, IndexedDocument, Occurrence, OccurrenceKind, PageId, Resolved, SourceRange,
    Target,
};
use logseq_gardener_sdk::page_titles::FilenameFormat;
use lsp_types::{Range, TextDocumentPositionParams};
use serde::Serialize;

use crate::dispatch::Answer;
use crate::server::Server;

pub(crate) const STATUS: &str = "lsg/status";
pub(crate) const OCCURRENCE: &str = "lsg/occurrence";

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Status {
    root: String,
    /// `triple-lowbar` or `legacy`, from `:file/name-format`.
    filename_format: &'static str,
    journal_title_format: String,
    journal_file_name_format: String,
    /// `utf-8` or `utf-16`.
    position_encoding: &'static str,
    /// Who watches for changes made outside the editor: `client`, `server` or `none`.
    file_watching: &'static str,
    documents: usize,
    pages: usize,
    journals: usize,
    open_documents: usize,
    /// How long the last load of the whole garden took.
    load_milliseconds: f64,
}

pub(crate) fn status(server: &mut Server, _: serde_json::Value) -> Answer<Status> {
    let index = &server.index;
    let journals = index
        .documents()
        .filter(|document| document.document.kind == DocumentKind::Journal)
        .count();
    Ok(Status {
        root: server.paths.root().to_string_lossy().into_owned(),
        filename_format: match index.settings().filename_format {
            FilenameFormat::TripleLowbar => "triple-lowbar",
            FilenameFormat::Legacy => "legacy",
        },
        journal_title_format: index.journal_formats().title.pattern().to_owned(),
        journal_file_name_format: index.journal_formats().file_name.pattern().to_owned(),
        position_encoding: match server.encoding {
            crate::positions::Encoding::Utf8 => "utf-8",
            crate::positions::Encoding::Utf16 => "utf-16",
        },
        file_watching: server.watching.label(),
        documents: index.len(),
        pages: index.len() - journals,
        journals,
        open_documents: index.buffers().count(),
        load_milliseconds: server.load_milliseconds,
    })
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OccurrenceReport {
    kind: &'static str,
    /// The whole syntax, such as `[[Page]]`.
    range: Range,
    /// The name inside it, such as `Page`.
    name_range: Range,
    target: TargetReport,
    resolved: ResolvedReport,
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum TargetReport {
    Page { name: String },
    Block { id: String },
    Property { key: String },
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
enum ResolvedReport {
    /// A page or journal with a file.
    Document {
        uri: String,
        title: Option<String>,
    },
    /// A journal date that has no file.
    JournalWithoutFile {
        date: String,
    },
    /// A page name that has no file.
    PageWithoutFile {
        name: String,
    },
    /// A name several files answer to.
    Ambiguous {
        uris: Vec<String>,
    },
    /// The block Logseq gives the id.
    Block {
        uri: String,
        range: Range,
    },
    BlockNotFound,
    /// How many lines in the garden write the key.
    Property {
        count: usize,
    },
}

pub(crate) fn occurrence(
    server: &mut Server,
    params: TextDocumentPositionParams,
) -> Answer<Option<OccurrenceReport>> {
    let Some(document) = server.document(params.text_document.uri.as_str()) else {
        return Ok(None);
    };
    let offset = server.encoding.document_offset(document, params.position);
    Ok(document
        .occurrence_at(offset)
        .map(|occurrence| report(server, document, occurrence)))
}

fn report(
    server: &Server,
    document: &IndexedDocument,
    occurrence: &Occurrence,
) -> OccurrenceReport {
    let range = |range| server.encoding.range(document, range);
    OccurrenceReport {
        kind: match occurrence.kind {
            OccurrenceKind::PageLink => "pageLink",
            OccurrenceKind::Tag => "tag",
            OccurrenceKind::BlockReference => "blockReference",
            OccurrenceKind::PageEmbed => "pageEmbed",
            OccurrenceKind::BlockEmbed => "blockEmbed",
            OccurrenceKind::PropertyKey => "propertyKey",
            OccurrenceKind::PropertyValue => "propertyValue",
            OccurrenceKind::BlockId => "blockId",
        },
        range: range(occurrence.range),
        name_range: range(occurrence.name_range),
        target: match &occurrence.target {
            Target::Page(name) => TargetReport::Page { name: name.clone() },
            Target::Block(id) => TargetReport::Block { id: id.clone() },
            Target::Property(key) => TargetReport::Property { key: key.clone() },
        },
        resolved: match server.index.resolve(&occurrence.target) {
            Resolved::Page(PageId::Document(path)) => ResolvedReport::Document {
                uri: server.paths.uri(&path),
                title: server
                    .index
                    .document(&path)
                    .and_then(|document| document.title.clone()),
            },
            Resolved::Page(PageId::Journal(date)) => ResolvedReport::JournalWithoutFile {
                date: format!("{:04}-{:02}-{:02}", date.year(), date.month(), date.day()),
            },
            Resolved::Page(PageId::Name(name)) => ResolvedReport::PageWithoutFile { name },
            Resolved::Page(PageId::Ambiguous(name)) => ResolvedReport::Ambiguous {
                uris: server
                    .index
                    .page_documents(&name)
                    .iter()
                    .map(|document| server.paths.uri(document.path()))
                    .collect(),
            },
            Resolved::Block(Some(BlockLocation { document, site })) => {
                // The first line of the block.
                let line = document.lines.position(site.block_start).line;
                let (_, end) = document.lines.line_range(line, document.source());
                ResolvedReport::Block {
                    uri: server.paths.uri(document.path()),
                    range: server
                        .encoding
                        .range(document, SourceRange::new(site.block_start, end)),
                }
            }
            Resolved::Block(None) => ResolvedReport::BlockNotFound,
            Resolved::Property { count, .. } => ResolvedReport::Property { count },
        },
    }
}
