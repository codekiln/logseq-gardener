## Purpose

The person publishing a garden can let readers install it as an app with its own name, icon and window, with one entry in the garden's `logseq/config.edn`. A garden without the entry publishes the same site as before. Installation adds no service worker, cache or browser storage, and works for a site served under a path prefix, with readable URLs, and opened from disk.

## Requirements

### Requirement: Opt in with `:lsg/install`

`lsg publish` SHALL make a site installable only when the garden's `logseq/config.edn` holds a top-level `:lsg/install` key whose value is a map with `:name` (a string), `:icon` (a string) and optionally `:short-name` (a string). `:name` SHALL hold 1 to 45 characters after trimming, `:short-name` SHALL hold 1 to 12 characters after trimming and default to `:name` when `:name` is at most 12 characters, and `:icon` SHALL be a path relative to the garden folder. Any other key, a missing required key, a value of another type or a nested use of the key SHALL stop the build before it creates the output directory, exit 1 with empty stdout, and write a message to stderr that names `logseq/config.edn`, the key and the problem. A `:name` longer than 12 characters without `:short-name` SHALL stop the build the same way. The key SHALL be the only switch: `lsg publish` SHALL have no flag for installation, and Rust callers of the SDK SHALL get the same result as the command line.

#### Scenario: Turned on in config.edn

- **WHEN** a garden whose `logseq/config.edn` holds `:lsg/install {:name "Ghost in the Patch" :short-name "GitP" :icon "assets/app-icon.png"}` is published
- **THEN** publishing succeeds, the report fields are unchanged and the output holds `lsg.webmanifest` and `lsg-icon.png`

#### Scenario: Misspelled key inside the map

- **WHEN** the map holds `:shortname "GitP"`
- **THEN** the build exits 1 with empty stdout, names `logseq/config.edn` and `:shortname`, lists the accepted keys on stderr, and creates no output directory

#### Scenario: Long name without a short name

- **WHEN** the map holds `:name "Logseq Gardener Documentation"` and no `:short-name`
- **THEN** the build exits 1 and the message asks for a `:short-name` of at most 12 characters

#### Scenario: No key

- **WHEN** a garden without `:lsg/install` is published by this release and by the previous release with the same options and `SOURCE_DATE_EPOCH`
- **THEN** the two outputs hold the same files with identical contents, and the site has no manifest, icon or install script

### Requirement: Check the icon before output

The `:icon` file SHALL exist inside the garden folder, SHALL be a PNG whose header names a square image of 512 to 1024 pixels on each side, and SHALL be at most 256 KB. A missing file, a path outside the garden folder, another format, a non-square or out-of-range image and an oversized file SHALL each stop the build before it creates the output directory, with a message that names the path and the failed check. `lsg publish` SHALL copy the file to `lsg-icon.png` unchanged and SHALL NOT resize, convert or recompress it.

#### Scenario: Valid icon

- **WHEN** `:icon` names a 512 by 512 PNG of 40 KB
- **THEN** `lsg-icon.png` is byte-identical to the file

#### Scenario: Rejected icons

- **WHEN** `:icon` names a missing file, an SVG, a 300 by 300 PNG, a 600 by 512 PNG, a 300 KB PNG or `../icon.png`
- **THEN** each build exits 1 before it creates the output directory and names the path and the check that failed

### Requirement: Manifest with a stable identity under any path

An installable site SHALL contain `lsg.webmanifest`, a JSON object with `name`, `short_name`, `start_url` set to `./`, `scope` set to `./`, `display` set to `standalone`, `theme_color` and `background_color` set to the light header colour of `lsg-site.css`, and `icons` holding one entry for `lsg-icon.png` with its `type` `image/png`, its real `sizes` and `purpose` `any`. The manifest SHALL NOT have `id`, so that its identity is its start URL. Every URL in the manifest SHALL be relative, and the manifest's bytes SHALL be identical wherever the site is hosted. The file SHALL NOT exceed 1 KB.

#### Scenario: Same bytes under any path

- **WHEN** the same garden is hosted at `/` and at `/gitpa/`
- **THEN** the manifest at each host is the same file, and each resolves its start URL and scope to that host's site folder

#### Scenario: Two gardens on one origin

- **WHEN** a reader installs the gardens at `/gitpa/` and at `/logseq-gardener/` on one GitHub Pages origin
- **THEN** the browser treats them as two apps, each with its own name and icon

#### Scenario: Manifest size

- **WHEN** a garden with a 45-character name is published
- **THEN** `lsg.webmanifest` is at most 1 KB

### Requirement: Head tags on every full page

Every full page of an installable site, including the home page, journals feed, All pages, graph view, namespace pages and page folders, SHALL carry in its head a link with `rel="manifest"` to `lsg.webmanifest`, a link with `rel="apple-touch-icon"` to `lsg-icon.png`, and two `theme-color` meta elements, one for `(prefers-color-scheme: light)` and one for `(prefers-color-scheme: dark)`, whose colours are the header colours of the light and dark themes. Each URL SHALL be relative to the output root and SHALL resolve through the page's `<base>` under readable URLs. Redirect pages, reference fragments and query fragments SHALL NOT carry the tags. The head SHALL gain no other element for installation, and no `apple-mobile-web-app-*` element.

#### Scenario: Nested readable page

- **WHEN** an installable site is published with `--readable-urls` and a reader opens `GitP/A/Session/` under `/gitpa/`
- **THEN** the browser requests `/gitpa/lsg.webmanifest` and `/gitpa/lsg-icon.png` and both return 200

#### Scenario: Fragments and redirects

- **WHEN** the site holds an old-address redirect page and a references fragment
- **THEN** neither contains a manifest link

#### Scenario: Plugin host navigation

