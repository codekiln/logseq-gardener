#!/usr/bin/env python3
"""Check lsg page sizes against the budgets, and that they stay the same as the garden grows."""
import argparse
import html.parser
import json
import re
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
# Enough filler for a full first All pages screen before the garden grows by FILLER_PAGES.
ALL_PAGES_BASE = 40
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
ALL_PAGES_SCRIPT = 'lsg-all-pages.js'
ALL_PAGES_LIST = '_pages/list.json'
GROWN_ROOT = 'Grown'
PLUGIN_FIXTURE = common.REPO / 'tests' / 'fixtures' / 'plugins' / 'recorder' / 'lsg-plugin.json'
# The tags a build with plugins adds before </head>: the page metadata and the host loader.
PLUGIN_TAGS = re.compile(r'<script type="application/json" data-lsg-page>.*?</script>'
                         r'|<script type="module" data-lsg-host>.*?</script>', re.S)
GROWN_BASE = 200



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


SIDEBAR_SCRIPT = 'lsg-sidebar.js'


def check_sidebar(site, pages, sidebar):
    """Return failure messages for the right sidebar's script and card transfers."""
    script = site / SIDEBAR_SCRIPT
    if not script.exists():
        return [f'{SIDEBAR_SCRIPT}: missing from the build']
    failures = []
    size = metrics.gzip_size(script.read_bytes())
    if size > sidebar['script_gzip_kb'] * 1000:
        failures.append(f'{SIDEBAR_SCRIPT}: {size:,} gzip exceeds {sidebar["script_gzip_kb"] * 1000:,}')
    for page in pages:
        data = (site / page).read_bytes()
        if SIDEBAR_SCRIPT.encode() in data:
            failures.append(f'{page}: requests {SIDEBAR_SCRIPT} before the sidebar is used')
        card = metrics.gzip_size(data)
        if card > sidebar['card_fetch_gzip_kb'] * 1000:
            failures.append(f'{page}: card transfer {card:,} gzip exceeds {sidebar["card_fetch_gzip_kb"] * 1000:,}')
    return failures


SLIDES_SCRIPT = 'lsg-slides.js'


def check_slides(site, pages, slides):
    """Return failure messages for View as slides: its script, and pages that request it."""
    script = site / SLIDES_SCRIPT
    if not script.exists():
        return [f'{SLIDES_SCRIPT}: missing from the build']
    failures = []
    size = metrics.gzip_size(script.read_bytes())
    if size > slides['script_gzip_kb'] * 1000:
        failures.append(f'{SLIDES_SCRIPT}: {size:,} gzip exceeds {slides["script_gzip_kb"] * 1000:,}')
    for page in pages:
        if SLIDES_SCRIPT.encode() in (site / page).read_bytes():
            failures.append(f'{page}: requests {SLIDES_SCRIPT} before a Slides card renders')
    return failures


MEDIA_SCRIPT = 'media.js'
MEDIA_STYLES = 'media.css'
MEDIA_MARKER = 'class="lsg-video"'


def placeholder_sizes(text):
    """Byte sizes of every video placeholder in a page's markup."""
    sizes, start = [], text.find('<div ' + MEDIA_MARKER)
    while start >= 0:
        end = text.find('</div>', start) + len('</div>')
        sizes.append(len(text[start:end].encode('utf-8')))
        start = text.find('<div ' + MEDIA_MARKER, end)
    return sizes


def check_media(site, pages, media):
    """Return failure messages for the media script and styles, placeholders, and pages that load
    media code or a third party before a click."""
    site = Path(site)
    failures = []
    for name in (MEDIA_SCRIPT, MEDIA_STYLES):
        if not (site / name).exists():
            return [f'{name}: missing from the build']
    script = (site / MEDIA_SCRIPT).stat().st_size
    if script > media['script_kb'] * 1000:
        failures.append(f'{MEDIA_SCRIPT}: {script:,} exceeds {media["script_kb"] * 1000:,}')
    styles = metrics.gzip_size((site / MEDIA_STYLES).read_bytes())
    if styles > media['styles_gzip_kb'] * 1000:
        failures.append(f'{MEDIA_STYLES}: {styles:,} gzip exceeds {media["styles_gzip_kb"] * 1000:,}')
    videos = 0
    for page in pages:
        text = (site / page).read_text(encoding='utf-8')
        sizes = placeholder_sizes(text)
        videos += len(sizes)
        loads = MEDIA_SCRIPT in text or MEDIA_STYLES in text
        if loads and not sizes:
            failures.append(f'{page}: loads media code without a video')
        if sizes and not loads:
            failures.append(f'{page}: has a video but does not load {MEDIA_SCRIPT}')
        if sizes and max(sizes) > media['placeholder_kb'] * 1000:
            failures.append(f'{page}: video placeholder {max(sizes):,} exceeds {media["placeholder_kb"] * 1000:,.0f}')
        if '<iframe' in text or 'youtube.com/iframe_api' in text:
            failures.append(f'{page}: first paint loads a third-party player')
    if not videos:
        failures.append('no fixture page has a video placeholder')
    return failures


