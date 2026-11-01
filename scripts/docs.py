#!/usr/bin/env python3
"""Check the docs graph and the remaining Markdown files, run README examples, and check README web links."""
import argparse
import os
import re
from pathlib import Path
import subprocess
import urllib.error
import urllib.parse
import urllib.request

DOCS = Path('docs')
CONFIG = DOCS / 'logseq' / 'config.edn'
# The kind index pages, which carry no kind tag of their own.
KIND_PAGES = {'diataxis', 'diataxis/how to', 'diataxis/tutorial', 'diataxis/reference', 'diataxis/explanation'}
KIND_TAGS = KIND_PAGES - {'diataxis'} | {'diataxis/concept'}
PROPERTY = re.compile(r'^([A-Za-z0-9_/-]+)::\s?(.*)$')
# The documentation site, where each page is published at its title, as in SITE + 'lsg/Ref/Command%20Line/'.
SITE = 'https://codekiln.github.io/logseq-gardener/'
HOMEPAGE = 'lsg'
# README commands that install software or change the reader's configuration go in ```bash blocks, which
# docs:check does not run; it runs every ```sh block.
CHANGES_MACHINE = re.compile(r'\b(?:mise\s+(?:use|install|trust)|cargo\s+install|brew\s+install)\b')


def old_documents():
    """Markdown files under docs/ that are not pages yet: outside docs/pages/ and Logseq's own folder."""
    return sorted(path for path in DOCS.rglob('*.md')
                  if path.relative_to(DOCS).parts[0] not in ('pages', 'logseq')
                  and not any(part.startswith('.') for part in path.parts))


def hidden_entries(config_text):
    """The string entries of :hidden, each with the number of strings on its line."""
    text = re.sub(r'"(?:[^"\\]|\\.)*"|;[^\n]*', lambda m: m.group(0) if m.group(0).startswith('"') else '',
                  config_text)
    match = re.search(r':hidden\s*\[(.*?)\]', text, re.S)
    entries = []
    for line in (match.group(1).splitlines() if match else []):
        strings = re.findall(r'"((?:[^"\\]|\\.)*)"', line)
        entries.extend((entry, len(strings)) for entry in strings)
    return entries


def check_hidden(documents):
    """Every remaining document has its own :hidden line, and every :hidden document still exists."""
    errors = []
    entries = hidden_entries(CONFIG.read_text(encoding='utf-8'))
    for entry, on_line in entries:
        if on_line > 1:
            errors.append(f'{CONFIG}: put :hidden entry "{entry}" on a line of its own')
    names = {entry for entry, _ in entries}
    for document in documents:
        entry = '/' + document.relative_to(DOCS).as_posix()
        if entry not in names:
            errors.append(f'{document}: add "{entry}" to :hidden in {CONFIG}, or convert the file into pages')
    for entry in sorted(names):
        if entry.endswith('.md') and not (DOCS / entry.lstrip('/')).exists():
            errors.append(f'{CONFIG}: remove the :hidden entry "{entry}"; docs{entry} does not exist')
    return errors


def page_properties(text):
    """The page properties: the key:: value lines at the top of a page file."""
    properties = {}
    for line in text.splitlines():
        match = PROPERTY.match(line)
        if not match:
            break
        properties[match.group(1).lower()] = match.group(2).strip()
    return properties


def tag_name(tag):
    """The page a tag names, lower-cased: `[[Diataxis/How To]]`, `#tag` and plain names."""
    tag = tag.strip().lstrip('#')
    if tag.startswith('[[') and tag.endswith(']]'):
        tag = tag[2:-2]
    return tag.strip().lower()


def expected_tag(title):
    for prefix, tag in (('lsg/How To/', 'Diataxis/How To'), ('lsg/Tutorial/', 'Diataxis/Tutorial'),
                        ('lsg/Ref/', 'Diataxis/Reference')):
        if title.startswith(prefix):
            return tag
    return 'Diataxis/Explanation' if title == 'lsg' or title.startswith('lsg/') else None


def page_title(page, properties):
    """A page's title: its title:: property, or its triple-lowbar file name."""
    return properties.get('title') or urllib.parse.unquote(page.stem.replace('___', '/'))


