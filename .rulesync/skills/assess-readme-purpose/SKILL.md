---
name: assess-readme-purpose
description: Evaluate whether README.md gives a first-time visitor the information needed to understand, trust, install, and try Logseq Gardener, with links to focused details. Use after changing the README or when asked whether it serves its purpose.
targets: ["agentsskills", "claudecode", "codexcli"]
license: MIT
---

# Assess README purpose

Evaluate the root `README.md` as a first-time visitor's path from discovery to first use. Report findings; do not edit the file unless the user separately asks for changes.

## Procedure

1. Read `README.md` with line numbers.
2. Check each question against repository evidence rather than expectation:
   - What will Logseq Gardener help users do, and what does the foundation currently implement?
   - Who is it for?
   - Why do garden relationships matter?
   - Why would someone choose Logseq Gardener's approach?
   - What is the project's maturity?
   - Is there a current release, and which operating systems and processors does it support?
   - Where can a reader verify continuous integration, maintenance, help, and the license?
   - Does the README show the global and the project mise install commands before any other installation method?
   - Does it link directly and descriptively to the site's install pages, starting with [lsg/How To/Install with mise](https://codekiln.github.io/logseq-gardener/lsg/How%20To/Install%20with%20mise/), for checksums, other platforms and building from source?
   - Can a reader run `lsg help outline` after the documented install, understand the shown result, and navigate a section and inspect the version?
   - Is the quick path designed to finish in under five minutes? If the text alone cannot establish timing, record uncertainty and request execution evidence.
   - Can a reader find focused guidance for CLI use, architecture, and contribution, with links to pages on the documentation site at https://codekiln.github.io/logseq-gardener/?
3. Check the order. Purpose, audience, value, status, installation, and first use must precede reference, development, and historical detail.
4. Apply progressive disclosure. A concise installation example plus a descriptive installation-guide link is complete. System-by-system procedures in the README are premature detail. Deferred information with no findable path is missing information.
5. Verify commands, claims, and links against repository files and current release evidence when available. State uncertainty when verification is unavailable.

## Output

Use this structure:

```text
Verdict: PASS | FAIL

Problems
- README.md:<line or heading> — <visitor question>: <missing, misplaced, or unsupported information>. <effect on the visitor>. <required correction direction>.

Suggestions
- README.md:<line or heading> — <optional improvement>.

Uncertainty
- <claim or timing that available evidence cannot establish>.
```

Write `None.` under an empty section. `PASS` means there are no unresolved problems; suggestions do not cause failure. Do not use a numeric score.

## Examples

Incomplete entry point:

> Logseq Gardener is written in Rust and emits a typed report document.

Expected findings: purpose, audience, value, and reason to choose Logseq Gardener remain unanswered. Implementation information appears before visitor questions.

Excessive installation section:

> Download these files and follow these separate extraction procedures for macOS, Linux, and Windows.

Expected finding: organization and progressive disclosure. Keep the mise install commands in the README and link to [lsg/How To/Install with mise](https://codekiln.github.io/logseq-gardener/lsg/How%20To/Install%20with%20mise/) for complete system-specific instructions.

Complete first-use path:

> Install with `mise use -g github:codekiln/logseq-gardener`, then run `lsg help outline`. For checksums, other platforms and building from source, see [Install with mise](https://codekiln.github.io/logseq-gardener/lsg/How%20To/Install%20with%20mise/).

Expected result: the installation structure passes if the commands work, the linked pages cover every supported download, and nearby text explains the first result and how to check another command.
