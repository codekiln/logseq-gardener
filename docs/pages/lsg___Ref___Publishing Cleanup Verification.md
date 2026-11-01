tags:: [[Diataxis/Reference]]
see-also:: [[lsg/How To/Finish an OpenSpec Change After Merge]], [[lsg/Static Publishing]]

- # Publishing Cleanup Verification Reference
	- ## Overview
		- The record of the cleanup that synced and archived the completed publishing changes, written on 2026-10-08 in commit [6e50105 sync and archive merged publishing changes](https://github.com/codekiln/logseq-gardener/commit/6e50105). After that cleanup, the main specifications include audio and artwork, explicit outline navigation, selected-page aliases and CLI behavior. The completed publishing changes were archived together, so later requirements keep the behavior that earlier ones added.
		- [[lsg/How To/Finish an OpenSpec Change After Merge]] describes the cleanup process this record follows.
	- ## Merged implementation
		- Cleanup started from [21eed30 foundation specification cleanup](https://github.com/codekiln/logseq-gardener/commit/21eed30). [PR #25 post-merge cleanup process](https://github.com/codekiln/logseq-gardener/pull/25) reached `main` through its merge commit [5a7c31a workflow documentation](https://github.com/codekiln/logseq-gardener/commit/5a7c31af9a3399bb4c8bd1af4f73ab5a834346c1).
		- [PR #19 SDK publisher](https://github.com/codekiln/logseq-gardener/pull/19) and [PR #21 publishing CLI](https://github.com/codekiln/logseq-gardener/pull/21) originally merged into dependent branches. [PR #33 main integration](https://github.com/codekiln/logseq-gardener/pull/33) brought the publisher into `main` together with [PR #36 media](https://github.com/codekiln/logseq-gardener/pull/36), [PR #39 outline navigation](https://github.com/codekiln/logseq-gardener/pull/39), [PR #43 trial guide](https://github.com/codekiln/logseq-gardener/pull/43) and [PR #45 aliases](https://github.com/codekiln/logseq-gardener/pull/45).
		- GitHub reports their stack integration commit as [0ec7d92 merged publishing stack](https://github.com/codekiln/logseq-gardener/commit/0ec7d92c32b3e404de6e1e05ed205296c750058e), an ancestor of the starting `main` commit. The merged tree contains `site::publish_site`, CLI dispatch and publishing tests, `page_names` alias lookup, and the cumulative media and outline renderer. The cleanup left the implementation files unchanged.
		- All selected changes had complete planning artifacts and tasks. The workflow's review task was marked complete after checking the fresh final empty reviews on [PR #25 cleanup process](https://github.com/codekiln/logseq-gardener/pull/25). The foundation archives from [PR #34 namespace, loader, and title cleanup](https://github.com/codekiln/logseq-gardener/pull/34) remained intact, and the parser research remained active.
	- ## Synchronization
		- The archive order is workflow, SDK publisher, CLI publisher, media, outline navigation, then aliases. The main specs keep the unaffected requirements and scenarios.
		- The final rendering description matches the alias delta, which keeps the media and outline behavior. The merged media and navigation implementations supersede earlier descriptions that required all remote images or block references to fall back; the exclusion and escaped-content scenarios of those descriptions remain intact.
		- The comparison below checks every archived delta requirement, every earlier scenario, the final replacement descriptions, and the unchanged requirements from the starting `main` specs. It passes on the specs as they stood at commit `6e50105`, run from the repository root of a checkout of that commit. Later changes have modified these specs, so on current `main` it stops at the first changed scenario.
		- ```sh
		  python3 - <<'PY'
		  from pathlib import Path
		  import re
		  import subprocess
		  
		  names = ['post-merge-openspec-cleanup', 'local-static-site',
		           'cli-static-publishing', 'selected-garden-media',
		           'selected-block-navigation', 'selected-page-aliases']
		  
		  def sections(text, heading):
		      pattern = rf'^{heading}: (.+)\n(?:(?!^#{{1,4}} (?:Requirement|Scenario):|^## ).|\n)*'
		      # Requirement blocks include scenarios; only the next requirement ends a block.
		      if heading == '### Requirement':
		          pattern = r'^### Requirement: (.+)\n(?:(?!^### Requirement:|^## ).|\n)*'
		      return {m.group(1).strip(): m.group(0).strip()
		              for m in re.finditer(pattern, text, re.M)}
		  
		  latest = {}
		  for name in names:
		      change = Path('openspec/changes/archive') / ('2026-10-08-' + name)
		      assert '- [ ]' not in (change / 'tasks.md').read_text(), name
		      for delta in sorted((change / 'specs').glob('*/spec.md')):
		          capability = delta.parent.name
		          main = sections((Path('openspec/specs') / capability / 'spec.md').read_text(),
		                          '### Requirement')
		          for title, block in sections(delta.read_text(), '### Requirement').items():
		              latest[capability, title] = block
		              actual = sections(main[title], '#### Scenario')
		              for scenario, text in sections(block, '#### Scenario').items():
		                  assert actual[scenario] == text, (name, title, scenario)
		  for (capability, title), block in latest.items():
		      main = sections((Path('openspec/specs') / capability / 'spec.md').read_text(),
		                      '### Requirement')
		      assert main[title] == block, (capability, title)
		  for capability in {cap for cap, _ in latest}:
		      path = f'openspec/specs/{capability}/spec.md'
		      before = subprocess.run(['git', 'show', f'21eed30:{path}'],
		                              capture_output=True, text=True)
		      if before.returncode:
		          continue
		      main = sections(Path(path).read_text(), '### Requirement')
		      for title, block in sections(before.stdout, '### Requirement').items():
		          if (capability, title) not in latest:
		              assert main[title] == block, (capability, title)
		  print('Archived requirements, cumulative scenarios, and unaffected requirements match.')
		  PY
		  ```
	- ## Functional evidence
		- The SDK publication tests cover selection, private-page withholding, output protection, escaping, assets, outline targets and aliases. The CLI publication tests cover native paths, streams and exit codes, excluded and private media, and source preservation. The archived verification records keep the original browser demonstrations and their limits.
		- A combined trial on merged `main` generated workshop and podcast pages with copied assets. Every emitted relative file and anchor link resolved. The generated files contained neither the excluded September episode's recording URL nor its distinctive description, and the excluded workshop and podcast titles had no generated destinations. Source fingerprints across pages, journals, assets and configuration matched before and after generation.
		- These are bounded checks of the source snapshot at the time, supported by the synthetic exclusion fixtures. Playback was demonstrated on the feature branch; the merged-main trial adds link and source-preservation evidence.
		- The trial guide, now [[lsg/Tutorial/Publish Part of a Public Garden]], creates its checkout from `origin/main`.
