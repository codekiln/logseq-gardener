## Purpose

Define the SDK operations that give a fenced code or example block its text without the outline's indentation and look up its fence language the way Logseq does, so the CLI, the language server and the publisher read the same code.

## Requirements

### Requirement: Give code text without the outline's indentation

The SDK SHALL give each fenced code block, `#+BEGIN_SRC` block and `#+BEGIN_EXAMPLE` block its text with the owning bullet's continuation indentation removed from every line once: the whitespace before the bullet's `-` followed by two spaces. A line that starts with less than that prefix SHALL lose only the leading whitespace it has. Whitespace the author wrote after the prefix SHALL remain, even when it starts with the same characters as the prefix. One trailing newline SHALL be dropped. The CLI, the language server and the publisher SHALL read code text through this operation.

#### Scenario: Nested bullet
- **WHEN** the `Rust/loop/label` page has a fenced `rust` block under a bullet indented one tab
- **THEN** the block's first line is `fn loop_labels_to_disambiguate_multiple_loops() {` and its indented body lines keep the indentation the author wrote inside the fence

#### Scenario: Top-level bullet
- **WHEN** a fenced block sits under a top-level bullet and its body lines start with two spaces
- **THEN** those two spaces are removed and deeper indentation remains

#### Scenario: Short line
- **WHEN** a blank line inside the fence has no leading whitespace
- **THEN** the line stays empty and the following lines are unaffected

#### Scenario: Nested source block
- **WHEN** a `#+BEGIN_SRC python` block sits under a bullet indented one tab and a code line is the bullet's prefix followed by `\treturn 1`
- **THEN** the line is `\treturn 1`, as Logseq shows it, and matches the same code in a fenced block or a `#+BEGIN_EXAMPLE` block

#### Scenario: Source block under a top-level bullet
- **WHEN** a `#+BEGIN_SRC` block sits under a top-level bullet and a code line is the bullet's two spaces followed by `  return 1`
- **THEN** the line is `  return 1`

### Requirement: Look up the fence language as Logseq does

The SDK SHALL map a fence language to the mode Logseq's published site highlights it with, by comparing the lower-cased language with the names and aliases `CodeMirror.findModeByName` accepts in Logseq's bundled modes, after mapping `edn`, `clj`, `cljc`, `cljs` and `clojurescript` to `clojure`. A language with no match SHALL map to no mode. The SDK SHALL also give the language text to show, lower-cased as typed, or nothing for a block without a language.

#### Scenario: Known language
- **WHEN** a fence is labelled `bash`, `sh` or `zsh`
- **THEN** the lookup returns Logseq's Shell mode and the label text is the fence language in lower case

#### Scenario: Language Logseq leaves plain
- **WHEN** a fence is labelled `py`, `rs`, `text`, `nix`, `jsonc`, `mermaid` or an unknown name
- **THEN** the lookup returns no mode and the label text is the fence language in lower case

#### Scenario: Clojure aliases
- **WHEN** a fence is labelled `edn`
- **THEN** the lookup returns Logseq's Clojure mode
