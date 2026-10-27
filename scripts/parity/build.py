#!/usr/bin/env python3
"""Publish every page of a garden that lsg can select into a new run directory."""
import argparse
import json
import subprocess
import sys
import unicodedata
import urllib.parse
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import common  # noqa: E402

FILENAME_FORMAT = 'triple-lowbar'


def page_title(file_name):
    """Decode a triple-lowbar page file name into its logical title."""
    stem = file_name.rsplit('.', 1)[0]
    return '/'.join(urllib.parse.unquote(part) for part in stem.split('___'))


def valid_root(root):
    """Apply lsg's selector rules to one namespace root."""
    for segment in root.split('/'):
        if not segment or segment != segment.strip():
            return False
        if any(unicodedata.category(ch) == 'Cc' for ch in segment):
            return False
    return True


def roots(pages_dir):
    """Return the selectable top-level roots and the files whose root fails lsg's rules."""
    selected, skipped = set(), []
    for path in sorted(Path(pages_dir).iterdir()):
        if path.suffix not in ('.md', '.org') or not path.is_file():
            continue
        root = urllib.parse.unquote(path.stem.split('___')[0])
        if valid_root(root):
            selected.add(root)
        else:
            skipped.append(path.name)
    return sorted(selected), skipped


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--garden', help='garden path (default: the encode garden from ghq)')
    args = parser.parse_args()
    garden = Path(args.garden) if args.garden else common.encode_garden()
    subprocess.run(['cargo', 'build', '--locked', '--release', '--bin', 'lsg'], cwd=common.REPO, check=True)
    lsg = common.lsg_binary()
    selected, skipped = roots(garden / 'pages')
    run = common.new_run_dir()
    options = ['--filename-format', FILENAME_FORMAT, '--journals', '--search']
    for root in selected:
        options += ['--include', root]
    command = [str(lsg), 'publish', '--graph', str(garden), '--output', str(run / 'site'), *options, '--format', 'json']
    with open(run / 'diagnostics.txt', 'w', encoding='utf-8') as diagnostics:
        result = subprocess.run(command, stdout=subprocess.PIPE, stderr=diagnostics, text=True, check=True)
    version = json.loads(subprocess.run([str(lsg), 'version', '--format', 'json'],
                                        capture_output=True, text=True, check=True).stdout)
    settings = {
        'garden': str(garden),
        'garden_commit': common.git_commit(garden),
        'lsg_version': version['version'],
        'lsg_commit': common.git_commit(common.REPO),
        'lsg_options': options,
        'skipped_files': skipped,
        'publish_result': json.loads(result.stdout),
    }
    (run / 'settings.json').write_text(json.dumps(settings, indent=2, ensure_ascii=False) + '\n', encoding='utf-8')
    published = settings['publish_result']
    print(f"Published {published['pages']} pages from {len(selected)} roots into {run / 'site'}")
    if skipped:
        print(f'Skipped {len(skipped)} files whose top-level namespace lsg cannot select; see settings.json')


if __name__ == '__main__':
    main()