- **WHEN** a site with a plugin turned on is installable and a reader follows a link without a full page load
- **THEN** the document still holds the manifest link and the browser keeps the installed identity

### Requirement: Installation without offline cost

An installable site SHALL NOT register a service worker, SHALL NOT call the Cache Storage API, `localStorage`, `sessionStorage` or IndexedDB for installation, and SHALL NOT contain a file named for a service worker. Installation SHALL NOT change which URLs a reader's browser requests when no one installs the app, except for the manifest, the icon and `lsg-install.js`.

#### Scenario: Reading without installing

- **WHEN** a reader opens an installable site's page in Chromium and navigates to three other pages
- **THEN** `navigator.serviceWorker.getRegistrations()` returns none, `caches.keys()` returns none and the site's origin holds no storage entry written by installation

#### Scenario: Install by menu

- **WHEN** a reader installs the site from Chrome's menu on desktop or Android, or from Safari's Share menu on iOS or macOS
- **THEN** the installed app opens the site's start page in its own window

### Requirement: Install app item in Chromium browsers

An installable site's More menu SHALL hold a button with `role="menuitem"` and the text "Install app", hidden by its `hidden` attribute. `lsg-install.js`, loaded with `defer` by installable pages only, SHALL keep the `beforeinstallprompt` event, call `preventDefault()` on it, and show the item. Activating the item SHALL call `prompt()` on the kept event once, hide the item and return focus to the More button. The item SHALL stay hidden in browsers that never fire the event, when the page runs in an installed window, and when the browser's `display-mode` is `standalone`. The item SHALL be a native button that a keyboard reaches and activates, with the focus style of the other menu items. The script SHALL NOT read or write cookies or browser storage and SHALL send no network request.

#### Scenario: Chromium offers installation

- **WHEN** Chromium fires `beforeinstallprompt` on an installable page
- **THEN** the More menu shows "Install app", and activating it from the keyboard opens the browser's install dialog

#### Scenario: Safari

- **WHEN** a reader opens an installable page in Safari
- **THEN** the More menu has no visible "Install app" item and the page raises no script error

#### Scenario: Installed window

- **WHEN** a reader opens the installed app
- **THEN** the item is hidden

### Requirement: Opened from disk and without scripts

An installable site opened from disk SHALL render and navigate as it does without the key, SHALL show no "Install app" item, and SHALL raise no uncaught script error. A page whose scripts do not run SHALL be as readable as without the key.

#### Scenario: Local file

- **WHEN** a reader opens `index.html` of an installable site from disk
- **THEN** the page renders, a link opens its target, the console holds no uncaught error and the item stays hidden

### Requirement: Performance budgets

`scripts/parity/budgets.toml` SHALL hold an `[install]` table with `page_growth_kb` 0.5, `page_growth_gzip_kb` 0.3, `script_kb` 1.5, `script_gzip_kb` 1, `manifest_kb` 1, `icon_kb` 256 and `build_ms` 1000. `parity:budgets` SHALL build the fixture garden with the key and fail when an installable page grows over its page row beyond the build without the key, when `lsg-install.js` or the manifest exceeds its row, when the first paint of any fixture page leaves the `[desktop]` and `[phone]` budgets, when the growth differs after the garden gains 2,000 pages, or when the search files differ from the build without the key. `parity:build-bench` SHALL fail when the key adds more than `build_ms` to the whole encode garden's build, measured as the median over five interleaved rounds of the build with the key less the build without it in the same round. `build_ms` SHALL stay above the run-to-run noise of that measurement, which was about 500 ms between the median of one CI run and the next.

#### Scenario: Page growth

- **WHEN** `parity:budgets` builds the fixture garden with and without the key
- **THEN** every page with the key is at most 0.5 KB larger, and 0.3 KB larger once gzip-compressed, than without it

#### Scenario: Garden grows

- **WHEN** `parity:budgets` adds 2,000 pages to the fixture garden and builds it with the key
- **THEN** the page growth of every fixture page is unchanged

#### Scenario: Script over budget

- **WHEN** `lsg-install.js` grows past 1.5 KB
- **THEN** `parity:budgets` fails and names the file and the row

#### Scenario: Slow build

- **WHEN** the key adds 80 ms to the whole encode garden's build
- **THEN** `parity:build-bench` fails and names the row

### Requirement: Publishing help and documentation

Publishing help SHALL describe `:lsg/install`, give one example and name the `lsg/Ref/Install Options` page with its documentation site URL. The documentation graph SHALL hold `lsg/How To/Install a Published Garden as an App`, which tells a reader how to install on iPhone, iPad, macOS Safari, Chrome and Edge on desktop and Chrome on Android, and says that Safari and Firefox show no in-page control and that offline reading is not part of installation; `lsg/Ref/Install Options`, which describes the key, every check, every generated file and the limits; and a pointer from `lsg/How To/Publish a Garden to GitHub Pages`. Both new pages SHALL use plain language and state which browser versions each claim covers.

#### Scenario: Discover offline

- **WHEN** a caller reads publishing help without a garden
- **THEN** the help describes `:lsg/install`, shows an example and names the `lsg/Ref/Install Options` page with its documentation site URL

#### Scenario: Documentation checks

- **WHEN** `mise run docs:check` runs
- **THEN** it passes with the new pages, their tags and their links

### Requirement: Real-device checks recorded before release

The change's `verification.md` SHALL record, for each row of the real-device checklist in the design, the device, the operating system and browser version, the date and the result. A row that cannot run SHALL be marked not run with its reason, and the implementation pull request SHALL list those rows.

#### Scenario: Checklist complete

- **WHEN** the implementation pull request is ready for review
- **THEN** `verification.md` holds every row of the checklist with a result or a reason it did not run
