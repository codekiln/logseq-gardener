---
name: assess-readme-style
description: Evaluate README.md against Logseq Gardener's communication style, including plain language, ambiguity, distracting wording, and progressive disclosure. Use after changing the README or when asked to review its writing.
targets: ["agentsskills", "claudecode", "codexcli"]
license: MIT
---

# Assess README style

Evaluate the root `README.md`. Report findings; do not edit the file unless the user separately asks for changes.

## Procedure

1. Read `.rulesync/rules/communication-style.md` completely.
2. Read `README.md` with line numbers.
3. Identify the intended first-time reader and the decision or task served by each section.
4. Review the whole document for main-point-first organization and progressive disclosure.
5. Review each section for information that appears before it is useful or detail that was deferred without a descriptive path to it.
6. Review each sentence against these named criteria:
   - reader context;
   - main point and information order;
   - plain and specific language;
   - one clear meaning;
   - active voice and visible actor;
   - sentence value;
   - jargon and uncommon abbreviations;
   - coined phrases and word salad;
   - double negatives and unnecessary contrasts;
   - distracting figurative language;
   - evidence and honest uncertainty;
   - calm and inclusive tone;
   - relevant implementation detail; and
   - separation of authoring instructions from human content.
7. Report a problem only when the README violates a criterion. Keep optional improvements separate.

## Output

Use this structure:

```text
Verdict: PASS | FAIL

Problems
- README.md:<line or range> — <criterion>: <quoted or precisely identified evidence>. <effect on the reader>. <required correction direction>.

Suggestions
- README.md:<line or range> — <optional improvement>.

Uncertainty
- <anything the repository evidence cannot establish>.
```

Write `None.` under an empty section. `PASS` means there are no unresolved problems; suggestions do not cause failure. Do not use a numeric score. Cite the tightest useful line range and state uncertainty instead of guessing.

## Examples

Unclear opening:

> Clilint operationalizes typed invocation evidence across versioned conformance packages.

Expected finding: `plain and specific language` and `reader context`. The sentence requires unexplained project vocabulary before it tells a new reader what Clilint helps them do.

Premature detail:

> Install on macOS with these six commands. Install on Linux with these seven commands. Install on Windows with these eight steps.

Expected finding: `progressive disclosure`. The README should show one common path and link to a focused installation guide for system-specific procedures.

Clear disclosure:

> Install with mise: `mise use -g github:codekiln/clilint`. See [Installation](docs/installation.md) for other methods and download verification.

Expected result: no style problem. The common action is available immediately and the descriptive link keeps deeper detail findable.
