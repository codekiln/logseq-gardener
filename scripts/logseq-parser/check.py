#!/usr/bin/env python3
"""Load a graph with Logseq's own file parser and with the SDK, and report where they disagree.

Both sides read the graph folder with its logseq/config.edn. The check compares the files
each side loads as pages, their titles, aliases, namespaces, journal dates and page
property names, the alias names and pages without a file that the graph names, the page
references and properties of each block, and each file's block identifiers and block
references. It prints every difference with its file and item, and exits 1 when it finds
one.

Logseq's side runs the graph parser of Logseq's version/file branch at the revision in
source.json, with the nbb-logseq and mldoc versions in package-lock.json, through
graph.cljs. These must be the revision and versions that `mise run parser:og-check` uses,
as the SDK relationship fixtures record them in
crates/logseq-gardener-sdk/tests/fixtures/og-relationships/expected.json; the check stops
when they differ. The parser source comes from the cache, otherwise from a local ghq
checkout of logseq/logseq that holds the revision, otherwise from GitHub, and every copy
must match the digest in source.json: the SHA-256 of a `shasum -a 256` listing of the
source files, sorted by path. To pin another revision, change `revision`, run the check,
review the upstream changes, and copy the digest the check reports into `sha256`.

The SDK's side is the ignored test crates/logseq-gardener-sdk/tests/logseq_parser.rs. It
derives page relationships with the code that tests/og_relationships.rs compares with
the recorded Logseq evidence, and Logseq's pages are projected the same way, with
scripts/og_relationship_fixtures.py.
"""
import argparse
import hashlib
import io
import json
import os
import re
import shutil
import subprocess
import sys
import tarfile
import tempfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent.parent
sys.path.insert(0, str(ROOT / 'scripts'))
from og_relationship_fixtures import configuration  # noqa: E402

SOURCE = json.loads((HERE / 'source.json').read_text())
FIXTURES = ROOT / 'crates/logseq-gardener-sdk/tests/fixtures/og-relationships/expected.json'


def run(command, **kwargs):
    """Run a command quietly; on failure print its output and stop."""
    result = subprocess.run([str(part) for part in command], capture_output=True, text=True, **kwargs)
    if result.returncode != 0:
        sys.stderr.write(result.stdout + result.stderr)
        sys.exit(f'{Path(str(command[0])).name} exited with status {result.returncode}')
    return result.stdout


def digest(tree):
    """SHA-256 of a `shasum -a 256` listing of every file under `tree`, sorted by path."""
    files = sorted(path.relative_to(tree).as_posix() for path in tree.rglob('*') if path.is_file())
    listing = ''.join(f'{hashlib.sha256((tree / file).read_bytes()).hexdigest()}  {file}\n' for file in files)
    return hashlib.sha256(listing.encode()).hexdigest()


def local_checkout():
    """A ghq checkout of logseq/logseq that holds the pinned revision, if there is one."""
    if not shutil.which('ghq'):
        return None
    found = subprocess.run(['ghq', 'list', '--full-path', '--exact', SOURCE['ghq']],
                           capture_output=True, text=True).stdout.split('\n')[0].strip()
    if not found:
        return None
    has = subprocess.run(['git', '-C', found, 'cat-file', '-e', SOURCE['revision'] + '^{commit}'],
                         capture_output=True)
    return Path(found) if has.returncode == 0 else None


def fetch(work):
    """Fetch the pinned commit from GitHub with only the parser folders' file contents."""
    repo = work / 'repo'
    run(['git', 'init', '-q', repo])
    run(['git', '-C', repo, 'remote', 'add', 'origin', SOURCE['repository']])
    run(['git', '-C', repo, 'fetch', '-q', '--depth', '1', '--filter=blob:none', 'origin', SOURCE['revision']])
    run(['git', '-C', repo, 'sparse-checkout', 'set', '--no-cone', *(f'/{path}/' for path in SOURCE['paths'])])
    run(['git', '-C', repo, 'checkout', '-q', '--detach', SOURCE['revision']])
    return repo


def parser_source():
    """The pinned parser source, from the cache or copied into it after its digest matches."""
    cache = Path(os.environ.get('XDG_CACHE_HOME') or Path.home() / '.cache') / 'logseq-gardener' / 'logseq-source'
    cached = cache / SOURCE['revision']
    if cached.is_dir() and digest(cached) == SOURCE['sha256']:
        return cached
    cache.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(dir=cache) as work:
        work = Path(work)
        repo = local_checkout()
        origin = f'the checkout {repo}' if repo else SOURCE['repository']
        if repo is None:
            print(f"Fetching Logseq {SOURCE['revision'][:12]} from {origin}", file=sys.stderr)
            repo = fetch(work)
        archive = subprocess.run(['git', '-C', repo, 'archive', SOURCE['revision'], *SOURCE['paths']],
                                 capture_output=True, check=True).stdout
        with tarfile.open(fileobj=io.BytesIO(archive)) as tar:
            tar.extractall(work / 'source', filter='data')
        found = digest(work / 'source')
        if found != SOURCE['sha256']:
            sys.exit(f"Logseq {SOURCE['revision']} from {origin} has digest {found}, "
                     f"but scripts/logseq-parser/source.json pins {SOURCE['sha256']}")
        if cached.is_dir() and digest(cached) == SOURCE['sha256']:
            return cached  # a parallel run filled the cache first
        shutil.rmtree(cached, ignore_errors=True)
        (work / 'source').rename(cached)
    return cached


