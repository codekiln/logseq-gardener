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
        html = html.replace('href="lsg-search-manifest-', 'href="/lsg-search-manifest-').replace('src="lsg-search-ui-', 'src="/lsg-search-ui-').replace('src="lsg-search-start-', 'src="/lsg-search-start-')
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
                # One frame evaluates only the newest rapid input; close cancels it.
                completed = field.evaluate('node => { const list=document.querySelector("#lsg-search-results"); const before=Number(list.dataset.lsgSearchComplete); node.value="obsolete"; node.dispatchEvent(new Event("input", {bubbles:true})); node.value="needle"; node.dispatchEvent(new Event("input", {bubbles:true})); return before; }')
                page.wait_for_function('(before) => Number(document.querySelector("#lsg-search-results").dataset.lsgSearchComplete) > before', arg=completed)
                assert page.locator('#lsg-search-results').get_attribute('data-lsg-search-complete') == str(completed + 1)
                expect(results).to_have_count(4)
                                # IME candidates own composing keys, including legacy keyCode229.
                field.focus()
                composition = field.evaluate('''node => {
                  node.dispatchEvent(new CompositionEvent('compositionstart', {bubbles: true, data: 'に'}));
                  const cases = [
                    {key: 'Escape', isComposing: true},
                    {key: 'ArrowDown', isComposing: true},
                    {key: 'ArrowUp', keyCode: 229},
                    {key: 'Enter', isComposing: true}
                  ];
                  const outcomes = cases.map(options => {
                    const event = new KeyboardEvent('keydown', {bubbles: true, cancelable: true, ...options});
                    node.dispatchEvent(event);
                    return {prevented: event.defaultPrevented, focused: document.activeElement === node, open: document.querySelector('#lsg-search-dialog').open};
                  });
                  node.dispatchEvent(new CompositionEvent('compositionend', {bubbles: true, data: 'に'}));
                  return outcomes;
                }''')
                assert all(not state['prevented'] and state['focused'] and state['open'] for state in composition), composition
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
                # Corrupt corpus and offline opens stay actionable and recover on retry.
                context = browser.new_context()
                page = context.new_page()
                page.route('**/lsg-search-*.json', lambda route: route.continue_() if 'manifest-' in route.request.url else route.fulfill(status=200, body='{broken'))
                page.goto(base + '/index.html')
                page.locator('[data-lsg-search]').click()
                expect(page.locator('#lsg-search-status')).to_contain_text('unavailable')
                expect(page.locator('#lsg-search-results a')).to_have_count(0)
                page.unroute('**/lsg-search-*.json')
                page.get_by_role('button', name='Retry search').click()
                expect(page.locator('#lsg-search-dialog')).to_have_attribute('data-lsg-search-state', 'ready')
                context.close()
                context = browser.new_context()
                page = context.new_page()
                page.goto(base + '/index.html')
                page.locator('[data-lsg-search]').click()
                expect(page.locator('#lsg-search-dialog')).to_have_attribute('data-lsg-search-state', 'ready')
                page.keyboard.press('Escape')
                page.evaluate("const link=document.head.querySelector('[rel=lsg-search]');link.remove();window.logseqGardenerInitSearch();document.head.append(link);window.logseqGardenerInitSearch()")
                context.set_offline(True)
                page.locator('[data-lsg-search]').click()
                expect(page.locator('#lsg-search-status')).to_contain_text('unavailable')
                context.set_offline(False)
                page.get_by_role('button', name='Retry search').click()
                expect(page.locator('#lsg-search-dialog')).to_have_attribute('data-lsg-search-state', 'ready')
                context.close()
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
                # Missing post-paint starter leaves the page usable and explains recovery.
                context = browser.new_context()
                page = context.new_page()
                page.route('**/lsg-search-start-*.mjs', lambda route: route.fulfill(status=404, body='missing'))
                page.goto(base + '/index.html')
                expect(page.get_by_role('status')).to_contain_text('Reload this page to retry search')
                expect(page.locator('main h1')).to_be_visible()
                expect(page.locator('[data-lsg-search]')).not_to_be_visible()
                assert page.url == base + '/index.html'
                context.close()
                # Host init before and during the post-paint starter import survives handoff.
                context = browser.new_context()
                context.add_init_script('''() => {
                  let api, first = true;
                  Object.defineProperty(window, 'logseqGardenerInitSearch', {configurable:true,
                    get: () => api, set: value => {api=value;if(first){first=false;value(document,{resolvePage:()=>'/before-starter/'});}}
                  });
                }''')
                page = context.new_page()
                starter = []
                page.route('**/lsg-search-start-*.mjs', lambda route: starter.append(route))
                page.goto(base + '/index.html')
                page.wait_for_timeout(100)
                assert starter
                expect(page.locator('[data-lsg-search]')).not_to_be_visible()
                page.evaluate("window.logseqGardenerInitSearch(document,{resolvePage:()=>'/during-starter/'})")
                starter.pop().continue_()
                expect(page.locator('[data-lsg-search]')).to_be_visible()
                page.locator('[data-lsg-search]').click()
                page.locator('#lsg-search-input').fill('needle')
                expect(page.locator('#lsg-search-results a')).to_have_count(4)
                assert page.locator('#lsg-search-results a').first.get_attribute('href') == '/during-starter/'
                context.close()
                # Bootstrap module failure remains retryable; latest host options survive import.
                context = browser.new_context()
                page = context.new_page()
                page.route('**/lsg-search-ui-*.mjs*', lambda route: route.fulfill(status=404, body='missing'))
                page.goto(base + '/index.html')
                page.locator('[data-lsg-search]').click()
                expect(page.get_by_role('status')).to_contain_text('could not load')
                page.unroute('**/lsg-search-ui-*.mjs*')
                held = []
                page.route('**/lsg-search-ui-*.mjs*', lambda route: held.append(route))
                page.locator('[data-lsg-search]').click()
                page.wait_for_timeout(100)
                assert held, 'module retry did not request the controller'
                page.keyboard.type('gatt')
                page.evaluate("for(let i=0;i<600;i++)document.dispatchEvent(new KeyboardEvent('keydown',{key:'x',bubbles:true,cancelable:true}))")
                assert page.url == base + '/index.html'
                assert page.evaluate('window.logseqGardenerTheme.choice()') == 'system'
                page.evaluate("window.logseqGardenerInitSearch(document,{resolvePage:()=>'/readable-result/'})")
                held.pop().continue_()
                expect(page.locator('#lsg-search-input')).to_be_focused()
                expect(page.locator('#lsg-search-input')).to_have_value('gatt' + 'x' * 508)
                page.locator('#lsg-search-input').fill('needle')
                expect(page.locator('#lsg-search-results a')).to_have_count(4)
                assert page.locator('#lsg-search-results a').first.get_attribute('href') == '/readable-result/'
                context.close()
                # Offline before the first UI import is actionable; prepared reopening stays offline.
                context = browser.new_context()
                page = context.new_page()
                page.goto(base + '/index.html')
                expect(page.locator('[data-lsg-search]')).to_be_visible()
                context.set_offline(True)
                page.locator('[data-lsg-search]').click()
                expect(page.get_by_role('status')).to_contain_text('could not load')
                expect(page.locator('main h1')).to_be_visible()
                context.set_offline(False)
                page.locator('[data-lsg-search]').click()
                expect(page.locator('#lsg-search-dialog')).to_have_attribute('data-lsg-search-state', 'ready')
                page.keyboard.press('Escape')
                context.set_offline(True)
                page.locator('[data-lsg-search]').click()
                page.locator('#lsg-search-input').fill('needle')
                expect(page.locator('#lsg-search-results a')).to_have_count(4)
                context.close()
                # Escape restores ordinary page typing/Tab while the module remains held.
                context = browser.new_context()
                page = context.new_page()
                held = []
                page.route('**/lsg-search-ui-*.mjs*', lambda route: held.append(route))
                page.goto(base + '/index.html')
                page.evaluate("const input=document.createElement('input');input.id='page-input';const next=document.createElement('button');next.id='page-next';next.textContent='Next';document.body.append(input,next)")
                page.locator('#page-input').focus()
                page.keyboard.press('Control+k')
                page.wait_for_timeout(100)
                assert held
                page.keyboard.press('Escape')
                page.keyboard.type('ordinary')
                expect(page.locator('#page-input')).to_have_value('ordinary')
                page.keyboard.press('Tab')
                expect(page.locator('#page-next')).to_be_focused()
                held.pop().continue_()
                page.wait_for_timeout(100)
                assert page.locator('#lsg-search-dialog').count() == 0
                page.locator('[data-lsg-search]').click()
                expect(page.locator('#lsg-search-input')).to_be_focused()
                page.keyboard.press('Escape')
                expect(page.locator('[data-lsg-search]')).to_be_focused()
                context.close()
                # A missing publication during module loading cannot be re-enabled by handoff.
                context = browser.new_context()
                page = context.new_page()
                held = []
                page.route('**/lsg-search-ui-*.mjs*', lambda route: held.append(route))
                page.goto(base + '/index.html')
                page.locator('[data-lsg-search]').click()
                page.wait_for_timeout(100)
                assert held
                page.evaluate("document.head.querySelector('[rel=lsg-search]').remove();window.logseqGardenerInitSearch()")
                held.pop().continue_()
                page.wait_for_timeout(100)
                expect(page.locator('[data-lsg-search]')).not_to_be_visible()
                assert page.locator('#lsg-search-dialog').count() == 0
                context.close()
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
