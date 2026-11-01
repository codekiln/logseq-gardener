#!/usr/bin/env python3
# /// script
# requires-python = ">=3.12"
# dependencies = ["playwright==1.60.0"]
# ///
"""Build Logseq 0.10.15's own export of a garden and the lsg site of it, and compare every page.

Logseq's side runs `logseq.tasks.dev.publishing` from Logseq's repository with nbb-logseq.
That script calls `logseq.publishing/export`, the function `logseq/publish-spa` wraps, and
writes `index.html`, `404.html`, `static/` and `assets/` into a folder. Its code comes from
the tag in logseq-export/source.json: from the cache, otherwise from a local ghq checkout
of logseq/logseq that holds the tag, otherwise from GitHub, and every copy must match the
pinned digest, the SHA-256 of a `shasum -a 256` listing of the files sorted by path. The
built publishing frontend, the `app` folder of a Logseq release, comes from --static-dir,
otherwise from Logseq.app, otherwise from the official release archive, whose SHA-256 is
pinned, and its files must match the pinned frontend digest.

The lsg side publishes the same garden with the checkout's lsg: the docs graph with the
options `docs:site` uses, another garden with the options given after `--`.

One local server serves both builds. For each page and journal in lsg-site.json, headless
Chromium reads the rendered text of the page's blocks and the rows of its properties box
in both builds, and the report names the words and rows only one build shows. The report
goes to target/parity/<UTC time>/logseq-export.md and .json, and the task exits 1 when a
page differs.
"""
import argparse
import collections
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
import time
import unicodedata
import zipfile
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import common  # noqa: E402
import logseq_db  # noqa: E402
import serve  # noqa: E402

PINS = HERE / 'logseq-export'
SOURCE = json.loads((PINS / 'source.json').read_text(encoding='utf-8'))
FRONTEND = SOURCE['frontend']
DOCS = common.REPO / 'docs'
# The options .mise/tasks/docs/site passes for the docs graph.
DOCS_OPTIONS = ['--filename-format', 'triple-lowbar', '--include', 'lsg', '--include', 'Diataxis',
                '--homepage', 'lsg', '--search', '--readable-urls']
COMPARED_KINDS = ('page', 'journal')
PAGE_TIMEOUT_MS = 30_000
SCROLL_PAUSE_MS = 150
STABLE_READS = 3
SCROLL_LIMIT = 400


def cache_root():
    return Path(os.environ.get('XDG_CACHE_HOME') or Path.home() / '.cache') / 'logseq-gardener'


def run(command, **kwargs):
    """Run a command quietly; on failure print its output and stop."""
    result = subprocess.run([str(part) for part in command], capture_output=True, text=True, **kwargs)
    if result.returncode != 0:
        sys.stderr.write(result.stdout + result.stderr)
        sys.exit(f'{Path(str(command[0])).name} exited with status {result.returncode}')
    return result.stdout


def tree_digest(root, names=None):
    """SHA-256 of a `shasum -a 256` listing of the files under `root`, or under its entries
    `names`, sorted by path."""
    root = Path(root)
    tops = [root / name for name in names] if names else [root]
    files = sorted({path.relative_to(root).as_posix() for top in tops
                    for path in ([top] if top.is_file() else top.rglob('*')) if path.is_file()})
    listing = ''.join(f'{hashlib.sha256((root / file).read_bytes()).hexdigest()}  {file}\n' for file in files)
    return hashlib.sha256(listing.encode()).hexdigest()


# Logseq's export code


def local_checkout():
    """A ghq checkout of logseq/logseq that holds the pinned commit, if there is one."""
    if not shutil.which('ghq'):
        return None
    found = subprocess.run(['ghq', 'list', '--full-path', '--exact', SOURCE['ghq']],
                           capture_output=True, text=True).stdout.split('\n')[0].strip()
    if not found:
        return None
    has = subprocess.run(['git', '-C', found, 'cat-file', '-e', SOURCE['revision'] + '^{commit}'],
                         capture_output=True)
    return Path(found) if has.returncode == 0 else None


