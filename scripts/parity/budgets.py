#!/usr/bin/env python3
"""Check lsg page sizes against the budgets, and that they stay the same as the garden grows."""
import argparse
import html.parser
import shutil
import subprocess
import sys
import tempfile
from datetime import date, timedelta
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import common  # noqa: E402
import metrics  # noqa: E402

FIXTURE_GARDEN = common.REPO / 'tests' / 'fixtures' / 'parity-garden'
FIXTURE_ROOTS = ['Tools']
FILLER_ROOT = 'Filler'
FILLER_PAGES = 2000
OLDER_JOURNALS = 1000
JOURNALS_SCRIPT = 'lsg-journals.js'
OUTLINE_SCRIPT = 'lsg-outline.js'
OUTLINE_STYLES = common.REPO / 'crates' / 'logseq-gardener-sdk' / 'src' / 'site' / 'outline.css'
INLINE_STYLES = common.REPO / 'crates' / 'logseq-gardener-sdk' / 'src' / 'site' / 'inline.css'
CODE_STYLES = common.REPO / 'crates' / 'logseq-gardener-sdk' / 'src' / 'site' / 'code.css'
CODE_MARKER = 'cp__fenced-code-block'
TRANSCLUSION_STYLES = common.REPO / 'crates' / 'logseq-gardener-sdk' / 'src' / 'site' / 'transclusion.css'
EMBED_MARKERS = ('class="embed-page"', 'class="embed-block"')
REFERENCES_SCRIPT = 'lsg-references.js'
REFERENCES_STYLES = common.REPO / 'crates' / 'logseq-gardener-sdk' / 'src' / 'site' / 'references.css'
REFERENCES_DIR = '_refs'



class _BlockingResources(html.parser.HTMLParser):
    """Collect the inline scripts and the local styles and scripts a page loads before rendering."""

    def __init__(self, all_scripts=False):
        super().__init__(convert_charrefs=False)
        self.all_scripts = all_scripts
        self.files = []
        self.inline_script = 0
        self._in_script = False

    def handle_starttag(self, tag, attrs):
        attrs = dict(attrs)
        if tag == 'script':
            if attrs.get('src'):
                if self.all_scripts or ('async' not in attrs and 'defer' not in attrs and attrs.get('type') != 'module'):
                    self.files.append(attrs['src'])
            else:
                self._in_script = True
        elif tag == 'link' and 'stylesheet' in (attrs.get('rel') or '').split() and attrs.get('href'):
            self.files.append(attrs['href'])

    def handle_data(self, data):
        if self._in_script:
            self.inline_script += len(data.encode('utf-8'))

    def handle_endtag(self, tag):
        if tag == 'script':
            self._in_script = False


def page_sizes(site, page):
    """Return (first-paint size, gzip first-paint size, script size) for one page."""
    data = (site / page).read_bytes()
    parser = _BlockingResources()
    parser.feed(data.decode('utf-8'))
    script = parser.inline_script
    for reference in parser.files:
        if '://' in reference:
            continue
        resource = (site / reference).read_bytes()
        data += resource
        if not reference.endswith('.css'):
            script += len(resource)
    return len(data), metrics.gzip_size(data), script


SHELL_SCRIPT = 'lsg-shell.js'


def frame_size(site, page):
    """Size of the header and sidebar markup between <body> and <main>."""
    text = (site / page).read_text(encoding='utf-8')
    start, end = text.find('<body>'), text.find('<main>')
    return len(text[start:end].encode('utf-8')) if 0 <= start < end else 0


def check_shell(small_site, large_site, pages, shell):
    """Return failure messages for the shared shell script and each page's frame."""
    failures = []
    script = small_site / SHELL_SCRIPT
    if not script.exists():
        return [f'{SHELL_SCRIPT}: missing from the build']
    size = script.stat().st_size
    if size > shell['script_kb'] * 1000:
        failures.append(f'{SHELL_SCRIPT}: {size:,} exceeds {shell["script_kb"] * 1000:,}')
    if (large_site / SHELL_SCRIPT).stat().st_size != size:
        failures.append(f'{SHELL_SCRIPT}: size changed when {FILLER_PAGES:,} unrelated pages were added')
    for page in pages:
        frame = frame_size(small_site, page)
        if frame > shell['frame_kb'] * 1000:
            failures.append(f'{page}: header and sidebar {frame:,} exceed {shell["frame_kb"] * 1000:,}')
    return failures


