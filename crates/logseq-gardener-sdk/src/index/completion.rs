//! Completion: what is being typed at a cursor, and what the garden offers for it.
//!
//! [`completion_context`] reads the text around an offset and reports a page link, a tag, a
//! block reference, a property key or a property value, with the text typed so far and the
//! range a chosen candidate replaces. It reads the line as text, because a half-typed
//! `[[Fru` is not a link the parser knows yet. [`GardenIndex::completions`] lists and ranks
//! the candidates for a context.
//!
//! ```
//! use logseq_gardener_sdk::index::{CompletionKind, completion_context};
//!
//! let text = "- See [[Fru]] today\n";
//! let context = completion_context(text, text.find("]]").unwrap()).unwrap();
//! assert_eq!(context.kind, CompletionKind::PageLink);
//! assert_eq!(context.typed, "Fru");
//! // A chosen page replaces the typed text and the closing pair after the cursor.
//! assert_eq!(context.replace.text(text), "Fru]]");
//! ```

use std::cmp::Ordering;
use std::collections::BTreeSet;

use super::{GardenIndex, SourceRange};
use crate::ast::Inline;
use crate::cards::is_hidden_property;
use crate::inline_references::tag_page_name;
use crate::journals::parse_journal_reference;
use crate::page_names::{has_outer_space, is_linkable_property, lookup_key, page_alias_spellings};
use crate::property_names::valid_property_name;
use crate::search::{fuzzy_score, normalize};

/// What is being typed at a cursor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompletionKind {
    /// A page name after `[[`, also in `{{embed [[` and `#[[`.
    PageLink,
    /// A page name after `#`.
    Tag,
    /// A block after `((`, also in `{{embed ((`.
    BlockReference,
    /// The key of a property line.
    PropertyKey,
    /// The value of the property with this key, in lower case.
    PropertyValue { key: String },
    /// One page name in the value of `tags::`, `alias::` or `aliases::`.
    PropertyPage,
}

/// What is being typed at a cursor, and the source a chosen candidate replaces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompletionContext {
    pub kind: CompletionKind,
    /// The text between the start of the name, key or value and the cursor.
    pub typed: String,
    /// The typed text, the rest of the name after the cursor, and a closing `]]` or `))`
    /// that follows it.
    pub replace: SourceRange,
    /// Whether the cursor directly follows `key::`, so a value needs a space before it.
    pub needs_space: bool,
}

/// What a [`Candidate`] is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CandidateKind {
    /// A page with a file, by its title.
    Page,
    /// A journal, by its title in the garden's journal title format.
    Journal,
    /// A namespace parent of a page title that has no file of its own.
    Namespace,
    /// An alias, as its page writes it.
    Alias,
    /// A block that writes an `id::`.
    Block,
    PropertyKey,
    PropertyValue,
}

/// One thing the garden offers at a cursor.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    pub kind: CandidateKind,
    /// The page name, alias, block text, property key or property value.
    pub label: String,
    /// The text that takes the place of [`CompletionContext::replace`].
    pub insert: String,
    /// The text the typed text matches. It differs from the label for a journal offered
    /// because the typed text spells its date in another format.
    pub filter: String,
    /// The title of the page an alias belongs to or a block is on.
    pub page: Option<String>,
    /// How many lines write a property key or value.
    pub uses: Option<usize>,
}

/// The ranked candidates for one context.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Completions {
    pub candidates: Vec<Candidate>,
    /// Whether more typing can bring candidates this list does not hold: the list was cut
    /// at the limit, or it holds page names, where a date candidate appears once the typed
    /// text spells a date.
    pub partial: bool,
}

/// The longest block text a candidate's label shows, in characters.
const BLOCK_LABEL_CHARS: usize = 120;

/// What is being typed at `offset` in `text`, or `None` in prose.
///
/// An open `[[` or `((` on the line wins over a tag and a property, and the later of the two
/// wins when both are open.
pub fn completion_context(text: &str, offset: usize) -> Option<CompletionContext> {
    let mut offset = offset.min(text.len());
    while !text.is_char_boundary(offset) {
        offset -= 1;
    }
    let line_start = text[..offset].rfind('\n').map_or(0, |at| at + 1);
    let line_end = text[offset..]
        .find('\n')
        .map_or(text.len(), |at| offset + at);
    let line = Line {
        offset,
        before: &text[line_start..offset],
        after: text[offset..line_end].trim_end_matches('\r'),
    };
    line.bracket()
        .or_else(|| line.tag())
        .or_else(|| line.property(&text[..line_start]))
}

