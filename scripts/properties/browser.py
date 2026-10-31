#!/usr/bin/env python3
# /// script
# requires-python = ">=3.12"
# dependencies = ["playwright==1.60.0"]
# ///
"""Properties preference proof on generated pages and host replacements."""
import argparse
import functools
import http.server
import json
from pathlib import Path
import subprocess
import tempfile
import threading
import sys

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "parity"))
from budgets import _BlockingResources
from playwright.sync_api import sync_playwright, expect


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    args = parser.parse_args()
    with tempfile.TemporaryDirectory(prefix='lsg-properties-') as temporary:
        root = Path(temporary)
        for name, collapsed in [('open', 'false'), ('closed', 'true')]:
            graph = root / (name + '-graph')
            (graph / 'pages').mkdir(parents=True)
            (graph / 'logseq').mkdir()
            (graph / 'logseq/config.edn').write_text('{:publishing/page-properties? true :publishing/page-properties-collapsed? ' + collapsed + '}')
            (graph / 'pages/Notes.md').write_text('- Page without properties\n')
            (graph / 'pages/Notes___One.md').write_text('rating:: 4\nsecret:: [[Private]]\n- Original content\n\t- Child content\n')
            (graph / 'pages/Notes___Two.md').write_text('rating:: 5\n- More content\n')
            before = {p: p.read_bytes() for p in graph.rglob('*.md')}
            subprocess.run([str(args.binary.resolve()), 'publish', '--graph', str(graph), '--output', str(root / name), '--filename-format', 'triple-lowbar', '--include', 'Notes', '--search'], check=True, capture_output=True)
            assert all(p.read_bytes() == content for p, content in before.items())
        # Count every initial script on the fixture's ordinary reading pages.
        # Graph and All pages have their own controller budgets in parity:budgets.
        sizes = []
        for path in root.glob('*/*.html'):
            if '<title>Notes' not in path.read_text():
                continue
            resources = _BlockingResources(all_scripts=True)
            resources.feed(path.read_text())
            size = resources.inline_script + sum((path.parent / ref).stat().st_size for ref in resources.files if not ref.endswith('.css'))
            sizes.append(size)
        assert len(sizes) == 6, sizes
        assert max(sizes) <= 16000, max(sizes)
        print(f'Enabled Properties + Search initial script: {max(sizes):,} (limit 16,000)')
        def route(base, title):
            for path in (root / base).glob('*.html'):
                if '<title>' + title + '</title>' in path.read_text():
                    return '/' + base + '/' + path.name
            raise AssertionError(title)
        one = route('open', 'Notes/One')
        two = route('open', 'Notes/Two')
        empty = route('open', 'Notes')
        closed = route('closed', 'Notes/One')
        # Observe the state immediately after the generated parse-time restoration.
        for path in root.glob('*/*.html'):
            text = path.read_text()
            text = text.replace('</script><summary id="page-properties"', '</script><script>window.parsedOpen=document.currentScript.parentElement.open</script><summary id="page-properties"')
            path.write_text(text)
        # A readable host route must retain the site asset directory as its identity.
        nested = root / 'open/readable/deep'
        nested.mkdir(parents=True)
        text = (root / one.lstrip('/')).read_text()
        text = text.replace('href="lsg-properties-', 'href="/open/lsg-properties-')
        (nested / 'index.html').write_text(text)
        class Quiet(http.server.SimpleHTTPRequestHandler):
            def log_message(self, *_):
                pass
            def copyfile(self, source, outputfile):
                try:
                    super().copyfile(source, outputfile)
                except (BrokenPipeError, ConnectionResetError):
                    pass
        server = http.server.ThreadingHTTPServer(('127.0.0.1', 0), functools.partial(Quiet, directory=root))
        threading.Thread(target=server.serve_forever, daemon=True).start()
        base = f'http://127.0.0.1:{server.server_port}'
        key = 'lsg:/open/'
        try:
            with sync_playwright() as playwright:
                browser = playwright.chromium.launch()
                context = browser.new_context()
                page = context.new_page()
                errors = []
                page.on('pageerror', lambda error: errors.append(str(error)))
                details = page.locator('.page-properties > details')
                summary = page.locator('#page-properties')
                page.goto(base + one)
                page.wait_for_function('!window.logseqGardenerInitProperties.k')
                expect(details).to_have_attribute('open', '')
                assert page.evaluate('parsedOpen') is True
                assert page.locator('.page-properties').inner_text().find('secret') < 0
                summary.click()
                expect(details).not_to_have_attribute('open', '')
                page.wait_for_function('(key) => JSON.parse(localStorage.getItem(key)).open === false', arg=key)
                page.goto(base + two)
                assert page.evaluate('parsedOpen') is False
                expect(details).not_to_have_attribute('open', '')
                page.wait_for_function('!window.logseqGardenerInitProperties.k')
                summary.press('Enter')
                page.wait_for_function('(key) => JSON.parse(localStorage.getItem(key)).open === true', arg=key)
                page.goto(base + one)
                assert page.evaluate('parsedOpen') is True
                # Rapid activation stores the resulting last activation, not a later DOM sample.
                page.wait_for_function('!window.logseqGardenerInitProperties.k')
                summary.evaluate('node => {node.click(); node.click()}')
                page.wait_for_timeout(20)
                assert page.evaluate('(key) => JSON.parse(localStorage.getItem(key)).open', key) is True
                # Cross-base-path and readable nested routes.
                page.evaluate('(key) => localStorage.setItem(key, JSON.stringify({open:false,seen:Date.now()}))', key)
                page.goto(base + closed)
                assert page.evaluate('parsedOpen') is False
                page.evaluate("localStorage.setItem('lsg:/closed/',JSON.stringify({open:true,seen:Date.now()}))")
                page.goto(base + closed)
                assert page.evaluate('parsedOpen') is True
                page.goto(base + '/open/readable/deep/index.html')
                assert page.evaluate('parsedOpen') is False
                page.goto(base + one)
                page.wait_for_function('!window.logseqGardenerInitProperties.k')
                # A host can begin on a page without a Properties section.
                page.goto(base + empty)
                page.wait_for_function('!window.logseqGardenerInitProperties.k')
                def replace(path, fragment=False):
                    page.evaluate("""async ([path, fragment]) => {
                      const doc=new DOMParser().parseFromString(await (await fetch(path)).text(),'text/html');
                      const main=document.importNode(doc.querySelector('main'),true);
                      main.querySelectorAll('script').forEach(node=>node.remove());
                      document.querySelector('main').replaceWith(main);
                      history.pushState({},'',path+(fragment?'#page-properties':''));
                      window.logseqGardenerInitProperties(main);
                      if(fragment)window.logseqGardenerRevealFragment('#page-properties',{scroll:false});
                    }""", [path, fragment])
                replace(one)
                expect(details).not_to_have_attribute('open', '')
                replace(two, True)
                expect(details).to_have_attribute('open', '')
                page.evaluate('logseqGardenerInitProperties(document.querySelector("main"))')
                expect(details).to_have_attribute('open', '')
                assert page.evaluate('(key) => JSON.parse(localStorage.getItem(key)).open', key) is False
                replace(one)
                expect(details).not_to_have_attribute('open', '')
                # Real SPA visits renew, expire and discard future/invalid timestamps.
                age = 29 * 86400000
                page.evaluate('([key,age]) => localStorage.setItem(key,JSON.stringify({open:false,seen:Date.now()-age}))', [key, age])
                replace(two)
                assert page.evaluate('(key) => Date.now()-JSON.parse(localStorage.getItem(key)).seen < 1000', key)
                for value in ['broken', {'open': False, 'seen': 1}, {'open': False, 'seen': 9999999999999}, {'open': False, 'seen': '123'}, {'open': 'false', 'seen': 1}]:
                    page.evaluate('([key,value])=>localStorage.setItem(key,typeof value==="string"?value:JSON.stringify(value))', [key,value])
                    replace(one)
                    expect(details).to_have_attribute('open', '')
                    assert page.evaluate('(key)=>localStorage.getItem(key)', key) is None
                # A hard-link fragment opens the necessary section without saving a preference.
                page.evaluate('(key)=>localStorage.setItem(key,JSON.stringify({open:false,seen:Date.now()}))', key)
                page.goto(base + one + '#page-properties')
                page.wait_for_function('!window.logseqGardenerInitProperties.k')
                expect(details).to_have_attribute('open', '')
                assert page.evaluate('(key)=>JSON.parse(localStorage.getItem(key)).open', key) is False
                page.goto(base + two)
                assert page.evaluate('parsedOpen') is False
                module = page.locator('link[rel=lsg-properties]').get_attribute('href')
                # Both early choices survive ordinary navigation and reload while the module is held.
                for choice in [False, True]:
                    for leave in ['navigate', 'reload']:
                        page.evaluate('([key,open])=>localStorage.setItem(key,JSON.stringify({open,seen:Date.now()}))', [key, not choice])
                        held = []
                        # Hold only the first page's module; the next page loads normally.
                        page.route('**/' + module, lambda route: route.continue_() if held else held.append(route))
                        page.goto(base + one, wait_until='domcontentloaded')
                        page.wait_for_function('(open)=>window.parsedOpen===open', arg=not choice)
                        while not held:
                            page.wait_for_timeout(10)
                        summary.click()
                        assert details.evaluate('(node)=>node.open') is choice
                        if leave == 'navigate':
                            page.goto(base + two, wait_until='domcontentloaded')
                        else:
                            page.reload(wait_until='domcontentloaded')
                        assert page.evaluate('parsedOpen') is choice, (choice, leave)
                        try:
                            held[0].abort()
                        except Exception:
                            pass  # The browser already canceled the left page's request.
                        page.wait_for_function('!window.logseqGardenerInitProperties.k')
                        page.unroute_all()
                page.goto(base + two)
                page.wait_for_function('!window.logseqGardenerInitProperties.k')
                # Both early choices reach a host replacement while the module is held.
                for choice in [False, True]:
                    page.evaluate('([key,open])=>localStorage.setItem(key,JSON.stringify({open,seen:Date.now()}))', [key, not choice])
                    held = []
                    page.route('**/' + module, lambda route: held.append(route))
                    page.goto(base + one, wait_until='domcontentloaded')
                    page.wait_for_function('(open)=>window.parsedOpen===open', arg=not choice)
                    summary.click()
                    assert details.evaluate('(node)=>node.open') is choice
                    while not held:
                        page.wait_for_timeout(10)
                    replace(two)
                    assert details.evaluate('(node)=>node.open') is choice
                    held[0].continue_()
                    page.wait_for_function('!window.logseqGardenerInitProperties.k')
                    page.wait_for_function('([key,open])=>JSON.parse(localStorage.getItem(key)).open===open', arg=[key, choice])
                    assert details.evaluate('(node)=>node.open') is choice
                    page.unroute_all(behavior='ignoreErrors')
                # Search and theme remain available alongside the preference controller.
                page.locator('[data-lsg-search]').click()
                expect(page.locator('#lsg-search-input')).to_be_focused()
                page.keyboard.press('Escape')
                page.evaluate("window.logseqGardenerTheme.set('dark')")
                expect(page.locator('html')).to_have_attribute('data-lsg-theme', 'dark')
                page.evaluate("window.logseqGardenerTheme.set('light')")
                expect(page.locator('html')).to_have_attribute('data-lsg-theme', 'light')
                page.goto(base + one)
                arrow = page.locator('.outline .lsg-arrow').first
                expect(arrow).to_have_attribute('aria-expanded', 'true')
                arrow.click()
                expect(arrow).to_have_attribute('aria-expanded', 'false')
                expect(details).to_have_attribute('open', '')
                page.locator('.outline .bullet').first.click()
                expect(page.locator('main')).to_have_attribute('data-lsg-zoomed', '')
                expect(page.locator('.page-properties')).not_to_be_visible()
                page.evaluate("history.pushState({},'',location.pathname);dispatchEvent(new HashChangeEvent('hashchange'))")
                expect(page.locator('.page-properties')).to_be_visible()
                expect(details).to_have_attribute('open', '')
                assert not errors, errors
                context.close()
                blocked = browser.new_context()
                blocked.add_init_script("Object.defineProperty(window,'localStorage',{get(){throw new Error('blocked')}})")
                p = blocked.new_page()
                p.goto(base + one)
                expect(p.locator('.page-properties > details')).to_have_attribute('open', '')
                p.goto(base + closed)
                expect(p.locator('.page-properties > details')).not_to_have_attribute('open', '')
                blocked.close()
                nojs = browser.new_context(java_script_enabled=False)
                p = nojs.new_page()
                p.goto(base + one)
                expect(p.locator('.page-properties > details')).to_have_attribute('open', '')
                nojs.close()
                browser.close()
                print('PASS: configured defaults, both choices, sliding expiry, invalid/blocked storage, base paths, early restore, SPA/fragment ordering, native keyboard and held-module activation')
        finally:
            server.shutdown()
            server.server_close()


if __name__ == '__main__':
    main()