def check_journals(small_site, older_site, journals):
    """Return failure messages for the feed loader and a home page that grew with older journals."""
    script = small_site / JOURNALS_SCRIPT
    if not script.exists():
        return [f'{JOURNALS_SCRIPT}: missing from the build']
    failures = []
    size = script.stat().st_size
    if size > journals['script_kb'] * 1000:
        failures.append(f'{JOURNALS_SCRIPT}: {size:,} exceeds {journals["script_kb"] * 1000:,}')
    for name in ('index.html', JOURNALS_SCRIPT):
        before, after = (small_site / name).stat().st_size, (older_site / name).stat().st_size
        if before != after:
            failures.append(f'{name}: size changed from {before:,} to {after:,} '
                            f'when {OLDER_JOURNALS:,} older journals were added')
    return failures


def check_outline(site, outline, styles=OUTLINE_STYLES):
    """Return failure messages for the outline script and the outline styles every page carries."""
    script = site / OUTLINE_SCRIPT
    if not script.exists():
        return [f'{OUTLINE_SCRIPT}: missing from the build']
    failures = []
    size = script.stat().st_size
    if size > outline['script_kb'] * 1000:
        failures.append(f'{OUTLINE_SCRIPT}: {size:,} exceeds {outline["script_kb"] * 1000:,}')
    packed = metrics.gzip_size(Path(styles).read_bytes())
    if packed > outline['styles_gzip_kb'] * 1000:
        failures.append(f'outline styles: {packed:,} gzip exceed {outline["styles_gzip_kb"] * 1000:,.0f}')
    return failures


def check_inline(pages, inline, styles=INLINE_STYLES):
    """Return failure messages for the inline styles and growth of the largest fixture page."""
    failures = []
    packed = metrics.gzip_size(Path(styles).read_bytes())
    if packed > inline['styles_gzip_kb'] * 1000:
        failures.append(f'inline styles: {packed:,} gzip exceed {inline["styles_gzip_kb"] * 1000:,.0f}')
    if pages:
        page, (_, largest, _) = max(pages.items(), key=lambda item: item[1][1])
        limit = (inline['largest_page_baseline_gzip_kb'] + inline['largest_page_growth_kb']) * 1000
        if largest > limit:
            failures.append(f'{page}: first paint {largest:,} gzip grew more than '
                            f'{inline["largest_page_growth_kb"] * 1000:,.0f} over the inline baseline')
    return failures


def check_code(site, pages, code, styles=CODE_STYLES):
    """Return failure messages for the code styles and growth of the largest fixture page with code."""
    failures = []
    packed = metrics.gzip_size(Path(styles).read_bytes())
    if packed > code['styles_gzip_kb'] * 1000:
        failures.append(f'code styles: {packed:,} gzip exceed {code["styles_gzip_kb"] * 1000:,.0f}')
    with_code = {page: sizes for page, sizes in pages.items()
                 if CODE_MARKER in (site / page).read_text(encoding='utf-8')}
    if not with_code:
        return failures + ['no fixture page has fenced code']
    page, (_, largest, _) = max(with_code.items(), key=lambda item: item[1][1])
    limit = (code['largest_page_baseline_gzip_kb'] + code['page_growth_gzip_kb']) * 1000
    if largest > limit:
        failures.append(f'{page}: first paint {largest:,} gzip grew more than '
                        f'{code["page_growth_gzip_kb"] * 1000:,.0f} over the code baseline')
    return failures