QUERIES_SCRIPT = 'lsg-queries.js'
QUERIES_STYLES = 'lsg-queries.css'
QUERIES_MARKER = 'class="custom-query'
QUERY_FRAGMENT = re.compile(r'-q\d+(-alt)?-\d+\.html$')


def query_boxes(text):
    """The markup of each query box in a page: from its opening tag to its closing tag."""
    boxes = []
    for match in re.finditer(r'<(div|details) class="custom-query', text):
        tag, depth, position = match.group(1), 0, match.start()
        for token in re.finditer(rf'<{tag}\b|</{tag}>', text[match.start():]):
            depth += -1 if token.group().startswith('</') else 1
            if depth == 0:
                boxes.append(text[position:match.start() + token.end()])
                break
    return boxes


def check_queries(site, pages, queries):
    """Return failure messages for the query script and styles, the pages that load them, each
    query box's in-page part and each query fragment file."""
    site = Path(site)
    failures = []
    for name in (QUERIES_SCRIPT, QUERIES_STYLES):
        if not (site / name).exists():
            return [f'{name}: missing from the build']
    script = (site / QUERIES_SCRIPT).stat().st_size
    if script > queries['script_kb'] * 1000:
        failures.append(f'{QUERIES_SCRIPT}: {script:,} exceeds {queries["script_kb"] * 1000:,}')
    styles = metrics.gzip_size((site / QUERIES_STYLES).read_bytes())
    if styles > queries['styles_gzip_kb'] * 1000:
        failures.append(f'{QUERIES_STYLES}: {styles:,} gzip exceeds {queries["styles_gzip_kb"] * 1000:,}')
    boxes = 0
    for page in pages:
        text = (site / page).read_text(encoding='utf-8')
        found = query_boxes(text)
        boxes += len(found)
        loads = QUERIES_SCRIPT in text or QUERIES_STYLES in text
        if loads and not found:
            failures.append(f'{page}: loads query code without a query')
        if found and not loads:
            failures.append(f'{page}: has a query but does not load {QUERIES_SCRIPT}')
        for box in found:
            size = metrics.gzip_size(box.encode('utf-8'))
            if size > queries['in_page_gzip_kb'] * 1000:
                failures.append(f'{page}: query box {size:,} gzip exceeds {queries["in_page_gzip_kb"] * 1000:,}')
    if not boxes:
        failures.append('no fixture page has a query box')
    for fragment in sorted((site / REFERENCES_DIR).glob('*.html')):
        if not QUERY_FRAGMENT.search(fragment.name):
            continue
        size = metrics.gzip_size(fragment.read_bytes())
        if size > queries['fragment_gzip_kb'] * 1000:
            failures.append(f'{fragment.name}: {size:,} gzip exceeds {queries["fragment_gzip_kb"] * 1000:,}')
    return failures


FLASHCARDS_SCRIPT = 'lsg-flashcards.js'
FLASHCARDS_STYLES = 'lsg-flashcards.css'
FLASHCARDS_ALL = 'flashcards/decks/all.json'
FLASHCARDS_BATCHES = 'flashcards/cards'


def without_deck_widgets(text):
    """A page without its deck widgets: the inline style blocks that hold the widget styles and each
    `cards-review` element, from its opening tag to its closing tag."""
    text = re.sub(r'<style>[^<]*\.cards-review[^<]*</style>', '', text)
    while True:
        match = re.search(r'<div class="cards-review', text)
        if not match:
            return text
        depth = 0
        for token in re.finditer(r'<div\b|</div>', text[match.start():]):
            depth += -1 if token.group().startswith('</') else 1
            if depth == 0:
                text = text[:match.start()] + text[match.start() + token.end():]
                break
        else:
            return text


