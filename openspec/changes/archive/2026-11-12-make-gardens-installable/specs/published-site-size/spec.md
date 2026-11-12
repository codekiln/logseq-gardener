# Spec Delta

## MODIFIED Requirements

### Requirement: Name files that every page references stably

Every file that every page references, including `lsg-site.css`, `lsg-search.css`, `lsg-theme.js`, `lsg-search-boot.js` and `lsg-shell.js`, SHALL have a fixed name, and its contents SHALL depend only on the `lsg` version. On a site made installable by `:lsg/install`, the files `lsg.webmanifest`, `lsg-icon.png` and `lsg-install.js` SHALL also have fixed names, and their contents SHALL depend only on the `lsg` version and the `:lsg/install` value and icon file. A change to the garden's pages SHALL leave these files and every page's references to them unchanged. With `--readable-urls`, these names SHALL be reserved like the site's other generated files.

#### Scenario: Garden edit
- **WHEN** the garden is published, one block is added to one page, and the garden is published again with the same `lsg`
- **THEN** `lsg-site.css`, `lsg-search.css`, `lsg-theme.js` and `lsg-search-boot.js` are identical in both outputs, and so is each page's markup that refers to them

#### Scenario: Garden edit on an installable site
- **WHEN** an installable garden is published, one block is added to one page, and the garden is published again with the same `lsg` and the same `:lsg/install` value
- **THEN** `lsg.webmanifest`, `lsg-icon.png` and `lsg-install.js` are identical in both outputs, and so is each page's markup that refers to them

#### Scenario: Reserved names
- **WHEN** an installable garden is published with `--readable-urls` and holds a page titled `lsg.webmanifest`
- **THEN** publication fails before output is created and the message names the page and `lsg.webmanifest`, as it does for the other generated names
