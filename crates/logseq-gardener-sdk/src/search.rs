//! Search over a garden's published pages and blocks, shared by the CLI, the language
//! server and the publisher.
//!
//! [`Records::build`] turns the selected documents into page and block records under the
//! static search contract's privacy rules. [`SearchIndex`] orders them and indexes every
//! block under its normalized words; [`SearchIndex::query`] answers a query with the
//! groups and order the published browser engine returns. [`SearchIndex::files`] splits
//! the index into the title index, word-prefix shards and record chunks the browser
//! loads, each named by its content digest.
//!
//! ```
//! use logseq_gardener_sdk::search::{BlockRecord, PageRecord, QueryOptions, Records, SearchIndex};
//!
//! let page = |id: &str, title: &str| PageRecord { id: id.into(), title: title.into(), aliases: vec![] };
//! let block = |page: &str, position, text: &str| BlockRecord {
//!     page: page.into(),
//!     position,
//!     fragment: format!("source-block-{position}"),
//!     text: text.into(),
//!     parents: vec![],
//! };
//! let index = SearchIndex::new(Records {
//!     pages: vec![page("p-1.html", "tmux"), page("p-2.html", "mise")],
//!     blocks: vec![block("p-2.html", 0, "mise runs tmux.conf tasks")],
//! });
//! let results = index.query("tmx", &QueryOptions::default()).unwrap();
//! // Titles match fuzzily; blocks need every word to start a word in the block.
//! assert_eq!(index.pages()[results.pages.hits[0].page].title, "tmux");
//! assert_eq!(results.blocks.count, 0);
//! let results = index.query("tmux con", &QueryOptions::default()).unwrap();
//! assert_eq!(results.blocks.hits, [0]);
//! ```
use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;

use flate2::{Compression, write::GzEncoder};
use serde_json::json;
use unicode_normalization::{UnicodeNormalization, char::is_combining_mark};

use crate::ast::{Block, Inline, Url};
use crate::garden::GardenDocument;
use crate::inline_references::tag_page_name;
use crate::page_names::{PageNameIndex, lookup_key, page_alias_spellings};
use crate::transclusion::{ReferenceTargets, page_targets};

/// The published search format; the browser engine rejects every other version.
pub const FORMAT_VERSION: u64 = 2;
/// Results per group, as Logseq's search returns at most 100 of each kind.
pub const MAX_RESULTS: usize = 100;
/// Longest accepted query, in Unicode scalar values.
pub const MAX_QUERY_CHARS: usize = 256;
/// Most distinct words in one query.
pub const MAX_TERMS: usize = 16;
/// Parent texts kept for a block's breadcrumb, nearest last.
pub const PARENTS: usize = 3;
/// Longest parent text kept for a breadcrumb, in Unicode scalar values.
const PARENT_CHARS: usize = 160;

/// Opt-in source-search settings. Publication selection and privacy still apply.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SearchOptions {
    /// Canonical page titles whose bodies are omitted while titles remain searchable.
    pub excluded_page_bodies: BTreeSet<String>,
}

/// One published page: its caller-supplied id (the publisher's route), stored title and
/// the aliases that name only this page.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageRecord {
    pub id: String,
    pub title: String,
    pub aliases: Vec<String>,
}

/// One searchable block: its page id, UTF-8 source offset, destination fragment, plain
/// text and up to [`PARENTS`] parent texts, nearest last.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockRecord {
    pub page: String,
    pub position: usize,
    pub fragment: String,
    pub text: String,
    pub parents: Vec<String>,
}

/// The page and block records of a garden, before ordering and indexing.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Records {
    pub pages: Vec<PageRecord>,
    pub blocks: Vec<BlockRecord>,
}