def check_flashcards(site, pages, flashcards):
    """Return failure messages for the flashcards script, styles, all-cards deck and card batches,
    for a page that links the review script, and for the deck page's growth over its size without
    the deck widgets."""
    site = Path(site)
    failures = []
    for name in (FLASHCARDS_SCRIPT, FLASHCARDS_STYLES, FLASHCARDS_ALL):
        if not (site / name).exists():
            return [f'{name}: missing from the build']
    script = (site / FLASHCARDS_SCRIPT).stat().st_size
    if script > flashcards['script_kb'] * 1000:
        failures.append(f'{FLASHCARDS_SCRIPT}: {script:,} exceeds {flashcards["script_kb"] * 1000:,.0f}')
    styles = metrics.gzip_size((site / FLASHCARDS_STYLES).read_bytes())
    if styles > flashcards['styles_gzip_kb'] * 1000:
        failures.append(f'{FLASHCARDS_STYLES}: {styles:,} gzip exceeds {flashcards["styles_gzip_kb"] * 1000:,.0f}')
    deck = metrics.gzip_size((site / FLASHCARDS_ALL).read_bytes())
    if deck > flashcards['deck_gzip_kb'] * 1000:
        failures.append(f'{FLASHCARDS_ALL}: {deck:,} gzip exceeds {flashcards["deck_gzip_kb"] * 1000:,.0f}')
    for batch in sorted((site / FLASHCARDS_BATCHES).glob('*.html')):
        size = metrics.gzip_size(batch.read_bytes())
        if size > flashcards['card_batch_gzip_kb'] * 1000:
            failures.append(f'{batch.name}: {size:,} gzip exceeds {flashcards["card_batch_gzip_kb"] * 1000:,.0f}')
    for page in pages:
        if FLASHCARDS_SCRIPT in (site / page).read_text(encoding='utf-8'):
            failures.append(f'{page}: links {FLASHCARDS_SCRIPT}, which loads when the review opens')
    title = flashcards['deck_page']
    deck_page = site / common.lsg_route(title)
    if not deck_page.exists():
        failures.append(f'{title}: no deck page in the build')
    else:
        text = deck_page.read_text(encoding='utf-8')
        if 'class="cards-review' not in text:
            failures.append(f'{title}: has no deck widget')
        growth = metrics.gzip_size(text.encode('utf-8')) - metrics.gzip_size(without_deck_widgets(text).encode('utf-8'))
        if growth > flashcards['deck_page_growth_gzip_kb'] * 1000:
            failures.append(f'{title}: deck widgets add {growth:,} gzip, over '
                            f'{flashcards["deck_page_growth_gzip_kb"] * 1000:,.0f}')
    return failures


def flashcards_summary(site, flashcards):
    """The sizes check_flashcards compares, as one line."""
    site = Path(site)
    text = (site / common.lsg_route(flashcards['deck_page'])).read_text(encoding='utf-8')
    growth = metrics.gzip_size(text.encode('utf-8')) - metrics.gzip_size(without_deck_widgets(text).encode('utf-8'))
    batches = [metrics.gzip_size(b.read_bytes()) for b in (site / FLASHCARDS_BATCHES).glob('*.html')]
    return (f"{FLASHCARDS_SCRIPT}: {(site / FLASHCARDS_SCRIPT).stat().st_size:,}; {FLASHCARDS_STYLES}: "
            f"{metrics.gzip_size((site / FLASHCARDS_STYLES).read_bytes()):,} gzip; {FLASHCARDS_ALL}: "
            f"{metrics.gzip_size((site / FLASHCARDS_ALL).read_bytes()):,} gzip; largest card batch "
            f"{max(batches, default=0):,} gzip; deck page growth {growth:,} gzip")


GRAPH_SCRIPT = 'lsg-graph.js'
GRAPH_STYLES = 'lsg-graph.css'
GRAPH_WORKER = 'lsg-graph-worker.js'
GRAPH_DATA = '_graph/default.bin'
GRAPH_NEIGHBOURHOODS = '_graph/p'


