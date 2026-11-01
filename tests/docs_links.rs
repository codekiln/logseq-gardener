//! Checks the links from `lsg` help to the documentation site.
//!
//! `help_links_name_built_pages` needs a built docs site, so it is ignored by default.
//! `mise run docs:check` runs it against the site it builds:
//! `LSG_DOCS_SITE=<site> cargo test --locked --test docs_links -- --ignored`.

use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};

const BASE_URL: &str = "https://codekiln.github.io/logseq-gardener/";

fn run(args: &[&str]) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_lsg"))
        .args(args)
        .stdin(Stdio::null())
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "lsg {}: {output:?}",
        args.join(" ")
    );
    String::from_utf8(output.stdout).unwrap()
}

/// Every command path, found by following `child_commands` in JSON help from the root.
fn command_paths() -> Vec<Vec<String>> {
    let mut paths = Vec::new();
    let mut pending = vec![Vec::<String>::new()];
    while let Some(path) = pending.pop() {
        let mut args: Vec<&str> = path.iter().map(String::as_str).collect();
        args.extend(["help", "--format", "json"]);
        let help: Value = serde_json::from_str(&run(&args)).unwrap();
        for child in help["child_commands"].as_array().unwrap() {
            let mut child_path = path.clone();
            child_path.push(child["name"].as_str().unwrap().to_owned());
            pending.push(child_path);
        }
        paths.push(path);
    }
    paths
}

/// The full help of every command in human and JSON output, with and without
/// `--programmatic`, keyed by the command line that printed it.
fn help_documents() -> BTreeMap<String, String> {
    let mut documents = BTreeMap::new();
    for path in command_paths() {
        for format in ["human", "json"] {
            for programmatic in [false, true] {
                let mut args: Vec<&str> = path.iter().map(String::as_str).collect();
                args.extend(["help", "--format", format]);
                if programmatic {
                    args.push("--programmatic");
                }
                documents.insert(format!("lsg {}", args.join(" ")), run(&args));
            }
        }
    }
    documents
}

/// The route after the base URL of each documentation link in `text`. A readable route holds
/// only unreserved characters, percent escapes and `/`; a `.` after the link ends a sentence.
fn routes(text: &str) -> Vec<&str> {
    text.match_indices(BASE_URL)
        .map(|(start, _)| {
            let rest = &text[start + BASE_URL.len()..];
            let end = rest
                .find(|c: char| !(c.is_ascii_alphanumeric() || "-._~%/".contains(c)))
                .unwrap_or(rest.len());
            rest[..end].trim_end_matches('.')
        })
        .collect()
}

#[test]
#[ignore = "needs a built docs site; mise run docs:check sets LSG_DOCS_SITE and runs it"]
fn help_links_name_built_pages() {
    let site = std::env::var_os("LSG_DOCS_SITE")
        .expect("set LSG_DOCS_SITE to a docs site built by mise run docs:site");
    let manifest_path = Path::new(&site).join("lsg-site.json");
    let manifest = fs::read_to_string(&manifest_path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", manifest_path.display()));
    let manifest: Value = serde_json::from_str(&manifest).unwrap();
    let pages: BTreeSet<&str> = manifest["pages"]
        .as_array()
        .unwrap()
        .iter()
        .map(|page| page["url"].as_str().unwrap())
        .collect();

    let mut links = 0;
    let mut missing = BTreeMap::<String, &str>::new();
    let documents = help_documents();
    for (command, text) in &documents {
        for route in routes(text) {
            links += 1;
            let url = if route.is_empty() {
                "index.html"
            } else {
                route
            };
            if !pages.contains(url) {
                missing
                    .entry(format!("{BASE_URL}{route}"))
                    .or_insert(command);
            }
        }
    }
    assert!(links > 0, "help links no documentation page");
    let report: Vec<String> = missing
        .iter()
        .map(|(url, command)| format!("  {url} (printed by {command})"))
        .collect();
    assert!(
        report.is_empty(),
        "help links pages that {} does not list:\n{}",
        manifest_path.display(),
        report.join("\n")
    );
}

#[test]
fn help_names_no_docs_files() {
    for (command, text) in help_documents() {
        assert!(!text.contains("docs/"), "{command} names a docs/ file");
    }
}

#[test]
fn publish_help_links_the_search_data_page() {
    let behavior = run(&["publish", "help", "section", "behavior"]);
    assert!(
        behavior.contains("lsg/Ref/Static Search Data (https://codekiln.github.io/logseq-gardener/lsg/Ref/Static%20Search%20Data/)"),
        "{behavior}"
    );
}

#[test]
fn cli_source_holds_the_base_url_once() {
    let mut count = 0;
    let mut folders = vec![Path::new(env!("CARGO_MANIFEST_DIR")).join("src")];
    while let Some(folder) = folders.pop() {
        for entry in fs::read_dir(folder).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                folders.push(path);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                count += fs::read_to_string(&path).unwrap().matches(BASE_URL).count();
            }
        }
    }
    assert_eq!(count, 1, "src/ should hold {BASE_URL} once");
}