fn compact(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

// Omitted nodes introduce a word boundary, so adjacent fragments cannot invent terms.
fn plain(nodes: &[Inline], names: &PageNameIndex<String>, depth: usize) -> String {
    if depth > 64 {
        return String::new();
    }
    let mut text = String::new();
    for node in nodes {
        match node {
            Inline::Plain { text: value, .. }
            | Inline::Code { text: value, .. }
            | Inline::Verbatim { text: value, .. } => text.push_str(value),
            Inline::Emphasis { children, .. }
            | Inline::Subscript { children, .. }
            | Inline::Superscript { children, .. } => {
                text.push_str(&plain(children, names, depth + 1))
            }
            Inline::Link {
                url: Url::PageRef { v },
                label,
                image: false,
                ..
            } if names.candidates(v).len() == 1 => {
                if label.is_empty() {
                    text.push_str(v);
                } else {
                    text.push_str(&plain(label, names, depth + 1));
                }
            }
            // Tags are references too: an unavailable name must never become a term.
            Inline::Tag { children, .. } => {
                let value = tag_page_name(children);
                if names.candidates(&value).len() == 1 {
                    text.push_str(&value);
                } else {
                    text.push(' ');
                }
            }
            // Explicit external-link display text is authored prose; URLs and asset
            // labels are omitted. Block refs contribute no copied target label.
            Inline::Link {
                url:
                    Url::Complex {
                        protocol: Some(protocol),
                        ..
                    },
                label,
                image: false,
                ..
            } if matches!(protocol.to_ascii_lowercase().as_str(), "http" | "https") => {
                text.push_str(&plain(label, names, depth + 1));
            }
            Inline::Macro { name, args, .. } if crate::media::is_timestamp_macro(name) => {
                match crate::media::Timestamp::from_args(args) {
                    Some(time) => text.push_str(&time.display()),
                    None => text.push(' '),
                }
            }
            _ => text.push(' '),
        }
    }
    text
}

impl Records {
    /// Records for `selected` `(title, id, document)` pages. `names` holds only the
    /// selected pages, so a reference to anything else contributes no text, and
    /// `targets` decides which authored UUIDs can serve as destinations.
    pub fn build(
        selected: &[(String, String, &GardenDocument)],
        names: &PageNameIndex<String>,
        targets: &ReferenceTargets<String>,
        options: &SearchOptions,
    ) -> Self {
        let mut records = Self::default();
        let excluded: BTreeSet<_> = options
            .excluded_page_bodies
            .iter()
            .map(|s| lookup_key(s))
            .collect();
        for (title, route, document) in selected {
            records.pages.push(PageRecord {
                id: route.clone(),
                title: title.clone(),
                aliases: page_alias_spellings(document, title)
                    .into_iter()
                    .filter(|alias| names.candidates(alias) == [route.clone()])
                    .collect(),
            });
            if excluded.contains(&lookup_key(title)) {
                continue;
            }
            let uuid_by_start: BTreeMap<_, _> = page_targets(&document.parsed.blocks)
                .into_iter()
                .filter(|(id, start, _)| targets.owns(id, route, *start))
                .map(|(id, start, _)| (start, id))
                .collect();
            records.bullets(
                &document.parsed.blocks,
                route,
                names,
                &uuid_by_start,
                &[],
                0,
            );
        }
        records
    }

    fn bullets(
        &mut self,
        blocks: &[Block],
        route: &str,
        names: &PageNameIndex<String>,
        uuids: &BTreeMap<usize, String>,
        inherited: &[String],
        depth: usize,
    ) {
        if depth > 64 {
            return;
        }
        let mut parents: Vec<(u32, String)> = Vec::new();
        let mut excluded_level = None;
        for (index, block) in blocks.iter().enumerate() {
            match block {
                Block::Bullet {
                    level,
                    inline,
                    span: Some(span),
                    ..
                } => {
                    if excluded_level.is_some_and(|excluded| *level > excluded) {
                        continue;
                    }
                    excluded_level = None;
                    while parents.last().is_some_and(|(l, _)| l >= level) {
                        parents.pop();
                    }
                    let excluded = matches!(blocks.get(index + 1), Some(Block::Properties { props, .. })
                        if props.iter().rev().find(|p| p.0.trim().eq_ignore_ascii_case("search-exclude"))
                            .is_some_and(|p| p.1.trim().eq_ignore_ascii_case("true")));
                    if excluded {
                        excluded_level = Some(*level);
                        continue;
                    }
                    let text = compact(&plain(inline, names, 0));
                    if !text.is_empty() {
                        let fragment = uuids.get(&span.0).map_or_else(
                            || format!("source-block-{}", span.0),
                            |id| format!("block-{id}"),
                        );
                        let ancestors: Vec<_> = inherited
                            .iter()
                            .cloned()
                            .chain(parents.iter().map(|(_, s)| s.clone()))
                            .filter(|s| !s.is_empty())
                            .collect();
                        self.blocks.push(BlockRecord {
                            page: route.to_owned(),
                            position: span.0,
                            fragment,
                            text: text.clone(),
                            parents: ancestors[ancestors.len().saturating_sub(PARENTS)..].to_vec(),
                        });
                    }
                    parents.push((*level, text.chars().take(PARENT_CHARS).collect()));
                }
                Block::Quote { children, .. } if excluded_level.is_none() => {
                    let ancestors: Vec<_> = inherited
                        .iter()
                        .cloned()
                        .chain(parents.iter().map(|(_, s)| s.clone()))
                        .collect();
                    self.bullets(children, route, names, uuids, &ancestors, depth + 1);
                }
                // Custom containers include queries and unsupported directives.
                _ => {}
            }
        }
    }
}

/// Lowercase, NFC and single spaces: the form both engines compare. JavaScript's
/// `toLowerCase` and Rust's `to_lowercase` agree, final sigma included.
pub fn normalize(text: &str) -> String {
    let folded: String = text.to_lowercase().nfc().collect();
    compact(&folded)
}

/// A word is a run of letters, digits or combining marks; everything else separates
/// words. The browser engine's `[\p{Alphabetic}\p{N}\p{M}]` is the same class.
pub fn words(normalized: &str) -> impl Iterator<Item = &str> {
    normalized
        .split(|c: char| !(c.is_alphanumeric() || is_combining_mark(c)))
        .filter(|word| !word.is_empty())
}

// The browser compares strings by UTF-16 code unit, so ties order the same way here.
fn utf16_cmp(a: &str, b: &str) -> std::cmp::Ordering {
    a.encode_utf16().cmp(b.encode_utf16())
}

// Logseq ignores spaces, brackets, slashes, underscores and parentheses in page names.
fn clean(text: &str) -> Vec<u16> {
    text.to_lowercase()
        .chars()
        .filter(|c| !"[ \\/_]()".contains(*c))
        .collect::<String>()
        .encode_utf16()
        .collect()
}

/// Logseq's fuzzy page score (`frontend/search.cljs`), the port All pages uses in
/// `site/fuzzy.js`: consecutive matches earn more, then a length-similarity term, and
/// an exact substring adds 1000. Zero when the query's characters are not all present
/// in order. It counts UTF-16 code units, as JavaScript does.
pub fn fuzzy_score(query: &str, name: &str) -> f64 {
    let (q, s) = (clean(query), clean(name));
    let (mut i, mut mult, mut total) = (0, 1.0, 0.0);
    for unit in &s {
        if i >= q.len() {
            break;
        }
        if q[i] == *unit {
            total += mult;
            mult += 1.0;
            i += 1;
        } else {
            mult = 1.0;
        }
    }
    if i < q.len() {
        return 0.0;
    }
    let (min, max) = (q.len().min(s.len()) as f64, q.len().max(s.len()) as f64);
    let similarity = if max > 0.0 {
        1.0 - (max - min) / max
    } else {
        1.0
    };
    total + similarity + if name.contains(query) { 1000.0 } else { 0.0 }
}

/// Normalized word → block record numbers that contain it, in record order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WordIndex(BTreeMap<String, Vec<u32>>);

impl WordIndex {
    /// Index each block under every distinct normalized word of its text.
    pub fn build<'a>(texts: impl IntoIterator<Item = &'a str>) -> Self {
        let mut words: BTreeMap<String, Vec<u32>> = BTreeMap::new();
        for (number, text) in texts.into_iter().enumerate() {
            let normalized = normalize(text);
            let distinct: BTreeSet<_> = self::words(&normalized).collect();
            for word in distinct {
                words
                    .entry(word.to_owned())
                    .or_default()
                    .push(number as u32);
            }
        }
        Self(words)
    }

    /// The blocks that contain exactly `word`.
    pub fn get(&self, word: &str) -> &[u32] {
        self.0.get(word).map_or(&[], Vec::as_slice)
    }

    /// Every indexed word with its blocks, in byte order.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &[u32])> {
        self.0.iter().map(|(w, b)| (w.as_str(), b.as_slice()))
    }

    /// The words that start with `prefix`, with their blocks.
    pub fn prefixed<'a>(&'a self, prefix: &'a str) -> impl Iterator<Item = (&'a str, &'a [u32])> {
        self.0
            .range::<str, _>((
                std::ops::Bound::Included(prefix),
                std::ops::Bound::Unbounded,
            ))
            .take_while(move |(word, _)| word.starts_with(prefix))
            .map(|(w, b)| (w.as_str(), b.as_slice()))
    }
}

