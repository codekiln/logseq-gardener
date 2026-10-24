#!/usr/bin/env python3
"""Report pages present in one build and missing from the other, by group."""
import argparse
import json
import re
import subprocess
import sys
import tarfile
import io
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import build  # noqa: E402
import common  # noqa: E402
import logseq_db  # noqa: E402

GROUPS = [
    ('journals', 'Journals'),
    ('reference_only', 'Reference-only pages (no content blocks)'),
    ('hidden', 'Hidden by :hidden in logseq/config.edn'),
    ('unselectable', 'Titles lsg cannot select'),
    ('other', 'Other pages'),
]


def hidden_patterns(config_text):
    """Read the string entries of :hidden from a config.edn text."""
    match = re.search(r'^\s*:hidden\s*\[(.*?)\]', config_text, re.S | re.M)
    return re.findall(r'"((?:[^"\\]|\\.)*)"', match.group(1)) if match else []


def is_hidden(relative_path, patterns):
    """Logseq's hidden? rule: '/' + path starts with the pattern, given a leading '/'."""
    path = '/' + relative_path
    for pattern in patterns:
        if not pattern.startswith('/'):
            pattern = '/' + pattern
        if path.startswith(pattern):
            return True
    return False


def hidden_titles(garden, patterns):
    titles = set()
    for folder in ('pages', 'journals'):
        for path in sorted((Path(garden) / folder).glob('*')):
            if path.is_file() and is_hidden(f'{folder}/{path.name}', patterns):
                titles.add(common.normalize(build.page_title(path.name)))
    return titles


def selectable(title):
    return all(build.valid_root(segment) for segment in title.split('/'))


def group_pages(logseq_pages, lsg_titles, hidden):
    """Place each Logseq-only page in one group and list lsg-only pages."""
    groups = {key: [] for key, _ in GROUPS}
    logseq_names = set()
    for page in sorted(logseq_pages, key=lambda p: p.name):
        name = common.normalize(page.name)
        logseq_names.add(name)
        if name in lsg_titles:
            continue
        if page.journal:
            key = 'journals'
        elif not page.has_content:
            key = 'reference_only'
        elif name in hidden:
            key = 'hidden'
        elif not selectable(page.title):
            key = 'unselectable'
        else:
            key = 'other'
        groups[key].append(page.title)
    groups['lsg_only'] = [
        {'title': title, 'hidden_in_logseq': name in hidden}
        for name, title in sorted(lsg_titles.items()) if name not in logseq_names
    ]
    return groups


def archive_gh_pages(garden, destination):
    destination.mkdir(parents=True)
    data = subprocess.run(['git', '-C', str(garden), 'archive', 'origin/gh-pages'],
                          capture_output=True, check=True).stdout
    with tarfile.open(fileobj=io.BytesIO(data)) as archive:
        archive.extractall(destination, filter='data')


def markdown(groups, summary):
    lines = ['# Parity page comparison', '']
    for key, value in summary.items():
        lines.append(f'- {key}: {value}')
    lines += ['', '| Group | Pages |', '| --- | --- |']
    for key, label in GROUPS:
        lines.append(f'| {label} | {len(groups[key])} |')
    lines.append(f"| Only in lsg | {len(groups['lsg_only'])} |")
    lines += ['', '## Other pages', '']
    lines += [f'- {title}' for title in groups['other']] or ['None.']
    lines += ['', '## Titles lsg cannot select', '']
    lines += [f'- {title}' for title in groups['unselectable']] or ['None.']
    lines += ['', '## Only in lsg', '']
    lines += [f"- {p['title']}" + (' (hidden in Logseq)' if p['hidden_in_logseq'] else '')
              for p in groups['lsg_only']] or ['None.']
    return '\n'.join(lines) + '\n'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--run', help='run directory from parity:build (default: the latest)')
    args = parser.parse_args()
    run = common.latest_run(args.run)
    settings = json.loads((run / 'settings.json').read_text(encoding='utf-8'))
    garden = Path(settings['garden'])
    subprocess.run(['git', '-C', str(garden), 'fetch', '--quiet', 'origin', 'gh-pages'], check=True)
    logseq_dir = run / 'logseq'
    if not logseq_dir.exists():
        archive_gh_pages(garden, logseq_dir)
    pages = logseq_db.read_pages(logseq_dir / 'static' / 'js' / 'db.gz')
    lsg_titles = {name: title for name, (title, _) in common.lsg_index(run / 'site').items()}
    patterns = hidden_patterns((garden / 'logseq' / 'config.edn').read_text(encoding='utf-8'))
    groups = group_pages(pages, lsg_titles, hidden_titles(garden, patterns))
    summary = {
        'garden commit': settings['garden_commit'],
        'gh-pages commit': common.git_commit(garden, 'origin/gh-pages'),
        'lsg version': f"{settings['lsg_version']} ({settings['lsg_commit'][:7]})",
        'Logseq pages': len(pages),
        'Logseq journals': sum(p.journal for p in pages),
        'Logseq pages with content': sum(p.has_content for p in pages),
        'lsg pages': len(lsg_titles),
    }
    (run / 'report.json').write_text(json.dumps({'summary': summary, 'groups': groups}, indent=2,
                                                ensure_ascii=False) + '\n', encoding='utf-8')
    (run / 'report.md').write_text(markdown(groups, summary), encoding='utf-8')
    print((run / 'report.md').read_text(encoding='utf-8').split('## Other pages')[0])
    print(f"Report: {run / 'report.md'}")


if __name__ == '__main__':
    main()
