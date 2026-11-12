# Design

## Context

[Issue #603](https://github.com/codekiln/logseq-gardener/issues/603) asks how a garden published by `lsg` can be installed as an app, and [Issue #604](https://github.com/codekiln/logseq-gardener/issues/604) asks for this proposal. [Issue #607](https://github.com/codekiln/logseq-gardener/issues/607) covers offline reading later. See proposal.md for the motivation.

What `lsg` has today:

- **Relative links.** Every link, script, stylesheet and asset in a generated page is relative to the output root. With `--readable-urls`, a page in a folder carries `<base href>` that climbs to the root, so the site works under a GitHub Pages path such as `/gitpa/` and opened from disk ([readable-page-urls](../../specs/readable-page-urls/spec.md)).
- **Fixed shared files.** Files that every page references, such as `lsg-site.css` and `lsg-shell.js`, have fixed names and contents that depend only on the `lsg` version ([published-site-size](../../specs/published-site-size/spec.md)).
- **Config keys.** `garden_settings.rs` reads top-level keys of `logseq/config.edn` with an EDN reader, including `lsg`'s own `:lsg/plugins`, which turns on first-party plugins and stops the build on a malformed entry ([first-party-plugins](../../specs/first-party-plugins/spec.md)). Logseq ignores both.
- **No installation support.** No page links a manifest or an icon, and no generated file is a manifest.
- **Budgets.** Every page meets 100 KB of data and 16 KB of script before its title, a site without opt-in features pays nothing for them, and `scripts/parity/budgets.toml` holds a table per feature ([publication-speed](../../specs/publication-speed/spec.md), [published-site-size](../../specs/published-site-size/spec.md)).
- **Publishing sites.** The documentation site in this repository, logseq-encode-garden (13,524 published pages) and gitpa each publish to GitHub Pages under a path of `codekiln.github.io`, and all three share that origin.

### What the sources say

Checked on 2026-11-11 against the pages named in #603. The device checklist below covers what no page settles.

| Claim | Source | Result |
| --- | --- | --- |
| Chrome installs from its menu without a service worker, from version 108 on Android and 112 on desktop | [Chrome install criteria](https://developer.chrome.com/blog/update-install-criteria) | Supported. The article adds that the install prompt algorithm still required a fetch handler when it was written, so the criteria page below is the one to rely on. |
| Chrome promotes installation when the page has a manifest with `name` or `short_name`, 192 and 512 pixel icons, `start_url` and a `display` of `standalone`, `fullscreen`, `minimal-ui` or `window-controls-overlay`, is served over HTTPS, and the reader has clicked the page and spent 30 seconds on it | [Install criteria](https://web.dev/articles/install-criteria) | Supported, and it does not list a service worker. It names two icon sizes, and this design supplies one 512 to 1024 pixel icon, so the Verify step checks that Chrome still promotes installation. |
| `beforeinstallprompt` exists only in Chromium browsers and is not supported on iOS | [MDN installability guide](https://developer.mozilla.org/en-US/docs/Web/Progressive_web_apps/Guides/Making_PWAs_installable), [MDN event page](https://developer.mozilla.org/en-US/docs/Web/API/Window/beforeinstallprompt_event) | Supported. MDN marks the event as not Baseline and takes its specification from a WICG incubation. |
| Safari on macOS Sonoma and later adds any page to the Dock, with or without a manifest, and lets the reader name it and change its icon | [Apple Mac guide](https://support.apple.com/en-us/104996), MDN guide | Supported. Apple's page does not say which icon the Dock uses by default. |
| Safari, Chrome, Edge, Firefox and Orion on iOS 16.4 and later can add to the Home Screen; a manifest with `display` of `standalone` or `fullscreen` opens as an app | [WebKit: Web Push for Web Apps](https://webkit.org/blog/13878/web-push-for-web-apps-on-ios-and-ipados/) | Supported. |
| iOS uses manifest icons since 15.4, and an `apple-touch-icon` link takes precedence; with no icon, iOS 16.4 draws a monogram | WebKit blog above | Supported. |
| `id` is resolved against the origin of `start_url`, so `foo`, `./foo` and `/foo` give the same identity; with no `id`, the identity is the processed `start_url`; `start_url` and `scope` resolve against the manifest's URL | [Web App Manifest](https://www.w3.org/TR/appmanifest/) | Supported. This is why the manifest has no `id`: a relative `id` would give every garden on `codekiln.github.io` one identity. |
| `navigator.install()` without arguments needs a manifest that declares an `id` | [Chrome guide](https://developer.chrome.com/docs/modern-web-guidance/guides/pwa/install-web-app) | Supported. The page gives no release or support status for the API, so this design does not use it. |
| Firefox on desktop does not install manifest-based apps; Firefox on Android adds a browser-badged shortcut | MDN guide | Supported. |

## Goals / Non-Goals

**Goals:**

- A garden's author turns installation on with one declarative key and gets an app with the chosen name and icon on iOS, iPadOS, macOS and Android and in desktop Chrome and Edge.
- A garden that does not turn it on publishes the same bytes as before.
- The installed app and the browser site keep working under a GitHub Pages path, with readable URLs and opened from disk.
- Installation adds no service worker, cache or browser storage and stays inside stated size budgets.

**Non-Goals:**

- Offline reading, a service worker and cache update rules ([Issue #607](https://github.com/codekiln/logseq-gardener/issues/607)).
- A maskable icon and one icon per size, which need an image resizer in `lsg`.
- Colour options for the manifest and the title bar.
- An in-page banner that explains the Share menu to Safari readers.

## Decisions

### The key

```clojure
:lsg/install {:name "Ghost in the Patch" :short-name "GitP" :icon "assets/app-icon.png"}
```

The key is a top-level entry of `logseq/config.edn`, read with the other garden settings. Its value is a map with `:name`, `:icon` and an optional `:short-name`. An absent key turns installation off. The SDK reads the key into one value:

```rust
pub struct InstallSettings { name: String, short_name: String, icon: Icon }
```

`Icon` holds the path inside the garden, the checked size in pixels and the bytes. Its only constructor reads the file and checks that it is a PNG inside the garden folder, square, 512 to 1024 pixels and at most 256 KB, so no later code can hold an unchecked icon. `short_name` is filled when `:name` has at most 12 characters; otherwise the reader must supply it, so no `InstallSettings` has a missing short name. A missing key, an unknown key, a wrong type, an out-of-range length and a nested use each stop the build before the output directory exists, exit 1 with empty stdout, and name `logseq/config.edn`, the key and the problem, as `:lsg/plugins` does. The key is the only switch, and the SDK reads it while it reads the garden, so a Rust caller gets the same result as the command line.

### Files and names

| File | Contents | Size limit |
| --- | --- | --- |
| `lsg.webmanifest` | The manifest, from the settings | 1 KB |
| `lsg-icon.png` | The author's icon, copied unchanged | 256 KB |
| `lsg-install.js` | The Install app item's script | 1.5 KB, 1 KB gzip |

Names are fixed, so every page's reference stays the same when pages change. They are reserved under `--readable-urls` like the other generated files. They are site files like `lsg-site.css`, so `lsg-site.json` does not list them.

### The manifest

```json
{"name":"Ghost in the Patch","short_name":"GitP","start_url":"./","scope":"./","display":"standalone","theme_color":"#fbfcfd","background_color":"#fbfcfd","icons":[{"src":"lsg-icon.png","type":"image/png","sizes":"512x512","purpose":"any"}]}
```

- **Relative URLs.** A browser resolves `start_url`, `scope` and `src` against the manifest's URL, which sits at the site root, so `./` is the garden's own folder at `/`, at `/gitpa/` and anywhere else. The manifest is the same bytes in every host.
- **No `id`.** The identity is then the processed `start_url`. The garden at `/gitpa/` and the one at `/logseq-gardener/` get different identities and install as separate apps. An absolute `id` would need the hosting path at build time, which `lsg` does not know. The cost is that moving a garden to another path or host gives it a new identity, and an installed copy stays on the old one.
- **`standalone`.** iOS 16.4 and later opens a manifest with `standalone` or `fullscreen` as an app, and Chromium promotes installation for it. The site's header already holds Home, search and menu controls.
- **Colours.** `theme_color` and `background_color` are the light header colour. The page's two `theme-color` meta elements follow the reader's appearance with the header colour of each theme.
- **Real size.** `sizes` is the PNG's own size, read from its header.

### Head tags

An installable page gains, in its head:

```html
<link rel="manifest" href="lsg.webmanifest"><link rel="apple-touch-icon" href="lsg-icon.png"><meta name="theme-color" media="(prefers-color-scheme: light)" content="#fbfcfd"><meta name="theme-color" media="(prefers-color-scheme: dark)" content="#151718"><script defer src="lsg-install.js"></script>
```

That is about 0.4 KB. The tags go after the `<base>` that a readable build writes, so each URL resolves to the site root. `apple-touch-icon` is the one icon link: it takes precedence on iOS and serves Safari versions that predate manifest icons. The page gets no `apple-mobile-web-app-*` element, because the manifest's `display` covers iOS 16.4 and later; older iOS adds a bookmark. A full page is any page the publisher writes with a shell; redirect pages and the fragments that scripts fetch get nothing.

### Menu installation and browser promotion

Two paths lead to an installed app, and they depend on different things.

- **Menu installation** is the browser's own command: Chrome's menu, Edge's menu, Safari's Share menu, Add to Dock. It works with the manifest and icon, and in Safari with nothing at all. The site needs no code for it.
- **Browser promotion** is a prompt that the browser decides to show: Chrome's address-bar install button and the `beforeinstallprompt` event. It needs the manifest criteria above, and Chrome adds reader engagement: a click and 30 seconds on the page.

The documentation tells readers about menu installation first, because it works in every browser that installs at all.

### The in-page affordance

`beforeinstallprompt` is the only in-page mechanism with support today: Chromium browsers fire it, Safari and Firefox do not, and `navigator.install()` needs a manifest `id` that this design leaves out. `lsg-install.js` therefore does one thing:

1. On `beforeinstallprompt`, call `preventDefault()`, keep the event and show the hidden Install app item in the More menu.
2. On activating the item, call `prompt()` once, hide the item and return focus to the More button.

The item is a native `<button role="menuitem">` in the existing menu, so it inherits keyboard behaviour, focus style and name from the other items; it is hidden by the `hidden` attribute in the markup, so a page whose script never runs shows nothing. The script is a separate file loaded with `defer` on installable pages only, so a site without the key loads nothing. It is not part of `lsg-shell.js`, whose contents depend only on the `lsg` version. The script reads and writes no storage and sends no request.

Safari and Firefox readers see no in-page control. The how-to page tells them where the Share menu is, and an author can link that page from the garden.

### Accessibility

- The manifest `name` is the label that screen readers announce for the installed app, so the key needs a readable name; a length check and a required `:name` stop empty and unreadable values.
- The Install app item is a native button with a text label, reachable by keyboard in the menu's tab order, with the same visible focus style as its neighbours, and focus returns to the More button after the browser dialog.
- The `theme-color` elements follow `prefers-color-scheme`, and both colours are the header colours of the themes, so the title bar matches the page header in light and dark.
- The icon is the author's file. The reference page asks authors for an icon that reads at 48 pixels and keeps its subject inside the central 80 percent, because some launchers mask it into a circle.
- The real-device checks below include VoiceOver on iOS and macOS and TalkBack on Android.

### Local file viewing

Installation needs HTTPS or localhost, so a page opened from disk cannot be installed, `beforeinstallprompt` does not fire there, and the item stays hidden. The page renders and navigates as before. The tags are inert and raise no script error.

### Plugins

The manifest link sits in the head of every full page. Same-document navigation by the plugin host swaps `<main>` and keeps the head, so the link stays and the installed identity stays. Plugins and installation share no file, no key and no budget row.

### What lives in the SDK and what lives in the publisher

The SDK holds every rule:

- reading and checking `:lsg/install` (`garden_settings.rs`) into `InstallSettings`;
- the icon check and the manifest's contents (`install.rs`), as pure functions of the settings;
- the colours, taken from the theme constants the publisher already uses;
- which pages are full pages.

The publisher writes the three files, adds the tags and the hidden menu item to full pages through the same page writer, reserves the names and puts the `[install]` rows into the budget checks. `install.js` reacts to one browser event and draws nothing else. The CLI only reports a rejected key through the existing error path and adds a paragraph to publishing help.

### Budgets

All measured by `mise run parity:budgets` and `parity:build-bench` on the fixture garden and the pinned encode garden, in addition to the existing first-paint, shell and search rows. Row names are the keys of the `[install]` table.

| Budget | Limit | Row |
| --- | --- | --- |
| Output without the key | Identical to the previous release | |
| Added bytes per page, uncompressed | 0.5 KB | `page_growth_kb` |
| Added bytes per page, gzip | 0.3 KB | `page_growth_gzip_kb` |
| `lsg-install.js` | 1.5 KB, 1 KB gzip | `script_kb`, `script_gzip_kb` |
| Manifest | 1 KB | `manifest_kb` |
| Icon | 256 KB | `icon_kb` |
| First paint of any page | Existing 100 KB of data and 16 KB of script before the title, desktop and phone | `[desktop]`, `[phone]` |
| Page growth when the garden gains 2,000 pages | 0 | |
| Search files | Identical to the build without the key | |
| Service workers, Cache Storage and browser storage | None | |
| Added build time on the encode garden | 50 ms | `build_ms` |

The icon and the manifest are not part of any page's first paint: a browser fetches them on its own schedule, and Safari fetches the icon when a reader adds the page. The 256 KB cap bounds that cost.

### Real-device checks

No source settles these. Each row is recorded in `verification.md` with the device, operating system, browser version, date and result. A row that cannot run is marked not run with its reason.

| Device and browser | Check |
| --- | --- |
| iPhone, Safari, iOS 17 or later | Share, Add to Home Screen shows the chosen name and icon; the app opens in its own window; Back by edge swipe and in-page navigation work; an external link opens; VoiceOver reads the name and the header controls |
| iPhone, Chrome, iOS 16.4 or later | Share, Add to Home Screen gives the same result |
| iPhone, iOS 16.3 or earlier, if a device exists | Add to Home Screen gives a bookmark and the page still works |
| iPad, Safari | Same as the iPhone row, in portrait and landscape and in split view |
| Mac, Safari 17 or later | File, Add to Dock shows the chosen name and the icon; the app opens; links stay inside the scope |
| Mac, Chrome and Edge | Menu, Install; the install button appears in the address bar after a click and 30 seconds; the Install app item appears in the More menu and opens the dialog; the installed window opens `start_url` |
| Android phone, Chrome, with Google Mobile Services | Menu, Install app; a WebAPK with the chosen name and icon; the Install app item appears |
| Android phone, Firefox | Add to Home Screen gives a shortcut; no Install app item |
| Two gardens on one origin | Install the documentation site and gitpa on `codekiln.github.io`; each opens as its own app, a link from one to the other leaves the app window |
| Garden at the origin root | Install it; its scope covers every project site on that origin, so record what happens to a link to another project site |
| Icon | Check the icon in each launcher for cropping and contrast; confirm that one 512 pixel icon passes Chrome's install criteria |
| Chrome with a throttled network | Installing from the menu adds no service worker and no cache entry in the Application panel |

Chromium checks that need no device run in CI: manifest validity, head tags on every kind of page, resolved URLs under a path prefix, no service worker, no storage entry, and the item hidden when the event never fires.

### Documentation

`lsg/How To/Install a Published Garden as an App` is for a reader: the steps per browser first, with versions, then what installation does and does not do. `lsg/Ref/Install Options` is for the author: the key, each check, the files, the budgets and the limits. `lsg/How To/Publish a Garden to GitHub Pages` gains one pointer. Publishing help gains a paragraph that names the reference page. Both pages state the versions that each claim covers, and the how-to tells a reader that offline reading is not part of installation.

### Existing publishing sites

| Site | Turns it on | Notes |
| --- | --- | --- |
| Documentation site (this repository) | Yes, in the implementation pull request, with an icon file in `logseq-gardener-docs/` | Exercises the key on the site that documents it. |
| logseq-encode-garden | After a release, in its own issue | Needs an icon file and a name. |
| gitpa | After a release, in its own issue | `scripts/build_site.py` already edits published pages; it needs no change, because `lsg` writes the tags. |

## Risks / Trade-offs

- A garden at the origin root, such as `owner.github.io`, has a scope that covers every project site on that origin, so a link to another project site may open inside the installed app → the device checklist records what each browser does, and the reference page says so.
- The identity is the start URL, so moving a garden to another path orphans installed copies → the reference page says so, and the cost is the price of paths that `lsg` cannot know.
- One icon file declared at its real size may fail a browser's size criteria for promotion (documentation names 192 and 512 pixels) → the Verify step checks Chrome on desktop and Android; if promotion fails, a later change adds image resizing and the key stays the same.
- Chrome's promotion needs a click and 30 seconds, so the Install app item can appear late or not at all → the menu path works at any time and the documentation leads with it.
- `beforeinstallprompt` is not Baseline and may change → the script is 1 KB, loads only on installable pages and is hidden when the event never fires.
- Safari may fetch the icon at page load for favorites, adding up to 256 KB on a first visit → the real-device check records requests; the cap is the bound.
- A garden without the key never pays: no tag, file or script. A garden that turns it on pays about 0.4 KB per page.

## Migration Plan

1. Merge this proposal, then implement the key, the files, the tags, the item, the budget rows, the documentation and the checks in one pull request with a cleanup sub-issue.
2. Release. Each consumer adds the key and an icon in its own repository and moves its `lsg` pin in the same commit, because an older `lsg` ignores the key and builds the site without installation and without an error.
3. To undo, remove the key; the next build drops the files and the tags. Installed copies keep working as bookmarks to their start page.

## Resolved Questions

### 1 - Should installation be opt-in?

Decided by the agent without codekiln: yes, by the `:lsg/install` key, absent by default. Parity with Logseq is the default for `lsg`, and Logseq's own published site has no manifest. An opt-in key leaves every existing garden's bytes unchanged, matches how `:lsg/plugins` works, and states the author's choice in the garden's own configuration.

### 2 - What does the configuration look like?

Decided by the agent without codekiln: a map under one key, `:lsg/install {:name .. :short-name .. :icon ..}`. The name and the icon are the two choices an author must make, so they are required; an installable site with no name or icon would repeat today's monogram result. A short name is required only when the name is longer than 12 characters, because iOS and Android truncate home-screen labels near that length. Unknown keys stop the build so a misspelling cannot silently drop a setting.

### 3 - Where does the icon come from, and does `lsg` resize it?

Decided by the agent without codekiln: the author supplies one square PNG of 512 to 1024 pixels and at most 256 KB, and `lsg` copies it unchanged. Resizing would add an image dependency to the SDK, with a licence and binary cost, for a result the author can produce once. The size check reads the PNG header, so no dependency is needed. A maskable icon and per-size icons move to a later change if the device checks show a need.

### 4 - Should the manifest declare an `id`?

Decided by the agent without codekiln: no. The manifest specification resolves a relative `id` against the origin, so every garden on `codekiln.github.io` would share one identity and a second installation would replace the first. An absolute `id` needs the hosting path, which the relative-path build does not know. Without `id`, the identity is `start_url`, which resolves to each garden's own folder.

### 5 - What are `start_url` and `scope`?

Decided by the agent without codekiln: both `./`, resolved against the manifest at the site root. This gives the garden's folder under any hosting path and is the same bytes everywhere. The start page is the home page or the journals feed, whichever the garden publishes at `index.html`.

### 6 - Which display mode?

Decided by the agent without codekiln: `standalone`. It opens as an app on iOS 16.4 and later and Chromium promotes it. The shell's header has Home, search and More, so a reader can move without browser chrome. Whether a reader can go back in an iOS standalone window is a device check.

### 7 - Which Apple metadata?

Decided by the agent without codekiln: an `apple-touch-icon` link to the same icon, and nothing else. Apple's guidance makes the link take precedence over manifest icons, and the manifest's `display` makes iOS 16.4 and later open the app. The older `apple-mobile-web-app-*` elements would serve iOS 16.3 and earlier only, and one extra element per page is a cost for a shrinking group.

### 8 - Do authors choose theme colours?

Decided by the agent without codekiln: no. The manifest uses the light header colour, and the page's `theme-color` elements use the header colour of each theme, so the title bar matches the page. Fewer keys means fewer states to check, and the colours already come from the site's themes.

### 9 - Should `lsg` offer an in-page install control, and where?

Decided by the agent without codekiln: an Install app item in the existing More menu, shown only when the browser fires `beforeinstallprompt`. That event is the only in-page mechanism with support: Chromium has it, Safari and Firefox do not, and `navigator.install()` needs an `id`. A control that appears only when it works avoids a button that does nothing in Safari. A banner or dialog for Safari would need browser sniffing, which breaks easily, and the documentation page serves those readers.

### 10 - Where does the install script live?

Decided by the agent without codekiln: in its own file, `lsg-install.js`, loaded with `defer` only on installable pages. `lsg-shell.js` must keep contents that depend only on the `lsg` version and is near its size budget, and a site without the key must load nothing new.

### 11 - Should installation add a service worker?

Decided by the agent without codekiln: no. Chrome installs from its menu without one since versions 108 and 112, Safari needs none, and the current promotion criteria list none. A service worker brings cache removal and update problems that belong to the offline change ([#607](https://github.com/codekiln/logseq-gardener/issues/607)).

### 12 - What are the performance budgets?

Decided by the agent without codekiln: the table in Budgets. The limits are set from the size of the tags and the script as designed, with margin: 0.4 KB of tags against a 0.5 KB row, a script written to stay under 1.5 KB, and a 256 KB icon cap that bounds a cost outside first paint. A site without the key has no budget change because its output does not change.

### 13 - How do installation and readable URLs, plugins, local files and the existing sites interact?

Decided by the agent without codekiln: the tags use relative URLs that resolve through the `<base>` of readable pages, so they work in folders, under a path prefix and from disk. Plugin navigation keeps the head. The documentation site turns the key on first, and the other two sites do so in their own issues after a release, because this repository may only change itself.

### 14 - What do the three generated files mean for the "files every page references" rule?

Decided by the agent without codekiln: they join it with fixed names, and their contents depend on the `lsg` version and the `:lsg/install` value rather than on the version alone, because the manifest holds the author's name. Editing pages changes none of them. This is a modification of the `published-site-size` requirement, not a new rule.

### 15 - Which devices must be checked before release?

Decided by the agent without codekiln: the checklist in Real-device checks: iPhone and iPad in Safari and Chrome, Mac in Safari, Chrome and Edge, Android in Chrome and Firefox, two gardens on one origin, a garden at the origin root, the icon and an offline-cost check. Rows that cannot run are recorded as not run with the reason, so a missing device shows up in review.

## Open Questions

None. Every question above is resolved.
