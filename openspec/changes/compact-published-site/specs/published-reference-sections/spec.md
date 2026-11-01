## MODIFIED Requirements

### Requirement: Load section bodies when opened

A page's HTML SHALL hold each reference section's header and counts, and no groups. The publisher SHALL write linked and unlinked groups to fragment files in the site's `_refs/` directory, in group order with about 50 groups per file, with the section's data for counting and filtering in its first file. Each linked and unlinked fragment file SHALL be stored gzip-compressed as `_refs/<page>-linked-<n>.html.gz` or `_refs/<page>-unlinked-<n>.html.gz`, with no file name or time in its gzip header, so the same groups always produce the same file. Opening a section SHALL fetch its first file, decompress it in the browser when it starts with the gzip header and read it as text otherwise, and fetch later files as the reader nears the end. Without scripts, each section SHALL show its header. Fragments SHALL follow the same rendering, anchor and withholding rules as page content. Fragment navigation to a referring block SHALL still open the block on its source page. When a fragment cannot be loaded or decompressed, the opened section SHALL say that its references could not be loaded.

#### Scenario: First paint independent of references
- **WHEN** the encode garden is built
- **THEN** `Answer/AI`, `note/my`, `mise` and `Claude/Code` each send at most 100 KB before their title, and no reference group is in their HTML

#### Scenario: Compressed fragment
- **WHEN** a reader opens Linked References on `mise`
- **THEN** the page fetches `_refs/` files ending in `.html.gz`, and the groups, counts and filters match those of the build before this change

#### Scenario: Host that decodes gzip files
- **WHEN** a server sends a fragment already decompressed
- **THEN** the section reads it as text and shows the same groups

#### Scenario: Same groups, same file
- **WHEN** the garden is published twice with no change to the blocks that refer to `mise`
- **THEN** the fragment files of `mise` are identical in both outputs

#### Scenario: Swapped content
- **WHEN** a host replaces page content and initializes the references script again
- **THEN** the new sections load their fragments and no handler is registered twice

### Requirement: Keep reference sections within budgets

The `[references]` table in `scripts/parity/budgets.toml` SHALL set: `references.js` at most 6 KB minified, references styles at most 1 KB gzip, each fragment file at most 100 KB as stored, first group visible at most 300 ms on desktop and 1,500 ms on the phone profile, peak phone heap at most 20 MB after opening both reference sections on `mise` and scrolling to the end, a filter change on `mise` at most 50 ms on desktop, and whole-garden build time at most 25% above the build before this change. `parity:budgets` SHALL check the size budgets in CI, and `parity:perf` SHALL mark the others.

#### Scenario: Oversized fragment
- **WHEN** a fragment file exceeds 100 KB as stored
- **THEN** `parity:budgets` fails and names the file