/// The cursor's line, split at the cursor.
struct Line<'a> {
    offset: usize,
    before: &'a str,
    after: &'a str,
}

impl Line<'_> {
    fn context(&self, kind: CompletionKind, typed: &str, tail: usize) -> CompletionContext {
        CompletionContext {
            kind,
            typed: typed.to_owned(),
            replace: SourceRange::new(self.offset - typed.len(), self.offset + tail),
            needs_space: false,
        }
    }

    /// An open `[[` or `((`.
    fn bracket(&self) -> Option<CompletionContext> {
        let open = |open: &str, close: &str| {
            let at = self.before.rfind(open)?;
            (!self.before[at + 2..].contains(close)).then_some(at)
        };
        let link = open("[[", "]]");
        let block = open("((", "))");
        let (at, kind, close, brackets) = match (link, block) {
            (Some(link), Some(block)) if block > link => {
                (block, CompletionKind::BlockReference, "))", ['(', ')'])
            }
            (Some(link), _) => (link, CompletionKind::PageLink, "]]", ['[', ']']),
            (None, Some(block)) => (block, CompletionKind::BlockReference, "))", ['(', ')']),
            (None, None) => return None,
        };
        // The rest of the name and its closing pair, when nothing else opens or closes
        // between the cursor and the pair.
        let tail = self
            .after
            .find(close)
            .filter(|end| !self.after[..*end].contains(brackets))
            .map_or(0, |end| end + close.len());
        Some(self.context(kind, &self.before[at + 2..], tail))
    }

    /// A `#` that starts a tag, and the text after it.
    fn tag(&self) -> Option<CompletionContext> {
        let at = self.before.rfind('#')?;
        let typed = &self.before[at + 1..];
        let starts_word = self.before[..at]
            .chars()
            .next_back()
            .is_none_or(|before| before.is_whitespace() || before == '(');
        if !starts_word || typed.contains(char::is_whitespace) || typed.starts_with('[') {
            return None;
        }
        let tail = self
            .after
            .find(|c: char| !(c.is_alphanumeric() || matches!(c, '_' | '-' | '/')))
            .unwrap_or(self.after.len());
        Some(self.context(CompletionKind::Tag, typed, tail))
    }

    /// A property key or value. `above` is the text before the cursor's line.
    fn property(&self, above: &str) -> Option<CompletionContext> {
        let indent = self.before.len() - self.before.trim_start().len();
        let body = &self.before[indent..];
        let (bullet, body) = match body.strip_prefix("- ") {
            Some(rest) => (true, rest),
            None => (false, body),
        };
        if let Some((key, rest)) = body.split_once("::") {
            if !valid_key(key) || !(rest.is_empty() || rest.starts_with([' ', '\t'])) {
                return None;
            }
            return Some(self.value(key, rest));
        }
        // A key: nothing but the key so far on a line where Logseq reads properties, and
        // nothing after the cursor but the rest of the key and its `::`.
        if bullet || !(body.is_empty() || valid_key(body)) || !property_line_follows(above) {
            return None;
        }
        let rest_of_key = self
            .after
            .find(|c: char| c.is_whitespace() || c == ':')
            .unwrap_or(self.after.len());
        let after_key = &self.after[rest_of_key..];
        let tail = match after_key.strip_prefix("::") {
            Some(value) => self.after.len() - value.trim_start_matches([' ', '\t']).len(),
            None if after_key.is_empty() => rest_of_key,
            None => return None,
        };
        Some(self.context(CompletionKind::PropertyKey, body, tail))
    }

    /// The value after `key::`, where `rest` is the text between the `::` and the cursor.
    fn value(&self, key: &str, rest: &str) -> CompletionContext {
        if !is_linkable_property(key) {
            let typed = rest.trim_start();
            let mut context = self.context(
                CompletionKind::PropertyValue {
                    key: key.to_lowercase(),
                },
                typed,
                self.after.trim_end().len(),
            );
            context.needs_space = rest.is_empty();
            return context;
        }
        // One name of a list separated by commas.
        let commas = [',', '，'];
        let typed = rest.rsplit(commas).next().unwrap_or(rest).trim_start();
        let tail = self.after.find(commas).unwrap_or(self.after.len());
        let mut context = self.context(
            CompletionKind::PropertyPage,
            typed,
            self.after[..tail].trim_end().len(),
        );
        context.needs_space = rest.is_empty();
        context
    }
}

fn valid_key(key: &str) -> bool {
    !key.contains(char::is_whitespace) && !key.contains(':') && valid_property_name(key)
}

