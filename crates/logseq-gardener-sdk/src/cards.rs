//! Flashcards as Logseq defines them: blocks that reference the page `card`, decks chosen
//! by `{{cards QUERY}}` with the shared query evaluator, and which cards are due at a
//! given time. The publisher, the CLI and the language server answer "which cards does
//! this deck hold" with the same operations.
//!
//! ```
//! use logseq_gardener_sdk::cards::{Schedule, deck_query};
//!
//! assert_eq!(deck_query(""), None);
//! assert_eq!(deck_query("yazi").as_deref(), Some("[[yazi]]"));
//! let schedule = Schedule::from_properties(&[
//!     ("card-repeats".into(), "2".into()),
//!     ("card-next-schedule".into(), "2026-11-01T00:00:00.000Z".into()),
//! ]);
//! assert!(!schedule.is_due(schedule.next_seconds().unwrap() - 1));
//! assert!(schedule.is_due(schedule.next_seconds().unwrap() + 1));
//! ```

use std::collections::BTreeSet;

use crate::ast::{Block, Inline, ListItem, Url};
use crate::garden::GardenDocument;
use crate::inline_references::tag_page_name;
use crate::journals::{JournalDate, JournalFormats};
use crate::page_names::lookup_key;
use crate::query::{EvaluateOptions, QueryError, QueryGraph, QueryResults, parse_simple};
use crate::references::{BlockId, ReferenceIndex};

const MAX_DEPTH: usize = 64;

/// The page every card references.
pub const CARD_PAGE: &str = "card";

/// The properties Logseq writes under a card when it is scored. They are built-in and
/// hidden: no page, reference, embed, search record or review shows them.
pub const HIDDEN_PROPERTIES: [&str; 6] = [
    "card-last-interval",
    "card-repeats",
    "card-ease-factor",
    "card-next-schedule",
    "card-last-reviewed",
    "card-last-score",
];

/// Whether `key`, lower-cased with `_` written as `-`, is a card scheduling property.
pub fn is_hidden_property(key: &str) -> bool {
    let key = key.trim().to_lowercase().replace('_', "-");
    HIDDEN_PROPERTIES.contains(&key.as_str())
}

/// Whether a written page name names the `card` page, in any case.
pub fn is_card_name(name: &str) -> bool {
    lookup_key(name) == CARD_PAGE
}

/// Whether a block's inline content references `card`, as `#card`, `#[[card]]`,
/// `[[card]]` or `[[Card]]`. Code is not a reference.
pub fn mentions_card(inline: &[Inline]) -> bool {
    mentions(inline, 0)
}

fn mentions(nodes: &[Inline], depth: usize) -> bool {
    if depth > MAX_DEPTH {
        return false;
    }
    nodes.iter().any(|node| match node {
        Inline::Tag { children, .. } => is_card_name(&tag_page_name(children)),
        Inline::Link {
            url: Url::PageRef { v },
            ..
        } => is_card_name(v),
        Inline::Emphasis { children, .. }
        | Inline::Subscript { children, .. }
        | Inline::Superscript { children, .. } => mentions(children, depth + 1),
        _ => false,
    })
}

/// A card's scheduling state from its six `card-*` properties.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Schedule {
    pub repeats: Option<i64>,
    /// `card-next-schedule` as written, an ISO 8601 UTC time.
    pub next: Option<String>,
    pub last_interval: Option<f64>,
    pub ease_factor: Option<f64>,
    pub last_reviewed: Option<String>,
    pub last_score: Option<i64>,
}

impl Schedule {
    /// Reads the schedule from a block's properties; `nil` and unparsable values are absent.
    pub fn from_properties(properties: &[(String, String)]) -> Self {
        let mut schedule = Self::default();
        for (key, value) in properties {
            let value = value.trim();
            if value.is_empty() || value == "nil" {
                continue;
            }
            match key.to_lowercase().replace('_', "-").as_str() {
                "card-repeats" => schedule.repeats = value.parse().ok(),
                "card-next-schedule" => schedule.next = Some(value.to_owned()),
                "card-last-interval" => schedule.last_interval = value.parse().ok(),
                "card-ease-factor" => schedule.ease_factor = value.parse().ok(),
                "card-last-reviewed" => schedule.last_reviewed = Some(value.to_owned()),
                "card-last-score" => schedule.last_score = value.parse().ok(),
                _ => {}
            }
        }
        schedule
    }

    /// `card-next-schedule` as Unix seconds, when it parses.
    pub fn next_seconds(&self) -> Option<i64> {
        self.next.as_deref().and_then(parse_utc)
    }

    /// Logseq's due rule: no repeats or fewer than 1, no schedule, or a schedule before
    /// `now` (Unix seconds).
    pub fn is_due(&self, now: i64) -> bool {
        self.repeats.is_none_or(|repeats| repeats < 1)
            || self.next_seconds().is_none_or(|next| next < now)
    }
}

