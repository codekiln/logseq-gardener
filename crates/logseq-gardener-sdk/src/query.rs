//! Logseq's simple query language and `#+BEGIN_QUERY` maps, parsed and evaluated over
//! the shared [`ReferenceIndex`], so the publisher, the CLI, the language server and the
//! flashcard decks get the same results for the same garden and build date.
//!
//! [`parse_simple`] reads the text of `{{query …}}`; [`parse_advanced`] reads the EDN map
//! of a `#+BEGIN_QUERY` block. [`QueryGraph::evaluate`] answers a parsed query with blocks
//! or pages in Logseq's order, after leaving out the query's own block and any result
//! whose parent is also a result.
//!
//! ```
//! use logseq_gardener_sdk::journals::{JournalDate, JournalFormats};
//! use logseq_gardener_sdk::query::{Clause, parse_simple};
//!
//! let dates = (JournalDate::new(2026, 10, 27).unwrap(), JournalFormats::default());
//! let query = parse_simple("(and [[tmux]] (not [[oh-my-tmux]]))", dates.0, &dates.1).unwrap();
//! assert_eq!(
//!     query.clause,
//!     Some(Clause::And(vec![
//!         Clause::Page("tmux".into()),
//!         Clause::Not(vec![Clause::Page("oh-my-tmux".into())]),
//!     ]))
//! );
//! ```
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::ast::Block;
use crate::garden::GardenDocument;
use crate::journals::{JournalDate, JournalFormats, parse_journal_reference};
use crate::page_names::lookup_key;
use crate::references::{BlockId, ReferenceIndex};

/// Why a query could not be read or run, with the byte offset of the problem in the text
/// that was parsed.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QueryError {
    pub kind: QueryErrorKind,
    pub offset: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum QueryErrorKind {
    /// A simple query that does not parse: Logseq's `Query Error:`.
    Syntax(String),
    /// A `#+BEGIN_QUERY` map that does not read: Logseq's `Invalid query:`.
    InvalidMap(String),
    /// A form the evaluator does not support, named as written: Logseq's `Query failed:`.
    Unsupported(String),
}

impl fmt::Display for QueryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.kind {
            QueryErrorKind::Syntax(message) => {
                write!(f, "{message} at byte {}", self.offset)
            }
            QueryErrorKind::InvalidMap(message) => {
                write!(f, "invalid query map: {message} at byte {}", self.offset)
            }
            QueryErrorKind::Unsupported(form) => write!(
                f,
                "the published site does not support the query form ({form})"
            ),
        }
    }
}

impl std::error::Error for QueryError {}

fn syntax(message: impl Into<String>, offset: usize) -> QueryError {
    QueryError {
        kind: QueryErrorKind::Syntax(message.into()),
        offset,
    }
}

/// A date in `(between A B)`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DateTerm {
    Today,
    Yesterday,
    Tomorrow,
    /// A journal page reference, as written.
    Journal(String),
    /// ±N days, weeks, months or years from the build date.
    Offset(i64, char),
}

/// One clause of a simple query.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Clause {
    /// `[[page]]` or `#tag`: blocks whose path references include the page.
    Page(String),
    /// `"text"`: blocks whose source contains the text, case-sensitive.
    Text(String),
    /// A bare word, which a published Logseq site does not search: no results.
    Word(String),
    And(Vec<Clause>),
    Or(Vec<Clause>),
    Not(Vec<Clause>),
    /// `(task TODO DOING)` and `(todo …)`, upper-cased.
    Task(Vec<String>),
    /// `(priority A)`, upper-cased.
    Priority(Vec<String>),
    Between(DateTerm, DateTerm),
    /// `(property key value)` or `(property key)` on blocks.
    Property(String, Option<String>),
    /// `(page name)`: blocks on a page.
    OnPage(String),
    /// `(page-property key value)` or `(page-property key)`: pages.
    PageProperty(String, Option<String>),
    /// `(page-tags a b)`: pages tagged with any of the names.
    PageTags(Vec<String>),
    /// `(all-page-tags)`: pages used as a tag.
    AllPageTags,
    /// `(namespace ns)`: the direct namespace children of a page.
    Namespace(String),
}

impl Clause {
    /// Whether the clause matches blocks, which makes the whole query return blocks.
    fn needs_blocks(&self) -> bool {
        match self {
            Clause::Page(_)
            | Clause::Text(_)
            | Clause::Task(_)
            | Clause::Priority(_)
            | Clause::Between(..)
            | Clause::Property(..)
            | Clause::OnPage(_) => true,
            Clause::And(items) | Clause::Or(items) | Clause::Not(items) => {
                items.iter().any(Clause::needs_blocks)
            }
            _ => false,
        }
    }
}

/// A parsed simple query.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SimpleQuery {
    /// `None` for a blank query such as `{{query }}` or `(and)`.
    pub clause: Option<Clause>,
    /// `(sort-by key asc|desc)`: the property and whether the order is descending.
    pub sort_by: Option<(String, bool)>,
    /// `(sample n)`.
    pub sample: Option<usize>,
    /// `(by-page false)` turns grouping by page off.
    pub group_by_page: Option<bool>,
    /// The query ends with the word `table`.
    pub table: bool,
}

impl SimpleQuery {
    /// Whether the query returns pages rather than blocks.
    pub fn returns_pages(&self) -> bool {
        self.clause
            .as_ref()
            .is_some_and(|clause| !clause.needs_blocks())
    }

    /// The clauses as Logseq's query builder shows them, such as `( AND [[yazi]] [[card]] )`.
    pub fn clause_text(&self) -> String {
        fn write(clause: &Clause, out: &mut Vec<String>) {
            let list = |name: &str, items: &[Clause], out: &mut Vec<String>| {
                out.push(format!("( {name}"));
                for item in items {
                    write(item, out);
                }
                out.push(")".into());
            };
            match clause {
                Clause::Page(name) => out.push(format!("[[{name}]]")),
                Clause::Text(text) => out.push(format!("\"{text}\"")),
                Clause::Word(word) => out.push(word.clone()),
                Clause::And(items) => list("AND", items, out),
                Clause::Or(items) => list("OR", items, out),
                Clause::Not(items) => list("NOT", items, out),
                Clause::Task(markers) => out.push(format!("task: {}", markers.join(" | "))),
                Clause::Priority(values) => out.push(format!("priority: {}", values.join(" | "))),
                Clause::Between(start, end) => {
                    out.push(format!("between: {} ~ {}", term(start), term(end)))
                }
                Clause::Property(key, value) | Clause::PageProperty(key, value) => {
                    let prefix = if matches!(clause, Clause::Property(..)) {
                        "property"
                    } else {
                        "page-property"
                    };
                    match value {
                        Some(value) => out.push(format!("{prefix} {key}: {value}")),
                        None => out.push(format!("{prefix}: {key}")),
                    }
                }
                Clause::OnPage(name) => out.push(format!("page: {name}")),
                Clause::PageTags(tags) => out.push(format!("page-tags: {}", tags.join(" | "))),
                Clause::AllPageTags => out.push("all-page-tags".into()),
                Clause::Namespace(name) => out.push(format!("namespace: {name}")),
            }
        }
        fn term(term: &DateTerm) -> String {
            match term {
                DateTerm::Today => "today".into(),
                DateTerm::Yesterday => "yesterday".into(),
                DateTerm::Tomorrow => "tomorrow".into(),
                DateTerm::Journal(name) => format!("[[{name}]]"),
                DateTerm::Offset(n, unit) => format!("{n:+}{unit}"),
            }
        }
        let mut out = Vec::new();
        if let Some(clause) = &self.clause {
            write(clause, &mut out);
        }
        out.join(" ")
    }
}

// ---------------------------------------------------------------------------------------
// Simple language reader

#[derive(Clone, Debug, PartialEq)]
enum Form {
    List(Vec<Form>, usize),
    Page(String, usize),
    Str(String, usize),
    Word(String, usize),
}

impl Form {
    fn offset(&self) -> usize {
        match self {
            Form::List(_, at) | Form::Page(_, at) | Form::Str(_, at) | Form::Word(_, at) => *at,
        }
    }
}

struct Lexer<'a> {
    text: &'a str,
    pos: usize,
}

