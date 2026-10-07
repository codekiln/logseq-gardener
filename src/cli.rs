use clap::{Args, Parser, Subcommand, ValueEnum};
use logseq_gardener_sdk::{
    page_titles::FilenameFormat, publishing::NamespaceSelection, site::publish_site,
};
use serde_json::json;
use std::path::PathBuf;

use crate::{help, output};

#[derive(Clone, Copy, Default, ValueEnum)]
pub enum Format {
    #[default]
    Human,
    Json,
}

#[derive(Parser)]
#[command(name = "lsg", disable_help_flag = true, disable_help_subcommand = true)]
struct Cli {
    #[arg(long, global = true, value_enum, default_value = "human")]
    format: Format,
    #[arg(short = 'h', long, global = true)]
    help: bool,
    #[arg(short = 'V', long, global = true)]
    version: bool,
    #[arg(long, global = true)]
    programmatic: bool,
    #[arg(long, global = true)]
    no_input: bool,
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    Help(HelpArgs),
    Version(VersionArgs),
    Publish(PublishArgs),
}

#[derive(Args, Default)]
#[command(disable_help_flag = true, disable_help_subcommand = true)]
pub struct HelpArgs {
    #[command(subcommand)]
    pub operation: Option<HelpOperation>,
}

#[derive(Subcommand)]
pub enum HelpOperation {
    Outline {
        #[arg(long, value_parser = clap::value_parser!(u8).range(1..=6), conflicts_with = "max_level")]
        level: Option<u8>,
        #[arg(long, value_parser = clap::value_parser!(u8).range(1..=6))]
        max_level: Option<u8>,
    },
    Section {
        section: String,
        #[arg(long)]
        recursive: bool,
    },
}

#[derive(Args)]
#[command(disable_help_flag = true, disable_help_subcommand = true)]
struct VersionArgs {
    #[command(subcommand)]
    command: Option<VersionCommand>,
}

#[derive(Subcommand)]
enum VersionCommand {
    Help(HelpArgs),
}

#[derive(Args)]
#[command(disable_help_flag = true, disable_help_subcommand = true)]
struct PublishArgs {
    #[arg(long)]
    graph: Option<PathBuf>,
    #[arg(long)]
    output: Option<PathBuf>,
    #[arg(long, value_enum)]
    filename_format: Option<PublishFilenameFormat>,
    #[arg(long)]
    include: Vec<String>,
    #[arg(long)]
    exclude: Vec<String>,
    #[command(subcommand)]
    command: Option<PublishCommand>,
}

#[derive(Clone, Copy, ValueEnum)]
enum PublishFilenameFormat {
    Legacy,
    TripleLowbar,
}

#[derive(Subcommand)]
enum PublishCommand {
    Help(HelpArgs),
}

pub struct Response {
    pub stdout: String,
    pub stderr: String,
}
impl From<String> for Response {
    fn from(stdout: String) -> Self {
        Self {
            stdout,
            stderr: String::new(),
        }
    }
}
pub struct Failure {
    pub message: String,
    pub code: u8,
}
impl From<String> for Failure {
    fn from(message: String) -> Self {
        Self { message, code: 2 }
    }
}

pub fn run() -> Result<Response, Failure> {
    let cli = Cli::try_parse().map_err(|e| e.to_string())?;
    let path: &[&str] = match &cli.command {
        Some(Command::Version(_)) => &["version"],
        Some(Command::Publish(_)) => &["publish"],
        _ => &[],
    };
    if cli.help {
        return Ok(help::render(path, &HelpArgs::default(), cli.format, cli.programmatic)?.into());
    }
    if cli.version {
        return Ok(version(cli.format)?.into());
    }
    let text = match cli.command {
        Some(Command::Version(VersionArgs { command: None })) => version(cli.format),
        Some(Command::Version(VersionArgs {
            command: Some(VersionCommand::Help(args)),
        }))
        | Some(Command::Help(args)) => help::render(path, &args, cli.format, cli.programmatic),
        Some(Command::Publish(PublishArgs {
            command: Some(PublishCommand::Help(args)),
            ..
        })) => help::render(path, &args, cli.format, cli.programmatic),
        Some(Command::Publish(args)) => return publish(args, cli.format),
        None => help::render(path, &HelpArgs::default(), cli.format, cli.programmatic),
    }?;
    Ok(text.into())
}

fn version(format: Format) -> Result<String, String> {
    Ok(match format {
        Format::Human => format!("lsg {}\n", env!("CARGO_PKG_VERSION")),
        Format::Json => {
            output::json_document(&json!({"format_version": 1, "command_path": ["version"],
                "program": "lsg", "version": env!("CARGO_PKG_VERSION"),
                "sdk_version": logseq_gardener_sdk::VERSION}))
        }
    })
}

fn publish(args: PublishArgs, format: Format) -> Result<Response, Failure> {
    let required =
        |name: &str| Failure::from(format!("missing required --{name}; use 'lsg publish help'"));
    let graph = args.graph.ok_or_else(|| required("graph"))?;
    let output = args.output.ok_or_else(|| required("output"))?;
    let filename_format = match args
        .filename_format
        .ok_or_else(|| required("filename-format"))?
    {
        PublishFilenameFormat::Legacy => FilenameFormat::Legacy,
        PublishFilenameFormat::TripleLowbar => FilenameFormat::TripleLowbar,
    };
    let included: Vec<&str> = args.include.iter().map(String::as_str).collect();
    let excluded: Vec<&str> = args.exclude.iter().map(String::as_str).collect();
    let selection = NamespaceSelection::new(&included, &excluded)
        .map_err(|error| Failure::from(format!("{error}; each slash-separated segment must be nonempty, have no surrounding whitespace or control characters; use 'lsg publish help'")))?;
    let report =
        publish_site(&graph, &output, filename_format, &selection).map_err(|error| Failure {
            message: format!("publication failed: {error}; use 'lsg publish help'"),
            code: 1,
        })?;
    let index = output.join("index.html");
    let stdout = match format {
        Format::Human => output::clean(&format!(
            "Published {} pages and {} assets; withheld {} pages; skipped {} journals; {} diagnostics. Open {}\n",
            report.pages,
            report.assets,
            report.withheld_pages,
            report.skipped_journals,
            report.diagnostics.len(),
            index.display()
        )),
        Format::Json => {
            output::json_document(&json!({"format_version": 1, "command_path": ["publish"],
            "pages": report.pages, "assets": report.assets, "withheld_pages": report.withheld_pages,
            "skipped_journals": report.skipped_journals, "diagnostic_count": report.diagnostics.len(),
            "output_directory": output.to_string_lossy(), "index_path": index.to_string_lossy()}))
        }
    };
    let stderr = report
        .diagnostics
        .iter()
        .map(|diagnostic| format!("{}: {}\n", diagnostic.source.display(), diagnostic.message))
        .collect();
    Ok(Response { stdout, stderr })
}
