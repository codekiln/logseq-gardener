# Spec Delta

## MODIFIED Requirements

### Requirement: Build plugins

A build plugin SHALL be a manifest with a `build` section passed to the publisher, a Rust value implementing the SDK's `BuildPlugin` trait in `SiteOptions::build_plugins`, or the build step of a first-party plugin that the garden turns on. All three kinds SHALL share one id namespace, receive the publication view filtered for their `pageProperties` and block choice, and return contributions of the same types. A first-party build step MAY return `metadata` contributions for its own module. Each plugin SHALL receive the same view; no plugin SHALL see another plugin's contributions.

#### Scenario: Rust plugin

- **WHEN** a Rust caller adds a `BuildPlugin` to `SiteOptions::build_plugins` and publishes
- **THEN** the plugin's `contribute` is called once with the view and its contributions appear in the output

#### Scenario: Shared id namespace

- **WHEN** a Rust plugin and a manifest declare the same id
- **THEN** publication fails before any file is written and the error names the id

#### Scenario: First-party build step with a module

- **WHEN** a garden turns on `podcast-episode` and is published
- **THEN** the plugin's build step runs once in the `lsg` process, starts no other process, and its `metadata` contributions reach its module through the page metadata