def filename_format(graph):
    """`triple-lowbar` when logseq/config.edn sets `:file/name-format :triple-lowbar`, else `legacy`."""
    config = graph / 'logseq' / 'config.edn'
    text = config.read_text(encoding='utf-8') if config.is_file() else ''
    text = re.sub(r'"(?:[^"\\]|\\.)*"|;[^\n]*', lambda m: m.group(0) if m.group(0).startswith('"') else '', text)
    return 'triple-lowbar' if re.search(r':file/name-format\s+:triple-lowbar\b', text) else 'legacy'


def check_versions():
    """Stop unless the pin and the npm packages match what the SDK relationship fixtures record."""
    recorded = json.loads(FIXTURES.read_text())
    installed = {name: json.loads((HERE / 'node_modules' / package / 'package.json').read_text())['version']
                 for name, package in (('mldoc', 'mldoc'), ('nbb', '@logseq/nbb-logseq'))}
    if SOURCE['revision'] != recorded['source']['revision'] or installed != recorded['runtime']:
        sys.exit(f"scripts/logseq-parser pins Logseq {SOURCE['revision']} with {installed}, but "
                 f"{FIXTURES.relative_to(ROOT)} records {recorded['source']['revision']} with "
                 f"{recorded['runtime']}; pin what mise run parser:og-check uses")


def read_logseq(graph, work):
    """What Logseq's parser builds from the graph: its pages and its blocks in outline order."""
    run(['npm', 'ci', '--ignore-scripts', '--no-audit', '--no-fund', '--silent'], cwd=HERE)
    check_versions()
    source = parser_source()
    output = work / 'logseq.json'
    classpath = ':'.join(str(source / path / 'src') for path in SOURCE['paths'])
    run([HERE / 'node_modules/.bin/nbb-logseq', '--classpath', classpath, HERE / 'graph.cljs', graph, output],
        cwd=HERE)
    return json.loads(output.read_text())


def read_sdk(graph, format, work):
    """What the SDK derives from the graph."""
    output = work / 'sdk.json'
    env = {**os.environ, 'LSG_PARSER_CHECK_GRAPH': str(graph), 'LSG_PARSER_CHECK_FILENAME_FORMAT': format,
           'LSG_PARSER_CHECK_OUTPUT': str(output)}
    run(['cargo', 'test', '--locked', '--workspace', '--all-features', '--test', 'logseq_parser', '--',
         '--ignored', '--exact', 'write_graph_relationships'], cwd=ROOT, env=env)
    return json.loads(output.read_text())


def project_logseq(raw, format):
    """Logseq's results in the SDK's shape.

    Pages, alias names and pages without a file go through the projection of the SDK
    relationship fixtures. Logseq records each property name, apart from its built-in ones,
    as a page the block references; those names are compared as properties instead.
    Logseq keeps `collapsed:: true` as the block's collapsed state rather than a property.
    """
    projected = configuration('triple-lowbar' if format == 'triple-lowbar' else 'default', raw)
    page_properties = {page['file']: page['properties'] for page in raw['pages'] if page['file']}
    documents = {}
    for block in raw['blocks']:
        document = documents.setdefault(block['file'], {
            'file': block['file'], 'properties': page_properties[block['file']],
            'block_ids': set(), 'block_refs': set(), 'blocks': []})
        names = set(block['properties'])
        properties = {key.lower(): str(value).strip()
                      for key, value in (block['properties-text-values'] or {}).items()}
        if block['collapsed?']:
            properties['collapsed'] = 'true'
        document['blocks'].append({
            'page_refs': sorted(set(block['refs']) - set(block['block-refs']) - names),
            'properties': properties})
        if 'id' in names:
            document['block_ids'].add(block['uuid'])
        document['block_refs'].update(block['block-refs'])
    for document in documents.values():
        document['block_ids'] = sorted(document['block_ids'])
        document['block_refs'] = sorted(document['block_refs'])
    for page in projected['pages']:
        documents.setdefault(page['file'], {'file': page['file'], 'properties': page_properties[page['file']],
                                            'block_ids': [], 'block_refs': [], 'blocks': []})
    projected['documents'] = sorted(documents.values(), key=lambda document: document['file'])
    return projected


def side(in_logseq):
    return 'Logseq reads it, the SDK does not' if in_logseq else 'the SDK reads it, Logseq does not'


def files_text(files):
    files = sorted(files)
    return ', '.join(files[:3]) + (f' and {len(files) - 3} more' if len(files) > 3 else '')