/// Whether text in the form `key:: value` is a property line.
fn is_property_line(line: &str) -> bool {
    let line = line.trim_start();
    let line = line.strip_prefix("- ").unwrap_or(line);
    line.split_once("::").is_some_and(|(key, rest)| {
        valid_key(key) && (rest.is_empty() || rest.starts_with([' ', '\t']))
    })
}

/// Whether Logseq reads properties on the line after `above`: every line back to the nearest
/// bullet, or to the top of the page, is blank or a property line.
fn property_line_follows(above: &str) -> bool {
    for line in above.lines().rev() {
        let content = line.trim();
        if content == "-" || content.starts_with("- ") {
            return true;
        }
        if !(content.is_empty() || is_property_line(content)) {
            return false;
        }
    }
    true
}

/// A page name that a link, a tag or a `tags::` value can write.
struct Name {
    kind: CandidateKind,
    name: String,
    /// The name in lower case, which scores compare.
    lower: String,
    page: Option<String>,
    filter: Option<String>,
}

/// Compare two scores, the higher first.
fn higher(a: f64, b: f64) -> Ordering {
    b.partial_cmp(&a).unwrap_or(Ordering::Equal)
}

/// How well a lower-case name matches the typed text: 0 for the same text, 1 for a name
/// that starts with it and 2 for another fuzzy match, with the fuzzy score. `None` when the
/// name does not match. Every name matches empty text equally.
fn matched(query: &str, name: &str) -> Option<(u8, f64)> {
    if query.is_empty() {
        return Some((2, 1.0));
    }
    let score = fuzzy_score(query, name);
    let bucket = if name == query {
        0
    } else if name.starts_with(query) {
        1
    } else {
        2
    };
    (score > 0.0).then_some((bucket, score))
}

/// Whether Logseq reads `#name` as one tag that names `name`.
fn bare_tag(name: &str) -> bool {
    match lsdoc::inline(&format!("#{name}"), "markdown").as_slice() {
        [Inline::Tag { children, .. }] => tag_page_name(children) == name,
        _ => false,
    }
}

impl GardenIndex {
    /// The candidates for a context, best first, and at most `limit` of them.
    pub fn completions(&self, context: &CompletionContext, limit: usize) -> Completions {
        let (mut candidates, pages) = match &context.kind {
            CompletionKind::PageLink | CompletionKind::Tag | CompletionKind::PropertyPage => {
                (self.page_candidates(context, limit), true)
            }
            CompletionKind::BlockReference => (self.block_candidates(&context.typed), false),
            CompletionKind::PropertyKey => (self.key_candidates(&context.typed), false),
            CompletionKind::PropertyValue { key } => (self.value_candidates(key, context), false),
        };
        let partial = pages || candidates.len() > limit;
        candidates.truncate(limit);
        Completions {
            candidates,
            partial,
        }
    }

    /// Every title, alias and namespace parent, one per name.
    fn page_names(&self) -> Vec<Name> {
        let mut names = Vec::new();
        let mut seen = BTreeSet::new();
        for document in self.documents() {
            let Some(title) = &document.title else {
                continue;
            };
            let lower = title.to_lowercase();
            if seen.insert(lower.clone()) {
                names.push(Name {
                    kind: match document.journal {
                        Some(_) => CandidateKind::Journal,
                        None => CandidateKind::Page,
                    },
                    name: title.clone(),
                    lower,
                    page: None,
                    filter: None,
                });
            }
        }
        // A title takes a name before an alias or a namespace parent does.
        let titled = names.len();
        for document in self.documents() {
            let Some(title) = &document.title else {
                continue;
            };
            if !document.aliases.is_empty() {
                for alias in page_alias_spellings(&document.document, title) {
                    let lower = alias.to_lowercase();
                    if !seen.contains(&lower) {
                        names.push(Name {
                            kind: CandidateKind::Alias,
                            name: alias,
                            lower,
                            page: Some(title.clone()),
                            filter: None,
                        });
                    }
                }
            }
        }
        seen.extend(names[titled..].iter().map(|name| name.lower.clone()));
        for at in 0..titled {
            let title = names[at].name.clone();
            let mut parent = title.as_str();
            while let Some((before, _)) = parent.rsplit_once('/') {
                parent = before;
                let lower = parent.to_lowercase();
                if parent.trim().is_empty() || !seen.insert(lower.clone()) {
                    continue;
                }
                names.push(Name {
                    kind: CandidateKind::Namespace,
                    name: parent.to_owned(),
                    lower,
                    page: None,
                    filter: None,
                });
            }
        }
        names
    }