def sparse_pattern(path):
    """A sparse-checkout pattern for a pinned folder or file."""
    return f'/{path}' if Path(path).suffix else f'/{path}/'


def fetch(work):
    """Fetch the pinned commit from GitHub with only the pinned paths' file contents."""
    repo = work / 'repo'
    run(['git', 'init', '-q', repo])
    run(['git', '-C', repo, 'remote', 'add', 'origin', SOURCE['repository']])
    run(['git', '-C', repo, 'fetch', '-q', '--depth', '1', '--filter=blob:none', 'origin', SOURCE['revision']])
    run(['git', '-C', repo, 'sparse-checkout', 'set', '--no-cone', *map(sparse_pattern, SOURCE['paths'])])
    run(['git', '-C', repo, 'checkout', '-q', '--detach', SOURCE['revision']])
    return repo


def logseq_source():
    """The pinned export code, from the cache or copied into it after its digest matches."""
    cache = cache_root() / 'logseq-export-source'
    cached = cache / SOURCE['revision']
    if cached.is_dir() and tree_digest(cached) == SOURCE['sha256']:
        return cached
    cache.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(dir=cache) as work:
        work = Path(work)
        repo = local_checkout()
        origin = f'the checkout {repo}' if repo else SOURCE['repository']
        if repo is None:
            print(f"Fetching Logseq {SOURCE['tag']} from {origin}", file=sys.stderr)
            repo = fetch(work)
        archive = subprocess.run(['git', '-C', repo, 'archive', SOURCE['revision'], *SOURCE['paths']],
                                 capture_output=True, check=True).stdout
        with tarfile.open(fileobj=io.BytesIO(archive)) as tar:
            tar.extractall(work / 'source', filter='data')
        found = tree_digest(work / 'source')
        if found != SOURCE['sha256']:
            sys.exit(f"Logseq {SOURCE['tag']} from {origin} has digest {found}, "
                     f"but scripts/parity/logseq-export/source.json pins {SOURCE['sha256']}")
        if cached.is_dir() and tree_digest(cached) == SOURCE['sha256']:
            return cached  # a parallel run filled the cache first
        shutil.rmtree(cached, ignore_errors=True)
        (work / 'source').rename(cached)
    return cached


# The built publishing frontend


def frontend_problem(folder):
    """Why `folder` is not the pinned frontend, or None when it is."""
    folder = Path(folder)
    version_file = folder / 'VERSION'
    version = version_file.read_text(encoding='utf-8').strip() if version_file.is_file() else 'missing'
    if version != FRONTEND['version']:
        return f"its VERSION is {version}; the comparison needs {FRONTEND['version']}"
    found = tree_digest(folder, FRONTEND['files'])
    if found != FRONTEND['sha256']:
        return (f"its files have digest {found}, but scripts/parity/logseq-export/source.json "
                f"pins {FRONTEND['sha256']}")
    return None


def download_frontend(cached):
    """Download the pinned release archive, check it, and unpack the frontend into `cached`."""
    release = FRONTEND['release']
    cached.parent.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(dir=cached.parent) as work:
        work = Path(work)
        archive = work / Path(release['url']).name
        print(f"Downloading {release['url']}", file=sys.stderr)
        subprocess.run(['curl', '--fail', '--location', '--retry', '3', '--silent', '--show-error',
                        '--output', str(archive), release['url']], check=True)
        digest = hashlib.sha256()
        with open(archive, 'rb') as data:
            for chunk in iter(lambda: data.read(1 << 20), b''):
                digest.update(chunk)
        if digest.hexdigest() != release['sha256']:
            sys.exit(f"{release['url']} has SHA-256 {digest.hexdigest()}, "
                     f"but scripts/parity/logseq-export/source.json pins {release['sha256']}")
        prefix = release['folder'].rstrip('/') + '/'
        frontend = work / 'frontend'
        with zipfile.ZipFile(archive) as bundle:
            for entry in bundle.infolist():
                relative = entry.filename[len(prefix):] if entry.filename.startswith(prefix) else ''
                parts = relative.split('/')
                if entry.is_dir() or parts[0] not in FRONTEND['files'] or '..' in parts:
                    continue
                target = frontend.joinpath(*parts)
                target.parent.mkdir(parents=True, exist_ok=True)
                with bundle.open(entry) as source, open(target, 'wb') as out:
                    shutil.copyfileobj(source, out)
        archive.unlink()
        problem = frontend_problem(frontend)
        if problem:
            sys.exit(f"The frontend in {release['url']} is unusable: {problem}")
        shutil.rmtree(cached, ignore_errors=True)
        frontend.rename(cached)
    return cached


