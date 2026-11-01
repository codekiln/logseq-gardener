tags:: [[Diataxis/Reference]]
see-also:: [[lsg/Ref/Published Pages]], [[lsg/Ref/Selection and Visibility]], [[lsg/Ref/Logseq Compatibility]]

- # Rendering and Assets Reference
	- ## Overview
		- `lsg publish` renders the blocks of each published page as HTML, keeps links and media working, and copies the local assets that published content uses into the site. Generation reads only local files; it performs no remote fetch.
		- [[lsg/Ref/Logseq Compatibility]] compares each rendered feature with Logseq's published site.
	- ## Blocks and inline syntax
		- The publisher renders outlines, headings, paragraphs, emphasis, literal code, quotes and tables.
		- Raw HTML and Hiccup appear as escaped literal content.
		- Unsupported syntax gets a visible fallback in the page and a diagnostic in the report.
	- ## Links
		- Ordinary HTTP, HTTPS and mailto links stay links.
		- An email autolink such as `<reader@example.org>` becomes a mailto link. An address in literal code stays literal, and an address inside another link's label stays plain label text.
		- [[lsg/Ref/Selection and Visibility]] describes page references, block references and embeds.
	- ## Remote media
		- A supported HTTPS raster image renders directly. The browser loads remote media when the site is viewed.
		- HTTP media and unsupported remote types get a fallback.
		- HTTPS media needs a supported extension at the end of its path, before any query or fragment. A URL with credentials, control characters, backslashes or no authority gets a fallback.
	- ## Local assets
		- A local asset URL resolves relative to its source document and must point beneath the graph's `assets/` folder.
		- The publisher copies only the assets that selected rendered content requests.
		- It rejects symlinks, paths that leave `assets/`, absolute URLs, URLs with a query or fragment, and unsupported file types.
		- Supported types are PNG, JPEG, GIF, WebP, AVIF, ICO, PDF, MP3, MP4, OGG, WAV and WebM.
	- ## Media syntax
		- | Image syntax pointing at | Renders as |
		  | --- | --- |
		  | A raster image | An image |
		  | An MP3, WAV or OGG recording | Labelled audio controls and a link to the recording |
		  | A PDF or a video | A link |
		- An ordinary Markdown link to an asset stays a link.
		- A link to a local recording requests a download. A link to a hosted recording may open it in the browser; whether it downloads depends on the host and the browser.
		- Browser playback depends on the recording's codec.
	- ## Page routes
		- Each page has a deterministic, fixed-length route derived from its title, `p-<digest of its title>.html` beside `index.html`, and the publisher checks routes for collisions.
		- Each page carries its own styling and links other pages with relative routes, so the site can be read from disk or served by any static file server.
		- With `--readable-urls`, each page is written to a folder named by its title instead, such as `GitP/A/Session/index.html` linked as `GitP/A/Session/`. [[lsg/Ref/Publish Options]] describes that layout and the titles it refuses.