def graph_sizes(site):
    """Sizes of the graph files in a site: the data file as stored (gzip), the script with its
    styles and the worker (gzip), and the largest neighbourhood file (gzip) with its name."""
    site = Path(site)
    neighbourhoods = sorted(((metrics.gzip_size(p.read_bytes()), p.name)
                             for p in (site / GRAPH_NEIGHBOURHOODS).glob('*.json')), reverse=True)
    return {
        'data': (site / GRAPH_DATA).stat().st_size,
        'script': metrics.gzip_size((site / GRAPH_SCRIPT).read_bytes() + (site / GRAPH_STYLES).read_bytes()),
        'worker': metrics.gzip_size((site / GRAPH_WORKER).read_bytes()),
        'largest_neighbourhood': neighbourhoods[0] if neighbourhoods else (0, None),
        'neighbourhoods': len(neighbourhoods),
    }


def check_graph(site, pages, graph):
    """Return failure messages for the graph files over budget and pages that load graph data."""
    site = Path(site)
    for name in (GRAPH_DATA, GRAPH_SCRIPT, GRAPH_STYLES, GRAPH_WORKER, 'graph.html'):
        if not (site / name).exists():
            return [f'{name}: missing from the build']
    sizes = graph_sizes(site)
    failures = []
    for key, limit, label in (('data', 'data_gzip_kb', GRAPH_DATA),
                              ('script', 'script_kb', f'{GRAPH_SCRIPT} and {GRAPH_STYLES}'),
                              ('worker', 'worker_kb', GRAPH_WORKER)):
        if sizes[key] > graph[limit] * 1000:
            failures.append(f'{label}: {sizes[key]:,} gzip exceeds {graph[limit] * 1000:,}')
    size, name = sizes['largest_neighbourhood']
    if size > graph['page_graph_fragment_gzip_kb'] * 1000:
        failures.append(f'{GRAPH_NEIGHBOURHOODS}/{name}: {size:,} gzip exceeds {graph["page_graph_fragment_gzip_kb"] * 1000:,}')
    for page in pages:
        if page == 'graph.html':
            continue
        text = (site / page).read_text(encoding='utf-8')
        if '_graph/' in text or GRAPH_SCRIPT in text or GRAPH_WORKER in text:
            failures.append(f'{page}: loads graph data or the graph script before use')
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
    packed = metrics.gzip_size(hierarchy_styles(styles, hierarchy=False).encode('utf-8'))
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


def hierarchy_styles(styles=REFERENCES_STYLES, hierarchy=True):
    """The rules in the references styles that only the Hierarchy section uses, or the others.

    The Hierarchy rules share the file with the reference sections but have their own budget.
    """
    rules = Path(styles).read_text(encoding='utf-8').replace('}', '}\n').splitlines()
    return ''.join(r for r in rules
                   if any(word in r.split('{')[0] for word in ('hierarchy', 'namespace')) == hierarchy)


def grown_root(lsg, temp, count):
    """Publish the fixture garden with `count` pages under a namespace; return its root page."""
    garden = temp / f'grown-{count}'
    shutil.copytree(FIXTURE_GARDEN, garden)
    (garden / 'pages' / f'{GROWN_ROOT}.md').write_text('- The grown namespace root.\n', encoding='utf-8')
    for number in range(count):
        (garden / 'pages' / f'{GROWN_ROOT}___{number:04d}.md').write_text(
            f'- Grown page {number}.\n', encoding='utf-8')
    site = temp / f'grown-{count}-site'
    publish(lsg, garden, site, [*FIXTURE_ROOTS, GROWN_ROOT])
    return site


