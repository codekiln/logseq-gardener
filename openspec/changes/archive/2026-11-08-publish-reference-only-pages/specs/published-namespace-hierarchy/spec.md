## MODIFIED Requirements

### Requirement: Preserve index and source publication contracts

The complete page index SHALL list each selected source page, generated namespace destination and reference-only page once with its full logical title and local route. SiteReport.pages SHALL continue counting selected source documents only. Page references, tags and page-valued properties SHALL resolve to generated namespaces through the shared page-name index. Generated namespaces SHALL remain separate from alias, block, transclusion, and homepage lookup. Homepage output, source routes, media rendering, and transclusion behavior SHALL retain their existing contracts.

#### Scenario: Indexed synthetic ancestors
- **WHEN** one selected source page requires two generated namespace ancestors
- **THEN** the complete index lists all three destinations and the source page count remains one

#### Scenario: Generated namespace is selected as homepage or embed
- **WHEN** a caller requests a source-absent generated namespace as homepage or a source page embeds it
- **THEN** homepage validation fails or the embed keeps its existing unavailable fallback, respectively

#### Scenario: Retained rendering
- **WHEN** a selected homepage includes supported media and selected-content transclusions
- **THEN** hierarchy navigation is added while its existing rendered content, media destinations, block links, and transclusions retain their behavior

#### Scenario: Text link to a generated namespace
- **WHEN** a selected page's block links `[[AI]]`, and `AI` is a generated namespace destination because `AI/General` is selected
- **THEN** the link opens the generated `AI` page and the build reports no unresolved reference for it

### Requirement: Distinguish namespace destinations

The publisher SHALL visibly label generated namespace pages as Namespace. Links to generated namespace destinations, including hierarchy segments, All pages rows, and page references and tags in page text, SHALL have a distinction beyond color and an accessible Namespace name; hierarchy rows and All pages rows SHALL carry no Namespace badge. Selected stored source pages SHALL retain ordinary page labels even when empty. Classification SHALL use only selected visible source routes: missing and ordinarily unselected source ancestors SHALL have the same Namespace presentation. Withheld and alias-conflicting ancestors SHALL retain their existing unavailable behavior. Full logical anchor text in the page index SHALL remain unchanged. Source-only alias, embed, homepage, route, and count contracts SHALL remain intact.

#### Scenario: Generated and stored destinations

- **WHEN** a selected stored page needs a generated ancestor, and another selected stored page has an empty body
- **THEN** only the generated ancestor has the Namespace page status and the distinct link style, including in hierarchy segments and All pages rows

#### Scenario: Unselected ancestor privacy

- **WHEN** one generated ancestor has an ordinary unselected source file and another has no source file
- **THEN** both have identical Namespace status without source file paths, content, or existence claims

#### Scenario: Safe index labels and private gaps

- **WHEN** the index and hierarchy include generated destinations while a private or alias-conflicting prefix is unavailable
- **THEN** generated index anchors keep their exact full logical names, generated links identify Namespace status, and unavailable prefixes receive no new route

#### Scenario: Rows without badges

- **WHEN** a reader opens `tmux`, whose rows include paths through the generated `tmux/Command`, and All pages, which lists `tmux/Command`
- **THEN** no hierarchy row or All pages row shows a Namespace badge and the generated links keep their style

#### Scenario: Page reference in text

- **WHEN** a block links `[[Ableton/Live]]`, a generated namespace destination
- **THEN** the link has the distinct link style and the accessible Namespace name
