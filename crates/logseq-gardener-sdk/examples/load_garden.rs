//! Run with: cargo run -p logseq-gardener-sdk --example load_garden -- <garden-root>

use std::env;
use std::error::Error;

use logseq_gardener_sdk::garden::{DocumentKind, load_garden};

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = env::args_os().skip(1);
    let root = args.next().ok_or("usage: load_garden <garden-root>")?;
    if args.next().is_some() {
        return Err("usage: load_garden <garden-root>".into());
    }
    let garden = load_garden(root)?;
    let pages = garden
        .documents
        .iter()
        .filter(|document| document.kind == DocumentKind::Page)
        .count();
    let journals = garden.documents.len() - pages;
    let page_refs: usize = garden
        .documents
        .iter()
        .map(|document| document.parsed.refs.page.len())
        .sum();
    let block_refs: usize = garden
        .documents
        .iter()
        .map(|document| document.parsed.refs.block.len())
        .sum();
    println!(
        "Loaded {} Markdown documents: {pages} pages, {journals} journals; {page_refs} page references, {block_refs} block references.",
        garden.documents.len()
    );
    Ok(())
}
