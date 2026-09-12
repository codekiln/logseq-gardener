use clap::{Args, Parser, Subcommand, ValueEnum};
use serde_json::json;

use crate::help;

#[derive(Clone, Copy, Default, ValueEnum)]
pub enum Format {
    #[default]
    Human,
    Json,
}

#[derive(Parser)]
#[command(name = "lsg",
    color = clap::ColorChoice::Never, disable_help_flag = true, disable_help_subcommand = true)]
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

pub fn run() -> Result<String, String> {
    let cli = Cli::try_parse().map_err(|e| e.to_string())?;
    let path: &[&str] = if matches!(cli.command, Some(Command::Version(_))) {
        &["version"]
    } else {
        &[]
    };
    if cli.help {
        return help::render(path, &HelpArgs::default(), cli.format, cli.programmatic);
    }
    if cli.version {
        return version(cli.format);
    }
    match cli.command {
        Some(Command::Version(VersionArgs { command: None })) => version(cli.format),
        Some(Command::Version(VersionArgs {
            command: Some(VersionCommand::Help(args)),
        }))
        | Some(Command::Help(args)) => help::render(path, &args, cli.format, cli.programmatic),
        None => help::render(path, &HelpArgs::default(), cli.format, cli.programmatic),
    }
}

fn version(format: Format) -> Result<String, String> {
    Ok(match format {
        Format::Human => format!("lsg {}\n", env!("CARGO_PKG_VERSION")),
        Format::Json => format!(
            "{}\n",
            json!({"format_version": 1, "command_path": ["version"],
                "program": "lsg", "version": env!("CARGO_PKG_VERSION")})
        ),
    })
}