impl Lexer<'_> {
    fn skip(&mut self) {
        while let Some(c) = self.text[self.pos..].chars().next() {
            if c.is_whitespace() || c == ',' {
                self.pos += c.len_utf8();
            } else {
                break;
            }
        }
    }

    fn rest(&self) -> &str {
        &self.text[self.pos..]
    }

    /// `[[name]]` with nested brackets kept in the name.
    fn page(&mut self) -> Result<String, QueryError> {
        let start = self.pos;
        self.pos += 2;
        let mut depth = 1;
        let body = self.pos;
        while self.pos < self.text.len() {
            if self.rest().starts_with("[[") {
                depth += 1;
                self.pos += 2;
            } else if self.rest().starts_with("]]") {
                depth -= 1;
                if depth == 0 {
                    let name = self.text[body..self.pos].trim().to_owned();
                    self.pos += 2;
                    return Ok(name);
                }
                self.pos += 2;
            } else {
                self.pos += self.rest().chars().next().map_or(1, char::len_utf8);
            }
        }
        Err(syntax("unclosed page reference", start))
    }

    fn form(&mut self, depth: usize) -> Result<Option<Form>, QueryError> {
        self.skip();
        let start = self.pos;
        let Some(c) = self.rest().chars().next() else {
            return Ok(None);
        };
        if depth > 64 {
            return Err(syntax("query nesting exceeds 64 levels", start));
        }
        match c {
            '(' => {
                self.pos += 1;
                let mut items = Vec::new();
                loop {
                    self.skip();
                    match self.rest().chars().next() {
                        None => return Err(syntax("unclosed (", start)),
                        Some(')') => {
                            self.pos += 1;
                            return Ok(Some(Form::List(items, start)));
                        }
                        Some(_) => {
                            items.push(self.form(depth + 1)?.expect("text remains"));
                        }
                    }
                }
            }
            ')' => Err(syntax("unexpected )", start)),
            '"' => {
                self.pos += 1;
                let body = self.pos;
                match self.rest().find('"') {
                    Some(end) => {
                        self.pos += end + 1;
                        Ok(Some(Form::Str(
                            self.text[body..body + end].to_owned(),
                            start,
                        )))
                    }
                    None => Err(syntax("unclosed string", start)),
                }
            }
            '[' if self.rest().starts_with("[[") => Ok(Some(Form::Page(self.page()?, start))),
            '#' if self.rest().starts_with("#[[") => {
                self.pos += 1;
                Ok(Some(Form::Page(self.page()?, start)))
            }
            '#' => {
                self.pos += 1;
                let body = self.pos;
                self.word_end();
                let name = &self.text[body..self.pos];
                if name.is_empty() {
                    return Err(syntax("empty tag", start));
                }
                Ok(Some(Form::Page(name.to_owned(), start)))
            }
            '[' | ']' | '{' | '}' => Err(syntax(format!("unexpected {c}"), start)),
            _ => {
                self.word_end();
                Ok(Some(Form::Word(
                    self.text[start..self.pos].to_owned(),
                    start,
                )))
            }
        }
    }

    fn word_end(&mut self) {
        while let Some(c) = self.rest().chars().next() {
            if c.is_whitespace() || "(),\"[]{}".contains(c) {
                break;
            }
            self.pos += c.len_utf8();
        }
    }
}

fn read_forms(text: &str) -> Result<Vec<Form>, QueryError> {
    let mut lexer = Lexer { text, pos: 0 };
    let mut forms = Vec::new();
    while let Some(form) = lexer.form(0)? {
        forms.push(form);
    }
    Ok(forms)
}

/// Replace `<% today %>`, `<% yesterday %>` and `<% tomorrow %>` with journal references.
fn expand_templates(text: &str, today: JournalDate, formats: &JournalFormats) -> String {
    let mut result = String::new();
    let mut rest = text;
    while let Some(start) = rest.find("<%") {
        let Some(end) = rest[start..].find("%>") else {
            break;
        };
        let name = rest[start + 2..start + end].trim().to_lowercase();
        let days = match name.as_str() {
            "today" => Some(0),
            "yesterday" => Some(-1),
            "tomorrow" => Some(1),
            _ => None,
        };
        result.push_str(&rest[..start]);
        match days {
            Some(days) => {
                let date = JournalDate::from_unix_days(today.unix_days() + days);
                result.push_str(&format!("[[{}]]", formats.title.format(date)));
            }
            None => result.push_str(&rest[start..start + end + 2]),
        }
        rest = &rest[start + end + 2..];
    }
    result.push_str(rest);
    result
}

/// Parse the simple query language, after expanding `<% today %>`-style templates against
/// `today`. Several top-level forms combine with `and`, as in Logseq, and a trailing word
/// `table` asks for the table view.
pub fn parse_simple(
    text: &str,
    today: JournalDate,
    formats: &JournalFormats,
) -> Result<SimpleQuery, QueryError> {
    let text = expand_templates(text, today, formats);
    let mut forms = read_forms(&text)?;
    let mut query = SimpleQuery {
        clause: None,
        sort_by: None,
        sample: None,
        group_by_page: None,
        table: false,
    };
    if matches!(forms.last(), Some(Form::Word(word, _)) if word.eq_ignore_ascii_case("table")) {
        forms.pop();
        query.table = true;
    }
    let mut clauses = Vec::new();
    for form in &forms {
        if let Some(clause) = to_clause(form, &mut query)? {
            clauses.push(clause);
        }
    }
    query.clause = match clauses.len() {
        0 => None,
        1 => clauses.pop(),
        _ => Some(Clause::And(clauses)),
    };
    Ok(query)
}

fn name_of(form: &Form) -> Option<String> {
    match form {
        Form::Page(name, _) | Form::Str(name, _) | Form::Word(name, _) => {
            let name = name.trim();
            let name = name.strip_prefix('#').unwrap_or(name);
            Some(name.to_owned())
        }
        Form::List(..) => None,
    }
}

fn property_key(form: &Form) -> Option<String> {
    name_of(form).map(|key| key.trim_start_matches(':').replace('_', "-").to_lowercase())
}

/// The list's arguments, or the items of a single list argument, as in `(task [TODO DOING])`.
fn names(args: &[Form]) -> Vec<String> {
    let args = match args {
        [Form::List(items, _)] => items.as_slice(),
        _ => args,
    };
    args.iter().filter_map(name_of).collect()
}

fn date_term(form: &Form) -> Result<DateTerm, QueryError> {
    let unsupported = || QueryError {
        kind: QueryErrorKind::Unsupported(format!("between {}", name_of(form).unwrap_or_default())),
        offset: form.offset(),
    };
    match form {
        Form::Page(name, _) => Ok(DateTerm::Journal(name.clone())),
        Form::Word(word, _) => {
            let word = word.trim_start_matches(':').to_lowercase();
            match word.as_str() {
                "today" => return Ok(DateTerm::Today),
                "yesterday" => return Ok(DateTerm::Yesterday),
                "tomorrow" => return Ok(DateTerm::Tomorrow),
                _ => {}
            }
            let unit = word.chars().last().ok_or_else(unsupported)?;
            if !"dwmy".contains(unit) {
                return Err(unsupported());
            }
            let number = word[..word.len() - 1].trim_start_matches('+');
            number
                .parse::<i64>()
                .map(|n| DateTerm::Offset(n, unit))
                .map_err(|_| unsupported())
        }
        _ => Err(unsupported()),
    }
}

fn to_clause(form: &Form, query: &mut SimpleQuery) -> Result<Option<Clause>, QueryError> {
    match form {
        Form::Page(name, _) => Ok(Some(Clause::Page(name.clone()))),
        Form::Str(text, _) => Ok(Some(Clause::Text(text.clone()))),
        Form::Word(word, _) => Ok(Some(Clause::Word(word.clone()))),
        Form::List(items, offset) => {
            let Some((head, args)) = items.split_first() else {
                return Ok(None);
            };
            let Form::Word(head, _) = head else {
                return Err(syntax("a list must start with a filter name", *offset));
            };
            let head = head.to_lowercase();
            let unsupported = |form: String| QueryError {
                kind: QueryErrorKind::Unsupported(form),
                offset: *offset,
            };
            let clause = match head.as_str() {
                "and" | "or" | "not" => {
                    let mut children = Vec::new();
                    for arg in args {
                        if let Some(child) = to_clause(arg, query)? {
                            children.push(child);
                        }
                    }
                    if children.is_empty() {
                        return Ok(None);
                    }
                    match head.as_str() {
                        "not" => Clause::Not(children),
                        _ if children.len() == 1 => children.pop().unwrap(),
                        "and" => Clause::And(children),
                        _ => Clause::Or(children),
                    }
                }
                "task" | "todo" => {
                    let markers: Vec<_> = names(args).iter().map(|m| m.to_uppercase()).collect();
                    if markers.is_empty() {
                        return Ok(None);
                    }
                    Clause::Task(markers)
                }
                "priority" => {
                    let values: Vec<_> = names(args).iter().map(|m| m.to_uppercase()).collect();
                    if values.is_empty() {
                        return Ok(None);
                    }
                    Clause::Priority(values)
                }
                "between" => match args {
                    [start, end] => Clause::Between(date_term(start)?, date_term(end)?),
                    [field, ..] => {
                        return Err(unsupported(format!(
                            "between {}",
                            name_of(field).unwrap_or_default()
                        )));
                    }
                    [] => return Err(syntax("between needs two dates", *offset)),
                },
                "property" | "page-property" => {
                    let key = args
                        .first()
                        .and_then(property_key)
                        .ok_or_else(|| syntax(format!("{head} needs a key"), *offset))?;
                    let value = args.get(1).and_then(name_of);
                    if head == "property" {
                        Clause::Property(key, value)
                    } else {
                        Clause::PageProperty(key, value)
                    }
                }
                "page-tags" => {
                    let tags = names(args);
                    if tags.is_empty() {
                        return Ok(None);
                    }
                    Clause::PageTags(tags)
                }
                "all-page-tags" => Clause::AllPageTags,
                "namespace" => Clause::Namespace(
                    args.first()
                        .and_then(name_of)
                        .ok_or_else(|| syntax("namespace needs a page", *offset))?,
                ),
                "page" => Clause::OnPage(
                    args.first()
                        .and_then(name_of)
                        .ok_or_else(|| syntax("page needs a page", *offset))?,
                ),
                "sort-by" => {
                    let key = args
                        .first()
                        .and_then(property_key)
                        .ok_or_else(|| syntax("sort-by needs a key", *offset))?;
                    let descending = !args
                        .get(1)
                        .and_then(name_of)
                        .is_some_and(|order| order.eq_ignore_ascii_case("asc"));
                    query.sort_by = Some((key, descending));
                    return Ok(None);
                }
                "sample" => {
                    let n = args
                        .first()
                        .and_then(name_of)
                        .and_then(|n| n.parse().ok())
                        .ok_or_else(|| syntax("sample needs a number", *offset))?;
                    query.sample = Some(n);
                    return Ok(None);
                }
                "by-page" => {
                    query.group_by_page = Some(
                        !args
                            .first()
                            .and_then(name_of)
                            .is_some_and(|value| value.eq_ignore_ascii_case("false")),
                    );
                    return Ok(None);
                }
                other => return Err(unsupported(other.to_owned())),
            };
            Ok(Some(clause))
        }
    }
}

