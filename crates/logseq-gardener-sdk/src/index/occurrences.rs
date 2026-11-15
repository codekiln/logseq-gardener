//! What one document writes: its links, tags, block references, embeds, properties and
//! block ids, each with the source range it occupies.

use crate::ast::{Block, Inline, ListItem, Url};
use crate::block_ids::id_property;
use crate::inline_references::tag_page_name;
use crate::outline::uuid;
use crate::page_names::is_linkable_property;
use crate::property_names::valid_property_name;

/// A range of a document's source, as offsets into its UTF-8 text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceRange {
    pub start: usize,
    /// The offset after the range's last character.
    pub end: usize,
}

impl SourceRange {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    /// Whether the range holds `offset`.
    pub fn contains(&self, offset: usize) -> bool {
        self.start <= offset && offset < self.end
    }

    /// The text of the range in `source`; empty when the range is outside it.
    pub fn text<'a>(&self, source: &'a str) -> &'a str {
        source.get(self.start..self.end).unwrap_or_default()
    }
}

/// What an [`Occurrence`] is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OccurrenceKind {
    /// `[[Page]]`, also as the target of a labelled link, `[label]([[Page]])`.
    PageLink,
    /// `#tag` or `#[[multi word]]`.
    Tag,
    /// `((uuid))`, also as the target of a labelled link.
    BlockReference,
    /// `{{embed [[Page]]}}`.
    PageEmbed,
    /// `{{embed ((uuid))}}`.
    BlockEmbed,
    /// The key of a `key:: value` line.
    PropertyKey,
    /// A page name written bare in an `alias::`, `aliases::` or `tags::` value.
    PropertyValue,
    /// The value of an `id::`, `custom-id::` or `custom_id::` line.
    BlockId,
}

/// What an [`Occurrence`] points at, as its author wrote it. [`super::GardenIndex::resolve`]
/// finds the page or block.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Target {
    /// A page, by the name written.
    Page(String),
    /// A block, by its id in lower case.
    Block(String),
    /// A property, by its key in lower case.
    Property(String),
}

/// One place where a document names a page, a block or a property.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Occurrence {
    pub kind: OccurrenceKind,
    /// The whole syntax, such as `[[Page]]`, `#tag` or `{{embed ((uuid))}}`.
    pub range: SourceRange,
    /// The name inside it, such as `Page`, `tag` or the uuid: the text a rename replaces.
    pub name_range: SourceRange,
    pub target: Target,
}

/// One `key:: value` line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropertyLine {
    /// The key in lower case.
    pub key: String,
    pub key_range: SourceRange,
    /// The value without the spaces around it.
    pub value: String,
    pub value_range: SourceRange,
    /// Where the block the line belongs to starts: the start of its bullet's line, before
    /// any indentation, or the properties themselves for the properties before a page's
    /// first bullet.
    pub block_start: usize,
}

/// A block that writes a valid `id::`, `custom-id::` or `custom_id::`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockIdSite {
    /// The id in lower case.
    pub id: String,
    /// The id as written in the property's value.
    pub range: SourceRange,
    /// Where the block starts, as in [`PropertyLine::block_start`].
    pub block_start: usize,
}

/// Everything [`extract`] reads from one document.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct Extracted {
    pub occurrences: Vec<Occurrence>,
    pub properties: Vec<PropertyLine>,
    pub block_ids: Vec<BlockIdSite>,
}

/// Read what a parsed document writes. Occurrences come back sorted by position.
pub(crate) fn extract(source: &str, blocks: &[Block]) -> Extracted {
    let mut reader = Reader {
        source,
        found: Extracted::default(),
    };
    reader.blocks(blocks, 0);
    reader
        .found
        .occurrences
        .sort_by_key(|occurrence| (occurrence.range.start, occurrence.range.end));
    reader.found
}

struct Reader<'a> {
    source: &'a str,
    found: Extracted,
}