def check_hierarchy(site, smaller, larger, hierarchy, styles=REFERENCES_STYLES):
    """Return failure messages for hierarchy rows, styles, script growth, fragments and a grown root.

    `smaller` and `larger` are sites whose grown namespace differs by 2,000 pages; both are past
    the rows a page holds, so the root page should not change.
    """
    failures = []
    packed = metrics.gzip_size(hierarchy_styles(styles).encode('utf-8'))
    if packed > hierarchy['styles_gzip_kb'] * 1000:
        failures.append(f'hierarchy styles: {packed:,} gzip exceed {hierarchy["styles_gzip_kb"] * 1000:,.0f}')
    script = site / REFERENCES_SCRIPT
    if script.exists():
        limit = (hierarchy['script_baseline_kb'] + hierarchy['script_growth_kb']) * 1000
        if script.stat().st_size > limit:
            failures.append(f'{REFERENCES_SCRIPT}: {script.stat().st_size:,} exceeds the hierarchy limit {limit:,.0f}')
    for built in (site, smaller, larger):
        for page in site_pages(built):
            text = (built / page).read_text(encoding='utf-8')
            section = text.partition('class="page-hierarchy')[2].partition('</section>')[0]
            rows = section.count('<li>')
            if rows > hierarchy['inline_rows']:
                failures.append(f'{page}: {rows:,} hierarchy rows exceed {hierarchy["inline_rows"]:,}')
        for fragment in sorted((built / REFERENCES_DIR).glob('*-hierarchy-*.html')):
            packed = metrics.gzip_size(fragment.read_bytes())
            if packed > hierarchy['fragment_gzip_kb'] * 1000:
                failures.append(f'{REFERENCES_DIR}/{fragment.name}: {packed:,} gzip exceed '
                                f'{hierarchy["fragment_gzip_kb"] * 1000:,.0f}')
    root = common.lsg_index(smaller).get(common.normalize(GROWN_ROOT), (None, None))[1]
    if root is None or not (larger / root).exists():
        return failures + [f'{GROWN_ROOT}: missing from the grown builds']
    before, after = (smaller / root).stat().st_size, (larger / root).stat().st_size
    if after - before > hierarchy['root_growth_kb'] * 1000:
        failures.append(f'{GROWN_ROOT}: grew from {before:,} to {after:,} with 2,000 more pages')
    if not list((larger / REFERENCES_DIR).glob('*-hierarchy-*.html')):
        failures.append(f'{GROWN_ROOT}: no hierarchy fragment in the grown build')
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


def is_all_pages(name):
    """All pages and its static paging pages, which list the garden and so grow with it."""
    return name == 'pages.html' or (name.startswith('pages-') and name.endswith('.html'))


def site_pages(site):
    return sorted(p.name for p in site.glob('*.html') if not is_all_pages(p.name))


def check_all_pages(small, large, budgets):
    """Return failure messages for All pages: first paint, paging pages, growth, script and list."""
    limits = budgets['all_pages']
    failures = []
    first = 'pages.html'
    if not (small / first).exists():
        return [f'{first}: missing from the build']
    raw, packed, script = page_sizes(small, first)
    print(f'{first}: first paint {raw:,} ({packed:,} gzip), script {script:,}')
    failures += [message.replace(f'{first}: ', f'{first} (All pages): ')
                 for message in check({first: (raw, packed, script)}, {first: (raw, packed, script)}, budgets)]
    for page in sorted(p.name for p in small.glob('pages-*.html')):
        size = metrics.gzip_size((small / page).read_bytes())
        if size > limits['paging_page_gzip_kb'] * 1000:
            failures.append(f'{page}: {size:,} gzip exceeds {limits["paging_page_gzip_kb"] * 1000:,.0f}')
    grown = page_sizes(large, first)[0]
    if grown - raw > limits['growth_kb'] * 1000:
        failures.append(f'{first}: first paint grew from {raw:,} to {grown:,} '
                        f'when {FILLER_PAGES:,} unrelated pages were added')
    script_size = metrics.gzip_size((small / ALL_PAGES_SCRIPT).read_bytes())
    print(f'{ALL_PAGES_SCRIPT}: {script_size:,} gzip')
    if script_size > limits['script_gzip_kb'] * 1000:
        failures.append(f'{ALL_PAGES_SCRIPT}: {script_size:,} gzip exceeds {limits["script_gzip_kb"] * 1000:,.0f}')
    list_size = metrics.gzip_size((small / ALL_PAGES_LIST).read_bytes())
    if list_size > limits['list_gzip_kb'] * 1000:
        failures.append(f'{ALL_PAGES_LIST}: {list_size:,} gzip exceeds {limits["list_gzip_kb"] * 1000:,.0f}')
    if any(ALL_PAGES_SCRIPT in (small / page).read_text(encoding='utf-8') for page in site_pages(small)):
        failures.append(f'{ALL_PAGES_SCRIPT}: loaded outside All pages')
    return failures


