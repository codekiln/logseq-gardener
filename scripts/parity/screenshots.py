#!/usr/bin/env python3
# /// script
# requires-python = ">=3.12"
# dependencies = ["playwright==1.60.0"]
# ///
"""Save full-page screenshots of the fixture pages from both builds, side by side."""
import argparse
import html
import json
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import common  # noqa: E402
import logseq_db  # noqa: E402
import serve  # noqa: E402

VIEWPORTS = {'desktop': {'width': 1280, 'height': 800}, 'phone': {'width': 390, 'height': 844}}
TITLE_TIMEOUT_MS = 60_000


def scroll_to_bottom(page):
    """Scroll in steps until the page height stops growing, so lazy sections render."""
    height, stable = 0, 0
    while stable < 3:
        page.mouse.wheel(0, page.viewport_size['height'])
        page.wait_for_timeout(250)
        current = page.evaluate('''() => {
            const main = document.querySelector('#main-content-container');
            if (main) main.scrollTop = main.scrollHeight;
            window.scrollTo(0, document.documentElement.scrollHeight);
            return Math.max(document.documentElement.scrollHeight, main ? main.scrollHeight : 0);
        }''')
        stable = stable + 1 if current == height else 0
        height = current
    page.evaluate('''() => {
        const main = document.querySelector('#main-content-container');
        if (main) main.scrollTop = 0;
        window.scrollTo(0, 0);
    }''')


def capture(page, url, selector, path):
    try:
        page.goto(url)
        page.wait_for_selector(selector, state='visible', timeout=TITLE_TIMEOUT_MS)
    except Exception as error:  # Playwright raises its own TimeoutError type.
        return f'title did not appear: {type(error).__name__}'
    page.wait_for_timeout(500)
    scroll_to_bottom(page)
    # Logseq scrolls inside #main-content-container; expand it so the capture holds the whole page.
    page.evaluate('''() => {
        for (const el of document.querySelectorAll('#main-content-container, #app-container, #root, .cp__sidebar-main-layout')) {
            el.style.height = 'auto'; el.style.maxHeight = 'none'; el.style.overflow = 'visible';
        }
    }''')
    page.screenshot(path=str(path), full_page=True)
    return None


SLIDES_PAGE = 'AI/ES/25/ws/What I Learned at the AI Engineer Summit Workshops 2025'


def capture_slides(browser, site_url, route, out):
    """Capture lsg's View as slides deck on its first slide at 1280 by 800, and in fullscreen."""
    context = browser.new_context(viewport=VIEWPORTS['desktop'], color_scheme='light')
    page = context.new_page()
    try:
        page.goto(site_url + route)
        page.click('[data-lsg-more]')
        page.click('[data-lsg-slides]')
        page.wait_for_selector('.reveal .slides>section.present', timeout=TITLE_TIMEOUT_MS)
        page.wait_for_timeout(1000)
        page.screenshot(path=str(out / 'slides-desktop-lsg.png'))
        page.keyboard.press('f')
        page.wait_for_function('document.fullscreenElement', timeout=5000)
        page.wait_for_timeout(1000)
        page.screenshot(path=str(out / 'slides-fullscreen-lsg.png'))
        return ['slides-desktop-lsg.png', 'slides-fullscreen-lsg.png']
    finally:
        context.close()


def capture_sidebar(browser, site_url, site, route, out):
    """Capture lsg's tmux at 1280 by 800 with a page card, a block card and a Block references card open."""
    context = browser.new_context(viewport=VIEWPORTS['desktop'], color_scheme='light')
    page = context.new_page()
    try:
        page.goto(site_url + route)
        page.wait_for_selector('h1', timeout=TITLE_TIMEOUT_MS)
        found = page.evaluate(common.SIDEBAR_TARGETS, [common.block_ref_href(site, [route]), 1])
        marks = [mark for mark, present in (('page-0', found['pages']), ('block', found['block']),
                                            ('refs', found['refs'])) if present]
        for count, mark in enumerate(marks, 1):
            page.click(f'[data-parity-sidebar="{mark}"]', modifiers=['Shift'])
            page.wait_for_function(f"document.querySelectorAll('.lsg-right-cards .lsg-card-page').length >= {count}",
                                   timeout=TITLE_TIMEOUT_MS)
        # Clicking the added block reference scrolled the page to its end.
        page.evaluate('''() => {
            document.querySelector('[data-parity-sidebar=refs]')?.remove();
            for (let node = document.querySelector('main'); node; node = node.parentElement) node.scrollTop = 0;
            window.scrollTo(0, 0);
        }''')
        page.wait_for_timeout(500)
        page.screenshot(path=str(out / 'sidebar-desktop-lsg.png'))
        return ['sidebar-desktop-lsg.png']
    finally:
        context.close()


