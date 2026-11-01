#!/usr/bin/env python3
# /// script
# requires-python = ">=3.12"
# dependencies = ["playwright==1.60.0"]
# ///
"""Check a --readable-urls publication in Chromium, served under a path prefix and from disk.

Builds a small garden with nested pages, a title that needs percent-encoding, a block
reference, an asset, a journal and search, once without plugins and once with the fixture
recorder plugin. Under the prefix it follows page links, zooms into a block and back, opens a
search result, a Shift-click sidebar card, the page graph, All pages and the journals feed, and
moves between pages with the plugin host. From disk it follows page links into folders. Every
page load must finish without a failed request. Prints one JSON line of results.
"""
import argparse
import base64
import functools
import http.server
import json
from pathlib import Path
import subprocess
import tempfile
import threading
from playwright.sync_api import sync_playwright, expect

REPO = Path(__file__).resolve().parents[2]
BLOCK = '6730a1b2-0000-4000-8000-0000000000aa'
# A one-pixel PNG.
PNG = base64.b64decode('iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR4nGNgYGD4DwABBAEAwS2OUAAAAABJRU5ErkJggg==')
START = 'Docs/Guide/Start/'
ENCODED = 'Docs/Guide/Q%26A%3A%202026/'
FILES = {
    'logseq/config.edn': '{:file/name-format :triple-lowbar\n :publishing/all-pages-public? true\n :journal/page-title-format "yyyy-MM-dd EEE"}\n',
    'pages/Docs___Guide___Start.md': ('- Start here. See [[Docs/Guide/Q&A: 2026]] and [[Other]].\n'
                                      '\t- Nested child with ![logo](../assets/logo.png)\n'
                                      f'- A block with an id\n  id:: {BLOCK}\n'
                                      '- The unicornword lives on this page\n'),
    'pages/Docs___Guide___QA.md': 'title:: Docs/Guide/Q&A: 2026\n\n- Questions and answers. Back to [[Docs/Guide/Start]].\n',
    'pages/Other.md': f'- Other page quotes (({BLOCK})) and links [[Docs/Guide/Start]].\n',
    'journals/2026_10_23.md': '- A journal entry about [[Docs/Guide/Start]].\n',
}


def build(binary, root, name, extra=()):
    garden = root / 'garden'
    if not garden.exists():
        for path, text in FILES.items():
            (garden / path).parent.mkdir(parents=True, exist_ok=True)
            (garden / path).write_text(text, encoding='utf-8')
        (garden / 'assets').mkdir()
        (garden / 'assets/logo.png').write_bytes(PNG)
    output = root / name
    subprocess.run([str(binary), 'publish', '--graph', str(garden), '--output', str(output),
                    '--filename-format', 'triple-lowbar', '--include', 'Docs', '--include', 'Other',
                    '--journals', '--search', '--readable-urls', *extra],
                   check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
                   env={'SOURCE_DATE_EPOCH': '1793000000', 'PATH': '/usr/bin:/bin'})
    return output


class Quiet(http.server.SimpleHTTPRequestHandler):
    def log_message(self, *args):
        pass


def serve(root):
    server = http.server.ThreadingHTTPServer(('127.0.0.1', 0), functools.partial(Quiet, directory=str(root)))
    threading.Thread(target=server.serve_forever, daemon=True).start()
    return server, f'http://127.0.0.1:{server.server_address[1]}'


def watch(page):
    """Collect page errors and requests that failed or answered with an error status."""
    problems = []
    page.on('pageerror', lambda error: problems.append(f'error: {error}'))
    # Leaving a page cancels its pending requests; those are not failures.
    page.on('requestfailed', lambda request: 'ERR_ABORTED' not in (request.failure or '')
            and problems.append(f'failed: {request.url} {request.failure}'))
    page.on('response', lambda response: response.status >= 400 and problems.append(f'{response.status}: {response.url}'))
    return problems


def styled(page):
    """Whether the page's built-in styles and shell script loaded."""
    return page.evaluate('''() => getComputedStyle(document.querySelector('main')).maxWidth !== 'none'
      && !!window.logseqGardenerShell && !!document.querySelector('.lsg-header')''')