// ---------------------------------------------------------------------------------------
// Advanced query maps

/// An advanced query's `:title`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum QueryTitle {
    /// A string, shown as 16px text.
    Text(String),
    /// A hiccup vector such as `[:h3 "…"]`: the element name and its text.
    Element { tag: String, text: String },
}

/// An advanced query's `:query`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AdvancedBody {
    /// A query in the simple language, as a string or a form.
    Simple(SimpleQuery),
    /// A Datalog vector with `:find`, which the build does not evaluate.
    Datalog,
    /// No `:query` key.
    Missing,
}

/// A `#+BEGIN_QUERY` map.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdvancedQuery {
    pub title: Option<QueryTitle>,
    pub query: AdvancedBody,
    pub collapsed: bool,
    pub group_by_page: Option<bool>,
    /// `false` keeps results whose parent is also a result.
    pub remove_block_children: bool,
    pub table_view: Option<bool>,
    pub breadcrumb_show: Option<bool>,
    pub inputs: bool,
    pub rules: bool,
    pub view: bool,
    pub result_transform: bool,
}

#[derive(Clone, Debug)]
enum Edn {
    Str(String),
    Keyword(String),
    Symbol(String),
    Vector(Vec<Edn>),
    List(Vec<Edn>),
    Map(Vec<(Edn, Edn)>),
    Other,
}

struct EdnReader<'a> {
    text: &'a [u8],
    pos: usize,
}

fn invalid(message: &str, offset: usize) -> QueryError {
    QueryError {
        kind: QueryErrorKind::InvalidMap(message.to_owned()),
        offset,
    }
}

impl EdnReader<'_> {
    fn skip(&mut self) {
        loop {
            while self
                .text
                .get(self.pos)
                .is_some_and(|c| c.is_ascii_whitespace() || *c == b',')
            {
                self.pos += 1;
            }
            if self.text.get(self.pos) == Some(&b';') {
                while self.text.get(self.pos).is_some_and(|c| *c != b'\n') {
                    self.pos += 1;
                }
            } else {
                break;
            }
        }
    }

    /// One form and its byte span.
    fn form(&mut self, depth: usize) -> Result<Option<(Edn, usize, usize)>, QueryError> {
        self.skip();
        while self.text.get(self.pos..self.pos + 2) == Some(b"#_") {
            self.pos += 2;
            self.form(depth + 1)?
                .ok_or_else(|| invalid("missing discarded form", self.pos))?;
            self.skip();
        }
        let start = self.pos;
        let Some(&c) = self.text.get(self.pos) else {
            return Ok(None);
        };
        if depth > 64 {
            return Err(invalid("nesting exceeds 64 levels", start));
        }
        self.pos += 1;
        let collection = |reader: &mut Self, close: u8| -> Result<Vec<Edn>, QueryError> {
            let mut items = Vec::new();
            loop {
                reader.skip();
                match reader.text.get(reader.pos) {
                    None => return Err(invalid("unbalanced brackets", start)),
                    Some(&b) if b == close => {
                        reader.pos += 1;
                        return Ok(items);
                    }
                    Some(b')' | b']' | b'}') => {
                        return Err(invalid("unbalanced brackets", reader.pos));
                    }
                    Some(_) => {
                        let (form, _, _) = reader.form(depth + 1)?.expect("text remains");
                        items.push(form);
                    }
                }
            }
        };
        let form = match c {
            b'"' => {
                let mut text = Vec::new();
                loop {
                    match self.text.get(self.pos) {
                        None => return Err(invalid("unterminated string", start)),
                        Some(b'\\') => {
                            let escaped = *self
                                .text
                                .get(self.pos + 1)
                                .ok_or_else(|| invalid("unterminated string", start))?;
                            text.push(match escaped {
                                b'n' => b'\n',
                                b't' => b'\t',
                                b'r' => b'\r',
                                other => other,
                            });
                            self.pos += 2;
                        }
                        Some(b'"') => {
                            self.pos += 1;
                            break;
                        }
                        Some(&other) => {
                            text.push(other);
                            self.pos += 1;
                        }
                    }
                }
                Edn::Str(String::from_utf8_lossy(&text).into_owned())
            }
            b'(' => Edn::List(collection(self, b')')?),
            b'[' => Edn::Vector(collection(self, b']')?),
            b'{' => {
                let items = collection(self, b'}')?;
                if items.len() % 2 != 0 {
                    return Err(invalid("map has an unmatched key", start));
                }
                let mut pairs = Vec::new();
                let mut items = items.into_iter();
                while let (Some(key), Some(value)) = (items.next(), items.next()) {
                    pairs.push((key, value));
                }
                Edn::Map(pairs)
            }
            b'#' if self.text.get(self.pos) == Some(&b'{') => {
                self.pos += 1;
                collection(self, b'}')?;
                Edn::Other
            }
            b'#' if self.text.get(self.pos) == Some(&b'(') => {
                self.pos += 1;
                collection(self, b')')?;
                Edn::Other
            }
            b')' | b']' | b'}' => return Err(invalid("unbalanced brackets", start)),
            b'\\' => {
                self.pos += 1;
                self.atom_end();
                Edn::Other
            }
            _ => {
                self.atom_end();
                let atom = String::from_utf8_lossy(&self.text[start..self.pos]).into_owned();
                if atom.starts_with('#') && !atom.starts_with("##") {
                    self.form(depth + 1)?
                        .ok_or_else(|| invalid("missing tagged value", self.pos))?;
                    Edn::Other
                } else if let Some(keyword) = atom.strip_prefix(':') {
                    Edn::Keyword(keyword.to_owned())
                } else {
                    Edn::Symbol(atom)
                }
            }
        };
        Ok(Some((form, start, self.pos)))
    }

    fn atom_end(&mut self) {
        while self
            .text
            .get(self.pos)
            .is_some_and(|c| !c.is_ascii_whitespace() && !b",{}[]()\";".contains(c))
        {
            self.pos += 1;
        }
    }
}

fn truthy(form: &Edn) -> bool {
    !matches!(form, Edn::Symbol(s) if s == "false" || s == "nil")
}

fn hiccup_text(form: &Edn, text: &mut String) {
    match form {
        Edn::Str(s) => text.push_str(s),
        Edn::Symbol(s) => text.push_str(s),
        Edn::Vector(items) | Edn::List(items) => {
            for item in items
                .iter()
                .skip(usize::from(matches!(items.first(), Some(Edn::Keyword(_)))))
            {
                hiccup_text(item, text);
            }
        }
        _ => {}
    }
}