/// Seconds since the Unix epoch of `YYYY-MM-DDTHH:MM:SS…`, read as UTC.
fn parse_utc(text: &str) -> Option<i64> {
    let number = |range: std::ops::Range<usize>| -> Option<i64> {
        let part = text.get(range)?;
        part.bytes()
            .all(|b| b.is_ascii_digit())
            .then(|| part.parse().ok())?
    };
    let date = JournalDate::new(
        i32::try_from(number(0..4)?).ok()?,
        u8::try_from(number(5..7)?).ok()?,
        u8::try_from(number(8..10)?).ok()?,
    )?;
    let (hours, minutes, seconds) = if text.len() >= 19 {
        (number(11..13)?, number(14..16)?, number(17..19)?)
    } else {
        (0, 0, 0)
    };
    Some(date.unix_days() * 86_400 + hours * 3_600 + minutes * 60 + seconds)
}

/// One card: its block and page, and its schedule. The front is the block and the back
/// its children.
#[derive(Debug, Clone, PartialEq)]
pub struct Card<K> {
    pub id: BlockId,
    pub page: K,
    pub schedule: Schedule,
}

/// Every card in the index's visible documents, in index order. `resolve` maps a written
/// page name to its page, for a garden that publishes a `card` page.
pub fn cards<K: Ord + Clone>(
    index: &ReferenceIndex<K>,
    graph: &QueryGraph<'_, K>,
    resolve: &dyn Fn(&str) -> Option<K>,
) -> Vec<Card<K>> {
    let card_page = resolve(CARD_PAGE);
    let mut found = Vec::new();
    for (document, contribution) in index.contributions().iter().enumerate() {
        for (block, refs) in contribution.graph_refs().iter().enumerate() {
            let id = BlockId { document, block };
            if index.block(id).properties {
                continue;
            }
            let direct = card_page
                .as_ref()
                .is_some_and(|page| refs.direct.contains(page));
            if direct || refs.unresolved.iter().any(|name| is_card_name(name)) {
                found.push(Card {
                    id,
                    page: contribution.page.clone(),
                    schedule: Schedule::from_properties(graph.block_properties(id)),
                });
            }
        }
    }
    found
}

/// The query a `{{cards}}` argument stands for: `None` for all cards, a bare name as a
/// page reference, and anything starting with `(` or `[` as written.
pub fn deck_query(argument: &str) -> Option<String> {
    let argument = argument.trim();
    if argument.is_empty() {
        None
    } else if argument.starts_with('(') || argument.starts_with('[') {
        Some(argument.to_owned())
    } else {
        Some(format!("[[{argument}]]"))
    }
}

/// The argument of every `{{cards}}` macro in `documents`, in the order given, each
/// document in outline order. Arguments are joined with `, ` as the parser split them.
pub fn deck_arguments<'d>(documents: impl IntoIterator<Item = &'d GardenDocument>) -> Vec<String> {
    let mut found = Vec::new();
    for document in documents {
        walk_blocks(&document.parsed.blocks, &mut found, 0);
    }
    found
}

fn walk_blocks(blocks: &[Block], found: &mut Vec<String>, depth: usize) {
    if depth > MAX_DEPTH {
        return;
    }
    for block in blocks {
        match block {
            Block::Paragraph { inline, .. }
            | Block::Heading { inline, .. }
            | Block::Bullet { inline, .. } => walk_inlines(inline, found, depth + 1),
            Block::Quote { children, .. } => walk_blocks(children, found, depth + 1),
            Block::List { items, .. } => walk_items(items, found, depth + 1),
            _ => {}
        }
    }
}

fn walk_items(items: &[ListItem], found: &mut Vec<String>, depth: usize) {
    if depth > MAX_DEPTH {
        return;
    }
    for item in items {
        walk_inlines(&item.name, found, depth + 1);
        walk_blocks(&item.content, found, depth + 1);
        walk_items(&item.items, found, depth + 1);
    }
}

fn walk_inlines(nodes: &[Inline], found: &mut Vec<String>, depth: usize) {
    if depth > MAX_DEPTH {
        return;
    }
    for node in nodes {
        match node {
            Inline::Macro { name, args, .. } if name.eq_ignore_ascii_case("cards") => {
                found.push(args.join(", "));
            }
            Inline::Emphasis { children, .. }
            | Inline::Subscript { children, .. }
            | Inline::Superscript { children, .. } => walk_inlines(children, found, depth + 1),
            _ => {}
        }
    }
}

/// A deck's cards: positions in the card list, due cards first in Logseq's order, then
/// the rest in index order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Deck {
    /// Positions in the card list of every card the deck holds, in index order.
    pub cards: Vec<usize>,
    /// Positions of the due cards, sorted by `card-next-schedule` text, unscheduled first.
    pub due: Vec<usize>,
}

impl Deck {
    pub fn total(&self) -> usize {
        self.cards.len()
    }
}

