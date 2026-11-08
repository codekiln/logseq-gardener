//! Run an advanced query over the published pages: the Datalog subset, then the
//! `:result-transform`, then the `:view`. The result is rows or a view's tree with no
//! HTML, so the publisher, the command line and the language server get the same answer
//! for the same garden.
//!
//! ```
//! use logseq_gardener_sdk::journals::{JournalDate, JournalFormats};
//! use logseq_gardener_sdk::query::parse_advanced;
//! use logseq_gardener_sdk::query_advanced::run_advanced;
//! use logseq_gardener_sdk::query_page::PageRecord;
//!
//! let today = JournalDate::new(2026, 10, 27).unwrap();
//! let query = parse_advanced(
//!     "{:query [:find (pull ?p [*]) :where [?p :block/name]]}",
//!     today,
//!     &JournalFormats::default(),
//! )
//! .unwrap();
//! let page = |name: &str| PageRecord {
//!     key: name.to_lowercase(),
//!     original_name: name.to_owned(),
//!     properties: Vec::new(),
//! };
//! let pages = [page("Beta"), page("Alpha")];
//! let result = run_advanced(&query, &pages).unwrap();
//! // Rows are indexes into `pages`, in lower-case page-name order.
//! assert_eq!(result.rows, vec![1, 0]);
//! ```
use std::rc::Rc;

use crate::query::{AdvancedBody, AdvancedQuery, QueryError, QueryErrorKind};
use crate::query_hiccup::{Hiccup, query_hiccup};
use crate::query_page::{PageRecord, Row};
use crate::query_script::{Meter, Script, Thrown, Value, items};

/// Which part of the query failed: the box says `Query failed:` or `Custom view failed:`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stage {
    Query,
    View,
}

/// Why a query could not show a result.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QueryFailure {
    pub stage: Stage,
    /// What went wrong, naming the form, the error or the limit.
    pub message: String,
    /// Where the form is in the block text, when the failure is a form outside the subset.
    pub offset: Option<usize>,
    /// The rows left after the transform, when the view failed: Logseq shows the count above
    /// the failure.
    pub count: Option<usize>,
}

/// What a query produced.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AdvancedResult {
    /// The number of rows left after the transform.
    pub count: usize,
    /// The pages among those rows, as indexes into the pages given, in the order to show.
    /// Empty when the query has a view, which takes the place of the rows.
    pub rows: Vec<usize>,
    /// The view's output, when the query has a view and a result.
    pub view: Option<Vec<Hiccup>>,
    /// Problems that did not stop the query, such as a transform that threw.
    pub warnings: Vec<String>,
    /// Evaluation steps used, the same for every run over the same pages.
    pub steps: u64,
}

fn failure(stage: Stage, message: impl Into<String>, offset: Option<usize>) -> QueryFailure {
    QueryFailure {
        stage,
        message: message.into(),
        offset,
        count: None,
    }
}

fn read_failure(stage: Stage, error: &QueryError) -> QueryFailure {
    let message = match &error.kind {
        QueryErrorKind::Unsupported(_) => error.to_string(),
        QueryErrorKind::Syntax(message) | QueryErrorKind::InvalidMap(message) => message.clone(),
    };
    failure(stage, message, Some(error.offset))
}

fn thrown(stage: Stage, error: &Thrown) -> QueryFailure {
    failure(stage, error.to_string(), None)
}