/// Read a `#+BEGIN_QUERY` map. A `:query` vector holding `:find` is Datalog; any other
/// `:query`, a string or a form, is the simple language.
pub fn parse_advanced(
    text: &str,
    today: JournalDate,
    formats: &JournalFormats,
) -> Result<AdvancedQuery, QueryError> {
    let mut reader = EdnReader {
        text: text.as_bytes(),
        pos: 0,
    };
    let Some((Edn::Map(pairs), _, _)) = reader.form(0)? else {
        return Err(invalid("the block must hold one EDN map", 0));
    };
    if let Some((_, start, _)) = reader.form(0)? {
        return Err(invalid("text after the map", start));
    }
    // Spans of the top-level values, to hand the simple language its own text.
    let mut spans = Vec::new();
    {
        let mut reader = EdnReader {
            text: text.as_bytes(),
            pos: 0,
        };
        reader.skip();
        reader.pos += 1;
        loop {
            reader.skip();
            if reader.text.get(reader.pos) == Some(&b'}') || reader.pos >= text.len() {
                break;
            }
            reader.form(1)?;
            match reader.form(1)? {
                Some((_, start, end)) => spans.push((start, end)),
                None => break,
            }
        }
    }
    let mut query = AdvancedQuery {
        title: None,
        query: AdvancedBody::Missing,
        collapsed: false,
        group_by_page: None,
        remove_block_children: true,
        table_view: None,
        breadcrumb_show: None,
        inputs: false,
        rules: false,
        view: false,
        result_transform: false,
    };
    for (position, (key, value)) in pairs.iter().enumerate() {
        let Edn::Keyword(key) = key else { continue };
        match key.as_str() {
            "title" => {
                query.title = match value {
                    Edn::Str(text) => Some(QueryTitle::Text(text.clone())),
                    Edn::Vector(items) => match items.first() {
                        Some(Edn::Keyword(tag)) => {
                            let mut text = String::new();
                            hiccup_text(value, &mut text);
                            Some(QueryTitle::Element {
                                tag: tag.clone(),
                                text,
                            })
                        }
                        _ => None,
                    },
                    _ => None,
                }
            }
            "query" => {
                let datalog = matches!(value, Edn::Vector(items) if items.iter().any(|item| matches!(item, Edn::Keyword(k) if k == "find")));
                query.query = if datalog {
                    AdvancedBody::Datalog
                } else {
                    let source = match value {
                        Edn::Str(text) => text.clone(),
                        _ => spans
                            .get(position)
                            .map(|(start, end)| text[*start..*end].to_owned())
                            .unwrap_or_default(),
                    };
                    AdvancedBody::Simple(parse_simple(&source, today, formats)?)
                };
            }
            "collapsed?" => query.collapsed = truthy(value),
            "group-by-page?" => query.group_by_page = Some(truthy(value)),
            "remove-block-children?" => query.remove_block_children = truthy(value),
            "table-view?" => query.table_view = Some(truthy(value)),
            "breadcrumb-show?" => query.breadcrumb_show = Some(truthy(value)),
            "inputs" => query.inputs = true,
            "rules" => query.rules = true,
            "view" => query.view = true,
            "result-transform" => query.result_transform = true,
            _ => {}
        }
    }
    Ok(query)
}

/// The source text of each map under `:default-queries {:journals [...]}` in a garden's
/// `logseq/config.edn`, for [`parse_advanced`]. An unreadable config gives none.
pub fn default_journal_queries(config: &str) -> Vec<String> {
    let bytes = config.as_bytes();
    let mut reader = EdnReader {
        text: bytes,
        pos: 0,
    };
    // Step through a map's entries, returning the span of the value under `key`.
    fn value_of(reader: &mut EdnReader, key: &str) -> Option<(usize, usize)> {
        reader.skip();
        if reader.text.get(reader.pos) != Some(&b'{') {
            return None;
        }
        reader.pos += 1;
        loop {
            reader.skip();
            if reader.text.get(reader.pos) == Some(&b'}') {
                return None;
            }
            let (found, _, _) = reader.form(1).ok()??;
            let (_, start, end) = reader.form(1).ok()??;
            if matches!(found, Edn::Keyword(ref k) if k == key) {
                return Some((start, end));
            }
        }
    }
    let Some((start, _)) = value_of(&mut reader, "default-queries") else {
        return Vec::new();
    };
    reader.pos = start;
    let Some((start, _)) = value_of(&mut reader, "journals") else {
        return Vec::new();
    };
    reader.pos = start;
    reader.skip();
    if bytes.get(reader.pos) != Some(&b'[') {
        return Vec::new();
    }
    reader.pos += 1;
    let mut maps = Vec::new();
    loop {
        reader.skip();
        if matches!(bytes.get(reader.pos), Some(b']') | None) {
            break;
        }
        match reader.form(1) {
            Ok(Some((_, start, end))) => maps.push(config[start..end].to_owned()),
            _ => break,
        }
    }
    maps
}

// ---------------------------------------------------------------------------------------
// Evaluation

/// Query results in Logseq's order.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum QueryResults<K> {
    Blocks(Vec<BlockId>),
    Pages(Vec<K>),
}

