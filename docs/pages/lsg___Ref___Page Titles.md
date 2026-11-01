tags:: [[Diataxis/Reference]]
see-also:: [[lsg/Ref/Garden Loading]], [[lsg/How To/Use the Rust SDK]]

- # Page Titles Reference
	- ## Overview
		- `lsg` and the SDK's `page_titles::page_title` derive the title of each page from its file's name or from a leading `title::` property. The caller chooses the filename format, which matches the garden's `:file/name-format` setting in `logseq/config.edn`: `pages/Names___Nested.md` is the page `Names/Nested` with the triple-lowbar format and `Names___Nested` with the legacy format.
		- The rules follow the Logseq OG source revision recorded in the [relationship experiment](https://github.com/codekiln/logseq-gardener/blob/main/openspec/changes/current-og-relationships/experiments/file-relationships/README.md).
	- ## Rules
		- ### Contents page
			- A file directly in `pages/` whose name starts with `contents.` is the page `Contents`, including `pages/contents.extra.md`. The rule uses native path separators and applies before properties.
		- ### Title property
			- A `title::` property in the first root properties block of the file overrides the file name. Property keys match without regard to ASCII letter case, and the last `title::` property in that block wins.
			- A `title::` in a later drawer or in a bullet's properties leaves the title from the file name.
		- ### File name
			- A title from the file name uses the final stem of the file's path. Folder names are file locations, and a file in any folder other than `journals/` is a page: `tools-docs/PARAMETERS.md` is the page `PARAMETERS`.
		- ### Legacy format
			- The legacy format replaces each `.` in the stem with `/`, then decodes the whole string as percent-encoded UTF-8.
			- A malformed escape or invalid UTF-8 keeps the whole string with its dots replaced and its escapes undecoded.
		- ### Triple-lowbar format
			- The triple-lowbar format replaces each non-overlapping `___` in the stem with `/`, decodes each ASCII percent escape once, and removes empty segments between slashes.
			- Escapes of non-ASCII bytes stay encoded: `%C3%A9` stays `%C3%A9`, where the legacy format decodes it to `é`.
	- ## Errors
		- `page_title` returns a `PageTitleError` with the file's relative path and the reason when the file is a journal, when its path is not a Markdown path outside `journals/`, when its file name is not valid UTF-8, or when the title is empty.
		- An empty or whitespace-only title is an error the SDK raises for publishing. Logseq OG itself accepts an empty `title::` override.
	- ## Technical Details
		- Titles keep their capitalization and every other character. A title is a display title; looking up a page by name uses a separate normalized key.
		- Journal pages take their titles from their dates, which `page_title` does not derive.