/// Selects a deck: every card for `None`, otherwise the cards that the query
/// `(and <query> [[card]])` returns, with their due order at `now` (Unix seconds).
#[allow(clippy::too_many_arguments)]
pub fn deck<K: Ord + Clone>(
    query: Option<&str>,
    cards: &[Card<K>],
    graph: &QueryGraph<'_, K>,
    today: JournalDate,
    formats: &JournalFormats,
    now: i64,
) -> Result<Deck, QueryError> {
    let chosen: Vec<usize> = match query {
        None => (0..cards.len()).collect(),
        Some(query) => {
            let parsed = parse_simple(&format!("(and {query} [[{CARD_PAGE}]])"), today, formats)?;
            let options = EvaluateOptions {
                own_block: None,
                keep_children: true,
                seed: 0,
            };
            let matched: BTreeSet<BlockId> = match graph.evaluate(&parsed, &options) {
                QueryResults::Blocks(blocks) => blocks.into_iter().collect(),
                QueryResults::Pages(_) => BTreeSet::new(),
            };
            cards
                .iter()
                .enumerate()
                .filter(|(_, card)| matched.contains(&card.id))
                .map(|(position, _)| position)
                .collect()
        }
    };
    Ok(order(chosen, cards, now))
}

fn order<K>(chosen: Vec<usize>, cards: &[Card<K>], now: i64) -> Deck {
    let mut due: Vec<usize> = chosen
        .iter()
        .copied()
        .filter(|&position| cards[position].schedule.is_due(now))
        .collect();
    // A stable sort keeps index order among equal schedules, as Logseq's query order.
    due.sort_by(|&a, &b| cards[a].schedule.next.cmp(&cards[b].schedule.next));
    Deck { cards: chosen, due }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_schedules_and_due_rule() {
        let props = |pairs: &[(&str, &str)]| {
            Schedule::from_properties(
                &pairs
                    .iter()
                    .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
                    .collect::<Vec<_>>(),
            )
        };
        let now = parse_utc("2026-10-27T04:33:00.000Z").unwrap();
        assert!(props(&[]).is_due(now));
        assert!(
            props(&[
                ("card-repeats", "0"),
                ("card-next-schedule", "2027-01-01T00:00:00.000Z")
            ])
            .is_due(now)
        );
        assert!(props(&[("card-repeats", "1"), ("card-next-schedule", "nil")]).is_due(now));
        assert!(
            props(&[
                ("card-repeats", "1"),
                ("card-next-schedule", "2026-10-27T04:32:59.000Z")
            ])
            .is_due(now)
        );
        assert!(
            !props(&[
                ("card-repeats", "1"),
                ("card-next-schedule", "2026-10-28T00:00:00.000Z")
            ])
            .is_due(now)
        );
        let full = props(&[
            ("card-last-interval", "4.1"),
            ("card-ease-factor", "2.6"),
            ("card-last-score", "5"),
            ("card-last-reviewed", "2026-10-01T00:00:00.000Z"),
        ]);
        assert_eq!(full.last_interval, Some(4.1));
        assert_eq!(full.ease_factor, Some(2.6));
        assert_eq!(full.last_score, Some(5));
    }

    #[test]
    fn parses_utc_times() {
        assert_eq!(parse_utc("1970-01-02T00:00:01.000Z"), Some(86_401));
        assert_eq!(parse_utc("1970-01-02"), Some(86_400));
        assert_eq!(parse_utc("not a date"), None);
    }

    #[test]
    fn normalizes_deck_queries() {
        assert_eq!(deck_query("  "), None);
        assert_eq!(deck_query("Zoxide").as_deref(), Some("[[Zoxide]]"));
        assert_eq!(deck_query("[[yazi]]").as_deref(), Some("[[yazi]]"));
        assert_eq!(
            deck_query("(and [[tmux]] (not [[oh-my-tmux]]))").as_deref(),
            Some("(and [[tmux]] (not [[oh-my-tmux]]))")
        );
    }

    #[test]
    fn names_card_in_any_form() {
        assert!(is_card_name("Card"));
        assert!(is_hidden_property("Card_Next_Schedule"));
        assert!(!is_hidden_property("favorite-deck"));
    }

    #[test]
    fn orders_due_cards_unscheduled_first() {
        let card = |next: Option<&str>, repeats: Option<i64>| Card {
            id: BlockId {
                document: 0,
                block: 0,
            },
            page: (),
            schedule: Schedule {
                repeats,
                next: next.map(str::to_owned),
                ..Schedule::default()
            },
        };
        let cards = [
            card(Some("2026-10-02T00:00:00.000Z"), Some(1)),
            card(None, None),
            card(Some("2026-10-01T00:00:00.000Z"), Some(2)),
            card(Some("2027-01-01T00:00:00.000Z"), Some(3)),
        ];
        let now = parse_utc("2026-10-27T00:00:00Z").unwrap();
        let deck = order((0..4).collect(), &cards, now);
        assert_eq!(deck.due, vec![1, 2, 0]);
        assert_eq!(deck.total(), 4);
    }
}
