//! Narrow recognition and finalization of original spoken-dialogue timestamps.
use std::collections::BTreeSet;

use crate::ast::Inline;

pub(super) const SCRIPT_ROUTE: &str = "dialogue.js";
pub(super) const SCRIPT: &str = include_str!("dialogue.js");
pub(super) const STARTUP: &str = include_str!("dialogue-start.js");
pub(super) const CONTROL_ATTRIBUTE: &str = " data-lsg-dialogue-time=\"";

pub(super) struct Candidate {
    pub node: usize,
    pub start: usize,
    pub end: usize,
    pub seconds: u64,
}

pub(super) struct Pending {
    pub start: usize,
    pub end: usize,
    pub seconds: u64,
}

fn speaker(text: &str) -> bool {
    let text = text.trim();
    !text.is_empty()
        && text.len() <= 160
        && text.chars().any(char::is_alphabetic)
        && !text.chars().any(|c| c.is_control() || "[]<>`".contains(c))
        && !text
            .split(|c: char| !c.is_alphanumeric())
            .any(|word| word.eq_ignore_ascii_case("written"))
}

fn seconds(text: &str) -> Option<u64> {
    let parts = text.split(':').collect::<Vec<_>>();
    if !(2..=3).contains(&parts.len()) {
        return None;
    }
    let mut total = 0_u64;
    for (index, part) in parts.iter().enumerate() {
        if part.is_empty() || !part.bytes().all(|c| c.is_ascii_digit()) {
            return None;
        }
        let value = part.parse::<u64>().ok()?;
        if index > 0 && (part.len() != 2 || value >= 60) {
            return None;
        }
        total = total.checked_mul(60)?.checked_add(value)?;
    }
    (total <= 9_007_199_254_740_991).then_some(total)
}

pub(super) fn recognize(nodes: &[Inline]) -> Option<Candidate> {
    let (node, text, dot) = match nodes.first()? {
        Inline::Plain { text, .. } => {
            let (label, _) = text.split_once('·')?;
            if !speaker(label) {
                return None;
            }
            (0, text.as_str(), label.len())
        }
        Inline::Emphasis { emph, children, .. } if matches!(emph.as_str(), "Bold" | "Italic") => {
            let mut label = String::new();
            for child in children {
                let Inline::Plain { text, .. } = child else {
                    return None;
                };
                label.push_str(text);
            }
            if !speaker(&label) {
                return None;
            }
            let Inline::Plain { text, .. } = nodes.get(1)? else {
                return None;
            };
            let trimmed = text.trim_start();
            if !trimmed.starts_with('·') {
                return None;
            }
            (1, text.as_str(), text.len() - trimmed.len())
        }
        _ => return None,
    };
    let after_dot = &text[dot + '·'.len_utf8()..];
    let trimmed = after_dot.trim_start();
    let token_end = trimmed.find(char::is_whitespace)?;
    let token = &trimmed[..token_end];
    let tail = trimmed[token_end..].trim_start();
    if !tail.starts_with(['—', '–', '-']) {
        return None;
    }
    let start = text.len() - trimmed.len();
    Some(Candidate {
        node,
        start,
        end: start + token.len(),
        seconds: seconds(token)?,
    })
}

pub(super) fn finalize(html: &mut String, pending: &[Pending], recordings: &[(usize, String)]) {
    let sources: BTreeSet<_> = recordings.iter().map(|(_, route)| route).collect();
    if pending.is_empty() || sources.len() != 1 {
        return;
    }
    let source = sources.first().unwrap();
    let mut edits = Vec::new();
    for item in pending {
        let label = &html[item.start..item.end];
        let link = format!(
            "<a class=\"dialogue-timestamp\" data-lsg-dialogue-time=\"{}\" href=\"{}#t={}\">{label}</a>",
            item.seconds,
            super::render::escape(source),
            item.seconds
        );
        edits.push((item.start, item.end, link));
    }
    for (position, _) in recordings {
        edits.push((*position, *position, " data-lsg-recording".to_owned()));
    }
    edits.sort_by_key(|edit| std::cmp::Reverse(edit.0));
    for (start, end, replacement) in edits {
        html.replace_range(start..end, &replacement);
    }
}
