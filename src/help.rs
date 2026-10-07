use serde::Serialize;
use serde_json::json;

use crate::cli::{Format, HelpArgs, HelpOperation};

#[derive(Serialize)]
struct Section {
    level: u8,
    title: &'static str,
    section: &'static str,
    content: String,
}

fn document(path: &[&str], programmatic: bool) -> Vec<Section> {
    let version = path == ["version"];
    let publish = path == ["publish"];
    let command = if version {
        "lsg version"
    } else if publish {
        "lsg publish"
    } else {
        "lsg"
    };
    let mut sections = vec![
        Section { level: 1, title: "Purpose", section: "purpose", content: if publish {
            "Generate local HTML pages and referenced assets from selected Logseq namespaces through the shared Rust SDK.".into()
        } else if version {
            "Report the installed Logseq Gardener version. Use this when reporting a bug or identifying a build.".into()
        } else {
            "Logseq Gardener provides a terminal interface for people and coding agents working with Logseq Markdown gardens. Use publish to generate selected local HTML pages, or version to identify the installed build.".into()
        }},
        Section { level: 2, title: "Usage", section: "usage", content: format!(
            "Usage: {command}{}\n       {command} help [outline | section <section> [--recursive]]\n       {command} --help\n\nOptions:\n  -h, --help          Show this documentation\n  -V, --version       Report the installed version\n  --format human|json Select output format (default: human)\n  --programmatic     Add guidance for scripts and agents\n  --no-input         Explicit noninteractive mode (the default)\n\nOutline options: --level N or --max-level N (1 through 6).\nSection option: --recursive includes descendant sections.",
            if publish { " --graph PATH --output PATH --filename-format legacy|triple-lowbar [--include NAMESPACE]... [--exclude NAMESPACE]..." } else if version { " [--format human|json]" } else { " [publish | version]" }
        )},
        Section { level: 2, title: "Commands", section: "commands", content: if version || publish {
            "No child commands. Use help to read this document, help outline to list headings, or help section <section> to retrieve a heading.".into()
        } else {
            "publish  Generate selected local HTML pages.\nversion  Report the installed version.\n\nUse lsg publish help for publication documentation and lsg version help for version documentation. Use lsg help outline to discover headings and lsg help section <section> to read a section.".into()
        }},
        Section { level: 2, title: "Behavior", section: "behavior", content: "Help displays documentation included in lsg; version reports the installed build. Both work offline and finish without prompts, a pager, or reading standard input. They leave garden files unchanged.".into() },
        Section { level: 3, title: "Permissions", section: "permissions", content: "You only need permission to run lsg.".into() },
        Section { level: 2, title: "Examples", section: "examples", content: if publish {
            "lsg publish --graph /path/to/garden --output /tmp/new-site --filename-format triple-lowbar --include My/AI --exclude My/AI/Agent\n\nPublishes My/AI and descendants, except My/AI/Agent and descendants. Open /tmp/new-site/index.html. Choose a fresh output directory with an existing parent outside the garden.".into()
        } else if version {
            "lsg version --format json\n\nReturns the program name and installed version in one JSON document, for use in a bug report or script.".into()
        } else {
            "lsg help outline\n\nLists the available documentation headings. Copy a section identifier into:\n\nlsg help section behavior --recursive\n\nPrints command behavior and its permissions section. To identify your installation, run lsg version.".into()
        }},
        Section { level: 2, title: "Output and exits", section: "output", content: "Results go to stdout and diagnostics go to stderr. JSON responses include format_version 1 and command_path. Invalid commands leave stdout empty. Exit 0 means success (including a closed downstream pipe), 2 means invalid arguments or an unknown section, and 1 means output could not be written. Redirected output contains no terminal control sequences.".into() },
    ];
    if path.is_empty() {
        sections.iter_mut().find(|s| s.section == "behavior").unwrap().content = "Help and version work offline without prompts, a pager, or reading stdin. Publish reads a garden and creates a fresh HTML directory outside it, leaving source files unchanged; use lsg publish help for selectors, destination protection and rendering limitations.".into();
        sections.iter_mut().find(|s| s.section == "permissions").unwrap().content = "Help and version only require permission to run lsg. Publishing requires read access to the garden and referenced assets, plus write access to the destination parent.".into();
    }
    if publish {
        sections.iter_mut().find(|s| s.section == "behavior").unwrap().content = "Reads the garden and writes a new directory outside it; source files stay unchanged. Selection is case-sensitive, matches roots and slash-separated descendants, and exclusions win. Repeat --include and --exclude; an empty include list selects nothing. Each namespace segment must be nonempty with no surrounding whitespace or controls. --graph, --output and --filename-format are required for publishing. Use triple-lowbar for :file/name-format :triple-lowbar in logseq/config.edn, otherwise legacy. Existing output is refused. A write failure may leave partial new output; retry with a fresh destination after fixing the error. Works offline with no prompts or stdin.".into();
        sections.iter_mut().find(|s| s.section == "permissions").unwrap().content = "Requires read access to the source garden and referenced assets, and write access to the existing parent of the new output directory.".into();
        sections.push(Section { level: 2, title: "Limitations", section: "limitations", content: "Any parsed public:: false property withholds the whole page; excessive nesting also withholds pages. Journals are skipped pending configured date identities. Selected display titles resolve page links; aliases and Markdown page-filename links need later lookup work. Block references, embeds, queries, remote images, raw HTML and Hiccup receive visible fallbacks and local diagnostics. Only supported referenced assets beneath assets/ are copied; unsafe asset paths are refused. See docs/local-site.md for supported types and rendering limits.".into() });
    }
    if !version {
        sections.iter_mut().find(|s| s.section == "output").unwrap().content = "Results go to stdout and diagnostics go to stderr. JSON responses include format_version 1 and command_path. Publish reports pages, assets, withheld_pages, skipped_journals, diagnostic_count, output_directory and index_path (paths are display strings). Unsupported content with a completed site exits 0. Fatal generation failures leave stdout empty and exit 1. Exit 2 means invalid arguments or an unknown section. Output write failure exits 1; a closed downstream pipe exits 0. Redirected output contains no terminal control sequences.".into();
    }
    if programmatic {
        sections.push(Section { level: 2, title: "Programmatic use", section: "programmatic", content: format!(
            "Pipe or redirect stdout directly; no interactive display or pager is used. Use {command} help --format json for immediate child_commands. Request each child's help to discover the hierarchy offline. Use {command} help outline --format json and {command} help section behavior --format json to retrieve sections. --recursive includes descendants. All JSON documents carry format_version 1. Stdin may be closed; --no-input is accepted explicitly. Check the exit code and keep stderr separate from JSON stdout."
        ) });
    }
    sections
}

