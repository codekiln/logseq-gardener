tags:: [[Diataxis/Reference]]

- # Spec: inline syntax
	- This board lists each inline Markdown or Logseq construct a block can contain, with what Logseq draws for it.
	- Board title: "Inline markup inside a block".
	- Gardener specs: `email-autolinks`, `selected-page-aliases`.
	- Logseq source: `block.cljs › inline, page-cp, macro-cp`; `common.css a.tag, mark, .external-link, :not(pre) > code`.
	- Board colours: link `#106ba3` (hover `#0b4f7a`), brackets `#6b7280`.
	- ## Page reference
		- ```markdown
		  [[tmux/Config]]
		  ```
		- Logseq renders: "tmux/Config" as a link between grey brackets.
		- DOM and rules:
			- `a.page-ref`, link colour; brackets are separate grey `span.text-gray-500` (`:ui/show-brackets?`).
			- Target matched case-insensitively; aliases resolve to the original page.
	- ## Labelled page reference
		- ```markdown
		  [that config]([[tmux/Config]])
		  ```
		- Logseq renders: "that config" as a link, no brackets.
		- DOM and rules:
			- Link text only, no brackets, same target.
	- ## Tags
		- ```markdown
		  #card   #[[Keyshort]]   #tmux/Keyshort
		  ```
		- Logseq renders: "`#card`", "`#Keyshort`", "`#tmux/Keyshort`" as small tag links (colour `#3e63dd` on the board).
		- ```html
		  <a class="tag" style="font-size:.9em;opacity:.7;padding:0 2px;border-radius:4px;color:#3e63dd">#card</a>
		  ```
		- DOM and rules:
			- `a.tag`: 0.9em, accent colour at 70% opacity, padding `0 2px`, radius 4; underline on hover.
			- A tag is a page ref: it appears in that page's Linked References and "Pages tagged with".
			- `#[[Keyshort]]` renders as `#Keyshort` (brackets dropped).
	- ## Journal reference
		- ```markdown
		  [[2026-07-04 Sat]]
		  ```
		- Logseq renders: "2026-07-04 Sat" as a link between grey brackets.
		- DOM and rules:
			- Journal refs use the graph's `:journal/page-title-format`; they open the journal page.
	- ## External link
		- ```markdown
		  [Panel | Visual Studio Code Extension API](https://code.visualstudio.com/api/ux-guidelines/panel)
		  ```
		- Logseq renders: "Panel | Visual Studio Code Extension API" as a link with a 1px solid bottom border (`#106ba3`).
		- DOM and rules:
			- `a.external-link`: 1px solid bottom border, opens a new tab.
	- ## Email autolink
		- ```markdown
		  <someone@example.com>
		  ```
		- Logseq renders: "someone@example.com" as a link styled like an external link.
		- DOM and rules:
			- Autolinked address becomes a `mailto:` link, styled like an external link.
	- ## Inline code
		- ```markdown
		  `<prefix> z`
		  ```
		- Logseq renders: `<prefix> z` in a mono chip (board: 0.9em mono, background `#e0e1e6`, colour `#60646c`, padding `1px 4px`, radius 3px).
		- DOM and rules:
			- `:not(pre) > code`: 0.9em mono, radius 3, background gray-05/06, text gray-11.
	- ## Emphasis, hidden text, highlight, strikethrough
		- ```markdown
		  **Zooms** · _hidden_ · ==fills the window== · ~~old~~
		  ```
		- Logseq renders: **Zooms** bold, _hidden_ italic, "fills the window" highlighted, ~~old~~ struck through.
		- DOM and rules:
			- `mark`: background `#fef3ac`, text `#262626`, padding `2px 4px`, radius 3.
			- Bold is `<b>`, italic `<i>`, strikethrough `<s>`.
	- ## Images
		- ```markdown
		  ![panel image](https://code.visualstudio.com/assets/api/ux-guidelines/examples/panel.png)
		  ![diagram](../assets/[file].png)
		  ```
		- Logseq renders: the image. The board shows a placeholder box (240×120, background `#f1f3f5`, radius 4px, label "[image]", 13px, `#687076`) instead of a real image; the placeholder is the board's, not a Logseq style.
		- DOM and rules:
			- Inline `img`, max-width 100%.
			- Relative `../assets/` paths must be copied into the build and rewritten.
	- ## Macros
		- ```markdown
		  {{youtube-timestamp 620}}
		  {{cloze <prefix> z}}
		  {{embed …}}  {{query …}}
		  ```
		- Logseq renders:
			- `{{youtube-timestamp 620}}` as a link "10:20".
			- `{{cloze <prefix> z}}` as a grey chip `[…]` (background `#e9ecef`, radius 3px, padding `0 6px`) while reviewing a card, plain text otherwise.
		- DOM and rules:
			- `youtube-timestamp` gives a seek link.
			- `cloze` hides its text during card review.
			- `embed` is on the transclusion sheet (see [[lsg/Ref/Logseq Published Site/Transclusion]]).
			- `query` runs live in the app, so a static build must pre-compute it or show it as unsupported.
			- Unknown macros render `span.warning` with the text "Unsupported macro name".