impl<K> QueryResults<K> {
    pub fn len(&self) -> usize {
        match self {
            QueryResults::Blocks(blocks) => blocks.len(),
            QueryResults::Pages(pages) => pages.len(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// Where a query sits and how its results are trimmed.
#[derive(Clone, Debug, Default)]
pub struct EvaluateOptions {
    /// The query's own block, left out of its results.
    pub own_block: Option<BlockId>,
    /// `false` keeps results whose parent is also a result.
    pub keep_children: bool,
    /// Seeds `(sample n)`, so two builds of one commit agree.
    pub seed: u64,
}

/// One block's source text, task marker, priority and properties.
struct BlockFacts<'a> {
    text: &'a str,
    marker: Option<String>,
    priority: Option<String>,
    properties: Vec<(String, String)>,
}

/// The selected visible documents as a query sees them: the reference index plus each
/// block's text, task marker, priority and properties, and each journal's date.
pub struct QueryGraph<'a, K> {
    index: &'a ReferenceIndex<K>,
    /// Global block number of each document's first block.
    offsets: Vec<usize>,
    facts: Vec<Vec<BlockFacts<'a>>>,
    /// Each document's page properties: the properties that open the page, the last value
    /// of a repeated key winning, as in Logseq. Properties after a heading are the
    /// heading's, not the page's.
    page_props: Vec<Vec<(String, String)>>,
    /// Each page's position in the index's contributions.
    positions: BTreeMap<K, usize>,
    /// Whether each document opens with a block of text before its first bullet.
    leading_text: Vec<bool>,
    journals: &'a BTreeMap<K, JournalDate>,
    resolve: &'a dyn Fn(&str) -> Option<K>,
    formats: &'a JournalFormats,
    today: JournalDate,
}

fn bit(set: &[u64], n: usize) -> bool {
    set[n / 64] & (1 << (n % 64)) != 0
}

impl<'a, K: Ord + Clone> QueryGraph<'a, K> {
    /// `documents` holds the source of each page in the index; `resolve` maps a written
    /// page name or alias to its page; `today` is the build date.
    pub fn new(
        index: &'a ReferenceIndex<K>,
        documents: &BTreeMap<K, &'a GardenDocument>,
        journals: &'a BTreeMap<K, JournalDate>,
        resolve: &'a dyn Fn(&str) -> Option<K>,
        formats: &'a JournalFormats,
        today: JournalDate,
    ) -> Self {
        let mut offsets = Vec::new();
        let mut facts = Vec::new();
        let mut page_props = Vec::new();
        let mut positions = BTreeMap::new();
        let mut leading_text = Vec::new();
        let mut total = 0;
        for (position, contribution) in index.contributions().iter().enumerate() {
            offsets.push(total);
            positions.insert(contribution.page.clone(), position);
            total += contribution.blocks.len();
            let Some(document) = documents.get(&contribution.page) else {
                facts.push(Vec::new());
                page_props.push(Vec::new());
                leading_text.push(false);
                continue;
            };
            leading_text.push(
                contribution
                    .blocks
                    .first()
                    .is_some_and(|first| first.level == 0 && !first.properties)
                    && !matches!(
                        document.parsed.blocks.first(),
                        Some(Block::Properties { .. })
                    ),
            );
            let mut props: Vec<(String, String)> = Vec::new();
            if let Some(Block::Properties { props: found, .. }) = document.parsed.blocks.first() {
                for prop in found {
                    let key = prop.0.trim().to_lowercase().replace('_', "-");
                    props.retain(|(k, _)| *k != key);
                    props.push((key, prop.1.trim().to_owned()));
                }
            }
            page_props.push(props);
            let blocks = &document.parsed.blocks;
            let units = &contribution.blocks;
            let mut page_facts = Vec::with_capacity(units.len());
            for (position, unit) in units.iter().enumerate() {
                let next = units.get(position + 1);
                let start = unit.anchor.unwrap_or(0);
                let end = next
                    .and_then(|next| next.anchor)
                    .unwrap_or(document.source.len());
                let end_block = next.map_or(blocks.len(), |next| next.start);
                let (marker, priority) = match blocks.get(unit.start) {
                    Some(Block::Bullet {
                        marker, priority, ..
                    }) => (
                        marker.as_ref().map(|m| m.to_uppercase()),
                        priority.as_ref().map(|p| p.to_uppercase()),
                    ),
                    _ => (None, None),
                };
                let mut properties = Vec::new();
                for block in blocks.get(unit.start..end_block).unwrap_or_default() {
                    if let Block::Properties { props, .. } = block {
                        for prop in props {
                            properties.push((
                                prop.0.trim().to_lowercase().replace('_', "-"),
                                prop.1.trim().to_owned(),
                            ));
                        }
                    }
                }
                page_facts.push(BlockFacts {
                    text: document
                        .source
                        .get(start..end.max(start))
                        .unwrap_or_default(),
                    marker,
                    priority,
                    properties,
                });
            }
            facts.push(page_facts);
        }
        Self {
            index,
            offsets,
            facts,
            page_props,
            positions,
            leading_text,
            journals,
            resolve,
            formats,
            today,
        }
    }

    fn total(&self) -> usize {
        self.offsets.last().map_or(0, |last| {
            last + self
                .index
                .contributions()
                .last()
                .map_or(0, |c| c.blocks.len())
        })
    }

    fn number(&self, id: BlockId) -> usize {
        self.offsets[id.document] + id.block
    }

    fn empty(&self) -> Vec<u64> {
        vec![0; self.total().div_ceil(64)]
    }

    fn set(set: &mut [u64], n: usize) {
        set[n / 64] |= 1 << (n % 64);
    }

    /// Blocks whose facts pass `test`.
    fn scan(&self, test: impl Fn(&BlockFacts) -> bool) -> Vec<u64> {
        let mut set = self.empty();
        for (document, facts) in self.facts.iter().enumerate() {
            for (block, fact) in facts.iter().enumerate() {
                if test(fact) {
                    Self::set(&mut set, self.offsets[document] + block);
                }
            }
        }
        set
    }

    /// Every block of the given pages.
    fn blocks_on(&self, pages: &BTreeSet<K>) -> Vec<u64> {
        let mut set = self.empty();
        for (document, contribution) in self.index.contributions().iter().enumerate() {
            if pages.contains(&contribution.page) {
                for block in 0..contribution.blocks.len() {
                    Self::set(&mut set, self.offsets[document] + block);
                }
            }
        }
        set
    }

    /// The block's parent as Logseq reads it. Bullets indented under text that starts a
    /// page without a bullet, such as `# [[Filed]]`, belong to that text's block.
    fn parent(&self, id: BlockId) -> Option<BlockId> {
        let blocks = &self.index.contributions()[id.document].blocks;
        let block = &blocks[id.block];
        match block.parent {
            Some(parent) => Some(BlockId {
                document: id.document,
                block: parent,
            }),
            None if id.block > 0 && block.level >= 2 && self.leads(id.document) => Some(BlockId {
                document: id.document,
                block: 0,
            }),
            None => None,
        }
    }

    /// Whether the page starts with text that is a block of its own: content before the
    /// first bullet that is neither the page's properties nor follows them. After page
    /// properties, Logseq starts a separate block that the reference index merges with
    /// them, so its bullets stay top-level here.
    fn leads(&self, document: usize) -> bool {
        self.leading_text[document]
    }

    /// Mark a block and its descendants.
    fn mark_subtree(&self, set: &mut [u64], id: BlockId) {
        let blocks = &self.index.contributions()[id.document].blocks;
        let first = self.number(id);
        Self::set(set, first);
        let block = &blocks[id.block];
        if block.properties {
            // As in Logseq, a page's properties are a block of their own and no other
            // block's ancestor.
            return;
        }
        let leading = id.block == 0 && self.leads(id.document);
        for (offset, next) in blocks[id.block + 1..].iter().enumerate() {
            let inside = if leading {
                next.level >= 2
            } else {
                next.start < block.subtree_end
            };
            if !inside {
                break;
            }
            Self::set(set, first + 1 + offset);
        }
    }

    /// Blocks whose path references include the page: blocks that name it, their
    /// descendants, and every block on the page itself. A name that no published page
    /// owns, such as a tag with no page file, matches the blocks that write it.
    fn page_ref(&self, name: &str) -> Vec<u64> {
        let mut set = self.empty();
        match (self.resolve)(name) {
            Some(page) => {
                set = self.blocks_on(&BTreeSet::from([page.clone()]));
                for &id in self.index.linked(&page) {
                    self.mark_subtree(&mut set, id);
                }
            }
            None => {
                let wanted = lookup_key(name);
                for (document, contribution) in self.index.contributions().iter().enumerate() {
                    for (block, refs) in contribution.graph_refs().iter().enumerate() {
                        if refs.unresolved.iter().any(|n| lookup_key(n) == wanted) {
                            self.mark_subtree(&mut set, BlockId { document, block });
                        }
                    }
                }
            }
        }
        set
    }

    fn date(&self, term: &DateTerm) -> Option<JournalDate> {
        let today = self.today;
        let days = |n: i64| Some(JournalDate::from_unix_days(today.unix_days() + n));
        match term {
            DateTerm::Today => days(0),
            DateTerm::Yesterday => days(-1),
            DateTerm::Tomorrow => days(1),
            DateTerm::Journal(name) => parse_journal_reference(name, self.formats),
            DateTerm::Offset(n, 'd') => days(*n),
            DateTerm::Offset(n, 'w') => days(n * 7),
            DateTerm::Offset(n, unit) => {
                let months = if *unit == 'y' { n * 12 } else { *n };
                let index = i64::from(today.year()) * 12 + i64::from(today.month()) - 1 + months;
                let (year, month) = (
                    index.div_euclid(12) as i32,
                    (index.rem_euclid(12) + 1) as u8,
                );
                (1..=today.day())
                    .rev()
                    .find_map(|day| JournalDate::new(year, month, day))
            }
        }
    }

    fn page_properties(&self, page: &K) -> &[(String, String)] {
        self.positions
            .get(page)
            .map_or(&[], |&document| self.page_props[document].as_slice())
    }

    /// The pages a page's `tags::` property names.
    fn page_tags(&self, document: usize) -> BTreeSet<K> {
        let Some((_, value)) = self.page_props[document].iter().find(|(k, _)| k == "tags") else {
            return BTreeSet::new();
        };
        let mut names = lsdoc::refs(value, "markdown").page;
        names.extend(value.split(',').map(|part| {
            let part = part.trim().trim_start_matches('#');
            part.strip_prefix("[[")
                .and_then(|p| p.strip_suffix("]]"))
                .unwrap_or(part)
                .to_owned()
        }));
        names
            .iter()
            .filter_map(|name| (self.resolve)(name))
            .collect()
    }

    /// Whether a property value names `wanted`: as a whole, as a page it links, or as one
    /// of its comma-separated entries, ignoring case.
    fn value_matches(value: &str, wanted: &str) -> bool {
        let wanted = lookup_key(wanted);
        if lookup_key(value) == wanted {
            return true;
        }
        lsdoc::refs(value, "markdown")
            .page
            .iter()
            .any(|name| lookup_key(name) == wanted)
            || value.split(',').any(|part| {
                let part = part.trim().trim_start_matches('#');
                let part = part
                    .strip_prefix("[[")
                    .and_then(|p| p.strip_suffix("]]"))
                    .unwrap_or(part);
                lookup_key(part) == wanted
            })
    }

    /// Pages that match a page clause, or `None` when the clause is about blocks.
    fn pages(&self, clause: &Clause) -> Option<BTreeSet<K>> {
        let all = || self.index.pages().map(|(page, _)| page.clone());
        Some(match clause {
            Clause::PageProperty(key, value) => all()
                .filter(|page| {
                    self.page_properties(page).iter().any(|(k, v)| {
                        k == key
                            && value
                                .as_ref()
                                .is_none_or(|value| Self::value_matches(v, value))
                    })
                })
                .collect(),
            Clause::PageTags(tags) => {
                let tags: BTreeSet<K> = tags.iter().filter_map(|tag| (self.resolve)(tag)).collect();
                self.index
                    .contributions()
                    .iter()
                    .enumerate()
                    .filter(|(document, _)| {
                        self.page_tags(*document)
                            .iter()
                            .any(|tag| tags.contains(tag))
                    })
                    .map(|(_, c)| c.page.clone())
                    .collect()
            }
            Clause::AllPageTags => (0..self.page_props.len())
                .flat_map(|document| self.page_tags(document))
                .collect(),
            Clause::Namespace(name) => {
                let parent = lookup_key(name);
                self.index
                    .pages()
                    .filter(|(_, title)| {
                        title
                            .rsplit_once('/')
                            .is_some_and(|(p, _)| lookup_key(p) == parent)
                    })
                    .map(|(page, _)| page.clone())
                    .collect()
            }
            Clause::And(items) => {
                let mut sets = items.iter().map(|item| self.pages(item));
                let mut result = sets.next()??;
                for set in sets {
                    let set = set?;
                    result.retain(|page| set.contains(page));
                }
                result
            }
            Clause::Or(items) => {
                let mut result = BTreeSet::new();
                for item in items {
                    result.extend(self.pages(item)?);
                }
                result
            }
            Clause::Not(items) => {
                let mut excluded = BTreeSet::new();
                for item in items {
                    excluded.extend(self.pages(item)?);
                }
                all().filter(|page| !excluded.contains(page)).collect()
            }
            _ => return None,
        })
    }

    fn blocks(&self, clause: &Clause) -> Vec<u64> {
        if let Some(pages) = self.pages(clause) {
            return self.blocks_on(&pages);
        }
        match clause {
            Clause::Page(name) => self.page_ref(name),
            Clause::Text(text) => self.scan(|fact| fact.text.contains(text.as_str())),
            Clause::Word(_) => self.empty(),
            Clause::Task(markers) => {
                self.scan(|fact| fact.marker.as_ref().is_some_and(|m| markers.contains(m)))
            }
            Clause::Priority(values) => {
                self.scan(|fact| fact.priority.as_ref().is_some_and(|p| values.contains(p)))
            }
            Clause::Between(start, end) => {
                let (Some(start), Some(end)) = (self.date(start), self.date(end)) else {
                    return self.empty();
                };
                let (start, end) = (start.min(end), start.max(end));
                let pages = self
                    .journals
                    .iter()
                    .filter(|(_, date)| **date >= start && **date <= end)
                    .map(|(page, _)| page.clone())
                    .collect();
                self.blocks_on(&pages)
            }
            Clause::Property(key, value) => self.scan(|fact| {
                fact.properties.iter().any(|(k, v)| {
                    k == key
                        && value
                            .as_ref()
                            .is_none_or(|value| Self::value_matches(v, value))
                })
            }),
            Clause::OnPage(name) => match (self.resolve)(name) {
                Some(page) => self.blocks_on(&BTreeSet::from([page])),
                None => self.empty(),
            },
            Clause::And(items) => {
                let mut sets = items.iter().map(|item| self.blocks(item));
                let mut set = sets.next().unwrap_or_else(|| self.empty());
                for other in sets {
                    for (word, other) in set.iter_mut().zip(other) {
                        *word &= other;
                    }
                }
                set
            }
            Clause::Or(items) => {
                let mut set = self.empty();
                for other in items.iter().map(|item| self.blocks(item)) {
                    for (word, other) in set.iter_mut().zip(other) {
                        *word |= other;
                    }
                }
                set
            }
            Clause::Not(items) => {
                let total = self.total();
                let mut set = self.empty();
                for other in items.iter().map(|item| self.blocks(item)) {
                    for (word, other) in set.iter_mut().zip(other) {
                        *word |= other;
                    }
                }
                for word in &mut set {
                    *word = !*word;
                }
                if !total.is_multiple_of(64)
                    && let Some(last) = set.last_mut()
                {
                    *last &= (1u64 << (total % 64)) - 1;
                }
                set
            }
            _ => self.empty(),
        }
    }

    /// Groups of blocks by page: journals newest first, then pages by lower-cased title;
    /// within a page, outline order.
    pub fn group_by_page(&self, blocks: &[BlockId]) -> Vec<(K, Vec<BlockId>)> {
        let mut groups: BTreeMap<usize, Vec<BlockId>> = BTreeMap::new();
        for &id in blocks {
            groups.entry(id.document).or_default().push(id);
        }
        let mut groups: Vec<(K, Vec<BlockId>)> = groups
            .into_iter()
            .map(|(document, mut ids)| {
                ids.sort();
                (self.index.contributions()[document].page.clone(), ids)
            })
            .collect();
        groups.sort_by_cached_key(|(page, _)| match self.journals.get(page) {
            Some(date) => (0, Some(std::cmp::Reverse(*date)), String::new()),
            None => (
                1,
                None,
                self.index.title(page).unwrap_or_default().to_lowercase(),
            ),
        });
        groups
    }

    /// The value of a block's property, for sorting and table cells.
    pub fn block_property(&self, id: BlockId, key: &str) -> Option<&str> {
        self.facts
            .get(id.document)?
            .get(id.block)?
            .properties
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, v)| v.as_str())
    }