def frontend_folder(static_dir=None, macos_app=None):
    """The pinned frontend: --static-dir, Logseq.app, the cache, or the release archive."""
    if static_dir:
        problem = frontend_problem(static_dir)
        if problem:
            sys.exit(f'--static-dir {static_dir}: {problem}')
        return Path(static_dir)
    app = Path(macos_app or FRONTEND['macos_app'])
    if app.is_dir():
        problem = frontend_problem(app)
        if problem is None:
            return app
        print(f"{app}: {problem}; using the {FRONTEND['version']} release instead", file=sys.stderr)
    cached = cache_root() / 'logseq-frontend' / FRONTEND['version']
    if cached.is_dir() and frontend_problem(cached) is None:
        return cached
    return download_frontend(cached)


# The two builds

FAILED_COPY = re.compile(r'^Failed to copy \{:from "((?:[^"\\]|\\.)*)"', re.M)


def failed_copies(log, garden):
    """The files Logseq's export logged it failed to copy, relative to the garden."""
    files = set()
    for match in FAILED_COPY.finditer(log):
        path = Path(match.group(1))
        files.add(path.relative_to(garden).as_posix() if path.is_relative_to(garden) else str(path))
    return sorted(files)


def build_logseq(source, static_dir, garden, output, log_path):
    """Run Logseq's export of the garden into `output`; return the files it failed to copy."""
    run(['npm', 'ci', '--ignore-scripts', '--no-audit', '--no-fund', '--silent'], cwd=PINS)
    classpath = ':'.join(str(source / path) for path in SOURCE['classpath'])
    # The script resolves relative paths against `..`, so every path is absolute.
    result = subprocess.run([str(PINS / 'node_modules/.bin/nbb-logseq'), '-cp', classpath,
                             '-m', 'logseq.tasks.dev.publishing', str(static_dir), str(garden), str(output)],
                            cwd=PINS, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    log_path.write_text(result.stdout, encoding='utf-8')
    if result.returncode != 0 or not (output / 'index.html').is_file():
        sys.exit(f"Logseq's export exited with status {result.returncode}; its log is {log_path}")
    return failed_copies(result.stdout, garden)


def build_lsg(garden, options, output, diagnostics_path):
    """Publish the garden with the checkout's lsg; return its JSON report and version."""
    subprocess.run(['cargo', 'build', '--locked', '--release', '--bin', 'lsg'], cwd=common.REPO, check=True)
    lsg = common.lsg_binary()
    with open(diagnostics_path, 'w', encoding='utf-8') as diagnostics:
        result = subprocess.run([str(lsg), 'publish', '--graph', str(garden), '--output', str(output), *options,
                                 '--format', 'json'], stdout=subprocess.PIPE, stderr=diagnostics, text=True)
    if result.returncode != 0:
        sys.exit(f'lsg publish exited with status {result.returncode}; its diagnostics are {diagnostics_path}')
    version = json.loads(run([lsg, 'version', '--format', 'json']))['version']
    return json.loads(result.stdout), version


def garden_status(garden):
    """`git status` of the garden folder, or None when the garden is not in a Git checkout."""
    result = subprocess.run(['git', '-C', str(garden), 'status', '--porcelain', '--ignored', '--', '.'],
                            capture_output=True, text=True)
    return result.stdout if result.returncode == 0 else None


def lsg_options(garden, options):
    """The lsg publish options: the docs:site options for the docs graph, else those given."""
    if options:
        return options
    if garden.resolve() == DOCS.resolve():
        return DOCS_OPTIONS
    sys.exit(f'{garden} is not the docs graph; pass the lsg publish options that select its pages '
             'after --, such as `-- --include notes --journals`')


# Which pages to compare


def logseq_pages(index_html):
    """Page records from the graph data inline in the export's index.html."""
    start = index_html.index(logseq_db.PREFIX)
    end = index_html.index('</script>', start)
    return logseq_db.pages_from_db(logseq_db.decode_assignment(index_html[start:end].rstrip().rstrip(';')))


def page_plan(manifest, logseq):
    """Split the pages into those to compare, those missing from Logseq's export, and Logseq
    pages with blocks that the lsg build does not publish."""
    names = {common.normalize(page.name) for page in logseq}
    entries = [entry for entry in manifest['pages'] if entry['kind'] in COMPARED_KINDS]
    published = {common.normalize(entry['title']) for entry in manifest['pages']}
    compared = [entry for entry in entries if common.normalize(entry['title']) in names]
    missing = [entry['title'] for entry in entries if common.normalize(entry['title']) not in names]
    outside = sorted(page.title for page in logseq
                     if page.has_content and common.normalize(page.name) not in published)
    return compared, missing, outside


# Reading a page in Chromium

WAIT_FOR_LOGSEQ_PAGE = '''(name) => document.title.trim().normalize('NFC').toLowerCase() === name
    && !!document.querySelector('.page-blocks-inner .ls-block')'''
LOGSEQ_TEXT_LENGTH = '''() => document.querySelector('.page-blocks-inner')?.innerText.length ?? -1'''
# Scroll Logseq's main column and the window down one screen; true when both are at the bottom.
SCROLL_ONE_SCREEN = '''() => {
  let atBottom = true;
  for (const scroller of [document.querySelector('#main-content-container'), document.scrollingElement]) {
    if (!scroller) continue;
    scroller.scrollBy(0, Math.max(100, scroller.clientHeight * 0.8));
    if (scroller.scrollTop + scroller.clientHeight < scroller.scrollHeight - 2) atBottom = false;
  }
  return atBottom;
}'''
# The properties box rows, then the blocks' rendered text without the properties block,
# the reference-count badges (the block-refs-count component in frontend/components/block.cljs)
# and CodeMirror's line numbers.
READ_LOGSEQ = '''() => {
  const root = document.querySelector('.page-blocks-inner');
  const rows = [...(root.querySelector('.page-properties')?.children ?? [])]
    .filter(row => row.querySelector('.page-property-key'))
    .map(row => [row.querySelector('.page-property-key').innerText,
                 row.querySelector('.page-property-value')?.innerText ?? '']);
  root.querySelectorAll('.pre-block, .open-block-ref-link, .CodeMirror-gutters, .CodeMirror-gutter-wrapper')
    .forEach(node => node.remove());
  return {rows, text: root.innerText};
}'''
READ_LSG = '''() => {
  const main = document.querySelector('main');
  const rows = [...main.querySelectorAll('.page-properties .page-property')]
    .map(row => [row.querySelector('dt')?.innerText ?? '', row.querySelector('dd')?.innerText ?? '']);
  return {rows, text: main.querySelector(':scope > ul.outline')?.innerText ?? ''};
}'''


def scroll_through(step, length, pause, stable_reads=STABLE_READS, limit=SCROLL_LIMIT):
    """Scroll one screen at a time until the page is at the bottom and its text length has stayed
    the same for `stable_reads` reads; return False when `limit` steps were not enough."""
    previous, unchanged = None, 0
    for _ in range(limit):
        at_bottom = step()
        pause()
        current = length()
        unchanged = unchanged + 1 if current == previous else 0
        previous = current
        if at_bottom and unchanged >= stable_reads:
            return True
    return False


def read_logseq(page, url, title):
    """Load a Logseq page from a blank page, render all of it, and read its rows and text."""
    page.goto('about:blank')
    page.goto(url)
    page.wait_for_function(WAIT_FOR_LOGSEQ_PAGE, arg=common.normalize(title).strip(), timeout=PAGE_TIMEOUT_MS)
    settled = scroll_through(lambda: page.evaluate(SCROLL_ONE_SCREEN), lambda: page.evaluate(LOGSEQ_TEXT_LENGTH),
                             lambda: page.wait_for_timeout(SCROLL_PAUSE_MS))
    return {**page.evaluate(READ_LOGSEQ), 'settled': settled}


def read_lsg(page, url):
    page.goto(url)
    page.wait_for_selector('main h1', timeout=PAGE_TIMEOUT_MS)
    return page.evaluate(READ_LSG)


# Comparing what the two builds show

WORD = re.compile(r'\w+')


def words(text):
    """Count the words of a text, ignoring letter case and punctuation."""
    return collections.Counter(WORD.findall(unicodedata.normalize('NFC', text).lower()))


def only_in(these, those):
    """The words `these` holds more often than `those`, with the difference when it is above one."""
    return sorted(word if count - those[word] == 1 else f'{word}×{count - those[word]}'
                  for word, count in these.items() if count > those[word])


def property_rows(rows):
    """Rows of a properties box as `key: value`, without page reference brackets."""
    found = set()
    for key, value in rows:
        value = ' '.join(value.replace('[[', ' ').replace(']]', ' ').split()).replace(' ,', ',')
        found.add(f"{key.strip().lower()}: {value}")
    return found


def compare_page(title, logseq, lsg):
    """What differs between Logseq's and lsg's reading of one page."""
    logseq_words, lsg_words = words(logseq['text']), words(lsg['text'])
    logseq_rows, lsg_rows = property_rows(logseq['rows']), property_rows(lsg['rows'])
    result = {
        'title': title,
        'logseq_words': sum(logseq_words.values()),
        'lsg_words': sum(lsg_words.values()),
        'words_only_in_logseq': only_in(logseq_words, lsg_words),
        'words_only_in_lsg': only_in(lsg_words, logseq_words),
        'rows_only_in_logseq': sorted(logseq_rows - lsg_rows),
        'rows_only_in_lsg': sorted(lsg_rows - logseq_rows),
    }
    if not logseq.get('settled', True):
        result['note'] = f"Logseq's page was still growing after {SCROLL_LIMIT} scrolls"
    result['differs'] = any(result[key] for key in ('words_only_in_logseq', 'words_only_in_lsg',
                                                    'rows_only_in_logseq', 'rows_only_in_lsg'))
    return result


def code_list(items):
    return ', '.join(f'`{item}`' for item in items)


def markdown(report):
    summary, pages = report['summary'], report['pages']
    lines = ['# Logseq export comparison', '']
    lines += [f'- {key}: {value}' for key, value in summary.items()]
    lines += ['', '## Pages that differ', '']
    differing = [page for page in pages if page['differs'] or page.get('error')]
    for page in differing:
        lines.append(f"### {page['title']}")
        lines.append('')
        if page.get('error'):
            lines.append(f"- Not compared: {page['error']}")
        for key, label in (('words_only_in_logseq', 'Words only Logseq shows'),
                           ('words_only_in_lsg', 'Words only lsg shows'),
                           ('rows_only_in_logseq', 'Property rows only Logseq shows'),
                           ('rows_only_in_lsg', 'Property rows only lsg shows')):
            if page.get(key):
                lines.append(f'- {label}: {code_list(page[key])}')
        if page.get('note'):
            lines.append(f"- {page['note']}")
        lines.append('')
    if not differing:
        lines += ['None.', '']
    lines += ["## Pages missing from Logseq's export", '']
    lines += [f'- {title}' for title in report['missing_from_logseq']] or ['None.']
    lines += ['', '## Logseq pages with blocks that the lsg build does not publish', '']
    lines += [f'- {title}' for title in report['outside_lsg']] or ['None.']
    lines += ['', "## Files Logseq's export failed to copy", '']
    lines += [f'- `{path}`' for path in report['failed_copies']] or ['None.']
    return '\n'.join(lines) + '\n'


def compare_builds(run_dir, compared):
    """Read every compared page in both builds from one local server."""
    from playwright.sync_api import sync_playwright

    results = []
    with serve.Server(run_dir) as server, sync_playwright() as playwright:
        browser = playwright.chromium.launch()
        context = browser.new_context(viewport={'width': 1280, 'height': 800}, color_scheme='light')
        logseq_page, lsg_page = context.new_page(), context.new_page()
        for number, entry in enumerate(compared, 1):
            title = entry['title']
            try:
                logseq = read_logseq(logseq_page, server.url + 'logseq/' + common.logseq_route(title), title)
                lsg = read_lsg(lsg_page, server.url + 'lsg/' + entry['url'])
                result = compare_page(title, logseq, lsg)
            except Exception as error:  # Playwright raises its own TimeoutError type.
                result = {'title': title, 'differs': True, 'error': f'{type(error).__name__}: {error}'.split('\n')[0]}
            results.append(result)
            print(f"[{number}/{len(compared)}] {title}: {'differs' if result['differs'] else 'same'}", flush=True)
        browser.close()
    return results


def main():
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument('--garden', type=Path, default=DOCS, help='garden folder (default: the docs graph)')
    parser.add_argument('--static-dir', type=Path, help=f"a Logseq {FRONTEND['version']} app folder, "
                        'holding js/publishing/ (default: Logseq.app, else the official release)')
    parser.add_argument('lsg_options', nargs='*', help='lsg publish options; required for a garden other than docs')
    args = parser.parse_args()
    garden = args.garden.resolve()
    options = lsg_options(garden, args.lsg_options)
    source = logseq_source()
    static_dir = frontend_folder(args.static_dir)
    status_before = garden_status(garden)
    run_dir = common.new_run_dir()
    started = time.monotonic()

    print(f"Building Logseq {SOURCE['tag']}'s export of {garden}", flush=True)
    copy_failures = build_logseq(source, static_dir, garden, run_dir / 'logseq', run_dir / 'logseq-export.log')
    print('Building the lsg site', flush=True)
    published, version = build_lsg(garden, options, run_dir / 'lsg', run_dir / 'lsg-diagnostics.txt')
    if garden_status(garden) != status_before:
        sys.exit(f'The garden {garden} changed while the task built the sites; see `git status`')

    manifest = json.loads((run_dir / 'lsg' / 'lsg-site.json').read_text(encoding='utf-8'))
    logseq = logseq_pages((run_dir / 'logseq' / 'index.html').read_text(encoding='utf-8'))
    compared, missing, outside = page_plan(manifest, logseq)
    pages = compare_builds(run_dir, compared)

    differing = sum(page['differs'] for page in pages)
    garden_commit = subprocess.run(['git', '-C', str(garden), 'rev-parse', '--short', 'HEAD'],
                                   capture_output=True, text=True).stdout.strip()
    shown = garden.relative_to(common.REPO) if garden.is_relative_to(common.REPO) else garden
    report = {
        'summary': {
            'Garden': f'{shown}' + (f' at {garden_commit}' if garden_commit else ''),
            'Logseq': f"{SOURCE['tag']} ({SOURCE['revision'][:7]}), frontend from {static_dir}",
            'lsg': f"{version} ({common.git_commit(common.REPO)[:7]}), {published['diagnostic_count']} diagnostics",
            'lsg options': ' '.join(options),
            'Pages compared': len(pages),
            'Pages that differ': differing,
            "Pages missing from Logseq's export": len(missing),
            'Minutes': round((time.monotonic() - started) / 60, 1),
        },
        'pages': pages,
        'missing_from_logseq': missing,
        'outside_lsg': outside,
        'failed_copies': copy_failures,
    }
    (run_dir / 'logseq-export.json').write_text(json.dumps(report, indent=2, ensure_ascii=False) + '\n',
                                                encoding='utf-8')
    (run_dir / 'logseq-export.md').write_text(markdown(report), encoding='utf-8')
    print(markdown(report))
    print(f"Report: {run_dir / 'logseq-export.md'}")
    return 1 if differing or missing else 0


if __name__ == '__main__':
    sys.exit(main())