def h1(page):
    return page.locator('main h1').first.inner_text()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', default=str(REPO / 'target' / 'debug' / 'lsg'))
    args = parser.parse_args()
    binary = Path(args.binary).resolve()
    results = {}
    with tempfile.TemporaryDirectory(prefix='lsg-readable-') as temp:
        temp = Path(temp)
        site = build(binary, temp, 'logseq-gardener')
        hosted = build(binary, temp, 'hosted', ['--plugin', str(REPO / 'tests/fixtures/plugins/badge/lsg-plugin.json')])
        server, base = serve(temp)
        prefix = f'{base}/{site.name}/'
        try:
            with sync_playwright() as playwright:
                browser = playwright.chromium.launch()

                # Under a path prefix: page links, assets, styles and scripts resolve from a folder.
                context = browser.new_context(viewport={'width': 1280, 'height': 800})
                page = context.new_page()
                problems = watch(page)
                page.goto(prefix + START, wait_until='load')
                assert styled(page)
                assert page.evaluate("document.querySelector('main img').naturalWidth") == 1
                assert page.evaluate('document.baseURI') == prefix
                page.locator(f'main a[href="{ENCODED}"]').first.click()
                page.wait_for_url(prefix + ENCODED)
                assert h1(page) == 'Docs/Guide/Q&A: 2026' and styled(page)
                page.locator(f'main a[href="{START}"]').first.click()
                page.wait_for_url(prefix + START)
                results['prefix_links'] = True

                # A bullet zooms within the page; the breadcrumb returns to the whole page.
                page.wait_for_function('!!window.logseqGardenerInitOutline')
                page.locator('main ul.outline > li > a.bullet').first.click()
                page.wait_for_url(prefix + START + '#zoom/1')
                expect(page.locator('.lsg-zoom')).to_be_visible()
                page.locator('.lsg-zoom .lsg-breadcrumb a').first.click()
                page.wait_for_url(prefix + START)
                expect(page.locator('.lsg-zoom')).to_have_count(0)
                results['zoom'] = True

                # A search result opens the page's folder at the block.
                page.locator('[data-lsg-search]').click()
                page.locator('#lsg-search-input').fill('unicornword')
                result = page.locator('#lsg-search-results a.lsg-search-row').first
                expect(result).to_be_visible()
                href = result.get_attribute('href')
                assert href.startswith(prefix + START + '#'), href
                page.keyboard.press('Enter')
                page.wait_for_url(href)
                results['search'] = href[len(prefix):]

                # Shift-click on a page link adds its card to the right sidebar.
                page.goto(prefix + 'Other/', wait_until='load')
                page.wait_for_function('!!window.logseqGardenerInitOutline')
                page.locator(f'main a[href="{START}"]').first.click(modifiers=['Shift'])
                card = page.locator('.lsg-right-sidebar .lsg-card').first
                expect(card).to_contain_text('Docs/Guide/Start')
                expect(card).to_contain_text('unicornword')
                results['sidebar'] = True

                # The page graph draws the current page from its neighbourhood file.
                assert problems == [], problems
                context.close()
                context = browser.new_context(viewport={'width': 1280, 'height': 800})
                page = context.new_page()
                problems = watch(page)
                page.goto(prefix + START, wait_until='load')
                page.keyboard.press('t')
                page.keyboard.press('r')
                page.locator('[data-add="graph"]').click()
                status = page.locator('.lsg-page-graph-canvas .lsg-graph-status')
                page.wait_for_function("() => [...performance.getEntriesByType('resource')].some(e => e.name.includes('/_graph/p/'))")
                page.wait_for_timeout(500)
                assert status.inner_text() == '', status.inner_text()
                results['page_graph'] = True

                # A graph node's route is the page's folder.
                page.goto(prefix + 'graph.html', wait_until='load')
                page.wait_for_function('!!window.logseqGardenerGraph')
                assert page.evaluate("window.logseqGardenerGraph.routeOf('Docs/Guide/Q&A: 2026')") == ENCODED
                results['graph_route'] = True

                # All pages and the journals feed link into folders.
                page.goto(prefix + 'pages.html', wait_until='load')
                page.locator(f'main a[href="{ENCODED}"]').first.click()
                page.wait_for_url(prefix + ENCODED)
                page.goto(prefix + 'index.html', wait_until='load')
                page.locator('main a.journal-title').first.click()
                page.wait_for_url(prefix + '2026-10-23%20Fri/')
                assert styled(page)
                results['lists'] = True
                assert problems == [], problems
                context.close()

                # With a plugin, the host adopts pages in folders and keeps the base at the root.
                context = browser.new_context(viewport={'width': 1280, 'height': 800})
                page = context.new_page()
                problems = watch(page)
                root = f'{base}/{hosted.name}/'
                page.goto(root + 'index.html', wait_until='load')
                page.wait_for_function('!!window.logseqGardenerPage')
                page.evaluate('window.__kept = true')
                for target in [START, 'Other/', ENCODED]:
                    before = page.evaluate("performance.getEntriesByName('lsg-host-adopted').length")
                    page.locator(f'a[href="{target}"]').first.click()
                    page.wait_for_function("(n) => performance.getEntriesByName('lsg-host-adopted').length > n", arg=before)
                    assert page.url == root + target, page.url
                    assert page.evaluate('window.__kept === true && document.baseURI') == root
                    assert page.evaluate("document.querySelectorAll('base').length") == 1
                page.go_back()
                page.wait_for_url(root + 'Other/')
                expect(page.locator('main h1')).to_have_text('Other')
                assert page.evaluate("document.querySelector('main a.page-ref').href").startswith(root + 'Docs/')
                results['host'] = True
                assert problems == [], problems
                context.close()

                # From disk, a link to a folder opens its index.html.
                context = browser.new_context(viewport={'width': 1280, 'height': 800})
                page = context.new_page()
                problems = watch(page)
                disk = site.as_uri() + '/'
                page.goto(disk + 'index.html', wait_until='load')
                page.locator('main a.journal-title').first.click()
                page.wait_for_url(disk + '2026-10-23%20Fri/index.html')
                page.locator(f'main a[href="{START}"]').first.click()
                page.wait_for_url(disk + START + 'index.html')
                assert styled(page) and h1(page) == 'Docs/Guide/Start'
                page.wait_for_function('!!window.logseqGardenerInitOutline')
                page.locator('main ul.outline > li > a.bullet').first.click()
                page.wait_for_url(disk + START + 'index.html#zoom/1')
                page.locator(f'main a[href="{ENCODED}"]').first.click()
                page.wait_for_url(disk + ENCODED + 'index.html')
                results['disk'] = True
                # Browsers refuse module scripts and fetch from disk in either layout, which
                # turns search and loaded sections off; everything else must load.
                problems = [p for p in problems if not p.endswith('.mjs net::ERR_FAILED') and p != 'error: Failed to fetch']
                assert problems == [], problems
                context.close()
                browser.close()
        finally:
            server.shutdown()
    print(json.dumps(results))


if __name__ == '__main__':
    main()
