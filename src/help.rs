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
    let version = !path.is_empty();
    let command = if version { "lsg version" } else { "lsg" };
    let mut sections = vec![
        Section { level: 1, title: "Purpose", section: "purpose", content: if version {
            "Report the installed Logseq Gardener version. Use this when reporting a bug or identifying a build.".into()
        } else {
            "Logseq Gardener provides a terminal interface for people and coding agents working with Logseq Markdown gardens. This foundation offers offline help and version reporting. Garden queries will follow the parser comparison.".into()
        }},
        Section { level: 2, title: "Usage", section: "usage", content: format!(
            "Usage: {command}{}\n       {command} help [outline | section <section> [--recursive]]\n       {command} --help\n\nOptions:\n  -h, --help          Show this documentation\n  -V, --version       Report the installed version\n  --format human|json Select output format (default: human)\n  --programmatic     Add guidance for scripts and agents\n  --no-input         Complete without prompts (also the default)\n\nOutline options: --level N or --max-level N (1 through 6).\nSection option: --recursive includes descendant sections.",
            if version { " [--format human|json]" } else { " [version]" }
        )},
        Section { level: 2, title: "Commands", section: "commands", content: if version {
            "No child commands. Use help to read this document, help outline to list headings, or help section <section> to retrieve a heading.".into()
        } else {
            "version  Report the installed version.\n\nUse lsg version help for version documentation. Use lsg help outline to discover headings and lsg help section <section> to read a section.".into()
        }},
        Section { level: 2, title: "Behavior", section: "behavior", content: "These commands read embedded documentation and version metadata. They work offline, never read standard input, and finish without prompts or a pager. They do not read or modify garden files, create caches, or access the network.".into() },
        Section { level: 3, title: "Permissions", section: "permissions", content: "Permission to execute lsg is sufficient. No garden access or account credentials are required.".into() },
        Section { level: 2, title: "Examples", section: "examples", content: if version {
            "lsg version --format json\n\nReturns the program name and installed version in one JSON document, for use in a bug report or script.".into()
        } else {
            "lsg help outline\n\nLists the available documentation headings. Copy a section identifier into:\n\nlsg help section behavior --recursive\n\nPrints command behavior and its permissions section. To identify your installation, run lsg version.".into()
        }},
        Section { level: 2, title: "Output and exits", section: "output", content: "Results go to stdout. JSON uses format_version 1 and command_path. Diagnostics go to stderr; invalid commands leave stdout empty. Exit 0 means success (including a closed downstream pipe), 2 means invalid arguments or an unknown section, and 1 means output could not be written. Redirected output contains no terminal control sequences.".into() },
    ];
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
                    if path.is_empty() {
                        "lsg"
                    } else {
                        "lsg version"
                    }
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
                vec![json!({"name": "version"})]
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
            format!("{value}\n")
        }
    })
}
