#!/usr/bin/env python3
"""Run README examples and check links without depending on a rendered site."""
import argparse
import re
from pathlib import Path
import subprocess
import urllib.parse
import urllib.request

parser = argparse.ArgumentParser()
parser.add_argument('--remote', action='store_true')
args = parser.parse_args()
readme = Path('README.md').read_text()
if not args.remote:
    for block in re.findall(r'```sh\n(.*?)```', readme, re.S):
        subprocess.run(['bash', '-e', '-c', block], check=True, timeout=300)
files = [Path('README.md'), Path('CONTRIBUTING.md'), *Path('docs').glob('*.md'), Path('THIRD_PARTY_NOTICES.md')]
urls = set()
for file in files:
    for link in re.findall(r'\[[^\]]*\]\(([^)]+)\)', file.read_text()):
        if link.startswith(('https://', 'http://')):
            if file.name == 'README.md':
                urls.add(link)
        else:
            target = urllib.parse.unquote(link.split('#')[0])
            if target and not (file.parent / target).exists():
                raise SystemExit(f'{file}: missing link {link}')
if args.remote:
    for url in sorted(urls):
        request = urllib.request.Request(url, headers={'User-Agent': 'lsg-documentation-check'})
        with urllib.request.urlopen(request, timeout=30) as response:
            print(f'{response.status} {url}')
print('README links checked' if args.remote else 'README commands and local documentation links passed')
