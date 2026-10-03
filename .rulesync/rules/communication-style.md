---
root: false
targets: ["*"]
description: "Reader-centered communication standard for codekiln/clilint"
---

# Communication style

Use this standard for repository documentation and prose written on behalf of the project. Adjust the amount of explanation to the reader and the task. Do not apply prose rules to Rust formatting or identifiers.

## Write for the reader

Assume the reader is intelligent and may not share your background.

- Put the main point first.
- Order information by the decisions and actions the reader needs to take, not by the order in which you discovered it.
- Prefer common, specific words. Use a technical term only when it adds precision, and explain it on first use.
- Prefer active voice and sentences with one clear meaning.
- State who or what acts. Replace vague references such as "this," "it," or "the system" when more than one meaning is possible.
- Keep a sentence only when it answers a likely reader question or reduces uncertainty.

Plain language must remain accurate. Do not replace a precise term with an easier word that changes the meaning.

## Use progressive disclosure

Progressive disclosure means presenting the main point and the information needed for the reader's current decision or task first, then providing a clear path to deeper detail when it becomes relevant.

This applies to human and agent readers:

- For a human, put the common action before special cases and troubleshooting. Link to a focused guide for the details.
- For an agent, provide enough context to select and begin the task. Direct the agent to load the relevant skill, rule, namespace, specification, or reference before it acts.

Progressive disclosure is not omission. When information moves out of an entry point, name and link or reference the deeper source where the reader needs it. A reader should not have to search for a detail the entry point promises.

Example for a human reader:

- Clear: "Install with mise: `mise use -g github:codekiln/clilint`. See [Installation](../../docs/installation.md) for other systems and verification steps."
- Unclear: four platform procedures appear before the reader sees how to run Clilint.

Example for an agent reader:

- Clear: "For an OpenSpec change, load the matching skill before editing an artifact."
- Unclear: paste every OpenSpec workflow into every task, including workflows unrelated to the requested change.

## Remove distracting language

Rewrite or remove:

- unexplained jargon and uncommon abbreviations;
- phrases coined for the occasion or words combined in ways a person would not normally use;
- strings of technical nouns that hide the action;
- double negatives and sentences built around several contrasts;
- metaphors that make the reader decode an image before the point;
- vague claims such as "easy," "powerful," or "comprehensive" without evidence;
- implementation details that do not affect the reader's current decision;
- dramatic framing, exaggerated certainty, and filler agreement; and
- language associated with exclusion, violence, slavery, or stereotypes when a neutral term says the same thing.

Prefer calm, direct statements. When evidence is incomplete, say what is known, what remains uncertain, and why.

Examples:

- Prefer "one-time setup" to "bootstrap the environment."
- Prefer "important part" to "load-bearing component."
- Prefer "the evidence supports this explanation" to "this confirms the smoking gun."
- Prefer "the file defines the project rules" to "the canonical rules artifact establishes a durable contract."

## Keep authoring instructions out of human documents

Rules about how an agent writes, reviews, or validates a document belong in project instructions and skills. The finished human document should contain what its reader needs, not a description of the writing process.

Apply only the rules relevant to the document. Do not add unrelated warnings merely because they are generally true.

## Sources

- [Favor Readers Over Writers](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Favor%20Readers%20Over%20Writers.md)
- [Dispel Ambiguity](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Principle___Dispel%20Ambiguity.md)
- [Writing preferences](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Writing.md)
- [Use plain language](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Writing___Use%20Plain%20language.md)
- [Use the simpler word](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Writing___Use%20the%20simpler%20word.md)
- [Be specific and explicit](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Writing___Be%20specific%20and%20explicit.md)
- [Avoid jargon](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Writing___Avoid%20Jargon%2C%20Especially%20Tech-speek.md)
- [Avoid double negatives](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Writing___Avoid%20double%20negatives.md)
- [Avoid distracting metaphors](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Writing___Avoid%20Distractors%20such%20as%20Awkward%20or%20Superfluous%20Metaphors.md)
- [Lower the drama](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Writing___Don%27t%20be%20an%20Attention%20Vampire%3B%20Lower%20the%20Drama.md)
- [Seek inclusive language](https://github.com/codekiln/logseq-encode-garden/blob/main/pages/My___Pref___Writing___Seek%20Inclusive%20Language.md)
