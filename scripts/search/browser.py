#!/usr/bin/env python3
# /// script
# requires-python = ">=3.12"
# dependencies = ["playwright==1.60.0"]
# ///
"""Functional search proof against an actual generated static publication."""
import argparse
import functools
import http.server
import json
from pathlib import Path
import subprocess
import tempfile
import threading
from playwright.sync_api import sync_playwright, expect


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    args = parser.parse_args()
    with tempfile.TemporaryDirectory(prefix='lsg-search-browser-') as temp:
        root = Path(temp)
        pages = root / 'garden/pages'
        pages.mkdir(parents=True)
        (pages / 'Notes.md').write_text('- Parent\n  collapsed:: true\n\t- needle child `safe <img src=x onerror=alert(1)>`\n- Other needle result\n- generated placeholder\n  search-exclude:: true\n')
        (pages / 'Notes___Needle.md').write_text('alias:: Find me\n- target needle content\n')
        (pages / 'Notes___Private.md').write_text('public:: false\n- PRIVATE_SENTINEL\n')
        site = root / 'site'
        subprocess.run([str(args.binary.resolve()), 'publish', '--graph', str(root / 'garden'), '--output', str(site), '--filename-format', 'triple-lowbar', '--include', 'Notes', '--search'], check=True, capture_output=True)
        nested = site / 'nested/page'
        nested.mkdir(parents=True)
        html = (site / 'index.html').read_text()
        html = html.replace('href="lsg-search-manifest-', 'href="/lsg-search-manifest-').replace('src="lsg-search-ui-', 'src="/lsg-search-ui-')
        (nested / 'index.html').write_text(html)
        class Quiet(http.server.SimpleHTTPRequestHandler):
            def log_message(self, *_):
                pass
        server = http.server.ThreadingHTTPServer(('127.0.0.1', 0), functools.partial(Quiet, directory=site))
        thread = threading.Thread(target=server.serve_forever, daemon=True)
        thread.start()
        base = f'http://127.0.0.1:{server.server_port}'
        try:
            with sync_playwright() as playwright:
                browser = playwright.chromium.launch()
                context = browser.new_context(viewport={'width': 1280, 'height': 800})
                page = context.new_page()
                errors = []
                page.on('pageerror', lambda error: errors.append(str(error)))
                requests = []
                page.on('request', lambda request: requests.append(request.url))
                page.goto(base + '/index.html')
                trigger = page.locator('[data-lsg-search]')
                expect(trigger).to_be_visible()
                assert not any('lsg-search-manifest-' in url or 'lsg-search-engine-' in url or ('lsg-search-' in url and url.endswith('.json')) for url in requests)
                trigger.focus()
                page.keyboard.press('Control+k')
                field = page.locator('#lsg-search-input')
                expect(field).to_be_focused()
                field.fill('needle')
                results = page.locator('#lsg-search-results a')
                expect(results).to_have_count(4)
                assert page.locator('#lsg-search-results img').count() == 0
                assert 'safe <img' in page.locator('#lsg-search-results').inner_text()
                assert 'PRIVATE_SENTINEL' not in page.locator('#lsg-search-results').inner_text()
                page.keyboard.press('ArrowDown')
                expect(results.nth(0)).to_be_focused()
                page.keyboard.press('ArrowDown')
                expect(results.nth(1)).to_be_focused()
                page.keyboard.press('ArrowUp')
                expect(results.nth(0)).to_be_focused()
                page.keyboard.press('g')
                page.keyboard.press('a')
                expect(page.locator('#lsg-search-dialog')).to_be_visible()
                page.keyboard.press('Control+k')
                expect(field).to_be_focused()
                page.keyboard.press('Escape')
                expect(page.locator('#lsg-search-dialog')).not_to_be_visible()
                expect(trigger).to_be_focused()
                before = len(requests)
                trigger.click()
                expect(results).to_have_count(4)
                assert not any('lsg-search-' in url for url in requests[before:]), 'cached reopen fetched data'
                field.fill('no-match-at-all')
                expect(results).to_have_count(0)
                expect(page.locator('#lsg-search-status')).to_have_text('No results.')
                field.fill('needle')
                expect(results).to_have_count(4)
                # Shared appearance variables follow dark/system and mobile geometry.
                page.evaluate("window.logseqGardenerTheme.set('dark')")
                dark = page.locator('#lsg-search-dialog').evaluate('node => getComputedStyle(node).backgroundColor')
                page.evaluate("window.logseqGardenerTheme.set('light')")
                light = page.locator('#lsg-search-dialog').evaluate('node => getComputedStyle(node).backgroundColor')
                assert light != dark
                page.emulate_media(color_scheme='dark')
                page.evaluate("window.logseqGardenerTheme.set('system')")
                assert page.locator('#lsg-search-dialog').evaluate('node => getComputedStyle(node).backgroundColor') == dark
                page.set_viewport_size({'width': 390, 'height': 844})
                bounds = page.locator('#lsg-search-dialog').bounding_box()
                assert bounds['x'] >= 0 and bounds['x'] + bounds['width'] <= 390
                # Native Tab stays within the modal; modified activation keeps it open.
                field.focus()
                page.keyboard.press('Shift+Tab')
                expect(page.get_by_role('button', name='Close search')).to_be_focused()
                page.keyboard.press('Tab')
                expect(field).to_be_focused()
                with context.expect_page() as popup_info:
                    results.nth(0).click(modifiers=['ControlOrMeta'])
                popup_info.value.close()
                expect(page.locator('#lsg-search-dialog')).to_be_visible()
                # Host delegates ordinary links after the controller closes, with persistent state.
                page.evaluate('''() => {
                  window.persistedState = {playhead: 123};
                  window.logseqGardenerInitSearch(document, {resolvePage: (_title, href) => new URL(href, location.href).href});
                  window.logseqGardenerInitSearch();
                  document.addEventListener('click', async event => {
                    const link = event.target.closest('#lsg-search-results a');
                    if (!link || event.ctrlKey || event.metaKey) return;
                    event.preventDefault();
                    window.wasClosedBeforeHost = !document.querySelector('#lsg-search-dialog').open;
                    const html = await (await fetch(link.href)).text();
                    const incoming = new DOMParser().parseFromString(html, 'text/html');
                    document.querySelector('main').replaceWith(incoming.querySelector('main'));
                    document.head.querySelector('[rel=lsg-search]').replaceWith(incoming.head.querySelector('[rel=lsg-search]'));
                    window.logseqGardenerInitSearch(document);
                    window.hostDone = true;
                  });
                }''')
                field.focus()
                page.keyboard.press('Enter')
                page.wait_for_function('window.hostDone')
                assert page.evaluate('window.wasClosedBeforeHost && window.persistedState.playhead === 123')
                assert page.locator('#lsg-search-dialog').count() == 1
                trigger.click()
                expect(field).to_be_focused()
                print('search proof: SPA complete', flush=True)
                # A changed identity closes the dialog and cannot reuse old results.
                page.evaluate("""const metadata=document.head.querySelector('[rel=lsg-search]'); metadata.href=new URL('lsg-search-manifest-fedcba9876543210.json', metadata.href); metadata.dataset.revision='fedcba9876543210'; window.logseqGardenerInitSearch()""")
                expect(page.locator('#lsg-search-dialog')).not_to_be_visible()
                trigger.click()
                expect(page.locator('#lsg-search-status')).to_contain_text('unavailable')
                expect(page.locator('#lsg-search-results a')).to_have_count(0)
                # Missing identity hides unusable controls.
                page.evaluate("document.head.querySelector('[rel=lsg-search]').remove(); window.logseqGardenerInitSearch()")
                expect(trigger).not_to_be_visible()
                expect(page.locator('#lsg-search-dialog')).not_to_be_visible()
                assert errors == [], errors
                context.close()
                # The default result route uses the manifest directory from a nested page.
                context = browser.new_context()
                page = context.new_page()
                page.goto(base + '/nested/page/index.html')
                page.locator('[data-lsg-search]').click()
                page.locator('#lsg-search-input').fill('needle')
                expect(page.locator('#lsg-search-results a')).to_have_count(4)
                destination = page.locator('#lsg-search-results a').first.get_attribute('href')
                assert destination.startswith(base + '/p-') and '/nested/' not in destination
                context.close()
                # A failed load is actionable, never an empty successful search or automatic reload.
                context = browser.new_context()
                page = context.new_page()
                page.route('**/lsg-search-manifest-*.json', lambda route: route.fulfill(status=404, body='missing'))
                page.goto(base + '/index.html')
                page.locator('[data-lsg-search]').click()
                expect(page.locator('#lsg-search-status')).to_contain_text('unavailable')
                expect(page.get_by_role('button', name='Retry search')).to_be_visible()
                page.unroute('**/lsg-search-manifest-*.json')
                page.get_by_role('button', name='Retry search').click()
                page.locator('#lsg-search-input').fill('needle')
                expect(page.locator('#lsg-search-results a')).to_have_count(4)
                context.close()
                print('search proof: retry complete', flush=True)
                # Closing during a deferred first fetch cannot publish stale results.
                context = browser.new_context()
                page = context.new_page()
                page.goto(base + '/index.html')
                page.evaluate("""() => { window.originalFetch=window.fetch; window.fetch=(...args)=>new Promise(resolve=>{window.releaseSearch=()=>resolve(window.originalFetch(...args));}); }""")
                page.locator('[data-lsg-search]').click()
                page.wait_for_function('Boolean(window.releaseSearch)')
                page.keyboard.press('Escape')
                page.evaluate('window.fetch=window.originalFetch; window.releaseSearch()')
                page.locator('[data-lsg-search]').click()
                page.locator('#lsg-search-input').fill('needle')
                expect(page.locator('#lsg-search-results a')).to_have_count(4)
                expect(page.locator('#lsg-search-status')).not_to_contain_text('unavailable')
                context.close()
                print('search proof: stale close complete', flush=True)
                nojs = browser.new_context(java_script_enabled=False)
                page = nojs.new_page()
                page.goto(base + '/index.html')
                expect(page.locator('[data-lsg-search]')).not_to_be_visible()
                expect(page.locator('[data-lsg-all-pages]')).to_have_attribute('href', 'index.html')
                nojs.close()
                browser.close()
            print(json.dumps({'search_browser': 'passed', 'keyboard': True, 'mobile_themes': True, 'lazy_cache': True, 'spa_persistence': True, 'failure_retry': True, 'no_javascript': True}))
        finally:
            server.shutdown()
            server.server_close()


if __name__ == '__main__':
    main()
