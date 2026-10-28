"""Paths, title matching and routes shared by the parity tasks."""
import html
import html.parser
import json
import os
import re
import subprocess
import unicodedata
import urllib.parse
from datetime import datetime, timezone
from pathlib import Path

REPO = Path(__file__).resolve().parents[2]
PARITY_ROOT = REPO / 'target' / 'parity'
GARDEN_ADDRESS = 'github.com/codekiln/logseq-encode-garden'


def encode_garden():
    result = subprocess.run(['ghq', 'list', '--full-path', '--exact', GARDEN_ADDRESS],
                            capture_output=True, text=True, check=True)
    path = result.stdout.strip().splitlines()
    if not path:
        raise SystemExit(f'ghq has no checkout of {GARDEN_ADDRESS}; pass --garden PATH')
    return Path(path[0])


def lsg_binary():
    target = Path(os.environ.get('CARGO_TARGET_DIR', REPO / 'target'))
    return target / 'release' / 'lsg'


def new_run_dir():
    stamp = datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%SZ')
    path = PARITY_ROOT / stamp
    path.mkdir(parents=True)
    return path


def runs():
    return sorted(p for p in PARITY_ROOT.glob('*') if (p / 'settings.json').exists())


def latest_run(explicit=None):
    if explicit:
        return Path(explicit)
    found = runs()
    if not found:
        raise SystemExit('no parity build found; run `mise run parity:build` first')
    return found[-1]


def git_commit(path, ref='HEAD'):
    return subprocess.run(['git', '-C', str(path), 'rev-parse', ref],
                          capture_output=True, text=True, check=True).stdout.strip()


def normalize(title):
    """Match page names the way Logseq does: lower case after NFC normalization."""
    return unicodedata.normalize('NFC', title).lower()


def logseq_route(title):
    return '#/page/' + urllib.parse.quote(normalize(title), safe='')


class _IndexLinks(html.parser.HTMLParser):
    def __init__(self):
        super().__init__(convert_charrefs=True)
        self.links = {}
        self._href = None
        self._text = []

    def handle_starttag(self, tag, attrs):
        if tag == 'a':
            href = dict(attrs).get('href') or ''
            if re.fullmatch(r'p-[0-9a-f]{16}\.html', href):
                self._href = href
                self._text = []

    def handle_data(self, data):
        if self._href:
            self._text.append(data)

    def handle_endtag(self, tag):
        if tag == 'a' and self._href:
            self.links[''.join(self._text)] = self._href
            self._href = None


def lsg_pages(index_html):
    """Map each title listed in an lsg index.html to its page file."""
    parser = _IndexLinks()
    parser.feed(index_html)
    return parser.links


LIST_SRC = re.compile(r'data-list-src="([^"]+)"')


def lsg_list(list_json):
    """Map each title in an All pages data file to its page file.

    Rows are [title, route, kind, ...]; a null route is the usual lsg_route(title).
    """
    return {row[0]: row[1] or lsg_route(row[0]) for row in json.loads(list_json)}


def lsg_index(site):
    """Map normalized titles to (title, file) for an lsg site directory.

    pages.html holds only the table's first rows, so read every title from the data file it names.
    """
    site = Path(site)
    index = site / 'pages.html'
    if not index.exists():
        index = site / 'index.html'
    index_html = index.read_text(encoding='utf-8')
    pages = lsg_pages(index_html)
    source = LIST_SRC.search(index_html)
    if source and (site / html.unescape(source.group(1))).exists():
        pages.update(lsg_list((site / html.unescape(source.group(1))).read_text(encoding='utf-8')))
    return {normalize(t): (t, f) for t, f in pages.items()}


def lsg_route(title):
    """An lsg page's file: p-<FNV-1a 64 of the title's UTF-8>.html."""
    digest = 0xcbf29ce484222325
    for byte in title.encode('utf-8'):
        digest = ((digest ^ byte) * 0x100000001b3) % (1 << 64)
    return f'p-{digest:016x}.html'


# The right sidebar on tmux: a page card, a block card, a Block references card, then ten page cards.
SIDEBAR_PAGE = 'tmux'
SIDEBAR_PAGE_CARDS = 10
BLOCK_REF = re.compile(r'<a class="block-ref-link" href="([^"]*#block-[^"]+)"')
# Marks the controls to Shift-click with data-parity-sidebar: `block` (the first visible bullet),
# `refs` (a block reference link added to the page, since tmux has none) and `page-<n>` (distinct
# page links, the current page excluded). Takes [block reference href, page links wanted].
SIDEBAR_TARGETS = '''([refHref, wanted]) => {
  const main = document.querySelector('main');
  const shown = node => node.getClientRects().length > 0;
  const bullet = [...main.querySelectorAll('ul.outline .bullet')].find(shown);
  bullet?.setAttribute('data-parity-sidebar', 'block');
  if (refHref) {
    const link = document.createElement('a');
    link.className = 'block-ref-link';
    link.setAttribute('href', refHref);
    link.setAttribute('data-parity-sidebar', 'refs');
    link.textContent = 'Block reference';
    main.append(link);
  }
  const seen = new Set([location.pathname.split('/').pop()]);
  const pages = [];
  for (const link of main.querySelectorAll('a[href^="p-"]:not(.block-ref-link)')) {
    const href = link.getAttribute('href');
    if (href.includes('#') || seen.has(href) || !shown(link)) continue;
    seen.add(href);
    link.setAttribute('data-parity-sidebar', 'page-' + pages.length);
    pages.push(href);
    if (pages.length === wanted) break;
  }
  return {block: !!bullet, refs: !!refHref, pages};
}'''


def block_ref_href(site, first=()):
    """A block reference link's href from the site, as a page file and #block- fragment; the pages
    in `first` are searched before the rest."""
    site = Path(site)
    for path in [*(site / name for name in first), *sorted(site.glob('p-*.html'))]:
        if not path.exists():
            continue
        match = BLOCK_REF.search(path.read_text(encoding='utf-8'))
        if match:
            href = match.group(1)
            return path.name + href if href.startswith('#') else href
    return None
