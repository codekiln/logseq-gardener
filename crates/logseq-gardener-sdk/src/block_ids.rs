//! Which block keeps an `id::` that more than one block gives.
//!
//! Logseq's file parser keeps a block id on the first block it reads with that id. Each
//! later block with the same id gets a new id, and Logseq removes the `id::` property from
//! it (`fix-block-id-if-duplicated!` and `fix-duplicate-id` in
//! `deps/graph-parser/src/logseq/graph_parser/block.cljs`). Logseq reads the files in the
//! order `filter-files` in `deps/graph-parser/src/logseq/graph_parser.cljs` gives them:
//! journals first, from the last path to the first, then the files whose path holds
//! `contents.`, then every other file by path. Within a file it reads the blocks in
//! source order.

use std::collections::HashSet;

use crate::ast::Block;
use crate::garden::GardenDocument;
use crate::outline::uuid;

/// The property names Logseq reads as a block's id. Logseq lower-cases a name and turns
/// spaces and underscores into hyphens, then reads `custom-id` as `id`.
fn id_property(name: &str) -> bool {
    let name = name.trim().to_ascii_lowercase().replace([' ', '_'], "-");
    name == "id" || name == "custom-id"
}

/// The order in which Logseq parses `documents`, as indexes into it.
pub(crate) fn parse_order(documents: &[GardenDocument]) -> Vec<usize> {
    let paths: Vec<String> = documents
        .iter()
        .map(|document| document.relative_path.to_string_lossy().replace('\\', "/"))
        .collect();
    let group = |path: &str| {
        if path.contains("journals/") {
            0
        } else if path.contains("contents.") {
            1
        } else {
            2
        }
    };
    let mut order: Vec<usize> = (0..documents.len()).collect();
    order.sort_by(|a, b| {
        let (a, b) = (&paths[*a], &paths[*b]);
        group(a)
            .cmp(&group(b))
            .then_with(|| if group(a) == 0 { b.cmp(a) } else { a.cmp(b) })
    });
    order
}

/// Remove the id properties from every block whose id an earlier block in Logseq's parse
/// order already holds, so each block id names one block.
pub(crate) fn keep_each_block_id_once(documents: &mut [GardenDocument]) {
    let mut seen = HashSet::new();
    for index in parse_order(documents) {
        visit(&mut documents[index].parsed.blocks, &mut seen, 0);
    }
}

fn visit(blocks: &mut [Block], seen: &mut HashSet<String>, depth: usize) {
    if depth > 64 {
        return;
    }
    for block in blocks {
        match block {
            Block::Properties { props, .. } => {
                let id = ["custom-id", "custom_id", "id"]
                    .into_iter()
                    .find_map(|key| {
                        props
                            .iter()
                            .rev()
                            .find(|prop| prop.0.trim().eq_ignore_ascii_case(key))
                    })
                    .and_then(|prop| uuid(&prop.1));
                if let Some(id) = id
                    && !seen.insert(id)
                {
                    props.retain(|prop| !id_property(&prop.0));
                }
            }
            Block::Quote { children, .. } | Block::Custom { children, .. } => {
                visit(children, seen, depth + 1);
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::garden::{DocumentKind, parse_document};
    use crate::outline::{BlockId, Outline};

    const ID: &str = "67c822b0-3271-41da-a14f-4f023a1f4556";

    fn document(path: &str, source: &str) -> GardenDocument {
        GardenDocument {
            relative_path: PathBuf::from(path),
            kind: if path.starts_with("journals/") {
                DocumentKind::Journal
            } else {
                DocumentKind::Page
            },
            source: source.to_owned(),
            parsed: parse_document(source),
            title_spellings: Default::default(),
        }
    }

    /// The paths of the documents whose blocks keep `ID`.
    fn owners(mut documents: Vec<GardenDocument>) -> Vec<String> {
        keep_each_block_id_once(&mut documents);
        let id = BlockId::Uuid(ID.to_owned());
        documents
            .iter()
            .flat_map(|document| {
                let outline = Outline::from_blocks(&document.parsed.blocks);
                let path = document.relative_path.display().to_string();
                outline
                    .blocks
                    .iter()
                    .filter(|block| block.id == id)
                    .map(move |block| format!("{path} {}", block.path[0]))
                    .collect::<Vec<_>>()
            })
            .collect()
    }

    #[test]
    fn logseq_reads_journals_from_the_last_then_contents_then_pages() {
        let paths = [
            "pages/b.md",
            "journals/2025_03_05.md",
            "pages/a.md",
            "pages/contents.md",
            "journals/2025_03_06.md",
        ];
        let documents: Vec<_> = paths.iter().map(|path| document(path, "- a\n")).collect();
        let order: Vec<_> = parse_order(&documents)
            .into_iter()
            .map(|index| paths[index])
            .collect();
        assert_eq!(
            order,
            [
                "journals/2025_03_06.md",
                "journals/2025_03_05.md",
                "pages/contents.md",
                "pages/a.md",
                "pages/b.md",
            ]
        );
    }

    #[test]
    fn the_first_page_by_path_keeps_the_id() {
        let block = |name: &str| format!("- About {name}\n  id:: {ID}\n");
        let documents = vec![
            document("pages/json___5.md", &block("JSON5")),
            document("pages/json___jsonc.md", &block("JSONC")),
        ];
        assert_eq!(owners(documents.clone()), ["pages/json___5.md 1"]);
        let mut fixed = documents;
        keep_each_block_id_once(&mut fixed);
        let props: Vec<_> = fixed[1]
            .parsed
            .blocks
            .iter()
            .filter_map(|block| match block {
                Block::Properties { props, .. } => Some(props.len()),
                _ => None,
            })
            .collect();
        assert_eq!(props, [0]);
    }

    #[test]
    fn a_journal_keeps_the_id_before_a_page_and_the_last_journal_first() {
        let block = format!("- block\n  id:: {}\n", ID.to_ascii_uppercase());
        let documents = vec![
            document("journals/2025_03_05.md", &block),
            document("journals/2025_03_06.md", &block),
            document("pages/a.md", &block),
        ];
        assert_eq!(owners(documents), ["journals/2025_03_06.md 1"]);
    }

    #[test]
    fn the_first_block_of_a_page_keeps_the_id_and_the_other_properties_stay() {
        let source = format!(
            "- first\n  id:: {ID}\n- second\n  custom_id:: {ID}\n  kind:: note\n- > quoted\n  id:: {ID}\n"
        );
        let mut documents = vec![document("pages/a.md", &source)];
        assert_eq!(owners(documents.clone()), ["pages/a.md 1"]);
        keep_each_block_id_once(&mut documents);
        let names: Vec<Vec<&str>> = documents[0]
            .parsed
            .blocks
            .iter()
            .filter_map(|block| match block {
                Block::Properties { props, .. } => {
                    Some(props.iter().map(|prop| prop.0.as_str()).collect())
                }
                _ => None,
            })
            .collect();
        assert_eq!(names, [vec!["id"], vec!["kind"], vec![]]);
    }

    #[test]
    fn ids_that_are_not_uuids_stay() {
        let documents = vec![
            document("pages/a.md", "- one\n  id:: not-a-uuid\n"),
            document("pages/b.md", "- two\n  id:: not-a-uuid\n"),
        ];
        let mut fixed = documents.clone();
        keep_each_block_id_once(&mut fixed);
        assert_eq!(fixed, documents);
    }
}