    /// Page names for a link, a tag or a `tags::` value. Only the best `limit` become
    /// candidates, because a garden holds thousands of names.
    fn page_candidates(&self, context: &CompletionContext, limit: usize) -> Vec<Candidate> {
        let typed = context.typed.as_str();
        let query = typed.to_lowercase();
        let mut names = self.page_names();

        // The journal whose date the typed text spells, in the garden's title format.
        let dated = (!has_outer_space(typed))
            .then(|| parse_journal_reference(typed, &self.journal_formats))
            .flatten()
            .map(|date| self.journal_formats.title.format(date));
        if let Some(title) = &dated {
            let lower = title.to_lowercase();
            match names.iter_mut().find(|name| name.lower == lower) {
                Some(name) => name.filter = Some(typed.to_owned()),
                None => names.push(Name {
                    kind: CandidateKind::Journal,
                    name: title.clone(),
                    lower,
                    page: None,
                    filter: Some(typed.to_owned()),
                }),
            }
        }
        // The namespace before the last `/`, with that slash, and the text after it.
        let level = query
            .rsplit_once('/')
            .map(|(parent, rest)| (format!("{}/", lookup_key(parent)), rest));

        // Dated first, then the pages one level beneath the typed namespace, then the rest.
        // Within each group an exact name comes first, then a name that starts with the
        // typed text, then the other fuzzy matches, as in the published search.
        let mut ranked: Vec<(u8, u8, f64, usize)> = Vec::new();
        for (at, name) in names.iter().enumerate() {
            if name.filter.is_some() {
                ranked.push((2, 0, 0.0, at));
                continue;
            }
            let child = level.as_ref().and_then(|(parent, rest)| {
                let segment = name.lower.strip_prefix(parent.as_str())?;
                if segment.is_empty() || segment.contains('/') {
                    return None;
                }
                matched(rest, segment)
            });
            if let Some((bucket, score)) = child {
                ranked.push((1, bucket, score, at));
            } else if let Some((bucket, score)) = matched(&query, &name.lower) {
                ranked.push((0, bucket, score, at));
            }
        }
        ranked.sort_by(|a, b| {
            b.0.cmp(&a.0)
                .then_with(|| a.1.cmp(&b.1))
                .then_with(|| higher(a.2, b.2))
                .then_with(|| names[a.3].lower.cmp(&names[b.3].lower))
                .then_with(|| names[a.3].kind.cmp(&names[b.3].kind))
                .then_with(|| names[a.3].page.cmp(&names[b.3].page))
        });
        // One more than the limit, so the caller sees that the list was cut.
        ranked.truncate(limit.saturating_add(1));

        let space = if context.needs_space { " " } else { "" };
        ranked
            .into_iter()
            .map(|(_, _, _, at)| {
                let name = &names[at];
                let insert = match context.kind {
                    CompletionKind::Tag if bare_tag(&name.name) => name.name.clone(),
                    CompletionKind::Tag => format!("[[{}]]", name.name),
                    CompletionKind::PropertyPage if name.name.contains([',', '，']) => {
                        format!("{space}[[{}]]", name.name)
                    }
                    CompletionKind::PropertyPage => format!("{space}{}", name.name),
                    _ => format!("{}]]", name.name),
                };
                Candidate {
                    kind: name.kind,
                    label: name.name.clone(),
                    insert,
                    filter: name.filter.clone().unwrap_or_else(|| name.name.clone()),
                    page: name.page.clone(),
                    uses: None,
                }
            })
            .collect()
    }

    /// Every block that writes an id and whose text holds each typed word.
    fn block_candidates(&self, typed: &str) -> Vec<Candidate> {
        let query = normalize(typed);
        let mut found = Vec::new();
        for id in self.block_ids.keys() {
            let Some(location) = self.block(id) else {
                continue;
            };
            let text = block_text(location.document.source(), location.site.block_start);
            let searched = normalize(&text);
            if !query.split(' ').all(|word| searched.contains(word)) {
                continue;
            }
            let score = if query.is_empty() {
                0.0
            } else {
                fuzzy_score(&query, &searched)
            };
            let page = match &location.document.title {
                Some(title) => title.clone(),
                None => location.document.path().to_string_lossy().into_owned(),
            };
            found.push((score, page, text, id));
        }
        found.sort_by(|a, b| {
            higher(a.0, b.0).then_with(|| (&a.1, &a.2, a.3).cmp(&(&b.1, &b.2, b.3)))
        });
        found
            .into_iter()
            .map(|(_, page, text, id)| {
                let label = match text.char_indices().nth(BLOCK_LABEL_CHARS) {
                    Some((end, _)) => format!("{}…", &text[..end]),
                    None if text.is_empty() => id.clone(),
                    None => text.clone(),
                };
                Candidate {
                    kind: CandidateKind::Block,
                    label,
                    insert: format!("{id}))"),
                    filter: text,
                    page: Some(page),
                    uses: None,
                }
            })
            .collect()
    }

