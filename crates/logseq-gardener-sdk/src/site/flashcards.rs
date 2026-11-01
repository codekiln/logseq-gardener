//! Flashcards: the `{{cards}}` deck widget, cloze, and the files the review dialog loads.
//! The SDK's [`crate::cards`] selects cards and decks and decides which are due; this
//! module writes them. Pages carry each deck's header, counts and first card as static
//! HTML; `lsg-flashcards.js` and its styles load only when a reader opens a review.
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::path::Path;

use serde_json::{Value, json};

use super::references::{Context, References};
use super::render::{QueryBox, escape};
use crate::cards::{Card, Deck, cards, deck, deck_arguments, deck_query};
use crate::journals::{JournalDate, JournalFormats};
use crate::query::QueryGraph;

/// The review script, written once at the site root and loaded on first use.
pub(super) const SCRIPT_ROUTE: &str = "lsg-flashcards.js";
/// The review styles, loaded with the script.
pub(super) const STYLES_ROUTE: &str = "lsg-flashcards.css";
/// `flashcards.js` without comments or indentation.
pub(super) fn script() -> String {
    super::references::minify(include_str!("flashcards.js"))
}
pub(super) const STYLES: &str = include_str!("flashcards.css");
/// Where [`WIDGET_STYLES`] is written; pages that hold a deck link it.
pub(super) const WIDGET_STYLES_ROUTE: &str = "lsg-flashcards-widget.css";
/// Deck widget styles, linked by pages that hold a deck.
pub(super) const WIDGET_STYLES: &str = concat!(
    ".cards-review{padding:12px}.cards-review[data-lsg-deck]{cursor:pointer}",
    ".cards-title{display:flex;align-items:center;justify-content:space-between;gap:8px;min-height:34px;box-sizing:border-box;padding:4px 6px;border-radius:4px;background:var(--lsg-surface)}",
    ".cards-name{display:flex;align-items:center;gap:4px;font-size:14px;font-weight:500;min-width:0;overflow-wrap:anywhere}",
    ".cards-name .lsg-icon{width:20px;height:20px}.cards-name .cards-caret{width:14px;height:14px;opacity:.6}",
    ".cards-tools{display:flex;align-items:center;flex:none}.cards-count{margin-right:8px;font-size:14px;opacity:.6}",
    ".cards-mode{display:inline-flex;align-items:center;justify-content:center;width:36px;height:24px;padding:0;border:0;border-radius:4px;background:none;color:inherit;opacity:.6;cursor:pointer}.cards-mode:hover,.cards-mode[aria-pressed=true]{opacity:1;background:var(--lsg-background)}.cards-mode .lsg-icon{width:16px;height:16px}",
    ".cards-first .breadcrumb{margin:8px 0;font-size:16px}.cards-first li.lsg-card{box-shadow:none;padding-top:0;padding-bottom:0;margin:0}.cards-first .lsg-card-ref{display:none}.cards-first ul{padding-left:1.25rem}",
    ".cards-review-open{height:24px;padding:0 12px;border:0;border-radius:4px;background:rgba(62,99,221,.9);color:#fff;font:inherit;font-size:12px;font-weight:500;cursor:pointer}",
    ".cards-empty{opacity:.6}.cards-empty code{padding:3px 5px;border-radius:4px;background:var(--lsg-surface);font-size:.8em}.cards-none{margin:8px 0 0 8px;font-weight:500}",
);
/// Every deck widget carries this class; pages that contain it link the widget styles.
pub(super) const MARKER: &str = "class=\"cards-review";
/// The deck list the review's `Switch to` menu shows.
pub(super) const DECK_LIST: &str = "flashcards/decks.json";
/// The directories the deck files and card batches go in, parents first.
pub(super) const DIRECTORIES: [&str; 3] = ["flashcards", "flashcards/decks", "flashcards/cards"];
/// Cards per batch file.
const BATCH: usize = 20;
/// The id of the deck of every card.
const ALL: &str = "all";

// Tabler Icons outline paths (MIT); see THIRD_PARTY_NOTICES.md.
pub(super) const INFINITY: &str = "M9.828 9.172a4 4 0 1 0 0 5.656a10 10 0 0 0 2.172 -2.828a10 10 0 0 1 2.172 -2.828a4 4 0 1 1 0 5.656a10 10 0 0 1 -2.172 -2.828a10 10 0 0 0 -2.172 -2.828";
const CARET: &str = "M6 10l6 6l6 -6h-12";
const LETTER_A: &str = "M7 20v-12a4 4 0 0 1 4 -4h2a4 4 0 0 1 4 4v12M7 13l10 0";
const SHUFFLE: &str = "M18 4l3 3l-3 3M18 20l3 -3l-3 -3M3 7h3a5 5 0 0 1 5 5a5 5 0 0 0 5 5h5M21 7h-5a4.978 4.978 0 0 0 -3 1m-4 8a4.984 4.984 0 0 1 -3 1h-3";

fn icon(path: &str, class: &str) -> String {
    format!(
        "<svg class=\"lsg-icon{class}\" viewBox=\"0 0 24 24\" aria-hidden=\"true\"><path d=\"{path}\"/></svg>"
    )
}

