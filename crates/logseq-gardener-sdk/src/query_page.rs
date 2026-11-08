//! A published page as an advanced query sees it: three attributes and the page's
//! properties with the types Logseq gives them.
//!
//! Logseq reads a property value with `parse-property`: a value with page references is a
//! set of the names as written; `true` and `false` are booleans; an all-digit value is a
//! number; a value in double quotes is the string inside the quotes; anything else is a
//! string. A key is lower case with `_` read as `-`.
use std::cell::OnceCell;
use std::rc::Rc;

use crate::query_script::Value;

/// One published page, as the caller holds it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PageRecord {
    /// The caller's name for the page, returned with the result (the publisher's route).
    pub key: String,
    /// The page's name as written, for `:block/original-name`.
    pub original_name: String,
    /// The page's leading properties in source order, keys as written.
    pub properties: Vec<(String, String)>,
}

/// A page row: `:block/name` (lower case), `:block/original-name` and
/// `:block/properties`.
#[derive(Debug)]
pub struct Row {
    /// The page's position among the pages the query ran over.
    pub index: usize,
    pub name: Rc<str>,
    pub original_name: Rc<str>,
    raw: Vec<(String, String)>,
    typed: OnceCell<Value>,
}

impl Row {
    pub fn new(index: usize, record: &PageRecord) -> Self {
        Self {
            index,
            name: Rc::from(record.original_name.to_lowercase()),
            original_name: Rc::from(record.original_name.as_str()),
            raw: record.properties.clone(),
            typed: OnceCell::new(),
        }
    }

    /// The typed properties, or nil for a page with none, as Logseq stores them.
    pub fn properties_value(&self) -> Value {
        if self.raw.is_empty() {
            return Value::Nil;
        }
        self.typed
            .get_or_init(|| typed_properties(&self.raw))
            .clone()
    }

    pub fn has_properties(&self) -> bool {
        !self.raw.is_empty()
    }
}

/// The property key as Logseq makes a keyword from it.
pub fn property_key(key: &str) -> String {
    key.trim().to_lowercase().replace('_', "-")
}

/// All of a page's properties as a map from keyword to typed value. A repeated key keeps
/// its last value, as Logseq does.
pub fn typed_properties(properties: &[(String, String)]) -> Value {
    let mut pairs: Vec<(Value, Value)> = Vec::new();
    for (key, raw) in properties {
        let key = property_key(key);
        let value = typed_value(&key, raw);
        let keyword = Value::keyword(&key);
        match pairs.iter_mut().find(|(k, _)| *k == keyword) {
            Some(entry) => entry.1 = value,
            None => pairs.push((keyword, value)),
        }
    }
    Value::Map(Rc::new(pairs))
}

/// One property value, typed by Logseq's rules.
pub fn typed_value(key: &str, raw: &str) -> Value {
    let text = raw.trim();
    if text.len() >= 2 && text.starts_with('"') && text.ends_with('"') {
        return Value::str(&text[1..text.len() - 1]);
    }
    match text {
        "true" => return Value::Bool(true),
        "false" => return Value::Bool(false),
        _ => {}
    }
    if !text.is_empty() && text.bytes().all(|b| b.is_ascii_digit()) {
        return text
            .parse::<f64>()
            .map_or_else(|_| Value::str(text), Value::Num);
    }
    let mut names = page_references(text);
    if names.is_empty() && matches!(key, "tags" | "alias") {
        names = text
            .split(',')
            .map(|name| name.trim().to_owned())
            .filter(|name| !name.is_empty())
            .collect();
    }
    if names.is_empty() {
        return Value::str(text);
    }
    let mut set: Vec<Value> = Vec::new();
    for name in names {
        let value = Value::str(&name);
        if !set.contains(&value) {
            set.push(value);
        }
    }
    Value::Set(Rc::new(set))
}