def compare(logseq, sdk):
    """Every difference between the two results, each naming its file and item."""
    problems = []

    def sets(where, item, in_logseq, in_sdk):
        for value in sorted(set(in_logseq) ^ set(in_sdk)):
            problems.append(f'{where}: {item} "{value}": {side(value in in_logseq)}')

    def values(where, item, in_logseq, in_sdk):
        if in_logseq != in_sdk:
            problems.append(f'{where}: {item}: Logseq reads {json.dumps(in_logseq)}, '
                            f'the SDK reads {json.dumps(in_sdk)}')

    logseq_pages = {page['file']: page for page in logseq['pages']}
    sdk_pages = {page['file']: page for page in sdk['pages']}
    for file in sorted(logseq_pages.keys() | sdk_pages.keys()):
        logseq_page, sdk_page = logseq_pages.get(file), sdk_pages.get(file)
        if logseq_page is None or sdk_page is None:
            problems.append(f'{file}: page "{(logseq_page or sdk_page)["title"]}": {side(sdk_page is None)}')
            continue
        values(file, 'title', logseq_page['title'], sdk_page['title'])
        values(file, 'page name', logseq_page['name'], sdk_page['name'])
        sets(file, 'alias', logseq_page['aliases'], sdk_page['aliases'])
        values(file, 'namespace', logseq_page['namespace'], sdk_page['namespace'])
        values(file, 'journal date', logseq_page['journal_day'], sdk_page['journal_day'])

    # Alias names and pages without a file are reported at the files that declare or name them.
    naming = {}
    for result in (logseq, sdk):
        for document in result['documents']:
            for block in document['blocks']:
                for name in block['page_refs']:
                    naming.setdefault(name, set()).add(document['file'])
    for kind, item in (('aliases', 'alias name'), ('fileless', 'page without a file')):
        logseq_names = {page['name']: page for page in logseq[kind]}
        sdk_names = {page['name']: page for page in sdk[kind]}
        for name in sorted(logseq_names.keys() | sdk_names.keys()):
            logseq_page, sdk_page = logseq_names.get(name), sdk_names.get(name)
            where = files_text((logseq_page or sdk_page).get('files') or naming.get(name, ())) or 'graph'
            if logseq_page is None or sdk_page is None:
                problems.append(f'{where}: {item} "{name}": {side(sdk_page is None)}')
                continue
            for field in sorted(logseq_page.keys() - {'name'}):
                values(where, f'{item} "{name}" {field}', logseq_page[field], sdk_page[field])

    logseq_documents = {document['file']: document for document in logseq['documents']}
    sdk_documents = {document['file']: document for document in sdk['documents']}
    for file in sorted(logseq_documents.keys() & sdk_documents.keys()):
        logseq_document, sdk_document = logseq_documents[file], sdk_documents[file]
        sets(file, 'page property', logseq_document['properties'], sdk_document['properties'])
        sets(file, 'block id', logseq_document['block_ids'], sdk_document['block_ids'])
        sets(file, 'block reference', logseq_document['block_refs'], sdk_document['block_refs'])
        if len(logseq_document['blocks']) != len(sdk_document['blocks']):
            problems.append(f'{file}: blocks: Logseq reads {len(logseq_document["blocks"])}, '
                            f'the SDK reads {len(sdk_document["blocks"])}')
            continue
        # Both list a file's blocks in outline order; the SDK knows each block's line.
        for logseq_block, sdk_block in zip(logseq_document['blocks'], sdk_document['blocks']):
            where = f'{file}:{sdk_block["line"]}'
            sets(where, 'page reference', logseq_block['page_refs'], sdk_block['page_refs'])
            logseq_properties, sdk_properties = logseq_block['properties'], sdk_block['properties']
            for key in sorted(logseq_properties.keys() | sdk_properties.keys()):
                if key in logseq_properties and key in sdk_properties:
                    values(where, f'property "{key}"', logseq_properties[key], sdk_properties[key])
                else:
                    problems.append(f'{where}: property "{key}": {side(key in logseq_properties)}')
    return problems


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument('graph', type=Path, help='graph folder, holding pages/ and logseq/config.edn')
    args = parser.parse_args()
    graph = args.graph.resolve()
    format = filename_format(graph)
    with tempfile.TemporaryDirectory() as work:
        logseq = project_logseq(read_logseq(graph, Path(work)), format)
        sdk = read_sdk(graph, format, Path(work))
    problems = compare(logseq, sdk)
    name = os.path.relpath(graph)
    if problems:
        print(f'Logseq and the SDK disagree about {len(problems)} items in {name}:')
        print('\n'.join(problems))
        return 1
    blocks = sum(len(document['blocks']) for document in sdk['documents'])
    print(f'Logseq and the SDK agree on {name}: {len(sdk["pages"])} pages and {blocks} blocks')
    return 0


if __name__ == '__main__':
    sys.exit(main())