/// Run `query` over `pages`. The pages are the published ones, in any order; rows reach
/// the transform in lower-case page-name order. A form outside the subset, a limit and
/// a thrown error in the query or the view are failures; a thrown transform leaves the
/// rows as they were, with a warning.
pub fn run_advanced(
    query: &AdvancedQuery,
    pages: &[PageRecord],
) -> Result<AdvancedResult, QueryFailure> {
    let datalog = match &query.query {
        AdvancedBody::Datalog(datalog) => match datalog.as_ref() {
            Ok(datalog) => datalog,
            Err(error) => return Err(read_failure(Stage::Query, error)),
        },
        _ => return Err(failure(Stage::Query, "the query is not Datalog", None)),
    };
    if query.inputs || query.rules {
        return Err(failure(
            Stage::Query,
            "This published site does not support :inputs or :rules.",
            None,
        ));
    }
    // Read every part before running any of it.
    let transform = match &query.result_transform {
        Some(Err(error)) => return Err(read_failure(Stage::Query, error)),
        Some(Ok(script)) => Some(script),
        None => None,
    };
    let view = match &query.view {
        Some(Err(error)) => return Err(read_failure(Stage::View, error)),
        Some(Ok(script)) => Some(script),
        None => None,
    };

    let mut meter = Meter::default();
    let mut order: Vec<usize> = (0..pages.len()).collect();
    order.sort_by_cached_key(|&index| pages[index].original_name.to_lowercase());
    let rows: Vec<Rc<Row>> = order
        .iter()
        .map(|&index| Rc::new(Row::new(index, &pages[index])))
        .collect();
    let matched = datalog
        .run(&rows, &mut meter)
        .map_err(|error| thrown(Stage::Query, &error))?;
    let mut warnings = Vec::new();
    let mut result = Value::vector(matched.into_iter().map(Value::Row).collect());
    if let Some(transform) = transform {
        match transform.call(vec![result.clone()], &mut meter) {
            Ok(value) => result = value,
            Err(Thrown::Error(message)) => warnings.push(format!(
                ":result-transform threw ({message}); results shown untransformed"
            )),
            Err(limit) => return Err(thrown(Stage::Query, &limit)),
        }
    }
    let sequence = items(&result).map_err(|error| thrown(Stage::Query, &error))?;
    let count = sequence.len();
    let mut shown = Vec::new();
    if view.is_none() {
        for value in sequence.iter() {
            match value {
                Value::Row(row) => shown.push(row.index),
                other => {
                    return Err(failure(
                        Stage::Query,
                        format!(
                            "The :result-transform returned {}, which a table cannot show; add a :view",
                            other.pr_str()
                        ),
                        None,
                    ));
                }
            }
        }
    }
    let tree = match view {
        Some(view) if count > 0 => Some(run_view(view, &sequence, &mut meter)?),
        _ => None,
    };
    Ok(AdvancedResult {
        count,
        rows: shown,
        view: tree,
        warnings,
        steps: meter.steps(),
    })
}