    /// The property keys the garden writes, without the hidden ones.
    fn key_candidates(&self, typed: &str) -> Vec<Candidate> {
        let query = typed.to_lowercase();
        let hidden = &self.settings.hidden_properties;
        let keys = self.property_keys().filter(|(key, count)| {
            // A key written once and equal to the typed text is the line being typed.
            !(is_hidden_property(key)
                || hidden.iter().any(|hidden| hidden == key)
                || (*key == query && *count == 1))
        });
        counted(keys, &query, CandidateKind::PropertyKey, |key| {
            format!("{key}:: ")
        })
    }

    /// The values the garden writes for a key.
    fn value_candidates(&self, key: &str, context: &CompletionContext) -> Vec<Candidate> {
        let typed = context.typed.trim();
        let space = if context.needs_space { " " } else { "" };
        let values = self
            .property_values(key)
            .filter(|(value, count)| !(value.is_empty() || (*value == typed && *count == 1)));
        counted(
            values,
            &typed.to_lowercase(),
            CandidateKind::PropertyValue,
            |value| format!("{space}{value}"),
        )
    }
}

/// Rank counted texts as page names are ranked, then by count.
fn counted<'a>(
    texts: impl Iterator<Item = (&'a str, usize)>,
    query: &str,
    kind: CandidateKind,
    insert: impl Fn(&str) -> String,
) -> Vec<Candidate> {
    let mut found: Vec<(u8, f64, usize, &str)> = texts
        .filter_map(|(text, count)| {
            let (bucket, score) = matched(query, &text.to_lowercase())?;
            Some((bucket, score, count, text))
        })
        .collect();
    found.sort_by(|a, b| {
        a.0.cmp(&b.0)
            .then_with(|| higher(a.1, b.1))
            .then_with(|| b.2.cmp(&a.2))
            .then_with(|| a.3.cmp(b.3))
    });
    found
        .into_iter()
        .map(|(_, _, count, text)| Candidate {
            kind,
            label: text.to_owned(),
            insert: insert(text),
            filter: text.to_owned(),
            page: None,
            uses: Some(count),
        })
        .collect()
}

