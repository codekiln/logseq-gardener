//! Publish a garden with an in-process build plugin that reads every block and contributes
//! nothing; `--no-plugin` publishes the same garden without it, for comparison.
use std::env;
use std::error::Error;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use logseq_gardener_sdk::contributions::{BuildPlugin, BuildPlugins, Contribution};
use logseq_gardener_sdk::page_titles::FilenameFormat;
use logseq_gardener_sdk::publication::PluginView;
use logseq_gardener_sdk::publishing::NamespaceSelection;
use logseq_gardener_sdk::site::{SiteOptions, publish_site_with_options};

/// Counts the view's blocks and returns no contributions.
struct NoOp {
    blocks: Arc<AtomicUsize>,
}

impl BuildPlugin for NoOp {
    fn id(&self) -> &str {
        "noop-in-process"
    }
    fn wants_blocks(&self) -> bool {
        true
    }
    fn contribute(&self, view: &PluginView<'_>) -> Result<Vec<Contribution>, String> {
        self.blocks.store(view.blocks().len(), Ordering::Relaxed);
        Ok(Vec::new())
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let usage = "usage: build_plugin <garden-root> <new-output-directory> <legacy|triple-lowbar> [--include <namespace>]... [--journals] [--no-plugin]";
    let mut args = env::args().skip(1);
    let root = PathBuf::from(args.next().ok_or(usage)?);
    let output = PathBuf::from(args.next().ok_or(usage)?);
    let format = match args.next().as_deref() {
        Some("legacy") => FilenameFormat::Legacy,
        Some("triple-lowbar") => FilenameFormat::TripleLowbar,
        _ => return Err(usage.into()),
    };
    let mut included = Vec::new();
    let mut options = SiteOptions::default();
    let mut plugin = true;
    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--include" => included.push(args.next().ok_or(usage)?),
            "--journals" => options.journals = true,
            "--no-plugin" => plugin = false,
            _ => return Err(usage.into()),
        }
    }
    let blocks = Arc::new(AtomicUsize::new(0));
    if plugin {
        let mut plugins = BuildPlugins::default();
        plugins.push(NoOp {
            blocks: Arc::clone(&blocks),
        });
        options.build_plugins = plugins;
    }
    let include: Vec<&str> = included.iter().map(String::as_str).collect();
    let policy = NamespaceSelection::new(&include, &[])?;
    let report = publish_site_with_options(root, &output, format, &policy, &options)?;
    println!(
        "Published {} pages and {} journals; the plugin saw {} blocks.",
        report.pages,
        report.journals,
        blocks.load(Ordering::Relaxed)
    );
    Ok(())
}