def check_transclusion(site, pages, transclusion, styles=TRANSCLUSION_STYLES):
    """Return failure messages for the transclusion styles and a fixture set with no embed."""
    failures = []
    packed = metrics.gzip_size(Path(styles).read_bytes())
    if packed > transclusion['styles_gzip_kb'] * 1000:
        failures.append(f'transclusion styles: {packed:,} gzip exceed {transclusion["styles_gzip_kb"] * 1000:,.0f}')
    if not any(marker in (site / page).read_text(encoding='utf-8')
               for page in pages for marker in EMBED_MARKERS):
        failures.append('no fixture page has an embed')
    return failures


def check_references(site, references, styles=REFERENCES_STYLES):
    """Return failure messages for the references script, its styles and each fragment file."""
    script = site / REFERENCES_SCRIPT
    if not script.exists():
        return [f'{REFERENCES_SCRIPT}: missing from the build']
    failures = []
    size = script.stat().st_size
    if size > references['script_kb'] * 1000:
        failures.append(f'{REFERENCES_SCRIPT}: {size:,} exceeds {references["script_kb"] * 1000:,}')
    packed = metrics.gzip_size(Path(styles).read_bytes())
    if packed > references['styles_gzip_kb'] * 1000:
        failures.append(f'references styles: {packed:,} gzip exceed {references["styles_gzip_kb"] * 1000:,.0f}')
    fragments = sorted((site / REFERENCES_DIR).glob('*.html'))
    if not fragments:
        failures.append('no fixture page has a reference fragment')
    for fragment in fragments:
        packed = metrics.gzip_size(fragment.read_bytes())
        if packed > references['fragment_gzip_kb'] * 1000:
            failures.append(f'{REFERENCES_DIR}/{fragment.name}: {packed:,} gzip exceed '
                            f'{references["fragment_gzip_kb"] * 1000:,.0f}')
    return failures


def add_older_journals(garden, count):
    """Add journals dated before every fixture journal."""
    start = date(2000, 1, 1)
    for offset in range(count):
        day = start + timedelta(days=offset)
        (garden / 'journals' / f'{day:%Y_%m_%d}.md').write_text(
            f'- Older journal {offset} that no fixture page refers to.\n', encoding='utf-8')


def profiles(budgets):
    return {name: limits for name, limits in budgets.items() if name in ('desktop', 'phone')}


def site_pages(site):
    return sorted(p.name for p in site.glob('*.html') if p.name != 'pages.html')


def publish(lsg, garden, output, roots, search=False):
    command = [str(lsg), 'publish', '--graph', str(garden), '--output', str(output),
               '--filename-format', 'triple-lowbar', '--journals']
    if search:
        command.append('--search')
    for root in roots:
        command += ['--include', root]
    subprocess.run(command, check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)


def add_filler(garden, count):
    for number in range(count):
        (garden / 'pages' / f'{FILLER_ROOT}___{number:04d}.md').write_text(
            f'- Filler page {number} with text that no fixture page refers to.\n', encoding='utf-8')


def check(small, large, budgets):
    """Return failure messages for budget overruns and pages that grew with the garden."""
    limit_data = min(b['data_before_title_kb'] for b in profiles(budgets).values()) * 1000
    limit_script = min(b['script_before_title_kb'] for b in profiles(budgets).values()) * 1000
    failures = []
    for page, (raw, packed, script) in sorted(small.items()):
        if packed > limit_data:
            failures.append(f'{page}: first paint {packed:,} gzip exceeds {limit_data:,}')
        if script > limit_script:
            failures.append(f'{page}: script {script:,} exceeds {limit_script:,}')
        if page not in large:
            failures.append(f'{page}: missing from the build with {FILLER_PAGES:,} added pages')
        elif large[page][0] != raw:
            failures.append(f'{page}: first paint grew from {raw:,} to {large[page][0]:,} '
                            f'when {FILLER_PAGES:,} unrelated pages were added')
    return failures