/// Why a query returned no results.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QueryError {
    /// Longer than [`MAX_QUERY_CHARS`].
    InvalidQuery,
    /// More than [`MAX_TERMS`] distinct words.
    TooManyTerms,
    /// A limit outside 1–[`MAX_RESULTS`].
    InvalidLimit,
}

/// Query settings: results kept per group, and the page whose blocks form the Current
/// page group.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryOptions {
    pub limit: usize,
    pub current_page: Option<String>,
}

impl Default for QueryOptions {
    fn default() -> Self {
        Self {
            limit: MAX_RESULTS,
            current_page: None,
        }
    }
}

/// One result group: the full match count and at most the limit's hits.
#[derive(Debug, Clone, PartialEq)]
pub struct Group<T> {
    pub count: usize,
    pub hits: Vec<T>,
}

impl<T> Default for Group<T> {
    fn default() -> Self {
        Self {
            count: 0,
            hits: Vec::new(),
        }
    }
}

/// A page result: its number in [`SearchIndex::pages`], and the alias it matched
/// through when that, not the title, matched best.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PageHit {
    pub page: usize,
    pub alias: Option<String>,
}

/// Grouped results. Block hits are record numbers in [`SearchIndex::blocks`].
#[derive(Debug, Clone, Default, PartialEq)]
pub struct QueryResults {
    pub pages: Group<PageHit>,
    pub blocks: Group<u32>,
    pub current_page: Group<u32>,
}

