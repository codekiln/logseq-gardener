"""Serve a build on 127.0.0.1 and resolve fixture pages to URLs in both builds."""
import functools
import gzip
import http.server
import threading
from pathlib import Path

import common

FIXTURES = Path(__file__).resolve().parent / 'fixtures.txt'
LOGSEQ_ROUTES = {'Home': '#/', 'All pages': '#/all-pages'}
LSG_ROUTES = {'Home': 'index.html', 'All pages': 'pages.html'}
TITLE_SELECTORS = {
    'logseq': '.page-title, .journal-title, h1.title',
    'lsg': 'h1',
}
COMPRESSED_TYPES = ('text/', 'application/javascript', 'application/json', 'image/svg+xml')


def fixtures(path=FIXTURES):
    lines = Path(path).read_text(encoding='utf-8').splitlines()
    return [line.strip() for line in lines if line.strip() and not line.startswith('#')]


class _Handler(http.server.SimpleHTTPRequestHandler):
    """Send text files gzip-compressed, as GitHub Pages does."""

    cache = {}

    def log_message(self, *args):
        pass

    def send_head(self):
        path = Path(self.translate_path(self.path))
        content_type = self.guess_type(str(path))
        wants_gzip = 'gzip' in self.headers.get('Accept-Encoding', '')
        if not (path.is_file() and wants_gzip and content_type.startswith(COMPRESSED_TYPES)):
            return super().send_head()
        body = self.cache.get(path)
        if body is None:
            body = self.cache[path] = gzip.compress(path.read_bytes(), 6)
        self.send_response(200)
        self.send_header('Content-Type', content_type)
        self.send_header('Content-Encoding', 'gzip')
        self.send_header('Content-Length', str(len(body)))
        self.send_header('Cache-Control', 'no-store')
        self.end_headers()
        self.wfile.write(body)
        return None


class Server:
    """A loopback HTTP server for one build directory, stopped on exit."""

    def __init__(self, directory):
        handler = functools.partial(type('Handler', (_Handler,), {'cache': {}}), directory=str(directory))
        self.httpd = http.server.ThreadingHTTPServer(('127.0.0.1', 0), handler)
        self.url = f'http://127.0.0.1:{self.httpd.server_address[1]}/'
        self.thread = threading.Thread(target=self.httpd.serve_forever, daemon=True)

    def __enter__(self):
        self.thread.start()
        return self

    def __exit__(self, *exc):
        self.httpd.shutdown()
        self.httpd.server_close()


def targets(name, logseq_names, lsg_index, site=None):
    """Return {'logseq': route or None, 'lsg': file or None} for a fixture name.

    All pages is `pages.html` when the lsg build has one (a journals feed or a homepage
    holds `index.html`), otherwise `index.html`.
    """
    if name in LOGSEQ_ROUTES:
        lsg = LSG_ROUTES[name]
        if site is not None and not (Path(site) / lsg).exists():
            lsg = 'index.html'
        return {'logseq': LOGSEQ_ROUTES[name], 'lsg': lsg}
    key = common.normalize(name)
    lsg = lsg_index.get(key)
    return {
        'logseq': common.logseq_route(name) if key in logseq_names else None,
        'lsg': lsg[1] if lsg else None,
    }
