//! Writes the SDK's search index as format version 2 files beside the generated HTML.
use std::collections::{BTreeMap, BTreeSet};

use crate::garden::GardenDocument;
use crate::page_names::PageNameIndex;
use crate::search::{FileOptions, Records, SearchIndex, SearchOptions};
use crate::transclusion::ReferenceTargets;

/// The marker in `search.mjs` and `all-pages.js` that the shared fuzzy score replaces.
pub(super) const FUZZY_MARKER: &str = "// __LSG_FUZZY__";

/// The browser engine: the shared fuzzy score spliced into `search.mjs`.
pub(super) fn engine() -> String {
    include_str!("search.mjs").replacen(FUZZY_MARKER, include_str!("fuzzy.js"), 1)
}

pub(super) struct Search {
    records: Records,
    pub anchors: BTreeMap<String, BTreeSet<usize>>,
    admitted: BTreeMap<String, BTreeSet<usize>>,
}

impl Search {
    pub fn new(
        selected: &[(String, String, &GardenDocument)],
        names: &PageNameIndex<String>,
        targets: &ReferenceTargets<String>,
        options: &SearchOptions,
    ) -> Self {
        let records = Records::build(selected, names, targets, options);
        let mut anchors: BTreeMap<String, BTreeSet<usize>> = BTreeMap::new();
        for block in &records.blocks {
            anchors
                .entry(block.page.clone())
                .or_default()
                .insert(block.position);
        }
        Self {
            records,
            anchors,
            admitted: BTreeMap::new(),
        }
    }

    pub fn admit(&mut self, route: &str, emitted: &BTreeSet<usize>) {
        self.admitted.insert(route.to_owned(), emitted.clone());
    }

    /// Every search file by route, and the manifest route and revision. A block is
    /// indexed only when the renderer emitted its anchor.
    pub fn artifacts(mut self) -> (BTreeMap<String, String>, String, String) {
        self.records.blocks.retain(|record| {
            self.admitted
                .get(&record.page)
                .is_some_and(|anchors| anchors.contains(&record.position))
        });
        let engine = engine();
        let engine_route = format!("lsg-search-engine-{}.mjs", super::digest(&engine));
        let files = SearchIndex::new(self.records).files(&engine_route, &FileOptions::default());
        let mut artifacts = files.files;
        artifacts.insert(engine_route, engine);
        (artifacts, files.manifest, files.revision)
    }
}