impl Reader<'_> {
    fn blocks(&mut self, blocks: &[Block], depth: usize) {
        if depth > 64 {
            return;
        }
        // Where the most recent bullet at this level starts.
        let mut owner = None;
        for block in blocks {
            match block {
                Block::Bullet { inline, span, .. } | Block::Heading { inline, span, .. } => {
                    owner = span.map(|span| span.0);
                    self.inlines(inline, 0, 0);
                }
                Block::Paragraph { inline, .. } | Block::FootnoteDef { inline, .. } => {
                    self.inlines(inline, 0, 0);
                }
                Block::Properties {
                    span: Some(span), ..
                } => self.properties(span.0, span.1, owner.unwrap_or(span.0)),
                Block::Quote { children, .. } | Block::Custom { children, .. } => {
                    self.blocks(children, depth + 1);
                }
                Block::List { items, .. } => self.list(items, depth + 1),
                Block::Table { header, rows, .. } => {
                    for cell in header.iter().flatten().chain(rows.iter().flatten()) {
                        self.inlines(cell, 0, 0);
                    }
                }
                _ => {}
            }
        }
    }

    fn list(&mut self, items: &[ListItem], depth: usize) {
        if depth > 64 {
            return;
        }
        for item in items {
            self.inlines(&item.name, 0, 0);
            self.blocks(&item.content, depth + 1);
            self.list(&item.items, depth + 1);
        }
    }

    /// Read inline nodes whose spans start counting at `base` in the source.
    fn inlines(&mut self, nodes: &[Inline], base: usize, depth: usize) {
        if depth > 64 {
            return;
        }
        for node in nodes {
            match node {
                Inline::Link {
                    url: Url::PageRef { v },
                    image: false,
                    span: Some(span),
                    ..
                } => self.named(
                    OccurrenceKind::PageLink,
                    base + span.0,
                    base + span.1,
                    "[[",
                    v,
                    "]]",
                ),
                Inline::Link {
                    url: Url::BlockRef { v },
                    span: Some(span),
                    ..
                } => self.named(
                    OccurrenceKind::BlockReference,
                    base + span.0,
                    base + span.1,
                    "((",
                    v,
                    "))",
                ),
                Inline::NestedLink {
                    content,
                    span: Some(span),
                } => {
                    if let Some(name) = content
                        .strip_prefix("[[")
                        .and_then(|inner| inner.strip_suffix("]]"))
                    {
                        self.named(
                            OccurrenceKind::PageLink,
                            base + span.0,
                            base + span.1,
                            "[[",
                            name,
                            "]]",
                        );
                    }
                }
                Inline::Tag {
                    children,
                    span: Some(span),
                } => self.tag(children, base + span.0, base + span.1),
                Inline::Macro {
                    name,
                    args,
                    span: Some(span),
                } if name.eq_ignore_ascii_case("embed") => {
                    let argument = args.first().map_or("", |argument| argument.trim());
                    let inner = |open: &str, close: &str| {
                        argument
                            .strip_prefix(open)
                            .and_then(|rest| rest.strip_suffix(close))
                    };
                    if let Some(name) = inner("[[", "]]") {
                        self.named(
                            OccurrenceKind::PageEmbed,
                            base + span.0,
                            base + span.1,
                            "[[",
                            name,
                            "]]",
                        );
                    } else if let Some(id) = inner("((", "))") {
                        self.named(
                            OccurrenceKind::BlockEmbed,
                            base + span.0,
                            base + span.1,
                            "((",
                            id,
                            "))",
                        );
                    }
                }
                Inline::Link { label, .. } => self.inlines(label, base, depth + 1),
                Inline::Emphasis { children, .. }
                | Inline::Subscript { children, .. }
                | Inline::Superscript { children, .. } => self.inlines(children, base, depth + 1),
                _ => {}
            }
        }
    }

    /// Record `open name close` written somewhere in `start..end`. The parser's range is
    /// checked against the source, so a range that does not hold the name records nothing.
    fn named(
        &mut self,
        kind: OccurrenceKind,
        start: usize,
        end: usize,
        open: &str,
        name: &str,
        close: &str,
    ) {
        let Some(text) = self.source.get(start..end) else {
            return;
        };
        let Some(at) = text.find(&format!("{open}{name}{close}")) else {
            return;
        };
        let name_start = start + at + open.len();
        let target = match kind {
            OccurrenceKind::BlockReference | OccurrenceKind::BlockEmbed => {
                Target::Block(name.trim().to_ascii_lowercase())
            }
            _ => Target::Page(name.to_owned()),
        };
        self.found.occurrences.push(Occurrence {
            kind,
            range: SourceRange::new(start, end),
            name_range: SourceRange::new(name_start, name_start + name.len()),
            target,
        });
    }

    fn tag(&mut self, children: &[Inline], start: usize, end: usize) {
        let Some(text) = self.source.get(start..end) else {
            return;
        };
        let Some(written) = text.strip_prefix('#') else {
            return;
        };
        let name = tag_page_name(children);
        if name.is_empty() {
            return;
        }
        let bracketed = written.starts_with("[[") && written.ends_with("]]") && written.len() >= 4;
        let name_range = if bracketed {
            SourceRange::new(start + 3, end - 2)
        } else {
            SourceRange::new(start + 1, end)
        };
        self.found.occurrences.push(Occurrence {
            kind: OccurrenceKind::Tag,
            range: SourceRange::new(start, end),
            name_range,
            target: Target::Page(name),
        });
    }

    /// Read the `key:: value` lines of a properties block that spans `start..end`.
    fn properties(&mut self, start: usize, end: usize, block_start: usize) {
        let Some(text) = self.source.get(start..end) else {
            return;
        };
        let first = self.found.properties.len();
        let mut line_start = start;
        for line in text.split_inclusive('\n') {
            let here = line_start;
            line_start += line.len();
            let content = line.trim_end_matches(['\n', '\r']);
            let indent = content.len() - content.trim_start().len();
            let Some(separator) = content[indent..].find("::").map(|at| at + indent) else {
                continue;
            };
            let after = &content[separator + 2..];
            let key = &content[indent..separator];
            if !(after.is_empty() || after.starts_with([' ', '\t']))
                || key.is_empty()
                || key.contains(char::is_whitespace)
                || !valid_property_name(key)
            {
                continue;
            }
            let value = after.trim();
            let value_start = here + separator + 2 + (after.len() - after.trim_start().len());
            let line = PropertyLine {
                key: key.to_lowercase(),
                key_range: SourceRange::new(here + indent, here + separator),
                value: value.to_owned(),
                value_range: SourceRange::new(value_start, value_start + value.len()),
                block_start,
            };
            self.found.occurrences.push(Occurrence {
                kind: OccurrenceKind::PropertyKey,
                range: line.key_range,
                name_range: line.key_range,
                target: Target::Property(line.key.clone()),
            });
            self.property_value(&line);
            self.found.properties.push(line);
        }
        // The id Logseq reads from the block: `custom-id` before `custom_id` before `id`, and
        // the last line of that key.
        let lines = &self.found.properties[first..];
        let id = ["custom-id", "custom_id", "id"]
            .into_iter()
            .find_map(|key| lines.iter().rev().find(|line| line.key == key))
            .and_then(|line| Some((uuid(&line.value)?, line.value_range)));
        if let Some((id, range)) = id {
            self.found.block_ids.push(BlockIdSite {
                id,
                range,
                block_start,
            });
        }
    }

    fn property_value(&mut self, line: &PropertyLine) {
        if id_property(&line.key) {
            if let Some(id) = uuid(&line.value) {
                self.found.occurrences.push(Occurrence {
                    kind: OccurrenceKind::BlockId,
                    range: line.value_range,
                    name_range: line.value_range,
                    target: Target::Block(id),
                });
            }
            return;
        }
        let value = line.value.as_str();
        let base = line.value_range.start;
        let nodes = lsdoc::inline(value, "markdown");
        self.inlines(&nodes, base, 0);
        // Logseq reads the plain text of these values as page names separated by commas, and
        // reads a value in double quotes as text.
        let quoted = value.len() >= 2 && value.starts_with('"') && value.ends_with('"');
        if !is_linkable_property(&line.key) || quoted {
            return;
        }
        for node in &nodes {
            let Inline::Plain {
                text,
                span: Some(span),
                ..
            } = node
            else {
                continue;
            };
            // A plain node whose text differs from its source, such as one with an escape,
            // gives no reliable ranges.
            if value.get(span.0..span.1) != Some(text.as_str()) {
                continue;
            }
            let mut part_start = span.0;
            for part in text.split([',', '，']) {
                let name = part.trim();
                if !name.is_empty() {
                    let start = base + part_start + (part.len() - part.trim_start().len());
                    let range = SourceRange::new(start, start + name.len());
                    self.found.occurrences.push(Occurrence {
                        kind: OccurrenceKind::PropertyValue,
                        range,
                        name_range: range,
                        target: Target::Page(name.to_owned()),
                    });
                }
                // Both separators are one character; step over the one that ended the part.
                let separator = text[part_start - span.0 + part.len()..]
                    .chars()
                    .next()
                    .map_or(0, char::len_utf8);
                part_start += part.len() + separator;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::garden::parse_document;

    const ID: &str = "66f1a2b0-0000-4000-8000-000000000001";

    fn read(source: &str) -> Extracted {
        extract(source, &parse_document(source).blocks)
    }

    /// Each occurrence as its kind, whole text, name text and target.
    fn summary(source: &str) -> Vec<(OccurrenceKind, String, String, Target)> {
        read(source)
            .occurrences
            .into_iter()
            .map(|occurrence| {
                (
                    occurrence.kind,
                    occurrence.range.text(source).to_owned(),
                    occurrence.name_range.text(source).to_owned(),
                    occurrence.target,
                )
            })
            .collect()
    }

    fn page(name: &str) -> Target {
        Target::Page(name.to_owned())
    }

    fn entry(
        kind: OccurrenceKind,
        whole: &str,
        name: &str,
        target: Target,
    ) -> (OccurrenceKind, String, String, Target) {
        (kind, whole.to_owned(), name.to_owned(), target)
    }

    #[test]
    fn reads_links_tags_references_and_embeds_in_nested_blocks() {
        let source = format!(
            "- Parent é [[Fruit/Apple]] and #recipes\n\t- child #[[multi word]] (({ID}))\n\t\t- {{{{embed [[Fruit]]}}}} {{{{embed (({ID}))}}}} [the apple]([[Fruit/Apple]]) **[[Bold]]**\n"
        );
        use OccurrenceKind::*;
        let block = Target::Block(ID.to_owned());
        assert_eq!(
            summary(&source),
            [
                entry(
                    PageLink,
                    "[[Fruit/Apple]]",
                    "Fruit/Apple",
                    page("Fruit/Apple")
                ),
                entry(Tag, "#recipes", "recipes", page("recipes")),
                entry(Tag, "#[[multi word]]", "multi word", page("multi word")),
                entry(BlockReference, &format!("(({ID}))"), ID, block.clone()),
                entry(PageEmbed, "{{embed [[Fruit]]}}", "Fruit", page("Fruit")),
                entry(BlockEmbed, &format!("{{{{embed (({ID}))}}}}"), ID, block),
                entry(
                    PageLink,
                    "[the apple]([[Fruit/Apple]])",
                    "Fruit/Apple",
                    page("Fruit/Apple")
                ),
                entry(PageLink, "[[Bold]]", "Bold", page("Bold")),
            ]
        );
    }

    #[test]
    fn reads_page_and_block_properties() {
        let source = format!(
            "tags:: [[Fruit]], recipes, #sweet\nalias:: Malus, \"quoted\"\n\n- Block\n  id:: {ID}\n  Type:: [[Recipe]] for two\n"
        );
        use OccurrenceKind::*;
        let key = |name: &str| {
            entry(
                PropertyKey,
                name,
                name,
                Target::Property(name.to_lowercase()),
            )
        };
        assert_eq!(
            summary(&source),
            [
                key("tags"),
                entry(PageLink, "[[Fruit]]", "Fruit", page("Fruit")),
                entry(PropertyValue, "recipes", "recipes", page("recipes")),
                entry(Tag, "#sweet", "sweet", page("sweet")),
                key("alias"),
                entry(PropertyValue, "Malus", "Malus", page("Malus")),
                entry(
                    PropertyValue,
                    "\"quoted\"",
                    "\"quoted\"",
                    page("\"quoted\"")
                ),
                key("id"),
                entry(BlockId, ID, ID, Target::Block(ID.to_owned())),
                key("Type"),
                entry(PageLink, "[[Recipe]]", "Recipe", page("Recipe")),
            ]
        );
        let found = read(&source);
        let bullet = source.find("- Block").unwrap();
        assert_eq!(
            found.block_ids,
            [BlockIdSite {
                id: ID.to_owned(),
                range: SourceRange::new(source.find(ID).unwrap(), source.find(ID).unwrap() + 36),
                block_start: bullet,
            }]
        );
        let lines: Vec<_> = found
            .properties
            .iter()
            .map(|line| (line.key.as_str(), line.value.as_str(), line.block_start))
            .collect();
        assert_eq!(
            lines,
            [
                ("tags", "[[Fruit]], recipes, #sweet", 0),
                ("alias", "Malus, \"quoted\"", 0),
                ("id", ID, bullet),
                ("type", "[[Recipe]] for two", bullet),
            ]
        );
    }

    #[test]
    fn leaves_out_code_and_quoted_values() {
        assert_eq!(
            summary("- `[[Not A Link]]` text\n- ```\n  [[Nor This]]\n  ```\n"),
            []
        );
        let quoted = summary("tags:: \"a, b\"\n");
        assert_eq!(quoted.len(), 1, "{quoted:?}");
    }

    #[test]
    fn every_block_that_writes_an_id_is_a_site() {
        let source = format!("- a\n  id:: {ID}\n- b\n  id:: {ID}\n- c\n  id:: not-a-uuid\n");
        let sites = read(&source).block_ids;
        let starts: Vec<_> = sites.iter().map(|site| site.block_start).collect();
        assert_eq!(
            starts,
            [source.find("- a").unwrap(), source.find("- b").unwrap()]
        );
    }
}
