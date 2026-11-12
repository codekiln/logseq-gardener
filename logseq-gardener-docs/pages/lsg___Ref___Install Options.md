tags:: [[Diataxis/Reference]]
see-also:: [[lsg/How To/Install a Published Garden as an App]], [[lsg/Ref/Publish Options]], [[lsg/Ref/First-Party Plugins]]

- # Install Options Reference
	- ## Overview
		- A garden lets readers install its site as an app with one top-level key in its own `logseq/config.edn`. The installed app has the name and icon the author chose and opens in its own window. Logseq ignores the key.
		- ~~~clojure
		  :lsg/install {:name "Ghost in the Patch" :short-name "GitP" :icon "assets/app-icon.png"}
		  ~~~
		- A garden without the key builds the same site as before: no manifest, icon, tag or script. The key is the only switch. `lsg publish` has no flag for installation, and Rust callers of `publish_site_with_options` get the same result as the command line.
		- [[lsg/How To/Install a Published Garden as an App]] tells readers how to install a site.
	- ## The key
		- | Entry | Required | Meaning |
		  | --- | --- | --- |
		  | `:name` | Yes | The app's name. 1 to 45 characters after trimming. |
		  | `:short-name` | When `:name` is longer than 12 characters | The name that a Home Screen shows under the icon. 1 to 12 characters after trimming. It defaults to `:name` when `:name` has at most 12 characters. |
		  | `:icon` | Yes | A path relative to the garden folder, such as `assets/app-icon.png`. |
		- The build stops before it creates the output directory, exits 1 with empty stdout, and writes a message to stderr that names `logseq/config.edn`, the key and the problem, when:
			- the value is not a map;
			- an entry other than `:name`, `:short-name` and `:icon` appears, such as `:shortname`;
			- `:name` or `:icon` is missing, an entry is not a string, or an entry repeats;
			- `:name` or `:short-name` is outside its length;
			- `:name` has more than 12 characters and there is no `:short-name`;
			- the key repeats, or sits inside another entry, such as `:default-home {:lsg/install {...}}`. It belongs at the top level.
		- ~~~text
		  logseq/config.edn: :lsg/install has the key :shortname; the accepted keys are :name, :short-name, :icon
		  ~~~
	- ## The icon
		- The icon is one file, and `lsg` copies it to `lsg-icon.png` unchanged. It does not resize, convert or compress the image. The build stops, naming the path and the check that failed, when:
			- the path is empty, absolute, holds `..` or a backslash, or the file resolves to a place outside the garden folder, as through a symlink;
			- the file does not exist or is not a regular file;
			- the file is larger than 256 KB, where 1 KB is 1,000 bytes;
			- the file is not a PNG, such as an SVG or a JPEG;
			- the image is not square, or a side is outside 512 to 1,024 pixels. `lsg` reads these from the PNG header.
		- Choose an icon that reads at 48 pixels and keeps its subject inside the central 80 percent of the image, because some launchers cut the icon to a circle.
	- ## What the build writes
		- | File | Contents | Limit |
		  | --- | --- | --- |
		  | `lsg.webmanifest` | The web app manifest | 1 KB |
		  | `lsg-icon.png` | The author's icon | 256 KB |
		  | `lsg-install.js` | The script for the Install app item | 1.5 KB, 1 KB gzip |
		- The three files sit at the site root with fixed names. Their contents depend only on the `lsg` version and the key's value and icon, so a change to a page leaves them and every page's references to them unchanged. They are not listed in [[lsg/Ref/Site Manifest]]. With `--readable-urls`, the names are reserved like the site's other generated files: a page titled `lsg.webmanifest` stops the build before it creates the output directory.
		- Every full page, which means the home page, the journals feed, All pages, the graph view, namespace pages and page folders, gains five elements in its head: a link to the manifest, an `apple-touch-icon` link to the icon, two `theme-color` elements for light and dark appearance, and the deferred install script. It also gains one hidden menu item. Redirect pages and the fragments that scripts fetch gain nothing. All paths are relative to the site root, so they resolve through the `<base>` of a readable page. A page grows by about 0.4 KB.
		- The manifest, for the key above:
		- ~~~json
		  {"name":"Ghost in the Patch","short_name":"GitP","start_url":"./","scope":"./","display":"standalone","theme_color":"#fbfcfd","background_color":"#fbfcfd","icons":[{"src":"lsg-icon.png","type":"image/png","sizes":"512x512","purpose":"any"}]}
		  ~~~
		- Non-ASCII characters in the names are written as `\u` escapes, so the file is plain ASCII. `theme_color` and `background_color` are the header colour of the light theme, and the page's two `theme-color` elements use the header colour of each theme.
	- ## Scope and identity
		- `start_url` and `scope` are `./`. A browser resolves them against the manifest's address, so they name the garden's own folder wherever it is hosted: at `/`, at `/gitpa/`, or anywhere else. The manifest is the same bytes at every address.
		- The manifest has no `id`, so its identity is its start address. Two gardens on one origin, such as `codekiln.github.io/gitpa/` and `codekiln.github.io/logseq-gardener/`, install as two apps. A relative `id` would give every garden on that origin one identity.
		- Moving a garden to another path or host gives it a new identity. An installed copy keeps opening the old address.
		- A garden published at the root of an origin, such as `owner.github.io`, has a scope that covers every project site on that origin. A link from the garden to a project site on the same origin can open inside the installed app instead of the browser.
	- ## The Install app item
		- Chromium browsers fire `beforeinstallprompt` when they offer installation. `lsg-install.js` keeps that event and shows an Install app item in the More menu. Activating the item opens the browser's install dialog once, hides the item and returns focus to the More button.
		- The item stays hidden in Safari and Firefox, which never fire the event, in an installed window, and on a page opened from disk. It is a native button, so a keyboard reaches and activates it. The script reads and writes no cookie or browser storage and sends no request.
		- Chrome offers the event only after the reader has clicked the page and spent about 30 seconds on it, so the item can appear late. The browser's menu installs at any time.
	- ## Limits
		- Installation adds no service worker, no cache and no browser storage, and the build writes no file named for a service worker. Offline reading is a separate decision.
		- Installing needs `https://` or `localhost`. A page opened from disk renders and navigates as it does without the key, and cannot be installed.
		- One icon at its real size serves every launcher. There is no maskable icon and no icon per size. Chrome documents 192 and 512 pixel icons for its install criteria; the checks below record what Chrome does with one 512 pixel icon.
		- The colours cannot be chosen.
		- Safari and Firefox readers get no in-page control; link [[lsg/How To/Install a Published Garden as an App]] from the garden to tell them where the Share menu is.
	- ## Size budgets
		- | Budget | Limit |
		  | --- | --- |
		  | Page growth with the key | 0.5 KB, 0.3 KB gzip |
		  | `lsg-install.js` | 1.5 KB, 1 KB gzip |
		  | `lsg.webmanifest` | 1 KB |
		  | Icon | 256 KB |
		  | First paint of any page | The existing 100 KB of data and 16 KB of script |
		  | Added build time on the encode garden | 1,000 ms, above the run-to-run noise of a 20 s build |
		- `mise run parity:budgets` and `mise run parity:build-bench` check these; [[lsg/Ref/Logseq Compatibility/Performance]] records the measurements. A garden without the key pays nothing.
	- ## What has been checked
		- Chromium checks run with `mise run plugins:browser-test`: the manifest and icon load from a nested page under a path prefix, Chromium parses the manifest and resolves `start_url` and `scope` to the site folder, reading adds no service worker, cache or storage entry, the item stays hidden until the event fires and then works from the keyboard, the manifest link survives plugin navigation, and a page opened from disk raises no script error. Unit and integration tests cover the key, the icon checks, the manifest bytes and the tags on every kind of page, with and without `--readable-urls`.
		- Checks on phones, tablets and computers are recorded in [Issue #605](https://github.com/codekiln/logseq-gardener/issues/605) and its pull request, with the device, system, browser version and date of each. A check that could not run says so.
	- ## Related
		- [[lsg/Ref/Publish Options]] lists the options of `lsg publish`.
		- [[lsg/Ref/First-Party Plugins]] describes the other key that a garden's `logseq/config.edn` holds for `lsg`.
