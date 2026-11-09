# Spec Delta

## Purpose

A garden whose pages form a sequence with `prev::` and `next::` can turn on `sequence-header`, so a reader finds the previous and next pages in the header at the top of the page.

## ADDED Requirements

### Requirement: Previous and next links in the header

The plugin SHALL apply to a published page with at least one valid neighbour under the authored page sequence rules. Its entry SHALL hold each neighbour's URL and title, and the module SHALL show them in the header as Previous and Next links with `rel="prev"` and `rel="next"`, escaped titles, visible keyboard focus and wrapping on narrow screens. A page without a valid neighbour SHALL get no entry.

#### Scenario: Both neighbours

- **WHEN** a reader opens `GitP/A/Session/26/10/24-Sat`, which names `GitP/A/Session/26/10/22-Thu` in `prev::` and `GitP/A/Session/26/10/26-Mon` in `next::`
- **THEN** the header shows a Previous link to the first and a Next link to the second, each with its page title

#### Scenario: Newest episode

- **WHEN** a reader opens `GitP/A/Session/26/11/01-Sun`, which has `prev::` and no `next::`
- **THEN** the header shows only the Previous link

#### Scenario: Page outside a sequence

- **WHEN** a reader opens `GitP/A` directly on a site with the plugin on
- **THEN** the header has no sequence links and the browser requests no file under `plugins/sequence-header/`

### Requirement: Same targets as the page sequence region

The header links SHALL point at the same pages as the page sequence region under the authored page sequence rules: a private, excluded, missing, ambiguous or self target SHALL give no link and expose no title or URL. The plugin SHALL leave the page sequence region and the property rows as they are without it.

#### Scenario: Withheld neighbour

- **WHEN** a page names a neighbour withheld by `public:: false` in `next::`
- **THEN** the header has no Next link and no generated file holds the withheld page's title or URL

### Requirement: Header follows navigation

After a same-document navigation, the header SHALL show the links of the adopted page, or none when that page has no entry.

#### Scenario: Navigate from an episode to the show page

- **WHEN** a reader opens `GitP/A/Session/26/10/24-Sat` and follows a link to `GitP/A`
- **THEN** the header shows no sequence links on `GitP/A`, and Back shows the episode's links again
