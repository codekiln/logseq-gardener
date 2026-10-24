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
    args = parser.parse_args()
    from playwright.sync_api import sync_playwright

    run = common.latest_run(args.run)
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
                target = serve.targets(name, names, lsg_index)
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
