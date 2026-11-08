//! The Datalog subset an advanced query's `:query` may use, read and run over pages:
//!
//! ```clojure
//! [:find (pull ?p [*])
//!  :where [?p :block/name]
//!         [?p :block/properties ?props]
//!         [(get ?props :kind) ?kinds]
//!         [(contains? ?kinds "Kind/A")]]
//! ```
//!
//! A clause is a data pattern over the page variable (`:block/name`,
//! `:block/original-name` or `:block/properties`, with an optional variable or literal in
//! the third place), a binding clause `[(get ?m :key) ?out]`, or a predicate clause
//! `[(contains? a b)]`, `[(= a b)]` or `[(not= a b)]`. Variables bind in order, as in
//! Datascript, so a clause that reads an unbound variable is an error when the query is
//! read. A binding that finds nothing, and a predicate that is false, drop the page.
use std::rc::Rc;

use crate::query::{QueryError, QueryErrorKind};
use crate::query_form::{Form, FormKind};
use crate::query_page::Row;
use crate::query_script::{Builtin, Meter, Thrown, Value, call_builtin, equal, get};

/// A page attribute a data pattern may match.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Attribute {
    Name,
    OriginalName,
    Properties,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Operand {
    /// A variable, by its slot.
    Var(usize),
    Lit(Value),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Clause {
    Data(Attribute, Option<Operand>),
    /// `[(get a b) ?out]`, with an optional default as the third argument.
    Bind(Vec<Operand>, usize),
    /// `[(contains? a b)]`, `[(= a b)]` or `[(not= a b)]`.
    Predicate(Builtin, Vec<Operand>),
}

/// A query in the subset, ready to run.
#[derive(Clone, Debug, PartialEq)]
pub struct DatalogQuery {
    clauses: Vec<Clause>,
    /// Slot 0 is the page variable; further slots hold bound values.
    variables: Vec<String>,
}

fn unsupported(name: String, offset: usize) -> QueryError {
    QueryError {
        kind: QueryErrorKind::Unsupported(name),
        offset,
    }
}

fn syntax(message: impl Into<String>, offset: usize) -> QueryError {
    QueryError {
        kind: QueryErrorKind::Syntax(message.into()),
        offset,
    }
}

fn is_variable(form: &Form) -> Option<&str> {
    match &form.kind {
        FormKind::Symbol(name) if name.starts_with('?') && name.len() > 1 => Some(name),
        _ => None,
    }
}

struct Reading {
    variables: Vec<String>,
    /// Whether the page variable (slot 0) has been introduced.
    page_seen: bool,
}

impl Reading {
    fn slot(&self, name: &str) -> Option<usize> {
        self.variables.iter().position(|v| v == name)
    }

    fn operand(&self, form: &Form) -> Result<Operand, QueryError> {
        if let Some(name) = is_variable(form) {
            return match self.slot(name) {
                Some(slot) if slot > 0 || self.page_seen => Ok(Operand::Var(slot)),
                _ => Err(syntax(format!("unbound variable {name}"), form.offset)),
            };
        }
        Ok(Operand::Lit(match &form.kind {
            FormKind::Nil => Value::Nil,
            FormKind::Bool(b) => Value::Bool(*b),
            FormKind::Num(n) => Value::Num(*n),
            FormKind::Str(s) => Value::str(s),
            FormKind::Keyword(k) => Value::keyword(k),
            _ => return Err(unsupported(form.name(), form.offset)),
        }))
    }

    fn bind(&mut self, name: &str) -> usize {
        self.variables.push(name.to_owned());
        self.variables.len() - 1
    }
}

/// Read a `:query` vector in the subset. An unbound variable is a syntax error; any other
/// form outside the subset is an unsupported-form error with the form's byte offset.
pub fn parse_datalog(form: &Form) -> Result<DatalogQuery, QueryError> {
    let FormKind::Vector(items) = &form.kind else {
        return Err(unsupported(form.name(), form.offset));
    };
    // `:find (pull ?p [*])`
    let mut position = 0;
    match items.get(position).map(|f| &f.kind) {
        Some(FormKind::Keyword(k)) if k == "find" => position += 1,
        _ => {
            return Err(unsupported(
                items.first().map_or(form.name(), Form::name),
                form.offset,
            ));
        }
    }
    let Some(pull) = items.get(position) else {
        return Err(syntax(":find needs (pull ?p [*])", form.offset));
    };
    position += 1;
    let pulled = match &pull.kind {
        FormKind::List(parts)
            if parts.len() == 3
                && matches!(&parts[0].kind, FormKind::Symbol(s) if s == "pull")
                && matches!(&parts[2].kind, FormKind::Vector(w) if w.len() == 1 && matches!(&w[0].kind, FormKind::Symbol(s) if s == "*")) =>
        {
            is_variable(&parts[1]).map(|name| (name, parts[1].offset))
        }
        _ => None,
    };
    let Some((page_variable, find_offset)) = pulled else {
        return Err(unsupported(pull.name(), pull.offset));
    };
    match items.get(position).map(|f| &f.kind) {
        Some(FormKind::Keyword(k)) if k == "where" => position += 1,
        Some(_) => {
            let other = &items[position];
            return Err(unsupported(other.name(), other.offset));
        }
        None => return Err(syntax(":where is missing", form.offset)),
    }
    let mut reading = Reading {
        variables: vec![page_variable.to_owned()],
        page_seen: false,
    };
    let mut clauses = Vec::new();
    for clause in &items[position..] {
        clauses.push(parse_clause(clause, &mut reading)?);
    }
    if !reading.page_seen {
        return Err(syntax(
            format!("no data pattern binds {page_variable}"),
            find_offset,
        ));
    }
    Ok(DatalogQuery {
        clauses,
        variables: reading.variables,
    })
}

fn parse_clause(clause: &Form, reading: &mut Reading) -> Result<Clause, QueryError> {
    let FormKind::Vector(parts) = &clause.kind else {
        // `(or …)`, `(not …)`, `(and …)` and rule calls.
        return Err(match &clause.kind {
            FormKind::List(parts) => match parts.first() {
                Some(head) => unsupported(head.name(), head.offset),
                None => unsupported(clause.name(), clause.offset),
            },
            _ => unsupported(clause.name(), clause.offset),
        });
    };
    let Some(head) = parts.first() else {
        return Err(unsupported(clause.name(), clause.offset));
    };
    if let FormKind::List(call) = &head.kind {
        return parse_function(call, head, &parts[1..], reading);
    }
    // A data pattern: `[?p :attribute]` or `[?p :attribute x]`.
    let Some(subject) = is_variable(head) else {
        return Err(unsupported(head.name(), head.offset));
    };
    let Some(attribute) = parts.get(1) else {
        return Err(syntax("a data pattern needs an attribute", clause.offset));
    };
    let attr = match &attribute.kind {
        FormKind::Keyword(k) if k == "block/name" => Attribute::Name,
        FormKind::Keyword(k) if k == "block/original-name" => Attribute::OriginalName,
        FormKind::Keyword(k) if k == "block/properties" => Attribute::Properties,
        _ => return Err(unsupported(attribute.name(), attribute.offset)),
    };
    if parts.len() > 3 {
        return Err(unsupported(parts[3].name(), parts[3].offset));
    }
    if subject != reading.variables[0] {
        return Err(unsupported(
            format!("{subject} (a second page variable)"),
            head.offset,
        ));
    }
    reading.page_seen = true;
    let target = match parts.get(2) {
        None => None,
        Some(target) => Some(match is_variable(target) {
            Some(name) => match reading.slot(name) {
                Some(slot) => Operand::Var(slot),
                None => Operand::Var(reading.bind(name)),
            },
            None => reading.operand(target)?,
        }),
    };
    Ok(Clause::Data(attr, target))
}

fn parse_function(
    call: &[Form],
    head: &Form,
    rest: &[Form],
    reading: &mut Reading,
) -> Result<Clause, QueryError> {
    let Some(Form {
        kind: FormKind::Symbol(name),
        ..
    }) = call.first()
    else {
        return Err(unsupported(head.name(), head.offset));
    };
    let function = &call[0];
    let arguments = &call[1..];
    match name.as_str() {
        "get" if !rest.is_empty() => {
            if !(2..=3).contains(&arguments.len()) {
                return Err(syntax("get takes two or three arguments", head.offset));
            }
            let operands = arguments
                .iter()
                .map(|argument| reading.operand(argument))
                .collect::<Result<Vec<_>, _>>()?;
            let [output] = rest else {
                return Err(unsupported(rest[1].name(), rest[1].offset));
            };
            let Some(variable) = is_variable(output) else {
                return Err(unsupported(output.name(), output.offset));
            };
            if reading.slot(variable).is_some() {
                return Err(syntax(
                    format!("{variable} is already bound"),
                    output.offset,
                ));
            }
            Ok(Clause::Bind(operands, reading.bind(variable)))
        }
        "contains?" | "=" | "not=" if rest.is_empty() => {
            let operands = arguments
                .iter()
                .map(|argument| reading.operand(argument))
                .collect::<Result<Vec<_>, _>>()?;
            let builtin = match name.as_str() {
                "contains?" => Builtin::Contains,
                "=" => Builtin::Eq,
                _ => Builtin::NotEq,
            };
            if (builtin == Builtin::Contains && operands.len() != 2) || operands.len() < 2 {
                return Err(syntax(
                    format!("{name} needs two operands"),
                    function.offset,
                ));
            }
            Ok(Clause::Predicate(builtin, operands))
        }
        other => Err(unsupported(other.to_owned(), function.offset)),
    }
}

impl DatalogQuery {
    /// The page variable's name, for messages.
    pub fn page_variable(&self) -> &str {
        &self.variables[0]
    }

    /// Run over `rows` in the order given, charging one step per page and clause. Returns
    /// the rows that satisfy every clause.
    pub fn run(&self, rows: &[Rc<Row>], meter: &mut Meter) -> Result<Vec<Rc<Row>>, Thrown> {
        let mut found = Vec::new();
        'rows: for row in rows {
            let mut slots: Vec<Option<Value>> = vec![None; self.variables.len()];
            slots[0] = Some(Value::Row(row.clone()));
            for clause in &self.clauses {
                meter.charge(1)?;
                if !self.step(clause, &mut slots, row, meter)? {
                    continue 'rows;
                }
            }
            found.push(row.clone());
        }
        Ok(found)
    }

    fn value(&self, operand: &Operand, slots: &[Option<Value>]) -> Value {
        match operand {
            Operand::Var(slot) => slots[*slot].clone().unwrap_or(Value::Nil),
            Operand::Lit(value) => value.clone(),
        }
    }

    fn step(
        &self,
        clause: &Clause,
        slots: &mut [Option<Value>],
        row: &Rc<Row>,
        meter: &mut Meter,
    ) -> Result<bool, Thrown> {
        match clause {
            Clause::Data(attribute, target) => {
                let value = match attribute {
                    Attribute::Name => Value::str(&row.name),
                    Attribute::OriginalName => Value::str(&row.original_name),
                    Attribute::Properties => {
                        if !row.has_properties() {
                            return Ok(false);
                        }
                        row.properties_value()
                    }
                };
                Ok(match target {
                    None => true,
                    Some(Operand::Lit(literal)) => equal(literal, &value),
                    Some(Operand::Var(slot)) => match &slots[*slot] {
                        Some(bound) => equal(bound, &value),
                        None => {
                            slots[*slot] = Some(value);
                            true
                        }
                    },
                })
            }
            Clause::Bind(operands, output) => {
                let arguments: Vec<Value> = operands
                    .iter()
                    .map(|operand| self.value(operand, slots))
                    .collect();
                let found = if arguments.len() == 2 {
                    get(&arguments[0], &arguments[1])?
                } else {
                    call_builtin(Builtin::Get, &arguments, meter)?
                };
                if matches!(found, Value::Nil) {
                    return Ok(false);
                }
                slots[*output] = Some(found);
                Ok(true)
            }
            Clause::Predicate(builtin, operands) => {
                let arguments: Vec<Value> = operands
                    .iter()
                    .map(|operand| self.value(operand, slots))
                    .collect();
                Ok(call_builtin(*builtin, &arguments, meter)?.truthy())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::query_form::read_form;
    use crate::query_page::PageRecord;

    const GITPA: &str = r#"[:find (pull ?p [*])
        :where
        [?p :block/name]
        [?p :block/properties ?props]
        [(get ?props :logseq-entity) ?entities]
        [(contains? ?entities "Logseq/Entity/Podcast/Episode")]
        [(get ?props :public) ?public]
        [(= ?public true)]]"#;

    fn parse(source: &str) -> Result<DatalogQuery, QueryError> {
        parse_datalog(&read_form(source, 0).unwrap())
    }

    fn pages(specs: &[(&str, &[(&str, &str)])]) -> Vec<Rc<Row>> {
        let mut records: Vec<PageRecord> = specs
            .iter()
            .map(|(name, properties)| PageRecord {
                key: (*name).to_owned(),
                original_name: (*name).to_owned(),
                properties: properties
                    .iter()
                    .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
                    .collect(),
            })
            .collect();
        records.sort_by_key(|r| r.original_name.to_lowercase());
        records
            .iter()
            .enumerate()
            .map(|(i, r)| Rc::new(Row::new(i, r)))
            .collect()
    }

    fn names(rows: &[Rc<Row>]) -> Vec<String> {
        rows.iter().map(|r| r.original_name.to_string()).collect()
    }

    #[test]
    fn reads_the_gitpa_form() {
        let query = parse(GITPA).unwrap();
        assert_eq!(query.page_variable(), "?p");
        assert_eq!(query.clauses.len(), 6);
    }

    #[test]
    fn rejects_unbound_variables_and_unsupported_forms() {
        let source = "[:find (pull ?p [*]) :where [(get ?props :kind) ?kinds] [?p :block/name]]";
        let error = parse(source).unwrap_err();
        assert_eq!(
            error.kind,
            QueryErrorKind::Syntax("unbound variable ?props".into())
        );
        assert_eq!(error.offset, source.find("?props").unwrap());
        let source = "[:find (pull ?b [*]) :where [?b :block/marker ?m]]";
        let error = parse(source).unwrap_err();
        assert_eq!(
            error.kind,
            QueryErrorKind::Unsupported(":block/marker".into())
        );
        assert_eq!(error.offset, source.find(":block/marker").unwrap());
        for (source, name) in [
            (
                "[:find (pull ?p [*]) :in $ ?x :where [?p :block/name]]",
                ":in",
            ),
            ("[:find (pull ?p [*]) :where (or [?p :block/name])]", "or"),
            ("[:find (pull ?p [*]) :where (not [?p :block/name])]", "not"),
            ("[:find ?p :where [?p :block/name]]", "?p"),
            (
                "[:find (pull ?p [:block/name]) :where [?p :block/name]]",
                "(",
            ),
            ("[:find (count ?p) :where [?p :block/name]]", "("),
            (
                "[:find (pull ?p [*]) :where [?p :block/name] [?q :block/name]]",
                "?q (a second page variable)",
            ),
            (
                "[:find (pull ?p [*]) :where [?p :block/name ?n] [(str ?n) ?s]]",
                "str",
            ),
            (
                "[:find (pull ?p [*]) :where [?p :block/name ?n] [(< ?n 3)]]",
                "<",
            ),
            (
                "[:find (pull ?p [*]) :where [?p :block/name ?n] [(re-find ?n \"x\")]]",
                "re-find",
            ),
            (
                "[:find (pull ?p [*]) :where [?p :block/page]]",
                ":block/page",
            ),
            (
                "[:find (pull ?p [*]) :with ?x :where [?p :block/name]]",
                ":with",
            ),
        ] {
            let error = parse(source).unwrap_err();
            assert_eq!(
                error.kind,
                QueryErrorKind::Unsupported(name.to_owned()),
                "{source}"
            );
        }
        // Reading the page variable before a pattern introduces it.
        assert!(matches!(
            parse("[:find (pull ?p [*]) :where [(= ?p 1)] [?p :block/name]]")
                .unwrap_err()
                .kind,
            QueryErrorKind::Syntax(_)
        ));
        // No clause binds the page variable.
        assert!(matches!(
            parse("[:find (pull ?p [*]) :where]").unwrap_err().kind,
            QueryErrorKind::Syntax(_)
        ));
    }

    #[test]
    fn gitpa_form_keeps_public_episodes_of_the_type() {
        let rows = pages(&[
            (
                "Ep/One",
                &[
                    ("logseq-entity", "[[Logseq/Entity/Podcast/Episode]]"),
                    ("public", "true"),
                ],
            ),
            (
                "Ep/Two",
                &[
                    ("logseq-entity", "[[Logseq/Entity/Podcast/Episode]]"),
                    ("public", "false"),
                ],
            ),
            (
                "Ep/Three",
                &[
                    ("logseq-entity", "[[Logseq/Entity/Podcast/Episode]]"),
                    ("public", "\"true\""),
                ],
            ),
            (
                "Ep/Four",
                &[("logseq-entity", "[[Logseq/Entity/Podcast/Episode]]")],
            ),
            (
                "Ep/Five",
                &[("logseq-entity", "[[Other/Type]]"), ("public", "true")],
            ),
            ("Ep/Six", &[("public", "true")]),
            ("No properties", &[]),
            (
                "Ep/Seven",
                &[
                    (
                        "logseq-entity",
                        "[[Other/Type]], [[Logseq/Entity/Podcast/Episode]]",
                    ),
                    ("public", "true"),
                ],
            ),
        ]);
        let found = parse(GITPA)
            .unwrap()
            .run(&rows, &mut Meter::default())
            .unwrap();
        assert_eq!(names(&found), ["Ep/One", "Ep/Seven"]);
    }

    #[test]
    fn set_membership_keeps_the_case_written() {
        let rows = pages(&[("A", &[("kind", "[[Kind/A]], [[Kind/B]]")])]);
        let yes = parse("[:find (pull ?p [*]) :where [?p :block/properties ?q] [(get ?q :kind) ?k] [(contains? ?k \"Kind/B\")]]").unwrap();
        let no = parse("[:find (pull ?p [*]) :where [?p :block/properties ?q] [(get ?q :kind) ?k] [(contains? ?k \"kind/b\")]]").unwrap();
        assert_eq!(yes.run(&rows, &mut Meter::default()).unwrap().len(), 1);
        assert_eq!(no.run(&rows, &mut Meter::default()).unwrap().len(), 0);
    }

    #[test]
    fn patterns_with_literals_and_variables() {
        let rows = pages(&[("Alpha", &[]), ("Beta", &[("x", "1")])]);
        let by_name = parse("[:find (pull ?p [*]) :where [?p :block/name \"beta\"]]").unwrap();
        assert_eq!(
            names(&by_name.run(&rows, &mut Meter::default()).unwrap()),
            ["Beta"]
        );
        let by_original =
            parse("[:find (pull ?p [*]) :where [?p :block/original-name ?n] [(= ?n \"Alpha\")]]")
                .unwrap();
        assert_eq!(
            names(&by_original.run(&rows, &mut Meter::default()).unwrap()),
            ["Alpha"]
        );
        let all = parse("[:find (pull ?p [*]) :where [?p :block/name]]").unwrap();
        assert_eq!(
            names(&all.run(&rows, &mut Meter::default()).unwrap()),
            ["Alpha", "Beta"]
        );
        let numbers = parse(
            "[:find (pull ?p [*]) :where [?p :block/properties ?q] [(get ?q :x) ?x] [(not= ?x 2)]]",
        )
        .unwrap();
        assert_eq!(
            names(&numbers.run(&rows, &mut Meter::default()).unwrap()),
            ["Beta"]
        );
    }

    #[test]
    fn rows_come_back_in_the_order_given_and_cost_steps() {
        let rows = pages(&[("b", &[]), ("a", &[]), ("C", &[])]);
        let query = parse("[:find (pull ?p [*]) :where [?p :block/name]]").unwrap();
        let mut meter = Meter::default();
        let found = query.run(&rows, &mut meter).unwrap();
        assert_eq!(names(&found), ["a", "b", "C"]);
        assert_eq!(meter.steps(), 3);
        let mut meter = Meter::with_limit(2);
        assert_eq!(
            query.run(&rows, &mut meter).unwrap_err(),
            Thrown::Limit(crate::query_script::Limit::Steps)
        );
    }
}
