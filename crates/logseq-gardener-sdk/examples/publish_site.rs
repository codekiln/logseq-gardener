//! Generate a selected local HTML garden; see docs/local-site.md for invocation.
use std::env;
use std::error::Error;
use std::path::PathBuf;

use logseq_gardener_sdk::page_titles::FilenameFormat;
use logseq_gardener_sdk::publishing::NamespaceSelection;
use logseq_gardener_sdk::site::publish_site;

fn main() -> Result<(), Box<dyn Error>> {
    let usage = "usage: publish_site <garden-root> <new-output-directory> <legacy|triple-lowbar> [--include <namespace>] [--exclude <namespace>] ...";
    let mut args = env::args_os().skip(1);
    let root = PathBuf::from(args.next().ok_or(usage)?);
    let output = PathBuf::from(args.next().ok_or(usage)?);
    let format = match args.next().as_deref().and_then(|value| value.to_str()) {
        Some("legacy") => FilenameFormat::Legacy,
        Some("triple-lowbar") => FilenameFormat::TripleLowbar,
        _ => return Err(usage.into()),
    };
    let mut included = Vec::new();
    let mut excluded = Vec::new();
    while let Some(flag) = args.next() {
        let roots = match flag.to_str() {
            Some("--include") => &mut included,
            Some("--exclude") => &mut excluded,
            _ => return Err(usage.into()),
        };
        roots.push(
            args.next()
                .ok_or(usage)?
                .into_string()
                .map_err(|_| "namespace must be UTF-8")?,
        );
    }
    let include: Vec<&str> = included.iter().map(String::as_str).collect();
    let exclude: Vec<&str> = excluded.iter().map(String::as_str).collect();
    let policy = NamespaceSelection::new(&include, &exclude)?;
    let report = publish_site(root, &output, format, &policy)?;
    println!(
        "Published {} pages and {} assets; withheld {} pages; skipped {} journals. Open {}/index.html",
        report.pages,
        report.assets,
        report.withheld_pages,
        report.skipped_journals,
        output.display()
    );
    for diagnostic in &report.diagnostics {
        eprintln!("{}: {}", diagnostic.source.display(), diagnostic.message);
    }
    eprintln!("{} publication diagnostics.", report.diagnostics.len());
    Ok(())
}