/// The first line of the block that starts at `block_start`, without its indentation and
/// bullet.
fn block_text(source: &str, block_start: usize) -> String {
    let line = source
        .get(block_start..)
        .and_then(|rest| rest.lines().next())
        .unwrap_or_default()
        .trim();
    let line = line.strip_prefix('-').map_or(line, str::trim_start);
    line.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};

    use super::*;

    const APPLE: &str = "66f1a2b0-0000-4000-8000-000000000001";
    const PEAR: &str = "66f1a2b0-0000-4000-8000-000000000003";

    static NEXT: AtomicUsize = AtomicUsize::new(0);

    struct Garden(PathBuf);

    impl Drop for Garden {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn garden(extra: &[(&str, &str)]) -> (Garden, GardenIndex) {
        let root = std::env::temp_dir().join(format!(
            "lsg-completion-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, AtomicOrdering::Relaxed)
        ));
        let apple = format!(
            "alias:: Malus, [[Pomme]]\ntags:: [[Fruit]], sweet\n\n- An apple is red or green.\n  id:: {APPLE}\n  colour:: red\n  card-last-score:: 5\n  secret:: yes\n"
        );
        let pear = format!("alias:: Pyrus\n\n- A pear ripens after picking.\n  id:: {PEAR}\n");
        let mut files = vec![
            (
                "logseq/config.edn",
                "{:file/name-format :triple-lowbar\n :journal/page-title-format \"MMM do, yyyy\"\n :block-hidden-properties #{:secret}}\n",
            ),
            ("pages/Fruit.md", "type:: [[Category]]\n\n- Fruit\n"),
            ("pages/Fruit___Apple.md", apple.as_str()),
            ("pages/Fruit___Pear.md", pear.as_str()),
            ("pages/Tools___Editor___Neovim.md", "- An editor\n"),
            ("pages/multi word.md", "- Two words\n"),
            (
                "pages/Recipes.md",
                "type:: [[Category]]\n\n- Cake\n  type:: recipe\n- Tart\n  type:: recipe\n- Jam\n  type:: preserve\n",
            ),
            ("journals/2026_01_02.md", "- Baked\n"),
        ];
        files.extend_from_slice(extra);
        for (path, text) in files {
            let path = root.join(path);
            fs::create_dir_all(path.parent().unwrap()).unwrap();
            fs::write(path, text).unwrap();
        }
        let index = GardenIndex::load(&root).unwrap();
        (Garden(root), index)
    }

    /// The context with the cursor at `|` in `text`.
    fn at(text: &str) -> Option<CompletionContext> {
        let offset = text.find('|').unwrap();
        completion_context(&text.replace('|', ""), offset)
    }

    /// The kind, typed text and replaced text with the cursor at `|`.
    fn read(text: &str) -> (CompletionKind, String, String) {
        let context = at(text).unwrap_or_else(|| panic!("no context in {text:?}"));
        let replaced = context.replace.text(&text.replace('|', "")).to_owned();
        (context.kind, context.typed, replaced)
    }

    fn entry(
        kind: CompletionKind,
        typed: &str,
        replaced: &str,
    ) -> (CompletionKind, String, String) {
        (kind, typed.to_owned(), replaced.to_owned())
    }

    fn labels(completions: &Completions) -> Vec<&str> {
        completions
            .candidates
            .iter()
            .map(|candidate| candidate.label.as_str())
            .collect()
    }

    fn complete(index: &GardenIndex, text: &str) -> Completions {
        index.completions(&at(text).unwrap(), 100)
    }

    fn candidate<'a>(completions: &'a Completions, label: &str) -> &'a Candidate {
        completions
            .candidates
            .iter()
            .find(|candidate| candidate.label == label)
            .unwrap_or_else(|| panic!("no {label} in {:?}", labels(completions)))
    }

    #[test]
    fn reads_links_and_block_references_from_the_line() {
        use CompletionKind::*;
        assert_eq!(read("- See [[Fru|"), entry(PageLink, "Fru", "Fru"));
        assert_eq!(
            read("- See [[Fru|]] today"),
            entry(PageLink, "Fru", "Fru]]")
        );
        assert_eq!(
            read("- See [[Fru|it]] today"),
            entry(PageLink, "Fru", "Fruit]]")
        );
        assert_eq!(read("- [[A]] and [[|"), entry(PageLink, "", ""));
        // The closing pair of a later link stays.
        assert_eq!(
            read("- [[Fru| and [[Other]]"),
            entry(PageLink, "Fru", "Fru")
        );
        assert_eq!(
            read("- {{embed [[Fru|]]}}"),
            entry(PageLink, "Fru", "Fru]]")
        );
        assert_eq!(
            read("- #[[multi w|]]"),
            entry(PageLink, "multi w", "multi w]]")
        );
        assert_eq!(
            read("- é ((pear ri|))"),
            entry(BlockReference, "pear ri", "pear ri))")
        );
        assert_eq!(read("- {{embed ((|))}}"), entry(BlockReference, "", "))"));
        assert_eq!(read("- [[A]] ((x|"), entry(BlockReference, "x", "x"));
        assert_eq!(read("- ((a)) [[x|"), entry(PageLink, "x", "x"));
        assert_eq!(at("- [[Fruit]] and|"), None);
        assert_eq!(at("- ((done)) and|"), None);
    }

    #[test]
    fn reads_tags() {
        use CompletionKind::*;
        assert_eq!(read("- Cake #swe|"), entry(Tag, "swe", "swe"));
        assert_eq!(read("- Cake #swe|et and"), entry(Tag, "swe", "sweet"));
        assert_eq!(read("#|"), entry(Tag, "", ""));
        assert_eq!(read("- (#a/b|)"), entry(Tag, "a/b", "a/b"));
        // A heading, a fragment of an address and a `#` inside a word.
        assert_eq!(at("- # Title|"), None);
        assert_eq!(at("- https://example.com/#top|"), None);
        assert_eq!(at("- C#x|"), None);
        assert_eq!(at("- #[|"), None);
    }

    #[test]
    fn reads_property_keys_and_values() {
        use CompletionKind::*;
        let value = |key: &str| PropertyValue { key: key.into() };
        assert_eq!(read("ty|"), entry(PropertyKey, "ty", "ty"));
        assert_eq!(read("alias:: A\n|"), entry(PropertyKey, "", ""));
        assert_eq!(read("- Block\n  ty|"), entry(PropertyKey, "ty", "ty"));
        assert_eq!(
            read("- Block\n  id:: x\n  ty|pe:: recipe"),
            entry(PropertyKey, "ty", "type:: ")
        );
        // The first line of a bullet, a line of prose and a line after prose.
        assert_eq!(at("- ty|"), None);
        assert_eq!(at("- Block\n  some wor|"), None);
        assert_eq!(at("- Block\n  more text\n  ty|"), None);
        assert_eq!(at("- Block\n  wor|d more"), None);
        assert_eq!(at("- Block\n  type:|"), None);

        assert_eq!(
            read("- Block\n  Type:: rec|"),
            entry(value("type"), "rec", "rec")
        );
        assert_eq!(
            read("- type:: rec|ipe  "),
            entry(value("type"), "rec", "recipe")
        );
        let context = at("type::|").unwrap();
        assert_eq!((context.typed.as_str(), context.needs_space), ("", true));
        assert_eq!(at("see http::|"), None);

        assert_eq!(
            read("tags:: sweet, Fru|"),
            entry(PropertyPage, "Fru", "Fru")
        );
        assert_eq!(
            read("alias:: Fr|u, Other"),
            entry(PropertyPage, "Fr", "Fru")
        );
        assert_eq!(read("tags:: [[Fru|]], b"), entry(PageLink, "Fru", "Fru]]"));
        assert_eq!(read("tags:: a, #sw|"), entry(Tag, "sw", "sw"));
    }

    #[test]
    fn offers_titles_aliases_and_namespace_parents() {
        let (_garden, index) = garden(&[]);
        let found = complete(&index, "- [[fru|");
        // Logseq's score puts the shorter of two equal matches first.
        assert_eq!(labels(&found), ["Fruit", "Fruit/Pear", "Fruit/Apple"]);
        assert!(found.partial);
        let apple = candidate(&found, "Fruit/Apple");
        assert_eq!(apple.kind, CandidateKind::Page);
        assert_eq!(apple.insert, "Fruit/Apple]]");
        assert_eq!(apple.filter, "Fruit/Apple");

        let found = complete(&index, "- [[pom|");
        let alias = candidate(&found, "Pomme");
        assert_eq!(alias.kind, CandidateKind::Alias);
        assert_eq!(alias.page.as_deref(), Some("Fruit/Apple"));
        assert_eq!(alias.insert, "Pomme]]");

        // `Tools` and `Tools/Editor` have no file.
        let found = complete(&index, "- [[tool|");
        assert_eq!(
            labels(&found),
            ["Tools", "Tools/Editor", "Tools/Editor/Neovim"]
        );
        assert_eq!(candidate(&found, "Tools").kind, CandidateKind::Namespace);

        // Nothing typed: every name, in name order.
        let found = complete(&index, "- [[|");
        assert_eq!(found.candidates.len(), 12, "{:?}", labels(&found));
        assert_eq!(labels(&found)[..2], ["Fruit", "Fruit/Apple"]);
        assert!(complete(&index, "- [[zzzz|").candidates.is_empty());
    }

    #[test]
    fn puts_an_exact_name_and_then_a_prefix_before_other_matches() {
        let (_garden, index) = garden(&[
            ("pages/Blog.md", "- b\n"),
            ("pages/Log.md", "- l\n"),
            ("pages/Logbook.md", "- l\n"),
        ]);
        let found = complete(&index, "- [[log|");
        assert_eq!(labels(&found), ["Log", "Logbook", "Blog"]);
        let found = complete(&index, "- [[lo|");
        assert_eq!(labels(&found)[..3], ["Log", "Logbook", "Blog"]);
        // A key that starts with the typed text comes before one that holds it.
        let found = complete(&index, "- Block\n  a|");
        assert_eq!(labels(&found), ["alias", "tags"]);
    }

    #[test]
    fn offers_one_namespace_level_first() {
        let (_garden, index) = garden(&[]);
        let found = complete(&index, "- [[Fruit/|");
        // The level beneath `Fruit` in name order, then the other matches.
        assert_eq!(labels(&found), ["Fruit/Apple", "Fruit/Pear", "Fruit"]);
        let found = complete(&index, "- [[tools/|");
        assert_eq!(
            labels(&found),
            ["Tools/Editor", "Tools/Editor/Neovim", "Tools"]
        );
        let found = complete(&index, "- [[fruit/pe|");
        assert_eq!(labels(&found)[0], "Fruit/Pear");
    }

    #[test]
    fn offers_a_journal_for_a_date_in_any_format() {
        let (_garden, index) = garden(&[]);
        let found = complete(&index, "- [[jan|");
        assert_eq!(labels(&found), ["Jan 2nd, 2026"]);
        assert_eq!(found.candidates[0].kind, CandidateKind::Journal);

        // A journal with a file, and one without.
        for (typed, title) in [
            ("2026-01-02", "Jan 2nd, 2026"),
            ("2026_01_04", "Jan 4th, 2026"),
        ] {
            let found = complete(&index, &format!("- [[{typed}|"));
            let journal = &found.candidates[0];
            assert_eq!(journal.label, title);
            assert_eq!(journal.kind, CandidateKind::Journal);
            assert_eq!(journal.insert, format!("{title}]]"));
            assert_eq!(journal.filter, typed);
        }
    }

    #[test]
    fn writes_a_tag_as_logseq_reads_it() {
        let (_garden, index) = garden(&[]);
        let found = complete(&index, "- #mul|");
        assert_eq!(candidate(&found, "multi word").insert, "[[multi word]]");
        let found = complete(&index, "- #fru|");
        assert_eq!(candidate(&found, "Fruit/Apple").insert, "Fruit/Apple");
        let found = complete(&index, "- #jan|");
        assert_eq!(
            candidate(&found, "Jan 2nd, 2026").insert,
            "[[Jan 2nd, 2026]]"
        );
    }

    #[test]
    fn offers_blocks_by_their_text() {
        let (_garden, index) = garden(&[]);
        let found = complete(&index, "- ((|");
        assert_eq!(
            labels(&found),
            ["An apple is red or green.", "A pear ripens after picking."]
        );
        assert!(!found.partial);
        let found = complete(&index, "- ((ripens PEAR|))");
        assert_eq!(labels(&found), ["A pear ripens after picking."]);
        let pear = &found.candidates[0];
        assert_eq!(pear.insert, format!("{PEAR}))"));
        assert_eq!(pear.page.as_deref(), Some("Fruit/Pear"));
        assert_eq!(pear.filter, "A pear ripens after picking.");
        assert!(complete(&index, "- ((plum|").candidates.is_empty());
    }

    #[test]
    fn offers_property_keys_without_hidden_ones() {
        let (_garden, index) = garden(&[]);
        let found = complete(&index, "- Block\n  |");
        // By count, then by name. `card-last-score` and `secret` are hidden.
        assert_eq!(labels(&found), ["type", "alias", "id", "colour", "tags"]);
        let found = complete(&index, "- Block\n  ty|");
        assert_eq!(labels(&found), ["type"]);
        assert_eq!(found.candidates[0].insert, "type:: ");
        assert_eq!(found.candidates[0].uses, Some(5));
        assert!(complete(&index, "- Block\n  sec|").candidates.is_empty());
    }

    #[test]
    fn offers_the_values_of_a_key() {
        let (_garden, index) = garden(&[]);
        let found = complete(&index, "- Block\n  type:: |");
        assert_eq!(labels(&found), ["[[Category]]", "recipe", "preserve"]);
        let found = complete(&index, "- Block\n  type:: rec|");
        assert_eq!(labels(&found), ["recipe"]);
        assert_eq!(found.candidates[0].insert, "recipe");
        let found = complete(&index, "- Block\n  type::|");
        assert_eq!(candidate(&found, "recipe").insert, " recipe");
        assert!(
            complete(&index, "- Block\n  unknown:: |")
                .candidates
                .is_empty()
        );

        // `tags::` takes page names.
        let found = complete(&index, "tags:: sweet, fru|");
        assert_eq!(labels(&found), ["Fruit", "Fruit/Pear", "Fruit/Apple"]);
        assert_eq!(candidate(&found, "Fruit").insert, "Fruit");
    }

    #[test]
    fn leaves_out_the_text_being_typed() {
        let text = "- Block\n  type:: rec\n";
        let (_garden, mut index) = garden(&[("pages/Draft.md", text)]);
        index.set_buffer("pages/Draft.md".as_ref(), text.to_owned());
        let context = completion_context(text, text.find("rec").unwrap() + 3).unwrap();
        assert_eq!(labels(&index.completions(&context, 100)), ["recipe"]);
    }

    #[test]
    fn cuts_the_list_at_the_limit() {
        let pages: Vec<(String, String)> = (0..12)
            .map(|n| {
                (
                    format!("pages/Note {n:02}.md"),
                    format!("- n\n  k{n:02}:: v\n"),
                )
            })
            .collect();
        let extra: Vec<(&str, &str)> = pages
            .iter()
            .map(|(path, text)| (path.as_str(), text.as_str()))
            .collect();
        let (_garden, index) = garden(&extra);
        let context = at("- [[note|").unwrap();
        let found = index.completions(&context, 5);
        assert_eq!(
            labels(&found),
            ["Note 00", "Note 01", "Note 02", "Note 03", "Note 04"]
        );
        assert!(found.partial);
        let keys = at("- Block\n  k|").unwrap();
        let found = index.completions(&keys, 5);
        assert_eq!(found.candidates.len(), 5);
        assert!(found.partial);
        assert!(!index.completions(&keys, 100).partial);
    }
}