/// Ordered records and their word index.
#[derive(Debug, Clone, Default)]
pub struct SearchIndex {
    pages: Vec<PageRecord>,
    keys: Vec<(String, Vec<String>)>,
    blocks: Vec<BlockRecord>,
    words: WordIndex,
}

/// Where the publisher splits the index: the largest gzip size of a shard before it
/// splits by the next character, the most block records in a chunk, and the largest gzip
/// size of a chunk. The default budgets are `shard_gzip_kb` and `chunk_gzip_kb` in
/// `scripts/parity/budgets.toml`, where a KB is 1,000 bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FileOptions {
    pub shard_budget: usize,
    pub chunk_size: usize,
    pub chunk_budget: usize,
}

impl Default for FileOptions {
    fn default() -> Self {
        Self {
            shard_budget: 64_000,
            chunk_size: 256,
            chunk_budget: 32_000,
        }
    }
}

/// The published search files by name, and the revision the manifest declares.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchFiles {
    pub revision: String,
    pub manifest: String,
    pub files: BTreeMap<String, String>,
}

/// 64-bit FNV-1a over UTF-8 bytes: the publisher's non-security content digest.
pub(crate) fn digest(value: &str) -> String {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in value.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

fn gzip_len(text: &str) -> usize {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(text.as_bytes()).expect("in-memory gzip");
    encoder.finish().expect("in-memory gzip").len()
}

fn shard_json(words: &[(&str, &[u32])]) -> String {
    let words: Vec<_> = words
        .iter()
        .map(|(word, blocks)| {
            let mut previous = 0;
            let deltas: Vec<_> = blocks
                .iter()
                .map(|n| {
                    let delta = n - previous;
                    previous = *n;
                    delta
                })
                .collect();
            json!([word, deltas])
        })
        .collect();
    json!({"format_version": FORMAT_VERSION, "words": words}).to_string()
}

fn prefix(word: &str, chars: usize) -> &str {
    word.char_indices()
        .nth(chars)
        .map_or(word, |(at, _)| &word[..at])
}

fn chunk_json(blocks: &[BlockRecord]) -> String {
    let blocks: Vec<_> = blocks
        .iter()
        .map(|b| json!([b.page, b.fragment, b.text, b.parents]))
        .collect();
    json!({"format_version": FORMAT_VERSION, "blocks": blocks}).to_string()
}

// Record chunks with the number of their first block. Blocks are cut into groups of
// `chunk_size` records; a group over `chunk_budget` gzip splits into the longest runs that
// fit, so an edit can move chunk boundaries only inside its own group. A single record over
// the budget still gets a chunk of its own.
fn chunk_files(blocks: &[BlockRecord], options: &FileOptions) -> Vec<(usize, String)> {
    let mut out = Vec::new();
    for (group, records) in blocks.chunks(options.chunk_size.max(1)).enumerate() {
        let mut start = 0;
        while start < records.len() {
            let rest = &records[start..];
            let fits = |count: usize| gzip_len(&chunk_json(&rest[..count])) <= options.chunk_budget;
            // The longest run that fits: `low` fits (or is one record), `high` does not.
            let mut count = rest.len();
            if !fits(count) {
                let (mut low, mut high) = (1, count);
                while high - low > 1 {
                    let middle = (low + high) / 2;
                    if fits(middle) {
                        low = middle
                    } else {
                        high = middle
                    }
                }
                count = low;
            }
            out.push((
                group * options.chunk_size.max(1) + start,
                chunk_json(&rest[..count]),
            ));
            start += count;
        }
    }
    out
}

// A shard over budget splits by the next character; the words no longer than its key
// stay behind in a terminal shard under the same key.
fn place<'a>(
    key: &'a str,
    words: Vec<(&'a str, &'a [u32])>,
    budget: usize,
    out: &mut BTreeMap<String, String>,
) {
    let depth = key.chars().count();
    let json = shard_json(&words);
    if gzip_len(&json) <= budget || words.iter().all(|(w, _)| w.chars().count() <= depth) {
        out.insert(key.to_owned(), json);
        return;
    }
    let mut terminal = Vec::new();
    let mut groups: BTreeMap<&str, Vec<_>> = BTreeMap::new();
    for (word, blocks) in words {
        if word.chars().count() == depth {
            terminal.push((word, blocks));
        } else {
            groups
                .entry(prefix(word, depth + 1))
                .or_default()
                .push((word, blocks));
        }
    }
    if !terminal.is_empty() {
        out.insert(key.to_owned(), shard_json(&terminal));
    }
    for (key, words) in groups {
        place(key, words, budget, out);
    }
}