    /// A block's properties in source order, with lower-cased keys.
    pub fn block_properties(&self, id: BlockId) -> &[(String, String)] {
        self.facts
            .get(id.document)
            .and_then(|facts| facts.get(id.block))
            .map_or(&[], |fact| fact.properties.as_slice())
    }

    /// A page's leading properties in source order, with lower-cased keys.
    pub fn page_property_list(&self, page: &K) -> &[(String, String)] {
        self.page_properties(page)
    }

    /// A block's source text, from its bullet to the next block.
    pub fn block_text(&self, id: BlockId) -> &str {
        self.facts
            .get(id.document)
            .and_then(|facts| facts.get(id.block))
            .map_or("", |fact| fact.text)
    }

    /// Evaluate a simple query. A query whose clauses all return pages returns pages.
    pub fn evaluate(&self, query: &SimpleQuery, options: &EvaluateOptions) -> QueryResults<K> {
        let Some(clause) = &query.clause else {
            return QueryResults::Blocks(Vec::new());
        };
        if query.returns_pages() {
            let mut pages: Vec<K> = self.pages(clause).unwrap_or_default().into_iter().collect();
            pages.sort_by_cached_key(|page| {
                self.index.title(page).unwrap_or_default().to_lowercase()
            });
            if let Some(n) = query.sample {
                sample(&mut pages, n, options.seed);
            }
            if let Some((key, descending)) = &query.sort_by {
                pages.sort_by(|a, b| {
                    let value = |page: &K| {
                        self.page_properties(page)
                            .iter()
                            .find(|(k, _)| k == key)
                            .map(|(_, v)| v.to_lowercase())
                    };
                    let order = value(a).cmp(&value(b));
                    if *descending { order.reverse() } else { order }
                });
            }
            return QueryResults::Pages(pages);
        }
        let set = self.blocks(clause);
        let mut matched = BTreeSet::new();
        for (document, contribution) in self.index.contributions().iter().enumerate() {
            for block in 0..contribution.blocks.len() {
                if bit(&set, self.offsets[document] + block) {
                    matched.insert(BlockId { document, block });
                }
            }
        }
        if let Some(own) = options.own_block {
            matched.remove(&own);
        }
        let mut blocks: Vec<BlockId> = matched
            .iter()
            .copied()
            .filter(|id| {
                options.keep_children
                    || !self
                        .parent(*id)
                        .is_some_and(|parent| matched.contains(&parent))
            })
            .collect();
        blocks = self
            .group_by_page(&blocks)
            .into_iter()
            .flat_map(|(_, ids)| ids)
            .collect();
        if let Some(n) = query.sample {
            sample(&mut blocks, n, options.seed);
        }
        if let Some((key, descending)) = &query.sort_by {
            blocks.sort_by(|a, b| {
                let value = |id: BlockId| self.block_property(id, key).map(str::to_lowercase);
                let order = value(*a).cmp(&value(*b));
                if *descending { order.reverse() } else { order }
            });
        }
        QueryResults::Blocks(blocks)
    }
}

/// Keep `n` items chosen by a generator seeded with `seed`, in their original order.
fn sample<T>(items: &mut Vec<T>, n: usize, seed: u64) {
    if items.len() <= n {
        return;
    }
    let mut state = seed ^ 0x9e37_79b9_7f4a_7c15;
    let mut next = || {
        // SplitMix64.
        state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    };
    let mut positions: Vec<usize> = (0..items.len()).collect();
    for i in 0..n {
        let j = i + (next() % (positions.len() - i) as u64) as usize;
        positions.swap(i, j);
    }
    let keep: BTreeSet<usize> = positions[..n].iter().copied().collect();
    let mut position = 0;
    items.retain(|_| {
        let kept = keep.contains(&position);
        position += 1;
        kept
    });
}

