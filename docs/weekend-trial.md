# Try a selected garden site

Build a local static site from the public Logseq garden, read workshop notes, and listen to a GitP episode. The trial selects both namespaces and removes a workshop subtree and a podcast session. Use the generated site to decide what still prevents replacing Logseq garden publish for your own garden.

## Get the reviewed publisher

The trial requires the publishing integration, media rendering, and block navigation changes. Until they reach `main`, use the branch for [PR #39 — selected block navigation](https://github.com/codekiln/logseq-gardener/pull/39), which includes [PR #36 — audio and artwork](https://github.com/codekiln/logseq-gardener/pull/36) and [PR #33 — publishing integration](https://github.com/codekiln/logseq-gardener/pull/33). These PRs need to merge in that order: integration, media, then navigation. A GitHub merged status on the earlier publishing stack alone does not put the publisher on `main`.

From the registered `logseq-gardener` checkout, create a separate trial checkout:

```sh
repo_root="$(ghq list --full-path --exact github.com/codekiln/logseq-gardener)"
git -C "$repo_root" fetch origin codex/38-selected-block-navigation
git -C "$repo_root" worktree add --detach "$repo_root/.worktrees/weekend-local-trial" FETCH_HEAD
cd "$repo_root/.worktrees/weekend-local-trial"
mise install
```

Choose another checkout name if `weekend-local-trial` already exists. After the dependent features reach `main`, create the checkout from an updated `origin/main`. The repository's mise configuration supplies Rust and the build tools; the trial also needs `ghq` and Python. The public garden must already be present in ghq.

## Build and serve

Run the following from the trial checkout. `mktemp` creates a fresh parent directory; `site` must not exist when generation begins. Each rerun creates another destination, so removed namespaces cannot leave files from an earlier build behind.

```sh
garden_root="$(ghq list --full-path --exact github.com/codekiln/logseq-encode-garden)"
trial_root="$(mktemp -d "${TMPDIR:-/tmp}/lsg-weekend.XXXXXX")"
mise exec -- cargo run --bin lsg -- publish \
  --graph "$garden_root" --output "$trial_root/site" \
  --filename-format triple-lowbar \
  --include AI/ES/25/ws --exclude AI/ES/25/ws/3 \
  --include GitP/A/Session --exclude GitP/A/Session/26/09/24-Thu \
  --format json > "$trial_root/result.json" 2> "$trial_root/report.txt"
cat "$trial_root/result.json"
printf 'Local report: %s\n' "$trial_root/report.txt"
python3 -m http.server 8000 --bind 127.0.0.1 --directory "$trial_root/site"
```

Open <http://127.0.0.1:8000/> while the server runs. Stop the server with Ctrl-C after reading. If port 8000 is occupied, choose another port in both the command and URL. The server serves only the generated site on this computer's loopback interface. Hosted artwork and recordings still load from their original HTTPS hosts when viewed.

The public encode garden uses triple-lowbar filenames. For another garden, inspect `logseq/config.edn`: use `triple-lowbar` for `:file/name-format :triple-lowbar`, otherwise use `legacy`. A leading `title::` property may supply the logical page name. Include and exclude roots match that logical name, with `/` separating descendants. Exclusions take precedence and matching is case-sensitive.

## Read the result

From the index, open “What I Learned at the AI Engineer Summit Workshops 2025.” Follow the reference below “Person/Mike Christensen”: the page should change to “Building Agents with Model Context Protocol” and jump to the referenced outline note. The target note's short excerpt is the link label. Some workshop headings use aliases and remain literal with diagnostics; [Issue #42 — selected publishing alias lookup](https://github.com/codekiln/logseq-gardener/issues/42) tracks the remaining compatibility gap.

Return to the index and open `GitP/A/Session/24/11/19-Tue`. Check that the artwork appears and the recording plays through the audio controls, then pause the recording. Listening requires access to the media host and a browser that supports the recording codec. “Open recording” follows the hosted file; host and browser behavior determine whether the file plays or downloads.

Check the index for the excluded `AI/ES/25/ws/3` subtree and September 24 GitP session. Their pages and target assets should be absent. Namespace selection does not redact text already written on retained pages: a workshop summary can still name an excluded workshop. Missing or excluded destinations keep the retained source's label and receive a diagnostic. Inspect retained pages before publishing sensitive content.

A page containing a parsed `public:: false` anywhere in its outline is withheld as a whole. Unmarked pages matching the namespace selection are published. Gitpa's [site preparation](https://github.com/codekiln/gitpa/blob/main/scripts/prepare_site.py) selects marked production-note sections and expands episode asset embeds before [Logseq publication](https://github.com/codekiln/gitpa/blob/main/.github/workflows/gh-pages.yml); the Rust trial currently uses whole-page privacy and literal fallbacks for embeds and queries. Journals, aliases, and precise private-outline publication also remain incomplete. See [publisher behavior](local-site.md) for the supported syntax and asset rules.

## Decide what to build next

Read the local report alongside a page where something useful is missing. Report the source page name, the syntax or reference, what the generated page shows, and what a reader should see instead. Include the namespace selection and filename format so the result can be repeated.

Give product feedback through [Issue #41 — reproducible weekend trial](https://github.com/codekiln/logseq-gardener/issues/41): would you use this output for your next garden site; which missing behavior prevents that; and which feature could wait? Focus on the actual reading or listening task. [The combined trial record](../openspec/changes/selected-block-navigation/experiments/weekend-trial/verification.md) describes the checked source and output; [Issue #11 — selected-namespace publishing](https://github.com/codekiln/logseq-gardener/issues/11) tracks implementation priorities.