pub fn render(
    path: &[&str],
    args: &HelpArgs,
    format: Format,
    programmatic: bool,
) -> Result<String, String> {
    let sections = document(path, programmatic);
    let selected: Vec<&Section> = match &args.operation {
        None => sections.iter().collect(),
        Some(HelpOperation::Outline { level, max_level }) => sections
            .iter()
            .filter(|s| {
                level.is_none_or(|l| s.level == l) && max_level.is_none_or(|l| s.level <= l)
            })
            .collect(),
        Some(HelpOperation::Section { section, recursive }) => {
            let mut selected = Vec::new();
            for (index, s) in sections.iter().enumerate() {
                if s.section == section {
                    selected.push(s);
                    if *recursive {
                        selected.extend(
                            sections[index + 1..]
                                .iter()
                                .take_while(|child| child.level > s.level),
                        );
                    }
                }
            }
            if selected.is_empty() {
                return Err(format!(
                    "unknown section {section:?}; use '{} help outline'",
                    format!("lsg {}", path.join(" ")).trim_end()
                ));
            }
            selected
        }
    };
    Ok(match format {
        Format::Human => selected
            .iter()
            .map(|s| {
                if matches!(args.operation, Some(HelpOperation::Outline { .. })) {
                    format!(
                        "{} {} ({})\n",
                        "#".repeat(usize::from(s.level)),
                        s.title,
                        s.section
                    )
                } else {
                    format!(
                        "{} {}\n\n{}\n\n",
                        "#".repeat(usize::from(s.level)),
                        s.title,
                        s.content
                    )
                }
            })
            .collect(),
        Format::Json => {
            let children = if path.is_empty() {
                vec![json!({"name": "publish"}), json!({"name": "version"})]
            } else {
                vec![]
            };
            let mut value = json!({"format_version": 1, "command_path": path, "programmatic": programmatic, "child_commands": children});
            if matches!(args.operation, Some(HelpOperation::Outline { .. })) {
                value["headings"] = json!(
                    selected
                        .iter()
                        .map(|s| json!({"level": s.level, "title": s.title, "section": s.section}))
                        .collect::<Vec<_>>()
                );
            } else {
                value["sections"] = json!(selected);
            }
            crate::output::json_document(&value)
        }
    })
}