impl SearchIndex {
    /// Order pages by normalized title then id, and blocks by page then source
    /// position, so one page's blocks are consecutive; then index the block words.
    /// Blocks whose page has no record are dropped.
    pub fn new(records: Records) -> Self {
        let mut pages = records.pages;
        pages.sort_by(|a, b| {
            utf16_cmp(&normalize(&a.title), &normalize(&b.title)).then_with(|| a.id.cmp(&b.id))
        });
        let order: BTreeMap<_, _> = pages
            .iter()
            .enumerate()
            .map(|(n, p)| (p.id.clone(), n))
            .collect();
        let mut blocks: Vec<_> = records
            .blocks
            .into_iter()
            .filter(|b| order.contains_key(&b.page))
            .collect();
        blocks.sort_by_key(|b| (order[&b.page], b.position));
        let keys = pages
            .iter()
            .map(|p| {
                (
                    normalize(&p.title),
                    p.aliases.iter().map(|a| normalize(a)).collect(),
                )
            })
            .collect();
        let words = WordIndex::build(blocks.iter().map(|b| b.text.as_str()));
        Self {
            pages,
            keys,
            blocks,
            words,
        }
    }

    /// Page records in title order.
    pub fn pages(&self) -> &[PageRecord] {
        &self.pages
    }

    /// Block records in page and position order; a block's number is its place here.
    pub fn blocks(&self) -> &[BlockRecord] {
        &self.blocks
    }

    /// The word index over [`Self::blocks`].
    pub fn words(&self) -> &WordIndex {
        &self.words
    }

    /// Answer `input` as the browser engine does.
    ///
    /// Pages match when the query's characters appear in order in a normalized title or
    /// alias, ranked as exact title, exact alias, prefix, then the higher fuzzy score,
    /// ties in title order. Blocks match when every query word starts a word of the
    /// block, ranked by how many query words equal a whole word, ties in record order.
    /// A one-character word matches only that whole word, and a query of one-character
    /// words searches titles alone.
    pub fn query(&self, input: &str, options: &QueryOptions) -> Result<QueryResults, QueryError> {
        if input.chars().count() > MAX_QUERY_CHARS {
            return Err(QueryError::InvalidQuery);
        }
        if !(1..=MAX_RESULTS).contains(&options.limit) {
            return Err(QueryError::InvalidLimit);
        }
        let query = normalize(input);
        let mut terms: Vec<&str> = Vec::new();
        for word in words(&query) {
            if !terms.contains(&word) {
                terms.push(word);
            }
        }
        if terms.len() > MAX_TERMS {
            return Err(QueryError::TooManyTerms);
        }
        let mut results = QueryResults::default();
        if query.is_empty() {
            return Ok(results);
        }
        let mut pages = Vec::new();
        for (page, (title, aliases)) in self.keys.iter().enumerate() {
            let candidates = std::iter::once((title, None)).chain(
                aliases
                    .iter()
                    .zip(&self.pages[page].aliases)
                    .map(|(k, a)| (k, Some(a))),
            );
            let mut best: Option<(u8, f64, Option<&String>)> = None;
            for (key, alias) in candidates {
                let score = fuzzy_score(&query, key);
                let bucket = if key == &query {
                    if alias.is_none() { 0 } else { 1 }
                } else if key.starts_with(&query) {
                    2
                } else if score > 0.0 {
                    3
                } else {
                    continue;
                };
                if best.is_none_or(|(b, s, _)| bucket < b || (bucket == b && score > s)) {
                    best = Some((bucket, score, alias));
                }
            }
            if let Some((bucket, score, alias)) = best {
                pages.push((bucket, score, page, alias.cloned()));
            }
        }
        pages.sort_by(|a, b| a.0.cmp(&b.0).then(b.1.total_cmp(&a.1)).then(a.2.cmp(&b.2)));
        results.pages = Group {
            count: pages.len(),
            hits: pages
                .into_iter()
                .take(options.limit)
                .map(|(_, _, page, alias)| PageHit { page, alias })
                .collect(),
        };
        if terms.iter().all(|t| t.chars().count() == 1) {
            return Ok(results);
        }
        let mut matched: Option<BTreeSet<u32>> = None;
        for term in &terms {
            let blocks: BTreeSet<u32> = if term.chars().count() == 1 {
                self.words.get(term).iter().copied().collect()
            } else {
                self.words
                    .prefixed(term)
                    .flat_map(|(_, b)| b.iter().copied())
                    .collect()
            };
            matched = Some(match matched {
                None => blocks,
                Some(previous) => previous.intersection(&blocks).copied().collect(),
            });
        }
        let mut blocks: Vec<_> = matched
            .unwrap_or_default()
            .into_iter()
            .map(|n| {
                let exact = terms
                    .iter()
                    .filter(|t| self.words.get(t).binary_search(&n).is_ok())
                    .count();
                (exact, n)
            })
            .collect();
        blocks.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
        let group = |hits: Vec<u32>| Group {
            count: hits.len(),
            hits: hits.into_iter().take(options.limit).collect(),
        };
        if let Some(current) = &options.current_page {
            results.current_page = group(
                blocks
                    .iter()
                    .map(|(_, n)| *n)
                    .filter(|n| &self.blocks[*n as usize].page == current)
                    .collect(),
            );
        }
        results.blocks = group(blocks.into_iter().map(|(_, n)| n).collect());
        Ok(results)
    }

