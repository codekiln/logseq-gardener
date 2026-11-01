//! Writes the page and block relationships the SDK derives from a whole graph, for
//! `mise run docs:parser-check` to compare with what Logseq's own file parser builds.
//!
//! The check sets `LSG_PARSER_CHECK_GRAPH` to the graph folder,
//! `LSG_PARSER_CHECK_FILENAME_FORMAT` to `legacy` or `triple-lowbar`, and
//! `LSG_PARSER_CHECK_OUTPUT` to the JSON file to write.

mod relationships;

use std::path::Path;

use logseq_gardener_sdk::page_titles::FilenameFormat;

fn variable(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("set {name}; mise run docs:parser-check does"))
}

#[test]
#[ignore = "mise run docs:parser-check runs it on a graph"]
fn write_graph_relationships() {
    let format = match variable("LSG_PARSER_CHECK_FILENAME_FORMAT").as_str() {
        "legacy" => FilenameFormat::Legacy,
        "triple-lowbar" => FilenameFormat::TripleLowbar,
        other => panic!("unknown filename format {other}"),
    };
    let derived = relationships::derive(Path::new(&variable("LSG_PARSER_CHECK_GRAPH")), format);
    let json = serde_json::to_string_pretty(&derived).unwrap();
    std::fs::write(variable("LSG_PARSER_CHECK_OUTPUT"), json + "\n").unwrap();
}