fn run_view(
    view: &Script,
    rows: &Rc<Vec<Value>>,
    meter: &mut Meter,
) -> Result<Vec<Hiccup>, QueryFailure> {
    let counted = |mut failed: QueryFailure| {
        failed.count = Some(rows.len());
        failed
    };
    let value = view
        .call(vec![Value::Vector(rows.clone())], meter)
        .map_err(|error| counted(thrown(Stage::View, &error)))?;
    query_hiccup(&value).map_err(|error| counted(failure(Stage::View, error.to_string(), None)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::journals::{JournalDate, JournalFormats};
    use crate::query::parse_advanced;

    const QUERY: &str = r#"[:find (pull ?p [*])
        :where
        [?p :block/name]
        [?p :block/properties ?props]
        [(get ?props :logseq-entity) ?entities]
        [(contains? ?entities "Type/Episode")]
        [(get ?props :public) ?public]
        [(= ?public true)]]"#;
    const TRANSFORM: &str = "(fn [rows] (sort-by (fn [row] (get-in row [:block/properties :at])) (fn [a b] (compare b a)) rows))";
    const VIEW: &str = "(fn [rows] (into [\"ul\"] (map (fn [row] [\"li\" [\"a\" {:href (str \"#/page/\" (clojure.string/replace (:block/name row) \"/\" \"%2F\"))} (:block/original-name row)]]) rows)))";

    fn page(name: &str, properties: &[(&str, &str)]) -> PageRecord {
        PageRecord {
            key: name.to_lowercase(),
            original_name: name.to_owned(),
            properties: properties
                .iter()
                .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
                .collect(),
        }
    }

    fn episode(name: &str, at: &str) -> PageRecord {
        page(
            name,
            &[
                ("logseq-entity", "[[Type/Episode]]"),
                ("public", "true"),
                ("at", at),
            ],
        )
    }

    fn parse(map: &str) -> AdvancedQuery {
        parse_advanced(
            map,
            JournalDate::new(2026, 10, 27).unwrap(),
            &JournalFormats::default(),
        )
        .unwrap()
    }

    fn map(transform: bool, view: bool) -> String {
        let mut text = format!("{{:title \"T\" :query {QUERY}");
        if transform {
            text.push_str(&format!(" :result-transform {TRANSFORM}"));
        }
        if view {
            text.push_str(&format!(" :view {VIEW}"));
        }
        text.push('}');
        text
    }

    fn pages() -> Vec<PageRecord> {
        vec![
            episode("Ep/Feb", "2026-02-01T10:00:00Z"),
            episode("Ep/Mar", "2026-03-01T10:00:00Z"),
            episode("Ep/Dec", "2025-12-31T23:00:00Z"),
            page(
                "Draft",
                &[("logseq-entity", "[[Type/Episode]]"), ("public", "false")],
            ),
            page("Other", &[("public", "true")]),
        ]
    }

    fn links(nodes: &[Hiccup], out: &mut Vec<(String, String)>) {
        for node in nodes {
            if let Hiccup::Element {
                tag,
                attrs,
                children,
            } = node
            {
                if tag == "a" {
                    let href = attrs.iter().find(|(k, _)| k == "href").unwrap().1.clone();
                    let Hiccup::Text(text) = &children[0] else {
                        panic!()
                    };
                    out.push((href, text.clone()));
                }
                links(children, out);
            }
        }
    }

    #[test]
    fn rows_without_a_view_come_back_in_name_order_or_transformed() {
        let pages = pages();
        let plain = run_advanced(&parse(&map(false, false)), &pages).unwrap();
        assert_eq!(plain.count, 3);
        assert_eq!(plain.rows, [2, 0, 1]);
        assert!(plain.view.is_none());
        // Newest first.
        let sorted = run_advanced(&parse(&map(true, false)), &pages).unwrap();
        assert_eq!(sorted.rows, [1, 0, 2]);
    }

    #[test]
    fn view_returns_the_tree_over_the_transformed_rows() {
        let result = run_advanced(&parse(&map(true, true)), &pages()).unwrap();
        assert_eq!(result.count, 3);
        assert!(result.rows.is_empty());
        let mut found = Vec::new();
        links(result.view.as_ref().unwrap(), &mut found);
        assert_eq!(
            found,
            [
                ("#/page/ep%2Fmar".to_owned(), "Ep/Mar".to_owned()),
                ("#/page/ep%2Ffeb".to_owned(), "Ep/Feb".to_owned()),
                ("#/page/ep%2Fdec".to_owned(), "Ep/Dec".to_owned()),
            ]
        );
        assert!(result.steps > 0);
    }

    #[test]
    fn zones_sort_as_strings() {
        let pages = vec![
            episode("A", "2026-01-01T00:30:00+02:00"),
            episode("B", "2025-12-31T23:00:00Z"),
        ];
        // Newest first by string: the +02:00 value sorts first, as in Logseq, although it
        // is the earlier instant.
        let result = run_advanced(&parse(&map(true, false)), &pages).unwrap();
        assert_eq!(result.rows, [0, 1]);
    }

    #[test]
    fn an_empty_result_calls_no_view() {
        let result = run_advanced(&parse(&map(true, true)), &[page("Other", &[])]).unwrap();
        assert_eq!(result.count, 0);
        assert!(result.view.is_none());
        // A view that would throw is not called.
        let source = format!("{{:query {QUERY} :view (fn [rows] (compare \"a\" 1))}}");
        assert!(run_advanced(&parse(&source), &[]).unwrap().view.is_none());
    }

    #[test]
    fn a_caller_without_a_view_gets_rows_without_html() {
        let result = run_advanced(&parse(&format!("{{:query {QUERY}}}")), &pages()).unwrap();
        assert_eq!(result.rows.len(), 3);
    }

    #[test]
    fn a_thrown_transform_leaves_the_rows_with_a_warning() {
        let source = format!("{{:query {QUERY} :result-transform (fn [rows] (sort [\"a\" 1]))}}");
        let result = run_advanced(&parse(&source), &pages()).unwrap();
        assert_eq!(result.rows, [2, 0, 1]);
        assert_eq!(result.warnings.len(), 1);
        assert!(result.warnings[0].contains("Cannot compare"));
    }

    #[test]
    fn failures_name_the_form_the_stage_and_the_limit() {
        let source = format!("{{:query {QUERY} :result-transform (fn [rows] (partition 2 rows))}}");
        let failure = run_advanced(&parse(&source), &pages()).unwrap_err();
        assert_eq!(failure.stage, Stage::Query);
        assert!(failure.message.contains("partition"));
        assert_eq!(failure.offset, Some(source.find("partition").unwrap()));
        let source = format!("{{:query {QUERY} :view (fn [rows] (slurp \"x\"))}}");
        let failure = run_advanced(&parse(&source), &pages()).unwrap_err();
        assert_eq!(failure.stage, Stage::View);
        assert!(failure.message.contains("slurp"));
        let source = format!("{{:query {QUERY} :view (fn [rows] (compare \"a\" 1))}}");
        let failure = run_advanced(&parse(&source), &pages()).unwrap_err();
        assert_eq!(failure.stage, Stage::View);
        assert!(failure.message.contains("Cannot compare a to 1"));
        assert_eq!(failure.count, Some(3));
        let source = format!("{{:query {QUERY} :view (fn [rows] (:db/id rows))}}");
        assert!(
            run_advanced(&parse(&source), &pages())
                .unwrap_err()
                .message
                .contains(":db/id")
        );
        let source = format!("{{:query {QUERY} :view (fn [rows] [:div.card])}}");
        let failure = run_advanced(&parse(&source), &pages()).unwrap_err();
        assert_eq!(failure.stage, Stage::View);
        assert!(failure.message.contains("div.card"));
        let source = "{:query [:find (pull ?b [*]) :where [?b :block/marker ?m]]}";
        let failure = run_advanced(&parse(source), &pages()).unwrap_err();
        assert_eq!(failure.stage, Stage::Query);
        assert!(failure.message.contains(":block/marker"));
        let source = format!("{{:query {QUERY} :inputs [:today]}}");
        assert_eq!(
            run_advanced(&parse(&source), &pages()).unwrap_err().stage,
            Stage::Query
        );
        // A transform that returns something a table cannot show.
        let source = format!("{{:query {QUERY} :result-transform (fn [rows] (map str rows))}}");
        let failure = run_advanced(&parse(&source), &pages()).unwrap_err();
        assert!(failure.message.contains("add a :view"));
        // A view that reads an attribute of a row at run time.
        let source = format!(
            "{{:query {QUERY} :view (fn [rows] (get (first rows) (first [:block/page])))}}"
        );
        assert!(run_advanced(&parse(&source), &pages()).is_err());
    }

    #[test]
    fn a_runaway_function_reaches_the_step_limit() {
        // Each `let` doubles the collection.
        let mut body = String::from("(let [a rows");
        let mut previous = "a".to_owned();
        for n in 0..40 {
            body.push_str(&format!(" b{n} (concat {previous} {previous})"));
            previous = format!("b{n}");
        }
        body.push_str(&format!("] {previous})"));
        let source = format!("{{:query {QUERY} :result-transform (fn [rows] {body})}}");
        let started = std::time::Instant::now();
        let failure = run_advanced(&parse(&source), &pages()).unwrap_err();
        assert_eq!(failure.stage, Stage::Query);
        assert!(failure.message.contains("limit of 20000000 steps"));
        assert!(started.elapsed().as_secs() < 2);
    }

    #[test]
    fn two_runs_give_the_same_rows_and_steps() {
        let query = parse(&map(true, true));
        let first = run_advanced(&query, &pages()).unwrap();
        let second = run_advanced(&query, &pages()).unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn a_renamed_property_and_type_give_the_same_result() {
        let renamed_query = map(true, true)
            .replace(":logseq-entity", ":kind-of")
            .replace("Type/Episode", "Other/Show")
            .replace(":public", ":listed")
            .replace(":at", ":when");
        let renamed_pages: Vec<PageRecord> = pages()
            .into_iter()
            .map(|mut page| {
                for (key, value) in &mut page.properties {
                    *key = match key.as_str() {
                        "logseq-entity" => "kind-of".into(),
                        "public" => "listed".into(),
                        "at" => "when".into(),
                        other => other.to_owned(),
                    };
                    *value = value.replace("Type/Episode", "Other/Show");
                }
                page
            })
            .collect();
        let original = run_advanced(&parse(&map(true, true)), &pages()).unwrap();
        let renamed = run_advanced(&parse(&renamed_query), &renamed_pages).unwrap();
        assert_eq!(original, renamed);
    }

    /// The worst case in the design: every page of the encode garden (16,693), sorted
    /// with gitpa's comparator and shown through a view of the first 1,000.
    #[test]
    fn sorting_every_page_of_the_encode_garden_fits_the_step_limit() {
        let pages: Vec<PageRecord> = (0..16_693)
            .map(|n| {
                page(
                    &format!("Page {:05}", (n * 7919) % 16_693),
                    &[("at", &format!("2026-01-{:02}T10:00:00Z", n % 28 + 1))],
                )
            })
            .collect();
        let source = format!(
            "{{:query [:find (pull ?p [*]) :where [?p :block/name]] :result-transform {TRANSFORM} :view (fn [rows] (into [\"ul\"] (map (fn [row] [\"li\" (:block/original-name row)]) (take 1000 rows))))}}"
        );
        let started = std::time::Instant::now();
        let result = run_advanced(&parse(&source), &pages).unwrap();
        eprintln!(
            "all pages: {} steps in {:?}",
            result.steps,
            started.elapsed()
        );
        assert_eq!(result.count, 16_693);
        assert!(result.steps < 20_000_000 / 2, "{}", result.steps);
    }
}