def publish(lsg, garden, output, roots, search=False, extra=()):
    command = [str(lsg), 'publish', '--graph', str(garden), '--output', str(output),
               '--filename-format', 'triple-lowbar', '--journals', *extra]
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


def plugin_bytes(text):
    """Bytes of plugin metadata and host loader in a page, and whether it names the host module."""
    return sum(len(tag.encode('utf-8')) for tag in PLUGIN_TAGS.findall(text)) + text.count('lsg-host-')


def check_plugins(plain, site, budgets):
    """Return failure messages for [plugins]: no added bytes without plugins, the host's gzip size,
    and every page's first paint with the fixture plugin against the [desktop] and [phone] budgets."""
    limits = budgets['plugins']
    failures = []
    added = max(plugin_bytes((plain / page).read_text(encoding='utf-8')) for page in site_pages(plain))
    print(f'plugins: {added} added bytes per page without plugins')
    if added > limits['added_bytes_per_page']:
        failures.append(f'a build without plugins adds {added:,} plugin bytes to a page, over {limits["added_bytes_per_page"]}')
    if any('lsg-site.json' in (plain / page).read_text(encoding='utf-8') for page in site_pages(plain)):
        failures.append('lsg-site.json: requested by a generated page')
    hosts = sorted(site.glob('lsg-host-*.mjs'))
    if len(hosts) != 1:
        return failures + [f'expected one host module in the plugin build, found {len(hosts)}']
    host = metrics.gzip_size(hosts[0].read_bytes())
    print(f'{hosts[0].name}: {hosts[0].stat().st_size:,} ({host:,} gzip)')
    if host > limits['host_gzip_kb'] * 1000:
        failures.append(f'{hosts[0].name}: {host:,} gzip exceeds {limits["host_gzip_kb"] * 1000:,.0f}')
    sizes = {page: page_sizes(site, page) for page in site_pages(site) if page != 'graph.html'}
    largest = max(sizes.items(), key=lambda item: item[1][1])
    print(f'plugins: largest first paint {largest[1][1]:,} gzip ({largest[0]}), script {max(s[2] for s in sizes.values()):,}')
    failures += [f'with a plugin, {message}' for message in check(sizes, sizes, budgets)]
    return failures


def check_search(site, limits, profiles):
    """Conservative initial script accounting includes deferred/module scripts too."""
    failures = []
    startup = next(site.glob('lsg-search-start-*.mjs'))
    if startup.stat().st_size > limits['bootstrap_kb'] * 1000:
        failures.append('search bootstrap exceeds its initial-script budget')
    for page in site_pages(site):
        # The graph view's renderer is that page's content; [graph] script_kb budgets it.
        if page == 'graph.html':
            continue
        parser = _BlockingResources(all_scripts=True)
        parser.feed((site / page).read_text(encoding='utf-8'))
        script = parser.inline_script + sum((site / p).stat().st_size for p in parser.files if p.endswith(('.js', '.mjs')))
        limit = min(p['script_before_title_kb'] for p in profiles.values()) * 1000
        if script > limit:
            failures.append(f'{page}: initial search-enabled script {script:,} exceeds {limit:,}')
    return failures


def search_index_sizes(site):
    """Sizes of the search format version 2 files in a site: the title index, every shard and chunk.

    Returns the title index's decoded and gzip sizes, the number of pages and blocks, and per
    shard and chunk its file name and gzip size, largest first.
    """
    site = Path(site)
    manifests = sorted(site.glob('lsg-search-manifest-*.json'))
    if len(manifests) != 1:
        raise ValueError(f'expected one search manifest in {site}, found {len(manifests)}')
    manifest = json.loads(manifests[0].read_text(encoding='utf-8'))
    if manifest.get('format_version') != 2:
        raise ValueError(f"{manifests[0].name}: search format version {manifest.get('format_version')}, expected 2")
    titles = (site / f"lsg-search-titles-{manifest['titles']}.json").read_bytes()

    def files(kind, digests):
        found = [(f'lsg-search-{kind}-{d}.json', metrics.gzip_size((site / f'lsg-search-{kind}-{d}.json').read_bytes()))
                 for d in digests]
        return sorted(found, key=lambda item: -item[1])

    return {'pages': len(json.loads(titles)['pages']), 'blocks': manifest['blocks'],
            'titles': f"lsg-search-titles-{manifest['titles']}.json",
            'titles_decoded': len(titles), 'titles_gzip': metrics.gzip_size(titles),
            'shards': files('shard', manifest['shards'].values()),
            'shard_keys': {f'lsg-search-shard-{d}.json': key for key, d in manifest['shards'].items()},
            'chunks': files('chunk', manifest['chunks'])}