/// The names a value refers to: `[[Name]]`, `#[[Name]]` and `#tag`, as written.
fn page_references(text: &str) -> Vec<String> {
    let bytes = text.as_bytes();
    let mut names = Vec::new();
    let mut at = 0;
    while at < bytes.len() {
        if bytes[at..].starts_with(b"[[") {
            // Find the matching close, allowing nested references in the name.
            let mut depth = 0;
            let mut end = None;
            let mut cursor = at;
            while cursor < bytes.len() {
                if bytes[cursor..].starts_with(b"[[") {
                    depth += 1;
                    cursor += 2;
                } else if bytes[cursor..].starts_with(b"]]") {
                    depth -= 1;
                    if depth == 0 {
                        end = Some(cursor);
                        break;
                    }
                    cursor += 2;
                } else {
                    cursor += 1;
                }
            }
            match end {
                Some(end) => {
                    let name = text[at + 2..end].trim();
                    if !name.is_empty() {
                        names.push(name.to_owned());
                    }
                    at = end + 2;
                }
                None => break,
            }
        } else if bytes[at] == b'#'
            && (at == 0 || bytes[at - 1].is_ascii_whitespace() || bytes[at - 1] == b',')
            && !bytes[at + 1..].starts_with(b"[[")
            && !bytes[at + 1..].starts_with(b" ")
        {
            let rest = &text[at + 1..];
            let length = rest
                .find(|c: char| c.is_whitespace() || c == ',')
                .unwrap_or(rest.len());
            if length > 0 {
                names.push(rest[..length].to_owned());
            }
            at += 1 + length;
        } else {
            at += 1;
        }
    }
    names
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(properties: &[(&str, &str)]) -> PageRecord {
        PageRecord {
            key: "k".into(),
            original_name: "Tools/Queries/Typed/One".into(),
            properties: properties
                .iter()
                .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
                .collect(),
        }
    }

    #[test]
    fn types_values_as_logseq_does() {
        assert_eq!(typed_value("public", "true"), Value::Bool(true));
        assert_eq!(typed_value("public", "false"), Value::Bool(false));
        assert_eq!(typed_value("public", "\"true\""), Value::str("true"));
        assert_eq!(typed_value("count", "42"), Value::Num(42.0));
        assert_eq!(typed_value("count", "4.5"), Value::str("4.5"));
        assert_eq!(typed_value("count", "-4"), Value::str("-4"));
        assert_eq!(
            typed_value("kind", "plain words"),
            Value::str("plain words")
        );
        assert_eq!(typed_value("kind", "True"), Value::str("True"));
        assert_eq!(
            typed_value("kind", "[[Kind/A]], [[Kind/B]]"),
            Value::Set(Rc::new(vec![Value::str("Kind/A"), Value::str("Kind/B")]))
        );
        assert_eq!(
            typed_value("kind", "#tag and #[[Other Tag]]"),
            Value::Set(Rc::new(vec![Value::str("tag"), Value::str("Other Tag")]))
        );
        assert_eq!(
            typed_value("tags", "a, b"),
            Value::Set(Rc::new(vec![Value::str("a"), Value::str("b")]))
        );
        // Duplicate names collapse.
        assert_eq!(
            typed_value("kind", "[[A]] [[A]]"),
            Value::Set(Rc::new(vec![Value::str("A")]))
        );
    }

    #[test]
    fn row_holds_three_attributes_and_the_original_case_in_sets() {
        let row = Row::new(
            3,
            &record(&[
                ("Podcast_Published_At", "2026-02-01T10:00:00Z"),
                ("kind", "[[Kind/A]], [[Kind/B]]"),
                ("public", "true"),
                ("public", "false"),
            ]),
        );
        assert_eq!(&*row.name, "tools/queries/typed/one");
        assert_eq!(&*row.original_name, "Tools/Queries/Typed/One");
        let Value::Map(pairs) = row.properties_value() else {
            panic!("properties are a map");
        };
        let get = |key: &str| {
            pairs
                .iter()
                .find(|(k, _)| *k == Value::keyword(key))
                .map(|(_, v)| v.clone())
        };
        assert_eq!(
            get("podcast-published-at"),
            Some(Value::str("2026-02-01T10:00:00Z"))
        );
        // The last value of a repeated key wins.
        assert_eq!(get("public"), Some(Value::Bool(false)));
        assert_eq!(get("missing"), None);
        assert_eq!(pairs.len(), 3);
    }

    #[test]
    fn a_page_without_properties_has_no_properties_attribute() {
        let row = Row::new(0, &record(&[]));
        assert_eq!(row.properties_value(), Value::Nil);
        assert!(!row.has_properties());
    }
}
