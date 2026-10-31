## ADDED Requirements

### Requirement: Remember the reader's site-wide Properties choice

The published site SHALL use the configured Properties default when no valid reader choice exists. Reader activation of the Properties summary SHALL remember the resulting open or closed state across pages and visits in localStorage, scoped to the site's base path. The entry SHALL contain only the boolean choice and last-seen timestamp. Every visit SHALL renew a valid entry. After 30 days without visits, or when its shape or timestamp is invalid, the site SHALL remove the entry and use the configured default. Storage access failures SHALL leave the disclosure usable with its configured default.

#### Scenario: Both reader choices cross page boundaries
- **WHEN** a reader closes or reopens Properties and visits another page under the same site base path
- **THEN** Properties uses that reader choice, while another garden on a different base path retains its own choice

#### Scenario: Expired or invalid storage
- **WHEN** the entry is expired, malformed or has a future timestamp, or storage is unavailable
- **THEN** Properties uses the configured default and any removable invalid entry is cleared

#### Scenario: Visits extend the remembered choice
- **WHEN** a reader returns within the absence window
- **THEN** the choice remains and the last-seen timestamp advances to that visit

### Requirement: Restore Properties during document and host navigation

The site SHALL restore a remembered state before the disclosure's content can flash in the configured state during normal parsing. It SHALL expose idempotent `window.logseqGardenerInitProperties(root=document)` for replaced page content, including calls before the full controller is available. Necessary fragment reveal SHALL take precedence on the current page without changing the remembered reader choice. Existing theme, search and outline behavior SHALL remain usable.

#### Scenario: Early restoration and host replacement
- **WHEN** a page is parsed or a host initializes new main content before its fragment handling
- **THEN** the Properties disclosure uses the remembered state and repeated initialization preserves subsequent disclosure state

#### Scenario: A fragment targets remembered closed properties
- **WHEN** navigation reveals the Properties fragment while the reader preference is closed
- **THEN** the target disclosure opens for that navigation and the next ordinary page still uses the closed preference

#### Scenario: A reader acts before the controller arrives
- **WHEN** a reader opens or closes Properties while the controller is loading and the host replaces the page before that controller arrives
- **THEN** replacement content uses the accepted choice before and after controller adoption, and canceled activations and fragment reveals do not become saved choices

#### Scenario: A reader leaves the page before the controller arrives
- **WHEN** a reader opens or closes Properties while the controller is loading and then follows an ordinary link or reloads
- **THEN** the next page uses that choice

#### Scenario: Back or Forward restores a cached page
- **WHEN** the browser restores a page from its Back/Forward cache after the reader changed Properties on another page
- **THEN** the restored page uses the current remembered choice, or the configured default when none is valid, and a fragment inside Properties keeps it open
