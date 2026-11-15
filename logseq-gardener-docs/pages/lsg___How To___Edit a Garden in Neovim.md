tags:: [[Diataxis/How To]]
see-also:: [[lsg/Ref/Command Line]], [[lsg/How To/Look Up a Page File]]

- # How To Edit a Garden in Neovim
	- ## Overview
		- `lsg lsp` is a language server for Logseq Markdown gardens. Neovim starts it when you open a page or journal, and the server reads the whole garden so it can tell the editor where each page link, tag, block reference and property leads.
		- With the server attached you can jump from a link to its page or block, list every place that links to a page or a block, and read a page or block in a hover window. The server reports broken block references and repeated block ids as diagnostics, and keeps its index of the garden current while you edit. Completion and rename arrive in later releases and need no change to the configuration on this page.
	- ## Prerequisites
		- Neovim 0.11 or later.
		- `lsg` on your `PATH`, installed as [[lsg/How To/Install with mise]] shows.
		- A Logseq Markdown garden: a folder that holds `logseq/config.edn`.
	- ## Steps
		- ### 1. Add the server to your Neovim configuration
			- Put these lines in `init.lua`, or in any Lua file your configuration loads:
				- ```lua
				  vim.lsp.config("logseq_gardener", {
				    cmd = { "lsg", "lsp" },
				    filetypes = { "markdown" },
				    root_dir = function(bufnr, on_dir)
				      local root = vim.fs.root(bufnr, function(name, path)
				        return name == "logseq" and vim.uv.fs_stat(path .. "/logseq/config.edn") ~= nil
				      end)
				      if root then
				        on_dir(root)
				      end
				    end,
				  })
				  vim.lsp.enable("logseq_gardener")
				  ```
			- `root_dir` starts the server only for a Markdown buffer inside a folder that holds `logseq/config.edn`, and passes that folder to the server as the garden. Markdown files anywhere else keep the servers you already use.
		- ### 2. Use the LazyVim form
			- With LazyVim, add the server to `nvim-lspconfig` in a plugin file such as `lua/plugins/logseq-gardener.lua`. `mason = false` tells LazyVim that you install `lsg` yourself:
				- ```lua
				  return {
				    "neovim/nvim-lspconfig",
				    opts = {
				      servers = {
				        logseq_gardener = {
				          mason = false,
				          cmd = { "lsg", "lsp" },
				          filetypes = { "markdown" },
				          root_dir = function(bufnr, on_dir)
				            local root = vim.fs.root(bufnr, function(name, path)
				              return name == "logseq" and vim.uv.fs_stat(path .. "/logseq/config.edn") ~= nil
				            end)
				            if root then
				              on_dir(root)
				            end
				          end,
				        },
				      },
				    },
				  }
				  ```
		- ### 3. Check that the server attached
			- Open a page of your garden and run `:checkhealth vim.lsp`. The report lists `logseq_gardener` under the active clients, with your garden folder as its root directory.
			- Ask the server what it loaded:
				- ```vim
				  :lua vim.print(vim.lsp.get_clients({ name = "logseq_gardener" })[1]:request_sync("lsg/status", vim.NIL, 5000, 0).result)
				  ```
			- The answer gives the garden root, the filename format and journal formats from `logseq/config.edn`, and the number of pages and journals.
		- ### 4. Ask what the cursor is on
			- Put the cursor on a page link, tag, block reference or property and run:
				- ```vim
				  :lua vim.print(vim.lsp.get_clients({ name = "logseq_gardener" })[1]:request_sync("lsg/occurrence", vim.lsp.util.make_position_params(0, "utf-8"), 5000, 0).result)
				  ```
			- The answer names what is under the cursor and the page or block it leads to. An alias gives the page that declares it, a journal date in any format Logseq accepts gives the journal, and a link to a page with no file says so.
		- ### 5. Follow a link, list references and read a hover
			- Put the cursor on a page link, a tag, an alias, a journal date, a `((block reference))`, an `{{embed}}` or a page name in a property such as `tags::`, then use one of these:
				- | Action | Neovim | LazyVim |
				  | --- | --- | --- |
				  | Go to the page or block | `CTRL-]` or `:lua vim.lsp.buf.definition()` | `gd` |
				  | List the places that name it | `grr` | `gr` |
				  | Show it in a hover window | `K` | `K` |
			- Go to definition opens the page's file, and puts the cursor on the block for a block reference. A link to a page or journal that has no file opens a new buffer at the path Logseq would create, such as `pages/Fruit___Quince.md` or `journals/2026_01_04.md`. The file appears when you save the buffer.
			- References from a link list every place that names the same page by its title or by one of its aliases. With the cursor in ordinary text, references list the places that link to the page you are in. On a block reference or an `id::` value they list the references and embeds of that block.
			- Hover on a page link shows the page's title, its aliases, how many places name it and its first lines. Hover on a block reference shows the block's text below its page and parent blocks.
		- ### 6. Read the diagnostics
			- The server marks three problems in a page:
				- a `((uuid))` or `{{embed ((uuid))}}` whose id no block in the garden has, as an error;
				- an `id::` value that more than one block writes, as an error on each of those blocks. Logseq keeps the id on one block and gives the others a new id, so references to them stop working;
				- a `{{embed [[page]]}}` whose page has no file, as a warning.
			- `:lua vim.diagnostic.setqflist()` lists the problems the server has reported, and `]d` and `[d` move between the ones in the current buffer. If your configuration turns diagnostics off, `:lua vim.diagnostic.enable(true)` shows them.
			- The marks follow your edits, saved or not. Giving a block the missing id in another buffer clears the error on the reference, and changing one of two equal ids clears both errors.
			- A link or tag to a page without a file gets no mark, because linking a page is how Logseq creates it.
	- ## What the server reads
		- The server reads `:file/name-format`, `:journal/page-title-format`, `:journal/file-name-format` and `:hidden` from `logseq/config.edn`, then every Markdown file beneath `pages/` and `journals/`. It skips names that start with a dot, `node_modules` and the paths `:hidden` lists. On the encode garden, about 9,300 pages and 525 journals, reading takes a little over a second, and Neovim shows the progress.
		- The text in an open buffer takes the place of its file until you close the buffer, so the server sees edits you have not saved.
		- When the Logseq app, git or another program changes a file, the server reads it again. A change to `logseq/config.edn` reads the whole garden again.
		- The server changes no file and offers no formatting, so the tab indentation of your outline stays as you wrote it.
	- ## Troubleshooting
		- If no client attaches, check that the buffer's filetype is `markdown` and that a folder at or above the file holds `logseq/config.edn`.
		- If Neovim reports that `lsg` is not executable, run `command -v lsg` in the shell you start Neovim from. [[lsg/How To/Install with mise]] shows how to put `lsg` on your `PATH`.
		- If another Markdown server reports every link to a page without a file as an error, turn that server off for folders that hold `logseq/config.edn`. In Logseq, linking a page is how you create it.
		- `:lua vim.cmd.edit(vim.lsp.log.get_filename())` opens the log that holds the server's messages.