def index_page(rows):
    parts = ['<!doctype html><meta charset="utf-8"><title>Parity screenshots</title>',
             '<style>body{font-family:system-ui;margin:16px}section{margin-bottom:48px}'
             '.pair{display:grid;grid-template-columns:1fr 1fr;gap:16px;align-items:start}'
             'img{width:100%;border:1px solid #ccc}</style><h1>Parity screenshots</h1>']
    for row in rows:
        parts.append(f'<section><h2>{html.escape(row["name"])}</h2><div class="pair">')
        for build in ('logseq', 'lsg'):
            label = 'Logseq' if build == 'logseq' else 'lsg'
            if row[build].get('image'):
                parts.append(f'<figure><figcaption>{label}</figcaption><img src="{row[build]["image"]}"></figure>')
            else:
                parts.append(f'<figure><figcaption>{label}</figcaption><p>{html.escape(row[build]["note"])}</p></figure>')
        parts.append('</div></section>')
    return '\n'.join(parts) + '\n'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--run', help='run directory from parity:build and parity:compare (default: the latest)')
    parser.add_argument('--fixtures', default=str(serve.FIXTURES), help='fixture list, one title per line')
    parser.add_argument('--viewport', choices=[*VIEWPORTS, 'all'], default='all',
                        help='desktop (1280 by 800), phone (390 by 844) or all (default)')
    parser.add_argument('--slides', action='store_true',
                        help=f'capture only the View as slides deck of {SLIDES_PAGE} in the lsg build')
    parser.add_argument('--sidebar', action='store_true',
                        help=f'capture only {common.SIDEBAR_PAGE} with a page, block and Block references card open in the lsg build')
    args = parser.parse_args()
    from playwright.sync_api import sync_playwright

    run = common.latest_run(args.run)
    if args.slides:
        out = run / 'screenshots'
        out.mkdir(exist_ok=True)
        route = common.lsg_route(SLIDES_PAGE)
        with serve.Server(run / 'site') as lsg, sync_playwright() as playwright:
            browser = playwright.chromium.launch()
            for image in capture_slides(browser, lsg.url, route, out):
                print(out / image)
            browser.close()
        return
    if args.sidebar:
        out = run / 'screenshots'
        out.mkdir(exist_ok=True)
        with serve.Server(run / 'site') as lsg, sync_playwright() as playwright:
            browser = playwright.chromium.launch()
            for image in capture_sidebar(browser, lsg.url, run / 'site', common.lsg_route(common.SIDEBAR_PAGE), out):
                print(out / image)
            browser.close()
        return
    logseq_dir = run / 'logseq'
    if not logseq_dir.exists():
        raise SystemExit('Logseq build missing; run `mise run parity:compare` first')
    names = {common.normalize(p.name) for p in logseq_db.read_pages(logseq_dir / 'static/js/db.gz')}
    lsg_index = common.lsg_index(run / 'site')
    out = run / 'screenshots'
    out.mkdir(exist_ok=True)
    rows = []
    with serve.Server(logseq_dir) as logseq, serve.Server(run / 'site') as lsg, sync_playwright() as playwright:
        browser = playwright.chromium.launch()
        viewports = VIEWPORTS if args.viewport == 'all' else {args.viewport: VIEWPORTS[args.viewport]}
        for label, viewport in viewports.items():
            context = browser.new_context(viewport=viewport, color_scheme='light')
            page = context.new_page()
            for number, name in enumerate(serve.fixtures(args.fixtures), 1):
                target = serve.targets(name, names, lsg_index, run / 'site')
                row = {'name': f'{name} ({label}, {viewport["width"]} by {viewport["height"]})'}
                for build, server in (('logseq', logseq), ('lsg', lsg)):
                    if target[build] is None:
                        row[build] = {'note': f'The {build} build lacks this page.'}
                        continue
                    image = f'{number:02d}-{label}-{build}.png'
                    error = capture(page, server.url + target[build], serve.TITLE_SELECTORS[build], out / image)
                    row[build] = {'note': error} if error else {'image': image}
                    print(f'{name} [{label} {build}]: {error or image}')
                rows.append(row)
            context.close()
        browser.close()
    (out / 'index.html').write_text(index_page(rows), encoding='utf-8')
    (out / 'screenshots.json').write_text(json.dumps(rows, indent=2, ensure_ascii=False) + '\n', encoding='utf-8')
    print(f"Screenshots: {out / 'index.html'}")


if __name__ == '__main__':
    main()