    /// The format version 2 files: the title index, the word-prefix shards and the
    /// record chunks, each named `lsg-search-<kind>-<digest>.json`, and the manifest
    /// `lsg-search-manifest-<revision>.json` that lists them for the engine at `engine`.
    /// Page ids must be routes beside the manifest.
    pub fn files(&self, engine: &str, options: &FileOptions) -> SearchFiles {
        let mut files = BTreeMap::new();
        let mut add = |kind: &str, content: String| {
            let digest = digest(&content);
            files.insert(format!("lsg-search-{kind}-{digest}.json"), content);
            digest
        };
        let mut counts = vec![0_usize; self.pages.len()];
        let order: BTreeMap<_, _> = self
            .pages
            .iter()
            .enumerate()
            .map(|(n, p)| (p.id.as_str(), n))
            .collect();
        for block in &self.blocks {
            counts[order[block.page.as_str()]] += 1;
        }
        let titles: Vec<_> = self
            .pages
            .iter()
            .zip(&counts)
            .map(|(p, count)| json!([p.title, p.id, p.aliases, count]))
            .collect();
        let titles = add(
            "titles",
            json!({"format_version": FORMAT_VERSION, "pages": titles}).to_string(),
        );
        let mut shards = BTreeMap::new();
        let mut groups: BTreeMap<&str, Vec<_>> = BTreeMap::new();
        for (word, blocks) in self.words.iter() {
            groups
                .entry(prefix(word, 2))
                .or_default()
                .push((word, blocks));
        }
        for (key, words) in groups {
            place(key, words, options.shard_budget, &mut shards);
        }
        let shards: serde_json::Map<_, _> = shards
            .into_iter()
            .map(|(key, json)| (key, add("shard", json).into()))
            .collect();
        let mut chunks = Vec::new();
        let mut starts = Vec::new();
        for (start, json) in chunk_files(&self.blocks, options) {
            starts.push(start);
            chunks.push(add("chunk", json));
        }
        let mut manifest = json!({
            "format_version": FORMAT_VERSION,
            "engine": engine,
            "titles": titles,
            "blocks": self.blocks.len(),
            "chunks": chunks,
            "chunk_starts": starts,
            "shards": shards,
        });
        // The revision names this exact set of files, so any change to one renames it.
        let revision = digest(&format!("search-v2:{manifest}"));
        manifest["revision"] = revision.clone().into();
        let manifest_name = format!("lsg-search-manifest-{revision}.json");
        files.insert(manifest_name.clone(), manifest.to_string());
        SearchFiles {
            revision,
            manifest: manifest_name,
            files,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn index(pages: &[(&str, &[&str])], blocks: &[(&str, &str)]) -> SearchIndex {
        SearchIndex::new(Records {
            pages: pages
                .iter()
                .map(|(title, aliases)| PageRecord {
                    id: format!("p-{}.html", digest(title)),
                    title: (*title).into(),
                    aliases: aliases.iter().map(|a| (*a).into()).collect(),
                })
                .collect(),
            blocks: blocks
                .iter()
                .enumerate()
                .map(|(n, (page, text))| BlockRecord {
                    page: format!("p-{}.html", digest(page)),
                    position: n,
                    fragment: format!("source-block-{n}"),
                    text: (*text).into(),
                    parents: vec![],
                })
                .collect(),
        })
    }

    fn titles(index: &SearchIndex, results: &QueryResults) -> Vec<String> {
        results
            .pages
            .hits
            .iter()
            .map(|hit| {
                let title = &index.pages()[hit.page].title;
                hit.alias
                    .as_ref()
                    .map_or(title.clone(), |alias| format!("{title} ← {alias}"))
            })
            .collect()
    }

    fn texts(index: &SearchIndex, hits: &[u32]) -> Vec<String> {
        hits.iter()
            .map(|n| index.blocks()[*n as usize].text.clone())
            .collect()
    }

    fn garden() -> SearchIndex {
        index(
            &[
                ("tmux", &[]),
                ("tmux/Config", &[]),
                ("tmux/Config/local", &["tmux.conf.local"]),
                ("Logseq", &[]),
                ("Logseq/Query", &["logseq query"]),
            ],
            &[
                ("tmux/Config", "tmux.conf: set -g mouse on"),
                ("tmux", "tmux runs inside a terminal"),
                ("Logseq", "Logseq query results find blocks"),
                ("Logseq/Query", "a query in logseq"),
                ("Logseq", "logseq alone"),
                ("tmux/Config/local", "tmux.conf.local overrides tmux.conf"),
            ],
        )
    }

    #[test]
    fn words_split_at_punctuation() {
        let index = garden();
        let block = index
            .blocks()
            .iter()
            .position(|b| b.text == "tmux.conf: set -g mouse on")
            .unwrap() as u32;
        for word in ["tmux", "conf", "set", "g", "mouse", "on"] {
            assert!(index.words().get(word).contains(&block), "{word}");
        }
        assert_eq!(
            words(&normalize("tmux.conf: set -g mouse on")).collect::<Vec<_>>(),
            ["tmux", "conf", "set", "g", "mouse", "on"]
        );
    }

    #[test]
    fn tmux_ranks_exact_title_then_prefixes() {
        let index = garden();
        let results = index.query("tmux", &QueryOptions::default()).unwrap();
        assert_eq!(
            titles(&index, &results),
            ["tmux", "tmux/Config", "tmux/Config/local"]
        );
        assert_eq!(results.blocks.count, 3);
        // Whole-word matches first, then record order.
        assert_eq!(
            texts(&index, &results.blocks.hits),
            [
                "tmux runs inside a terminal",
                "tmux.conf: set -g mouse on",
                "tmux.conf.local overrides tmux.conf"
            ]
        );
    }

    #[test]
    fn tmx_finds_tmux_first() {
        let index = garden();
        let results = index.query("tmx", &QueryOptions::default()).unwrap();
        assert_eq!(titles(&index, &results)[0], "tmux");
        assert_eq!(results.blocks.count, 0);
    }

    #[test]
    fn logseq_query_needs_both_words() {
        let index = garden();
        let results = index
            .query("logseq query", &QueryOptions::default())
            .unwrap();
        assert_eq!(titles(&index, &results)[0], "Logseq/Query ← logseq query");
        assert_eq!(
            texts(&index, &results.blocks.hits),
            ["Logseq query results find blocks", "a query in logseq"]
        );
    }

    #[test]
    fn alias_returns_its_page_once() {
        let index = garden();
        let results = index
            .query("tmux.conf.local", &QueryOptions::default())
            .unwrap();
        assert_eq!(
            titles(&index, &results),
            ["tmux/Config/local ← tmux.conf.local"]
        );
        assert_eq!(
            texts(&index, &results.blocks.hits),
            ["tmux.conf.local overrides tmux.conf"]
        );
    }

    #[test]
    fn counts_beyond_the_limit_and_current_page() {
        let blocks: Vec<_> = (0..250).map(|_| ("tmux", "tmux again")).collect();
        let index = index(&[("tmux", &[]), ("other", &[])], &blocks);
        let options = QueryOptions {
            current_page: Some(index.pages()[0].id.clone()),
            ..QueryOptions::default()
        };
        let results = index.query("tmux", &options).unwrap();
        assert_eq!(
            (results.blocks.count, results.blocks.hits.len()),
            (250, 100)
        );
        assert_eq!(results.current_page, Group::default());
        assert_eq!(
            index.query("t", &QueryOptions::default()).unwrap().blocks,
            Group::default()
        );
    }

    #[test]
    fn bounded_input() {
        let index = garden();
        let options = QueryOptions::default();
        assert_eq!(
            index.query(&"a".repeat(257), &options),
            Err(QueryError::InvalidQuery)
        );
        let many: Vec<_> = (0..17).map(|n| format!("w{n}")).collect();
        assert_eq!(
            index.query(&many.join(" "), &options),
            Err(QueryError::TooManyTerms)
        );
        let limit = QueryOptions {
            limit: 0,
            ..QueryOptions::default()
        };
        assert_eq!(index.query("tmux", &limit), Err(QueryError::InvalidLimit));
        assert_eq!(
            index.query(" \n", &options).unwrap(),
            QueryResults::default()
        );
    }

    #[test]
    fn large_shards_split_by_the_next_character() {
        let blocks: Vec<_> = (0..400)
            .map(|n| ("tmux", if n % 2 == 0 { "this" } else { "those th" }))
            .collect();
        let index = index(&[("tmux", &[])], &blocks);
        let files = index.files(
            "engine.mjs",
            &FileOptions {
                shard_budget: 100,
                chunk_size: 256,
                chunk_budget: 32_000,
            },
        );
        let manifest: serde_json::Value =
            serde_json::from_str(&files.files[&files.manifest]).unwrap();
        let keys: Vec<_> = manifest["shards"]
            .as_object()
            .unwrap()
            .keys()
            .cloned()
            .collect();
        assert_eq!(keys, ["th", "thi", "tho"]);
        assert_eq!(manifest["chunks"].as_array().unwrap().len(), 2);
    }

    #[test]
    fn one_edited_block_changes_only_its_shards_and_chunk() {
        let build = |edited: &str| {
            let mut blocks: Vec<(&str, &str)> =
                (0..600).map(|_| ("other", "filler text")).collect();
            blocks.push(("tmux", "keep this"));
            blocks.push(("tmux", edited));
            let index = index(&[("tmux", &[]), ("other", &[])], &blocks);
            index.files(
                "engine.mjs",
                &FileOptions {
                    shard_budget: 64 * 1024,
                    chunk_size: 256,
                    chunk_budget: 32_000,
                },
            )
        };
        let (before, after) = (build("alpha"), build("beta"));
        let changed: BTreeSet<_> = before
            .files
            .iter()
            .filter(|(name, content)| after.files.get(*name) != Some(content))
            .map(|(name, content)| {
                let kind = name.split('-').nth(2).unwrap().to_owned();
                if kind == "shard" {
                    let shard: serde_json::Value = serde_json::from_str(content).unwrap();
                    format!("shard {}", shard["words"][0][0].as_str().unwrap())
                } else {
                    kind
                }
            })
            .collect();
        assert_eq!(
            changed,
            BTreeSet::from(["manifest".into(), "chunk".into(), "shard alpha".into()])
        );
        assert_eq!(before.files.len(), after.files.len());
        assert_ne!(before.revision, after.revision);
    }

    // Text that gzip cannot shrink much: `count` pseudo-random words.
    fn noise(seed: u64, count: usize) -> String {
        let mut state = seed.wrapping_mul(0x9e3779b97f4a7c15) | 1;
        (0..count)
            .map(|_| {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                format!("{:x}", state & 0xff_ffff)
            })
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn chunks(files: &SearchFiles) -> Vec<(u64, String)> {
        let manifest: serde_json::Value =
            serde_json::from_str(&files.files[&files.manifest]).unwrap();
        let starts = manifest["chunk_starts"].as_array().unwrap();
        let digests = manifest["chunks"].as_array().unwrap();
        assert_eq!(starts.len(), digests.len());
        starts
            .iter()
            .zip(digests)
            .map(|(s, d)| (s.as_u64().unwrap(), d.as_str().unwrap().to_owned()))
            .collect()
    }

    #[test]
    fn chunks_close_at_the_record_count_or_the_gzip_budget() {
        let texts: Vec<_> = (0..40).map(|n| noise(n, 60)).collect();
        let mut blocks: Vec<(&str, &str)> = texts.iter().map(|t| ("long", t.as_str())).collect();
        blocks.extend((0..20).map(|_| ("short", "a few words")));
        let index = index(&[("long", &[]), ("short", &[])], &blocks);
        let options = FileOptions {
            shard_budget: 64_000,
            chunk_size: 16,
            chunk_budget: 2_000,
        };
        let files = index.files("engine.mjs", &options);
        let chunks = chunks(&files);
        let starts: Vec<_> = chunks.iter().map(|(start, _)| *start).collect();
        // Every group of 16 starts a chunk; the long groups split further by size.
        for group in [0, 16, 32, 48] {
            assert!(starts.contains(&group), "{starts:?}");
        }
        assert!(starts.len() > 6, "{starts:?}");
        assert_eq!(starts[0], 0);
        assert!(starts.windows(2).all(|w| w[0] < w[1]));
        let mut total = 0;
        for (n, (start, digest)) in chunks.iter().enumerate() {
            let json = &files.files[&format!("lsg-search-chunk-{digest}.json")];
            assert!(gzip_len(json) <= options.chunk_budget, "chunk {start}");
            let records = serde_json::from_str::<serde_json::Value>(json).unwrap()["blocks"]
                .as_array()
                .unwrap()
                .len() as u64;
            let end = chunks.get(n + 1).map_or(60, |(next, _)| *next);
            assert_eq!(records, end - start);
            assert!(records <= 16);
            total += records;
        }
        assert_eq!(total, 60);
        // The short records fill whole groups of 16.
        assert!(chunks.iter().any(|(start, _)| *start == 48));
        assert!(!starts.contains(&49));
    }

    #[test]
    fn an_edit_that_moves_chunk_boundaries_stays_inside_its_group() {
        let texts: Vec<_> = (0..64).map(|n| noise(n, 30)).collect();
        let build = |edited: &str| {
            let mut blocks: Vec<(&str, &str)> =
                texts.iter().map(|t| ("long", t.as_str())).collect();
            blocks[20] = ("long", edited);
            let index = index(&[("long", &[])], &blocks);
            index.files(
                "engine.mjs",
                &FileOptions {
                    shard_budget: 64_000,
                    chunk_size: 16,
                    chunk_budget: 1_500,
                },
            )
        };
        let longer = noise(999, 200);
        let (before, after) = (chunks(&build("short")), chunks(&build(&longer)));
        assert_ne!(
            before.iter().map(|c| c.0).collect::<Vec<_>>(),
            after.iter().map(|c| c.0).collect::<Vec<_>>(),
            "the longer block moves a boundary"
        );
        // Chunks outside the edited block's group of 16 (blocks 16 to 31) keep their files.
        let outside = |chunks: &[(u64, String)]| -> Vec<(u64, String)> {
            chunks
                .iter()
                .filter(|(start, _)| !(16..32).contains(start))
                .cloned()
                .collect()
        };
        assert_eq!(outside(&before), outside(&after));
    }
}
