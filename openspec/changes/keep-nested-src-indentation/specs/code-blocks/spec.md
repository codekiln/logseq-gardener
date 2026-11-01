## MODIFIED Requirements

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