/// A deck the garden names in a `{{cards}}` macro, or `All`.
struct Entry {
    id: String,
    label: String,
    deck: Result<Deck, String>,
}

/// Every card and deck in the selected documents, computed once per build.
pub(super) struct Flashcards<'r, 'a> {
    refs: &'r References<'a>,
    context: Context<'r, 'a>,
    cards: Vec<Card<String>>,
    /// Each card's number: its position in `All`'s build-time order.
    numbers: Vec<usize>,
    /// Card positions in number order.
    order: Vec<usize>,
    /// `All` first, then each distinct deck argument in page-title order.
    entries: Vec<Entry>,
    by_argument: BTreeMap<String, usize>,
    diagnostics: RefCell<Vec<String>>,
}

impl<'r, 'a> Flashcards<'r, 'a> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        refs: &'r References<'a>,
        graph: &QueryGraph<'r, String>,
        resolve: &dyn Fn(&str) -> Option<String>,
        today: JournalDate,
        formats: &JournalFormats,
        now: i64,
        context: Context<'r, 'a>,
    ) -> Self {
        let found = cards(refs.index(), graph, resolve);
        let all = deck(None, &found, graph, today, formats, now).unwrap_or_default();
        let mut order = all.due.clone();
        order.extend(all.cards.iter().filter(|p| !all.due.contains(p)));
        let mut numbers = vec![0; found.len()];
        for (number, &position) in order.iter().enumerate() {
            numbers[position] = number;
        }
        let mut entries = vec![Entry {
            id: ALL.to_owned(),
            label: "All".to_owned(),
            deck: Ok(all),
        }];
        let mut by_argument = BTreeMap::new();
        let mut routes: Vec<&String> = refs.sources().keys().collect();
        routes.sort_by_cached_key(|route| (refs.page_title(route).to_lowercase(), *route));
        let documents = routes.iter().map(|route| refs.sources()[*route]);
        for argument in deck_arguments(documents) {
            let argument = argument.trim().to_owned();
            if by_argument.contains_key(&argument) {
                continue;
            }
            // An empty argument is `All`, which the deck menu already lists last.
            let Some(query) = deck_query(&argument) else {
                by_argument.insert(argument, 0);
                continue;
            };
            let selected = deck(Some(&query), &found, graph, today, formats, now)
                .map_err(|error| error.to_string());
            by_argument.insert(argument.clone(), entries.len());
            entries.push(Entry {
                id: format!("d{}", entries.len()),
                label: argument,
                deck: selected,
            });
        }
        Self {
            refs,
            context,
            cards: found,
            numbers,
            order,
            entries,
            by_argument,
            diagnostics: RefCell::new(Vec::new()),
        }
    }

    /// Cards due at the build time, for the sidebar badge.
    pub fn due(&self) -> usize {
        self.entries[0]
            .deck
            .as_ref()
            .map_or(0, |deck| deck.due.len())
    }

    /// Diagnostics from rendering the card files.
    pub fn take_diagnostics(&self) -> Vec<String> {
        std::mem::take(&mut self.diagnostics.borrow_mut())
    }

    fn parts(
        &self,
        position: usize,
        asset: &mut dyn FnMut(&Path, &str) -> Result<String, String>,
        diagnostics: &mut Vec<String>,
    ) -> (String, String, String) {
        self.refs
            .card_parts(self.cards[position].id, &self.context, asset, diagnostics)
    }

    /// The deck header; in the dialog its name is the button that opens `Switch to`.
    fn header(&self, label: &str, count: &str, dialog: bool) -> String {
        let (open, close) = if dialog {
            (
                "<button type=\"button\" class=\"cards-name\" aria-haspopup=\"listbox\" aria-expanded=\"false\">",
                "</button>",
            )
        } else {
            ("<span class=\"cards-name\">", "</span>")
        };
        format!(
            "<div class=\"cards-title\">{open}{}<span class=\"cards-query\">{}</span>{}{close}<span class=\"cards-tools\"><span class=\"cards-count\" title=\"Overdue/Total\">{count}</span><button type=\"button\" class=\"cards-mode\" data-lsg-mode=\"preview\" aria-label=\"Toggle preview mode\">{}</button><button type=\"button\" class=\"cards-mode\" data-lsg-mode=\"random\" aria-label=\"Toggle random mode\">{}</button></span></div>",
            icon(INFINITY, ""),
            escape(label),
            icon(CARET, " cards-caret"),
            icon(LETTER_A, ""),
            icon(SHUFFLE, ""),
        )
    }

    /// The `{{cards}}` widget for a macro whose arguments, joined by `, `, are `text`.
    pub fn widget(
        &self,
        text: &str,
        asset: &mut dyn FnMut(&Path, &str) -> Result<String, String>,
    ) -> QueryBox {
        let argument = text.trim();
        let mut diagnostics = Vec::new();
        let Some(entry) = self.by_argument.get(argument).map(|&i| &self.entries[i]) else {
            return QueryBox {
                html: String::new(),
                diagnostics,
            };
        };
        let deck = match &entry.deck {
            Ok(deck) => deck,
            Err(error) => {
                diagnostics.push(format!("Query Error: {error}"));
                return QueryBox {
                    html: format!(
                        "<div class=\"lsg-query-error\"><h5>Query Error:</h5><pre><code>{}</code></pre></div>",
                        escape(argument)
                    ),
                    diagnostics,
                };
            }
        };
        if deck.total() == 0 {
            return QueryBox {
                html: format!(
                    "<div class=\"cards-review cards-empty\"><code>Cards: {}</code><div class=\"cards-none\">No matched cards</div></div>",
                    escape(argument)
                ),
                diagnostics,
            };
        }
        let first = match deck.due.first() {
            Some(&position) => {
                let (crumb, front, _) = self.parts(position, asset, &mut diagnostics);
                format!("{crumb}<div class=\"cards-front\">{front}</div>")
            }
            None => "<p>Congrats, you've reviewed all the cards for this query, see you next time! 💯</p>".to_owned(),
        };
        QueryBox {
            html: format!(
                "<div class=\"cards-review\" data-lsg-deck=\"{}\">{}<div class=\"cards-first\">{first}</div><button type=\"button\" class=\"cards-review-open\">Review cards</button></div>",
                entry.id,
                self.header(
                    &entry.label,
                    &format!("{}/{}", deck.due.len(), deck.total()),
                    false
                ),
            ),
            diagnostics,
        }
    }

    /// The deck list, a file per deck and the card batches.
    pub fn files(
        &self,
        asset: &mut dyn FnMut(&Path, &str) -> Result<String, String>,
    ) -> Vec<(String, String)> {
        let mut files = Vec::new();
        let menu: Vec<Value> = self.entries[1..]
            .iter()
            .chain(std::iter::once(&self.entries[0]))
            .map(|entry| json!([entry.id, entry.label]))
            .collect();
        let dialog = format!(
            "<div class=\"lsg-review\" role=\"dialog\" aria-modal=\"true\" aria-label=\"Flashcards\" tabindex=\"-1\"><button type=\"button\" class=\"lsg-review-close\" aria-label=\"Close\">×</button><div class=\"cards-review\">{}<div class=\"lsg-review-menu\" hidden><input placeholder=\"Switch to\" aria-label=\"Switch to\"><ul role=\"listbox\"></ul></div><div class=\"lsg-review-card\"></div><div class=\"lsg-review-actions\"></div></div></div>",
            self.header("", "", true)
        );
        files.push((
            DECK_LIST.to_owned(),
            json!({ "decks": menu, "batch": BATCH, "dialog": dialog }).to_string(),
        ));
        for entry in &self.entries {
            let Ok(deck) = &entry.deck else { continue };
            let mut positions = deck.due.clone();
            positions.extend(deck.cards.iter().filter(|p| !deck.due.contains(p)));
            let rows: Vec<Value> = positions
                .iter()
                .map(|&position| {
                    let schedule = &self.cards[position].schedule;
                    json!([
                        self.numbers[position],
                        schedule.repeats,
                        schedule.next,
                        schedule.last_interval,
                        schedule.ease_factor
                    ])
                })
                .collect();
            files.push((
                format!("flashcards/decks/{}.json", entry.id),
                json!({ "label": entry.label, "cards": rows }).to_string(),
            ));
        }
        let mut diagnostics = Vec::new();
        for (batch, numbers) in self.order.chunks(BATCH).enumerate() {
            let mut html = String::new();
            for (offset, &position) in numbers.iter().enumerate() {
                let (crumb, front, back) = self.parts(position, asset, &mut diagnostics);
                html.push_str(&format!(
                    "<article data-card=\"{}\">{crumb}<div class=\"lsg-card-front\">{front}</div><div class=\"lsg-card-back\">{back}</div></article>\n",
                    batch * BATCH + offset
                ));
            }
            files.push((format!("flashcards/cards/{batch}.html"), html));
        }
        self.diagnostics.borrow_mut().extend(diagnostics);
        files
    }
}

/// `{{cloze answer}}` as `[...]`, or `(cue)` for `{{cloze answer \\ cue}}`; activating
/// it reveals `[answer]`.
pub(super) fn cloze(text: &str) -> String {
    let (answer, cue) = match text.split_once("\\\\") {
        Some((answer, cue)) => (answer.trim(), Some(cue.trim())),
        None => (text.trim(), None),
    };
    let shown = cue.map_or_else(|| "[...]".to_owned(), |cue| format!("({cue})"));
    format!(
        "<span class=\"cloze\" data-cloze=\"[{}]\">{}</span>",
        escape(answer),
        escape(&shown)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_cloze_with_and_without_a_cue() {
        assert_eq!(
            cloze("Paris"),
            "<span class=\"cloze\" data-cloze=\"[Paris]\">[...]</span>"
        );
        assert!(cloze("Paris \\\\ capital").contains(">(capital)</span>"));
        assert!(cloze("a<b").contains("data-cloze=\"[a&lt;b]\""));
    }
}