/// A stable seed for `(sample n)` from text that names the query's block.
pub fn seed(text: &str) -> u64 {
    // FNV-1a.
    text.bytes().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
        (hash ^ u64::from(byte)).wrapping_mul(0x0100_0000_01b3)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::garden::DocumentKind;
    use crate::references::{Contribution, PageEntry};

    fn today() -> JournalDate {
        JournalDate::new(2026, 10, 27).unwrap()
    }

    fn parse(text: &str) -> Result<SimpleQuery, QueryError> {
        parse_simple(text, today(), &JournalFormats::default())
    }

    #[test]
    fn parses_every_form() {
        let clause = |text: &str| parse(text).unwrap().clause.unwrap();
        assert_eq!(clause("[[tmux]]"), Clause::Page("tmux".into()));
        assert_eq!(clause("#tmux"), Clause::Page("tmux".into()));
        assert_eq!(clause("#[[a b]]"), Clause::Page("a b".into()));
        assert_eq!(
            clause("\"lightweight database\""),
            Clause::Text("lightweight database".into())
        );
        assert_eq!(
            clause("(or [[a]] (not [[b]]))"),
            Clause::Or(vec![
                Clause::Page("a".into()),
                Clause::Not(vec![Clause::Page("b".into())])
            ])
        );
        assert_eq!(
            clause("(task TODO doing)"),
            Clause::Task(vec!["TODO".into(), "DOING".into()])
        );
        assert_eq!(clause("(todo now)"), Clause::Task(vec!["NOW".into()]));
        assert_eq!(
            clause("(priority a b)"),
            Clause::Priority(vec!["A".into(), "B".into()])
        );
        assert_eq!(
            clause("(between -30d today)"),
            Clause::Between(DateTerm::Offset(-30, 'd'), DateTerm::Today)
        );
        assert_eq!(
            clause("(between [[Oct 1st, 2026]] +1w)"),
            Clause::Between(
                DateTerm::Journal("Oct 1st, 2026".into()),
                DateTerm::Offset(1, 'w')
            )
        );
        assert_eq!(
            clause("(property type book)"),
            Clause::Property("type".into(), Some("book".into()))
        );
        assert_eq!(
            clause("(property type)"),
            Clause::Property("type".into(), None)
        );
        assert_eq!(
            clause("(page-property logseq-entity [[Logseq/Entity/Proxy/Page]])"),
            Clause::PageProperty(
                "logseq-entity".into(),
                Some("Logseq/Entity/Proxy/Page".into())
            )
        );
        assert_eq!(
            clause("(page-tags [[CLI/Tool]])"),
            Clause::PageTags(vec!["CLI/Tool".into()])
        );
        assert_eq!(clause("(all-page-tags)"), Clause::AllPageTags);
        assert_eq!(clause("(namespace mise)"), Clause::Namespace("mise".into()));
        assert_eq!(clause("(page [[tmux]])"), Clause::OnPage("tmux".into()));
        assert_eq!(
            clause("[[a]] [[b]]"),
            Clause::And(vec![Clause::Page("a".into()), Clause::Page("b".into())])
        );
        let query =
            parse("(and [[a]] (sort-by rating asc) (sample 3) (by-page false)) table").unwrap();
        assert_eq!(query.clause, Some(Clause::Page("a".into())));
        assert_eq!(query.sort_by, Some(("rating".into(), false)));
        assert_eq!(query.sample, Some(3));
        assert_eq!(query.group_by_page, Some(false));
        assert!(query.table);
        assert_eq!(
            parse("query").unwrap().clause,
            Some(Clause::Word("query".into()))
        );
        assert_eq!(parse("").unwrap().clause, None);
        assert_eq!(parse("(and)").unwrap().clause, None);
    }

    #[test]
    fn templates_expand_to_journal_references() {
        assert_eq!(
            parse("(between <% yesterday %> <% today %>)")
                .unwrap()
                .clause,
            Some(Clause::Between(
                DateTerm::Journal("Oct 26th, 2026".into()),
                DateTerm::Journal("Oct 27th, 2026".into())
            ))
        );
    }

    #[test]
    fn errors_name_the_problem_and_its_offset() {
        let error = parse("(and [[tmux]]").unwrap_err();
        assert_eq!(error.offset, 0);
        assert!(matches!(error.kind, QueryErrorKind::Syntax(_)));
        let error = parse("(and [[a]] (frobnicate x))").unwrap_err();
        assert_eq!(error.kind, QueryErrorKind::Unsupported("frobnicate".into()));
        assert_eq!(error.offset, 11);
        let error = parse("(between created-at -7d today)").unwrap_err();
        assert_eq!(
            error.kind,
            QueryErrorKind::Unsupported("between created-at".into())
        );
        assert!(matches!(
            parse("[[open").unwrap_err().kind,
            QueryErrorKind::Syntax(_)
        ));
    }

    #[test]
    fn clause_text_reads_like_the_query_builder() {
        assert_eq!(
            parse("(and [[yazi]] [[card]])").unwrap().clause_text(),
            "( AND [[yazi]] [[card]] )"
        );
        assert_eq!(
            parse("(between -30d today)").unwrap().clause_text(),
            "between: -30d ~ today"
        );
    }

    #[test]
    fn advanced_maps() {
        let formats = JournalFormats::default();
        let query = parse_advanced(
            "{:title \"Blocks\" :query [[tmux]] :collapsed? true}",
            today(),
            &formats,
        )
        .unwrap();
        assert_eq!(query.title, Some(QueryTitle::Text("Blocks".into())));
        assert!(query.collapsed);
        let AdvancedBody::Simple(simple) = &query.query else {
            panic!()
        };
        assert_eq!(simple.clause, Some(Clause::Page("tmux".into())));
        let query = parse_advanced(
            "{:title [:h3 \"Blocks that mention tmux\"]\n :query (and [[tmux]] (task TODO))}",
            today(),
            &formats,
        )
        .unwrap();
        assert_eq!(
            query.title,
            Some(QueryTitle::Element {
                tag: "h3".into(),
                text: "Blocks that mention tmux".into()
            })
        );
        assert!(matches!(query.query, AdvancedBody::Simple(_)));
        let query = parse_advanced(
            "{:title \"NOW\" :query [:find (pull ?b [*]) :where [?b :block/marker ?m]] :inputs [:14d :today] :result-transform (fn [r] r) :group-by-page? false}",
            today(),
            &formats,
        )
        .unwrap();
        assert_eq!(query.query, AdvancedBody::Datalog);
        assert_eq!(query.title, Some(QueryTitle::Text("NOW".into())));
        assert!(query.inputs && query.result_transform);
        assert_eq!(query.group_by_page, Some(false));
        let query = parse_advanced(
            "{:query \"(task TODO)\" :remove-block-children? false}",
            today(),
            &formats,
        )
        .unwrap();
        assert!(!query.remove_block_children);
        assert!(matches!(query.query, AdvancedBody::Simple(_)));
        let error =
            parse_advanced("{:title \"x\" :query (and [[tmux]]}", today(), &formats).unwrap_err();
        assert!(matches!(error.kind, QueryErrorKind::InvalidMap(_)));
    }

    #[test]
    fn reads_default_journal_queries() {
        let config = "{:a 1 ;; note\n :default-queries\n {:journals\n  [{:title \"NOW\" :query [:find (pull ?b [*]) :where [?b :block/marker]]}\n   {:title \"Tasks\" :query (task TODO)}]}\n :b [1 2]}";
        let maps = default_journal_queries(config);
        assert_eq!(maps.len(), 2);
        let formats = JournalFormats::default();
        assert_eq!(
            parse_advanced(&maps[0], today(), &formats).unwrap().query,
            AdvancedBody::Datalog
        );
        assert!(matches!(
            parse_advanced(&maps[1], today(), &formats).unwrap().query,
            AdvancedBody::Simple(_)
        ));
        assert!(default_journal_queries("{:a 1}").is_empty());
    }

    fn document(path: &str, kind: DocumentKind, source: &str) -> GardenDocument {
        GardenDocument {
            relative_path: path.into(),
            kind,
            source: source.to_owned(),
            parsed: lsdoc::parse_to_projection(source),
        }
    }

    struct Fixture {
        index: ReferenceIndex<String>,
        documents: Vec<(String, GardenDocument)>,
        journals: BTreeMap<String, JournalDate>,
    }

    fn fixture() -> Fixture {
        let documents = vec![
            (
                "tmux".to_owned(),
                document(
                    "pages/tmux.md",
                    DocumentKind::Page,
                    "alias:: terminal multiplexer\ntags:: CLI/Tool\n\n- tmux splits panes\n",
                ),
            ),
            (
                "yazi".to_owned(),
                document(
                    "pages/yazi.md",
                    DocumentKind::Page,
                    "tags:: CLI/Tool\ntype:: [[file manager]]\n\n- TODO [#A] try [[terminal multiplexer]] with yazi\n\t- child names [[tmux]] too\n- lightweight database\n- rating:: 3\n",
                ),
            ),
            (
                "j1".to_owned(),
                document(
                    "journals/2026_10_20.md",
                    DocumentKind::Journal,
                    "- DOING read about [[tmux]]\n\t- {{query [[tmux]]}}\n",
                ),
            ),
            (
                "j2".to_owned(),
                document(
                    "journals/2026_08_01.md",
                    DocumentKind::Journal,
                    "- old [[yazi]] note\n- rating:: 5\n",
                ),
            ),
            (
                "CLI/Tool".to_owned(),
                document("pages/CLI___Tool.md", DocumentKind::Page, "- a tag page\n"),
            ),
        ];
        let titles = [
            ("tmux", "tmux"),
            ("yazi", "yazi"),
            ("j1", "Oct 20th, 2026"),
            ("j2", "Aug 1st, 2026"),
            ("CLI/Tool", "CLI/Tool"),
            ("CLI", "CLI"),
        ];
        let resolve = |name: &str| resolve_name(name);
        let contributions = documents
            .iter()
            .map(|(page, document)| {
                Contribution::from_document(page.clone(), document, &resolve, None)
            })
            .collect();
        let index = ReferenceIndex::build(
            titles.iter().map(|(id, title)| PageEntry {
                id: (*id).to_owned(),
                title: (*title).to_owned(),
                aliases: if *id == "tmux" {
                    vec!["terminal multiplexer".into()]
                } else {
                    vec![]
                },
            }),
            contributions,
        );
        let journals = BTreeMap::from([
            ("j1".to_owned(), JournalDate::new(2026, 10, 20).unwrap()),
            ("j2".to_owned(), JournalDate::new(2026, 8, 1).unwrap()),
        ]);
        Fixture {
            index,
            documents,
            journals,
        }
    }

    fn resolve_name(name: &str) -> Option<String> {
        match lookup_key(name).as_str() {
            "tmux" | "terminal multiplexer" => Some("tmux".into()),
            "yazi" => Some("yazi".into()),
            "cli/tool" => Some("CLI/Tool".into()),
            "cli" => Some("CLI".into()),
            "oct 20th, 2026" => Some("j1".into()),
            _ => None,
        }
    }

    fn run(fixture: &Fixture, text: &str, options: &EvaluateOptions) -> QueryResults<String> {
        let documents: BTreeMap<String, &GardenDocument> = fixture
            .documents
            .iter()
            .map(|(page, document)| (page.clone(), document))
            .collect();
        let formats = JournalFormats::default();
        let resolve = |name: &str| resolve_name(name);
        let graph = QueryGraph::new(
            &fixture.index,
            &documents,
            &fixture.journals,
            &resolve,
            &formats,
            today(),
        );
        graph.evaluate(&parse(text).unwrap(), options)
    }

    fn texts(fixture: &Fixture, results: &QueryResults<String>) -> Vec<String> {
        let QueryResults::Blocks(blocks) = results else {
            panic!("{results:?}")
        };
        blocks
            .iter()
            .map(|id| {
                let (page, document) = &fixture.documents[id.document];
                let block = fixture.index.block(*id);
                let start = block.anchor.unwrap_or(0);
                format!(
                    "{page}: {}",
                    document.source[start..].lines().next().unwrap().trim()
                )
            })
            .collect()
    }

    #[test]
    fn page_references_follow_paths_aliases_and_the_top_level_rule() {
        let fixture = fixture();
        let options = EvaluateOptions::default();
        assert_eq!(
            texts(&fixture, &run(&fixture, "[[tmux]]", &options)),
            vec![
                "j1: - DOING read about [[tmux]]",
                "tmux: alias:: terminal multiplexer",
                "tmux: - tmux splits panes",
                "yazi: - TODO [#A] try [[terminal multiplexer]] with yazi",
            ]
        );
        // The child repeats the reference but its parent is already a result.
        let keep = EvaluateOptions {
            keep_children: true,
            ..EvaluateOptions::default()
        };
        assert_eq!(run(&fixture, "[[tmux]]", &keep).len(), 6);
        // The query's own block is left out.
        let own = EvaluateOptions {
            own_block: Some(BlockId {
                document: 2,
                block: 1,
            }),
            keep_children: true,
            ..EvaluateOptions::default()
        };
        assert_eq!(run(&fixture, "[[tmux]]", &own).len(), 5);
    }

    #[test]
    fn block_filters() {
        let fixture = fixture();
        let options = EvaluateOptions::default();
        let count = |text: &str| run(&fixture, text, &options).len();
        assert_eq!(count("(and [[yazi]] (task TODO DOING))"), 1);
        assert_eq!(count("(task doing)"), 1);
        assert_eq!(count("(priority A)"), 1);
        assert_eq!(count("\"lightweight database\""), 1);
        assert_eq!(count("\"Lightweight database\""), 0);
        assert_eq!(count("lightweight"), 0);
        assert_eq!(count("(property rating 3)"), 1);
        assert_eq!(count("(property rating)"), 2);
        // The child of the task is on the page too, but its parent is already a result.
        assert_eq!(count("(page yazi)"), 4);
        // -30d from 2026-10-27 includes 2026-10-20 but not 2026-08-01; each journal's
        // top-level blocks count.
        assert_eq!(count("(between -30d today)"), 1);
        assert_eq!(count("(between -3m today)"), 3);
        assert_eq!(count("(between [[Oct 20th, 2026]] [[Oct 20th, 2026]])"), 1);
        // A query of only `not` matches every other block.
        let keep = EvaluateOptions {
            keep_children: true,
            ..EvaluateOptions::default()
        };
        let all = run(&fixture, "(not [[nothing-here]])", &keep).len();
        // yazi's five blocks and the journal block that names yazi.
        assert_eq!(run(&fixture, "(not [[yazi]])", &keep).len(), all - 6);
    }

    #[test]
    fn page_filters_return_pages() {
        let fixture = fixture();
        let options = EvaluateOptions::default();
        let pages = |text: &str| match run(&fixture, text, &options) {
            QueryResults::Pages(pages) => pages,
            other => panic!("{other:?}"),
        };
        assert_eq!(pages("(page-tags [[CLI/Tool]])"), vec!["tmux", "yazi"]);
        assert_eq!(pages("(page-property type [[file manager]])"), vec!["yazi"]);
        assert_eq!(pages("(page-property alias)"), vec!["tmux"]);
        assert_eq!(pages("(all-page-tags)"), vec!["CLI/Tool"]);
        assert_eq!(pages("(namespace CLI)"), vec!["CLI/Tool"]);
        // A page filter inside a block query keeps blocks on those pages.
        assert_eq!(
            run(
                &fixture,
                "(and (page-tags [[CLI/Tool]]) (task TODO))",
                &options
            )
            .len(),
            1
        );
    }

    #[test]
    fn sort_by_and_seeded_sample() {
        let fixture = fixture();
        let options = EvaluateOptions {
            seed: seed("x"),
            ..EvaluateOptions::default()
        };
        assert_eq!(
            texts(
                &fixture,
                &run(
                    &fixture,
                    "(and (property rating) (sort-by rating asc))",
                    &options
                )
            ),
            vec!["yazi: - rating:: 3", "j2: - rating:: 5"]
        );
        let first = run(&fixture, "(and (not [[nothing]]) (sample 2))", &options);
        assert_eq!(first.len(), 2);
        assert_eq!(
            first,
            run(&fixture, "(and (not [[nothing]]) (sample 2))", &options)
        );
    }

    #[test]
    fn text_before_the_first_bullet_is_a_parent_unless_properties_come_first() {
        let documents = [
            (
                "heading".to_owned(),
                document(
                    "pages/heading.md",
                    DocumentKind::Page,
                    "# [[tmux]]\n\t- under the heading\n- top level\n",
                ),
            ),
            (
                "props".to_owned(),
                document(
                    "pages/props.md",
                    DocumentKind::Page,
                    "see:: [[tmux]]\n# Heading\n\t- names [[tmux]]\n",
                ),
            ),
            (
                "tmux".to_owned(),
                document("pages/tmux.md", DocumentKind::Page, "- tmux\n"),
            ),
        ];
        let resolve = |name: &str| (lookup_key(name) == "tmux").then(|| "tmux".to_owned());
        let index = ReferenceIndex::build(
            ["heading", "props", "tmux"].map(|id| PageEntry {
                id: id.to_owned(),
                title: id.to_owned(),
                aliases: vec![],
            }),
            documents
                .iter()
                .map(|(page, d)| Contribution::from_document(page.clone(), d, &resolve, None))
                .collect(),
        );
        let map: BTreeMap<String, &GardenDocument> =
            documents.iter().map(|(p, d)| (p.clone(), d)).collect();
        let journals = BTreeMap::new();
        let formats = JournalFormats::default();
        let graph = QueryGraph::new(&index, &map, &journals, &resolve, &formats, today());
        let QueryResults::Blocks(blocks) =
            graph.evaluate(&parse("[[tmux]]").unwrap(), &EvaluateOptions::default())
        else {
            panic!()
        };
        let per_page: Vec<_> = graph
            .group_by_page(&blocks)
            .into_iter()
            .map(|(page, ids)| (page, ids.len()))
            .collect();
        // The heading's bullet is its child; after properties, the bullet stays top-level.
        assert_eq!(
            per_page,
            vec![
                ("heading".to_owned(), 1),
                ("props".to_owned(), 2),
                ("tmux".to_owned(), 1)
            ]
        );
    }

    #[test]
    fn groups_put_journals_first_newest_first() {
        let fixture = fixture();
        let documents: BTreeMap<String, &GardenDocument> = fixture
            .documents
            .iter()
            .map(|(page, document)| (page.clone(), document))
            .collect();
        let formats = JournalFormats::default();
        let resolve = |name: &str| resolve_name(name);
        let graph = QueryGraph::new(
            &fixture.index,
            &documents,
            &fixture.journals,
            &resolve,
            &formats,
            today(),
        );
        let QueryResults::Blocks(blocks) = graph.evaluate(
            &parse("(or [[yazi]] [[tmux]])").unwrap(),
            &EvaluateOptions::default(),
        ) else {
            panic!()
        };
        let order: Vec<_> = graph
            .group_by_page(&blocks)
            .into_iter()
            .map(|(page, _)| page)
            .collect();
        assert_eq!(order, vec!["j1", "j2", "tmux", "yazi"]);
    }
}
