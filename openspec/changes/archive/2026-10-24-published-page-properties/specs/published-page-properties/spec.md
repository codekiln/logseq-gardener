## ADDED Requirements

### Requirement: Publish selected leading page properties safely

When explicitly enabled, the publisher SHALL render safe leading page properties in authored order with escaped keys and scalar values, keeping the last duplicate key. Direct page-reference values SHALL link only to canonical unique selected visible source pages. Standalone public HTTPS values SHALL render as escaped links after nonempty, credential-free authority and safe-delimiter validation. A property containing an unavailable or ambiguous reference, unsafe expression, block reference, unsafe scheme or local/asset path SHALL be omitted completely. Internal/control/source metadata and configured hidden keys SHALL be omitted from property markup and enabled property backlink counts/options. Publication SHALL preserve source files.

#### Scenario: Ordered scalars and aliases
- **WHEN** enabled properties contain a number, boolean, ordinary text and a reference through a unique alias
- **THEN** visible rows retain authored order and scalar text, and the reference uses the canonical source label and route

#### Scenario: Hidden and mixed values
- **WHEN** a property mixes public and withheld references or uses an internal/configured hidden key
- **THEN** its key and values supply no visible property row or enabled property backlink count or option

#### Scenario: Unsafe values and markup
- **WHEN** properties contain raw expressions, asset/file URLs, block references or markup characters
- **THEN** unsupported values are omitted and retained plain text is escaped without executing markup

### Requirement: Configure property disclosure and stable source navigation

The publisher SHALL read boolean `:publishing/page-properties?`, `:publishing/page-properties-collapsed?` and `:publishing/page-sequence?` from the graph config, defaulting to false, true and true. It SHALL honor keyword collections in `:block-hidden-properties`. Invalid settings SHALL retain defaults with bounded read-only diagnostics. Visible properties SHALL appear in a native Properties disclosure with summary fragment `page-properties`, readable focus and narrow-screen wrapping. Generated namespace pages SHALL not invent properties. Fragment navigation SHALL reveal the necessary disclosure using the shared fragment helper.

#### Scenario: Opt-in and collapse override
- **WHEN** properties are enabled without a collapse override
- **THEN** the section starts closed, while an explicit false collapse setting starts it open

#### Scenario: Stable fragment
- **WHEN** a reader follows a source link to a closed property section
- **THEN** the summary exists inside the native disclosure and the shared reveal helper opens that containing disclosure

#### Scenario: Default and malformed settings
- **WHEN** properties are not enabled or config contains invalid or nested setting lookalikes
- **THEN** the existing presentation remains and supported top-level defaults apply
