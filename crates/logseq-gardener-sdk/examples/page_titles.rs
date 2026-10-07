//! Run with: cargo run -p logseq-gardener-sdk --example page_titles -- <garden-root> <legacy|triple-lowbar>

use std::env;
use std::error::Error;

use logseq_gardener_sdk::garden::{DocumentKind, load_garden};
use logseq_gardener_sdk::page_titles::{FilenameFormat, page_title};

fn main() -> Result<(), Box<dyn Error>> {
    let usage = "usage: page_titles <garden-root> <legacy|triple-lowbar>";
    let mut args = env::args_os().skip(1);
    let root = args.next().ok_or(usage)?;
    let format = match args.next().as_deref().and_then(|arg| arg.to_str()) {
        Some("legacy") => FilenameFormat::Legacy,
        Some("triple-lowbar") => FilenameFormat::TripleLowbar,
        _ => return Err(usage.into()),
    };
    if args.next().is_some() {
        return Err(usage.into());
    }
    let garden = load_garden(root)?;
    let mut pages = 0;
    let mut namespaces = 0;
    let mut journals = 0;
    for document in &garden.documents {
        if document.kind == DocumentKind::Journal {
            journals += 1;
            continue;
        }
        let title = page_title(document, format)?;
        pages += 1;
        if title.contains('/') {
            namespaces += 1;
        }
    }
    println!(
        "Derived {pages} page titles: {namespaces} contain namespace separators; skipped {journals} journals."
    );
    Ok(())
}