def check_search(site, limits, profiles):
    """Conservative initial script accounting includes deferred/module scripts too."""
    failures = []
    startup = next(site.glob('lsg-search-start-*.mjs'))
    if startup.stat().st_size > limits['bootstrap_kb'] * 1000:
        failures.append('search bootstrap exceeds its initial-script budget')
    for page in site_pages(site):
        parser = _BlockingResources(all_scripts=True)
        parser.feed((site / page).read_text(encoding='utf-8'))
        script = parser.inline_script + sum((site / p).stat().st_size for p in parser.files if p.endswith(('.js', '.mjs')))
        limit = min(p['script_before_title_kb'] for p in profiles.values()) * 1000
        if script > limit:
            failures.append(f'{page}: initial search-enabled script {script:,} exceeds {limit:,}')
    return failures


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--lsg', default=str(common.lsg_binary()), help='lsg binary (default: the release build)')
    args = parser.parse_args()
    lsg = Path(args.lsg)
    if not lsg.exists():
        raise SystemExit(f'{lsg} is missing; run `mise run build:release` first')
    budgets = metrics.load_budgets()
    with tempfile.TemporaryDirectory(prefix='lsg-budgets-') as temp:
        temp = Path(temp)
        publish(lsg, FIXTURE_GARDEN, temp / 'small', FIXTURE_ROOTS)
        garden = temp / 'garden'
        shutil.copytree(FIXTURE_GARDEN, garden)
        add_filler(garden, FILLER_PAGES)
        publish(lsg, garden, temp / 'large', [*FIXTURE_ROOTS, FILLER_ROOT])
        small = {page: page_sizes(temp / 'small', page) for page in site_pages(temp / 'small')}
        large = {page: page_sizes(temp / 'large', page) for page in small if (temp / 'large' / page).exists()}
        shell_failures = check_shell(temp / 'small', temp / 'large', small, budgets['shell'])
        older = temp / 'older-garden'
        shutil.copytree(FIXTURE_GARDEN, older)
        add_older_journals(older, OLDER_JOURNALS)
        publish(lsg, older, temp / 'older', FIXTURE_ROOTS)
        publish(lsg, FIXTURE_GARDEN, temp / 'search', FIXTURE_ROOTS, search=True)
        search_failures = check_search(temp / 'search', budgets['search'], profiles(budgets))
        journal_failures = check_journals(temp / 'small', temp / 'older', budgets['journals'])
        print(f"{SHELL_SCRIPT}: {(temp / 'small' / SHELL_SCRIPT).stat().st_size:,}")
        outline_failures = check_outline(temp / 'small', budgets['outline'])
        code_failures = check_code(temp / 'small', small, budgets['code'])
        transclusion_failures = check_transclusion(temp / 'small', small, budgets['transclusion'])
        references_failures = check_references(temp / 'small', budgets['references'])
        print(f"{REFERENCES_SCRIPT}: {(temp / 'small' / REFERENCES_SCRIPT).stat().st_size:,}")
        print(f"{JOURNALS_SCRIPT}: {(temp / 'small' / JOURNALS_SCRIPT).stat().st_size:,}")
        print(f"{OUTLINE_SCRIPT}: {(temp / 'small' / OUTLINE_SCRIPT).stat().st_size:,}")
    for page, (raw, packed, script) in sorted(small.items()):
        print(f'{page}: first paint {raw:,} ({packed:,} gzip), script {script:,}')
    inline_failures = check_inline(small, budgets['inline'])
    print(f'inline styles: {metrics.gzip_size(INLINE_STYLES.read_bytes()):,} gzip')
    print(f'code styles: {metrics.gzip_size(CODE_STYLES.read_bytes()):,} gzip')
    print(f'transclusion styles: {metrics.gzip_size(TRANSCLUSION_STYLES.read_bytes()):,} gzip')
    print(f'references styles: {metrics.gzip_size(REFERENCES_STYLES.read_bytes()):,} gzip')
    failures = (check(small, large, budgets) + shell_failures + journal_failures + outline_failures
                + inline_failures + code_failures + transclusion_failures + references_failures
                + search_failures)
    if failures:
        print('\n'.join(failures), file=sys.stderr)
        raise SystemExit(1)
    print(f'{len(small)} pages within budget and unchanged with {FILLER_PAGES:,} added pages; '
          f'home page unchanged with {OLDER_JOURNALS:,} older journals')


if __name__ == '__main__':
    main()