def check_kind_tags():
    """Every page other than contents.md and the kind index pages carries the kind tag its title names."""
    errors = []
    for page in sorted((DOCS / 'pages').glob('*.md')):
        if page.name == 'contents.md':
            continue
        properties = page_properties(page.read_text(encoding='utf-8'))
        title = page_title(page, properties)
        if title.lower() in KIND_PAGES:
            continue
        expected = expected_tag(title)
        if expected is None:
            errors.append(f'{page}: {title} needs a title that names its kind: lsg/How To/..., lsg/Tutorial/..., '
                          f'lsg/Ref/... or lsg/...')
            continue
        tags = {tag_name(tag) for tag in properties.get('tags', '').split(',') if tag.strip()}
        if tags & KIND_TAGS != {expected.lower()}:
            errors.append(f'{page}: {title} needs the kind tag [[{expected}]] in tags::, and no other kind tag')
    return errors


def without_code(text):
    """Markdown text with its fenced code blocks and code spans removed, so example syntax is not a link."""
    text = re.sub(r'(?ms)^[ \t]*(`{3,}|~{3,}).*?^[ \t]*\1[ \t]*$', '', text)
    return re.sub(r'(`+).+?\1', '', text)


def check_relative_links(files):
    errors = []
    for file in files:
        for link in re.findall(r'\[[^\]]*\]\(([^)]+)\)', without_code(file.read_text(encoding='utf-8'))):
            if link.startswith(('https://', 'http://')):
                continue
            target = urllib.parse.unquote(link.split('#')[0])
            if target and not (file.parent / target).exists():
                errors.append(f'{file}: missing link {link}')
    return errors


def web_links(file):
    return sorted({link for link in re.findall(r'\[[^\]]*\]\(([^)]+)\)', file.read_text(encoding='utf-8'))
                   if link.startswith(('https://', 'http://'))})


def check_site_links(file):
    """Every link to the documentation site names a page of the docs graph by its title."""
    titles = {page_title(page, page_properties(page.read_text(encoding='utf-8')))
              for page in (DOCS / 'pages').glob('*.md')}
    errors = []
    for link in web_links(file):
        if not link.startswith(SITE):
            continue
        path = urllib.parse.unquote(link[len(SITE):].split('#')[0]).removesuffix('index.html').strip('/')
        if (path or HOMEPAGE) not in titles:
            errors.append(f'{file}: {link} names no page in docs/pages; a page is published at its title, '
                          f'such as {SITE}lsg/Ref/Command%20Line/')
    return errors


def check_web_links(file):
    """Request every web link in the file, including its links to the documentation site."""
    failures = []
    for url in web_links(file):
        request = urllib.request.Request(url, headers={'User-Agent': 'lsg-documentation-check'})
        try:
            with urllib.request.urlopen(request, timeout=30) as response:
                print(f'{response.status} {url}')
        except (urllib.error.URLError, TimeoutError) as error:
            failures.append(f'{file}: {url}: {getattr(error, "code", None) or error}')
    return failures


def run_commands(file):
    """Run every ```sh block in the file with this checkout's release build first on PATH."""
    blocks = re.findall(r'```sh\n(.*?)```', file.read_text(encoding='utf-8'), re.S)
    for block in blocks:
        if CHANGES_MACHINE.search(block):
            raise SystemExit(f'{file}: a ```sh block installs software or changes configuration, and docs:check '
                             f'runs every ```sh block; put that command in a ```bash block:\n{block}')
    release = Path(os.environ.get('CARGO_TARGET_DIR', 'target')).resolve() / 'release'
    env = {**os.environ, 'PATH': f'{release}{os.pathsep}{os.environ.get("PATH", "")}'}
    for block in blocks:
        subprocess.run(['bash', '-e', '-c', block], check=True, timeout=300, env=env)


parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--remote', action='store_true', help='check README web links (requires network)')
args = parser.parse_args()
readme = Path('README.md')
if args.remote:
    failures = check_web_links(readme)
    if failures:
        raise SystemExit('\n'.join(failures))
    print('README web links, including its links to the documentation site, checked')
    raise SystemExit
documents = old_documents()
errors = [*check_relative_links([readme, Path('CONTRIBUTING.md'), Path('THIRD_PARTY_NOTICES.md'), *documents]),
          *check_site_links(readme), *check_hidden(documents), *check_kind_tags()]
if errors:
    raise SystemExit('\n'.join(errors))
run_commands(readme)
print('README commands and site links, docs graph tags, :hidden entries and local documentation links passed')