def check_search_index(sizes, limits):
    """Return failure messages for a title index, shard or chunk over its budget, naming the largest."""
    failures = []
    if sizes['titles_gzip'] > limits['titles_gzip_kb'] * 1000:
        failures.append(f"{sizes['titles']}: title index {sizes['titles_gzip']:,} gzip exceeds "
                        f"{limits['titles_gzip_kb'] * 1000:,.0f}")
    if sizes['titles_decoded'] > limits['titles_decoded_mb'] * 1_000_000:
        failures.append(f"{sizes['titles']}: title index {sizes['titles_decoded']:,} decoded exceeds "
                        f"{limits['titles_decoded_mb'] * 1_000_000:,.0f}")
    for kind, limit in (('shards', limits['shard_gzip_kb'] * 1000), ('chunks', limits['chunk_gzip_kb'] * 1000)):
        over = [(name, size) for name, size in sizes[kind] if size > limit]
        if over:
            name, size = over[0]
            key = sizes['shard_keys'].get(name)
            label = f'{name} (words starting {key!r})' if key else name
            others = f' and {len(over) - 1:,} more' if len(over) > 1 else ''
            failures.append(f'{label}: largest {kind[:-1]} {size:,} gzip exceeds {limit:,.0f}{others}')
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
        # A full first All pages screen, without git history like the grown garden, so both
        # list pages by name; the grown garden then gains FILLER_PAGES more.
        add_filler(garden, ALL_PAGES_BASE)
        publish(lsg, garden, temp / 'all-pages-base', [*FIXTURE_ROOTS, FILLER_ROOT])
        add_filler(garden, ALL_PAGES_BASE + FILLER_PAGES)
        publish(lsg, garden, temp / 'large', [*FIXTURE_ROOTS, FILLER_ROOT])
        small = {page: page_sizes(temp / 'small', page) for page in site_pages(temp / 'small')}
        large = {page: page_sizes(temp / 'large', page) for page in small if (temp / 'large' / page).exists()}
        shell_failures = check_shell(temp / 'small', temp / 'large', small, budgets['shell'])
        older = temp / 'older-garden'
        shutil.copytree(FIXTURE_GARDEN, older)
        add_older_journals(older, OLDER_JOURNALS)
        publish(lsg, older, temp / 'older', FIXTURE_ROOTS)
        publish(lsg, FIXTURE_GARDEN, temp / 'search', FIXTURE_ROOTS, search=True)
        publish(lsg, FIXTURE_GARDEN, temp / 'plugins', FIXTURE_ROOTS, extra=('--plugin', str(PLUGIN_FIXTURE)))
        plugin_failures = check_plugins(temp / 'small', temp / 'plugins', budgets)
        search_failures = check_search(temp / 'search', budgets['search'], profiles(budgets))
        index = search_index_sizes(temp / 'search')
        search_failures += check_search_index(index, budgets['search'])
        print(f"search title index: {index['titles_gzip']:,} gzip, {index['titles_decoded']:,} decoded; "
              f"largest shard {index['shards'][0][1]:,} gzip, largest chunk {index['chunks'][0][1]:,} gzip")
        sidebar_failures = check_sidebar(temp / 'small', small, budgets['sidebar'])
        slides_failures = check_slides(temp / 'small', small, budgets['slides'])
        print(f"{SLIDES_SCRIPT}: {metrics.gzip_size((temp / 'small' / SLIDES_SCRIPT).read_bytes()):,} gzip")
        graph_failures = check_graph(temp / 'small', small, budgets['graph'])
        media_failures = check_media(temp / 'small', small, budgets['media'])
        print(f"{MEDIA_SCRIPT}: {(temp / 'small' / MEDIA_SCRIPT).stat().st_size:,}; {MEDIA_STYLES}: "
              f"{metrics.gzip_size((temp / 'small' / MEDIA_STYLES).read_bytes()):,} gzip; largest placeholder "
              f"{max(max(placeholder_sizes((temp / 'small' / p).read_text(encoding='utf-8')), default=0) for p in small):,}")
        queries_failures = check_queries(temp / 'small', small, budgets['queries'])
        print(f"{QUERIES_SCRIPT}: {(temp / 'small' / QUERIES_SCRIPT).stat().st_size:,}; {QUERIES_STYLES}: "
              f"{metrics.gzip_size((temp / 'small' / QUERIES_STYLES).read_bytes()):,} gzip")
        flashcards_failures = check_flashcards(temp / 'small', small, budgets['flashcards'])
        if not any('missing from the build' in f or 'no deck page' in f for f in flashcards_failures):
            print(flashcards_summary(temp / 'small', budgets['flashcards']))
        sizes = graph_sizes(temp / 'small')
        print(f"graph: data {sizes['data']:,} gzip, script and styles {sizes['script']:,} gzip, "
              f"worker {sizes['worker']:,} gzip, {sizes['neighbourhoods']} neighbourhood files, "
              f"largest {sizes['largest_neighbourhood'][0]:,} gzip")
        print(f"{SIDEBAR_SCRIPT}: {metrics.gzip_size((temp / 'small' / SIDEBAR_SCRIPT).read_bytes()):,} gzip")
        journal_failures = check_journals(temp / 'small', temp / 'older', budgets['journals'])
        print(f"{SHELL_SCRIPT}: {(temp / 'small' / SHELL_SCRIPT).stat().st_size:,}")
        outline_failures = check_outline(temp / 'small', budgets['outline'])
        code_failures = check_code(temp / 'small', small, budgets['code'])
        transclusion_failures = check_transclusion(temp / 'small', small, budgets['transclusion'])
        references_failures = check_references(temp / 'small', budgets['references'])
        all_pages_failures = check_all_pages(temp / 'all-pages-base', temp / 'large', budgets)
        hierarchy_failures = check_hierarchy(temp / 'small', grown_root(lsg, temp, GROWN_BASE),
                                             grown_root(lsg, temp, GROWN_BASE + FILLER_PAGES),
                                             budgets['hierarchy'])
        print(f"{REFERENCES_SCRIPT}: {(temp / 'small' / REFERENCES_SCRIPT).stat().st_size:,}")
        print(f"{JOURNALS_SCRIPT}: {(temp / 'small' / JOURNALS_SCRIPT).stat().st_size:,}")
        print(f"{OUTLINE_SCRIPT}: {(temp / 'small' / OUTLINE_SCRIPT).stat().st_size:,}")
    for page, (raw, packed, script) in sorted(small.items()):
        print(f'{page}: first paint {raw:,} ({packed:,} gzip), script {script:,}')
    inline_failures = check_inline(small, budgets['inline'])
    print(f'inline styles: {metrics.gzip_size(INLINE_STYLES.read_bytes()):,} gzip')
    print(f'code styles: {metrics.gzip_size(CODE_STYLES.read_bytes()):,} gzip')
    print(f'transclusion styles: {metrics.gzip_size(TRANSCLUSION_STYLES.read_bytes()):,} gzip')
    print(f'references styles: {metrics.gzip_size(hierarchy_styles(hierarchy=False).encode("utf-8")):,} gzip')
    print(f'hierarchy styles: {metrics.gzip_size(hierarchy_styles().encode("utf-8")):,} gzip')
    failures = (check(small, large, budgets) + shell_failures + journal_failures + outline_failures
                + inline_failures + code_failures + transclusion_failures + references_failures
                + hierarchy_failures + sidebar_failures + slides_failures + graph_failures
                + search_failures + all_pages_failures + media_failures + queries_failures + flashcards_failures
                + plugin_failures)
    if failures:
        print('\n'.join(failures), file=sys.stderr)
        raise SystemExit(1)
    print(f'{len(small)} pages within budget and unchanged with {FILLER_PAGES:,} added pages; '
          f'home page unchanged with {OLDER_JOURNALS:,} older journals')


if __name__ == '__main__':
    main()
