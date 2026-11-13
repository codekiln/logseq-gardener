#!/usr/bin/env python3
# /// script
# requires-python = ">=3.12"
# dependencies = ["playwright==1.60.0"]
# ///
"""Check offline reading (:lsg/offline) in Chromium against generated publications.

Publishes a small garden with :lsg/offline true and --readable-urls at the paths /a/ and /ab/ of one
origin and at the root of another, then checks: nothing is registered or stored before Save; Save
and Remove from the keyboard; the registered scope; the cache name and isolation of /a/ from /ab/;
no worker at an origin root; a page opened from disk; network first online; a stored copy with its
saved-on notice when the server stops and when the browser is offline; the browser's offline page
for an unvisited page; the 4-second fallback; what is stored (shared files, not the search index
or a 400 KB image); 404 deletion; the check of stored pages after a new publish, a host that
answers HEAD with 405, and 30-day expiry; the 250-copy and 32 MB limits and a quota error; the
removing worker of :lsg/offline false; and what Chromium does when a registered worker's script
returns 404. It then measures the [offline] rows nav_overhead_ms, memory_growth_mb, cache_pages
and cache_mb against scripts/parity/budgets.toml.

Prints one JSON line of results and exits 1 when a check or a budget fails.
"""
import argparse
import functools
import http.server
import json
import os
import random
import shutil
import statistics
import struct
import subprocess
import sys
import tempfile
import threading
import time
import tomllib
import zlib
from pathlib import Path

from playwright.sync_api import expect, sync_playwright

REPO = Path(__file__).resolve().parents[2]
BUDGETS = REPO / 'scripts' / 'parity' / 'budgets.toml'
PAGES = {
    'Off___One.md': '- The first page. See [[Off/Two]], [[Off/Three]] and [[Off/Image]].\n',
    'Off___Two.md': '- The second page. Back to [[Off/One]].\n',
    'Off___Three.md': '- The third page. Back to [[Off/One]].\n',
    'Off___Four.md': '- A page the reader never opens.\n',
    'Off___Gone.md': '- A page the author will unpublish.\n',
    'Off___Image.md': '- A page with a large image.\n- ![big](../assets/big.png)\n',
}
CONFIG = '{:publishing/all-pages-public? true %s :lsg/install {:name "Offline Garden" :short-name "Offline" :icon "assets/app-icon.png"}}\n'
DAY_MS = 24 * 60 * 60 * 1000

# What the test server does: refuse every connection, delay page navigations, answer HEAD with a
# status, and the HEAD requests it received.
SERVER = {'down': False, 'slow': 0, 'head': None, 'heads': []}


class Handler(http.server.SimpleHTTPRequestHandler):
    def log_message(self, *args):
        pass

    def end_headers(self):
        # Keep the browser's HTTP cache out of the way, so only the worker can answer offline.
        self.send_header('Cache-Control', 'no-store')
        super().end_headers()

    def do_GET(self):
        if SERVER['down']:
            self.close_connection = True
            return
        if SERVER['slow'] and self.path.endswith('/'):
            time.sleep(SERVER['slow'])
        super().do_GET()

    def do_HEAD(self):
        if SERVER['down']:
            self.close_connection = True
            return
        SERVER['heads'].append(self.path)
        if SERVER['head']:
            self.send_error(SERVER['head'])
            return
        super().do_HEAD()


def serve(root):
    server = http.server.ThreadingHTTPServer(('127.0.0.1', 0), functools.partial(Handler, directory=str(root)))
    threading.Thread(target=server.serve_forever, daemon=True).start()
    return server, f'http://127.0.0.1:{server.server_address[1]}'


def png(side, noisy=False):
    """A square RGB PNG; a noisy one is stored without compression, so its size is about 3 x side²."""
    def chunk(kind, data):
        body = kind + data
        return struct.pack('>I', len(data)) + body + struct.pack('>I', zlib.crc32(body))
    rng = random.Random(7)
    rows = b''.join(b'\x00' + (rng.randbytes(side * 3) if noisy else bytes([0x2f, 0x6f, 0x4f]) * side)
                    for _ in range(side))
    return (b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', struct.pack('>IIBBBBB', side, side, 8, 2, 0, 0, 0))
            + chunk(b'IDAT', zlib.compress(rows, 0 if noisy else 9)) + chunk(b'IEND', b''))


class Garden:
    def __init__(self, binary, temp):
        self.binary, self.root = binary, temp / 'garden'
        (self.root / 'pages').mkdir(parents=True)
        (self.root / 'assets').mkdir()
        (self.root / 'logseq').mkdir()
        (self.root / 'assets' / 'app-icon.png').write_bytes(png(512))
        (self.root / 'assets' / 'big.png').write_bytes(png(370, noisy=True))
        for name, text in PAGES.items():
            (self.root / 'pages' / name).write_text(text, encoding='utf-8')

    def publish(self, output, key=':lsg/offline true', without=()):
        """Publish into `output`, replacing it, with `key` and without the pages in `without`."""
        (self.root / 'logseq' / 'config.edn').write_text(CONFIG % key, encoding='utf-8')
        for name in PAGES:
            path = self.root / 'pages' / name
            if name in without:
                path.unlink(missing_ok=True)
            elif not path.exists():
                path.write_text(PAGES[name], encoding='utf-8')
        staging = output.with_name(output.name + '.new')
        shutil.rmtree(staging, ignore_errors=True)
        subprocess.run([str(self.binary), 'publish', '--graph', str(self.root), '--output', str(staging),
                        '--filename-format', 'triple-lowbar', '--include', 'Off', '--readable-urls', '--search'],
                       check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        shutil.rmtree(output, ignore_errors=True)
        staging.rename(output)


def stored(page, name):
    """The addresses stored in cache `name`, without the record, and the record."""
    return page.evaluate('''async (name) => {
      if (!(await caches.has(name))) return null;
      const cache = await caches.open(name), keys = (await cache.keys()).map((r) => r.url);
      const recordKey = keys.find((url) => url.endsWith('?lsg-offline-record'));
      const record = recordKey ? await (await cache.match(recordKey)).json() : null;
      return { keys: keys.filter((url) => url !== recordKey).sort(), record };
    }''', name)


def storage(page):
    return page.evaluate('''async () => ({
      workers: (await navigator.serviceWorker.getRegistrations()).map((r) => r.scope).sort(),
      caches: (await caches.keys()).sort(),
      local: Object.keys(localStorage), session: Object.keys(sessionStorage),
      databases: (await indexedDB.databases()).map((d) => d.name) })''')


def menu_item(page, selector):
    """Open the More menu from the keyboard, arrow to `selector` and press Enter on it."""
    page.locator('[data-lsg-more]').focus()
    page.keyboard.press('Enter')
    for _ in range(6):
        if page.evaluate('(s) => document.activeElement?.matches(s)', selector):
            break
        page.keyboard.press('ArrowDown')
    assert page.evaluate('(s) => document.activeElement?.matches(s)', selector), selector
    page.keyboard.press('Enter')


def save(page):
    menu_item(page, '[data-lsg-offline-save]')
    page.wait_for_function('navigator.serviceWorker.controller !== null')
    expect(page.locator('[data-lsg-offline-status]')).to_have_text(
        'Saving is on. Pages you open from now on are kept for offline reading.')
    assert page.evaluate("document.activeElement?.hasAttribute('data-lsg-more')")


def settle(page, name, predicate, timeout=15):
    """Poll cache `name` until `predicate(state)` holds; return the state."""
    deadline = time.time() + timeout
    while True:
        state = stored(page, name)
        if predicate(state):
            return state
        if time.time() > deadline:
            raise AssertionError(f'{name}: {state}')
        page.wait_for_timeout(100)


def browse_until(page, url, name, predicate, timeout=20):
    """Open `url` again every second, as a reader moving between pages would, until cache `name`
    satisfies `predicate`: the browser checks for a new worker after a navigation, and the new
    worker checks the stored pages after it answers a request."""
    deadline = time.time() + timeout
    while True:
        visit(page, url)
        page.wait_for_timeout(1000)
        state = stored(page, name)
        if predicate(state):
            return state
        if time.time() > deadline:
            raise AssertionError(f'{name}: {state}')


def has(state, url):
    return state is not None and url in state['keys']


def visit(page, url):
    response = page.goto(url, wait_until='load')
    page.wait_for_timeout(150)
    return response


def check_reading(browser, garden, served, base, root_base, results):
    """The first visit, Save, copies, notices, limits on what is stored, and 404 deletion."""
    a = f'{base}/a/'
    name = 'lsg-offline:/a/'
    context = browser.new_context()
    page = context.new_page()
    errors, requests = [], []
    page.on('pageerror', lambda error: errors.append(str(error)))
    page.on('request', lambda request: requests.append(request.url))

    # Three pages without opening the More menu: no worker, cache or storage, and no worker request.
    for title in ('One', 'Two', 'Three'):
        visit(page, f'{a}Off/{title}/')
    state = storage(page)
    assert state == {'workers': [], 'caches': [], 'local': [], 'session': [], 'databases': []}, state
    assert not [url for url in requests if url.endswith('lsg-sw.js')], requests
    results['first_visit_stores_nothing'] = state

    # The menu shows Save and hides Remove; Save from the keyboard registers the folder scope.
    page.locator('[data-lsg-more]').click()
    expect(page.locator('[data-lsg-offline-save]')).to_be_visible()
    expect(page.locator('[data-lsg-offline-remove]')).to_be_hidden()
    page.keyboard.press('Escape')
    save(page)
    page.locator('[data-lsg-more]').click()
    expect(page.locator('[data-lsg-offline-remove]')).to_be_visible()
    expect(page.locator('[data-lsg-offline-save]')).to_be_hidden()
    page.keyboard.press('Escape')
    assert storage(page)['workers'] == [a], storage(page)
    results['save_registers_scope'] = a

    # Pages opened from now on are kept, with the shared files they load.
    for title in ('One', 'Two', 'Three', 'Gone'):
        visit(page, f'{a}Off/{title}/')
    state = settle(page, name, lambda s: has(s, f'{a}Off/Gone/') and has(s, f'{a}lsg-site.css'))
    shared = sorted(url.removeprefix(a) for url in state['keys'] if '/Off/' not in url)
    for expected in ('lsg-site.css', 'lsg-shell.js', 'lsg-offline.js', 'lsg-theme.js'):
        assert expected in shared, shared
    assert page.evaluate('caches.keys()') == [name]
    results['stored_after_save'] = {'pages': sorted(url.removeprefix(a) for url in state['keys'] if '/Off/' in url),
                                    'shared': shared, 'manifest_stored': 'lsg.webmanifest' in shared}

    # The search index and a 400 KB image stay online only; the image still appears online.
    visit(page, f'{a}Off/Image/')
    image = page.locator('main img').first
    expect(image).to_be_visible()
    page.wait_for_function("document.querySelector('main img').naturalWidth > 0")
    image_url = page.evaluate("document.querySelector('main img').src")
    page.evaluate("fetch('lsg-search-manifest.json').then((r) => r.json())")
    state = settle(page, name, lambda s: has(s, f'{a}Off/Image/'))
    assert image_url not in state['keys'], image_url
    assert not [url for url in state['keys'] if '/lsg-search-' in url and url.endswith('.json')], state['keys']
    results['large_image_and_search_online_only'] = {'image_bytes': (served / 'a' / image_url.removeprefix(a)).stat().st_size}

    # Online, a changed page is shown and replaces the copy.
    one = served / 'a' / 'Off' / 'One' / 'index.html'
    one.write_text(one.read_text(encoding='utf-8').replace('The first page.', 'The first page, changed.'), encoding='utf-8')
    visit(page, f'{a}Off/One/')
    expect(page.locator('main')).to_contain_text('The first page, changed.')
    settle(page, name, lambda s: 'changed' in page.evaluate(
        "async (u) => (await (await caches.open('lsg-offline:/a/')).match(u))?.text() ?? ''", f'{a}Off/One/'))
    results['online_shows_current_page_and_replaces_copy'] = True

    # The server stops: a visited page appears from its copy with the saved-on notice.
    SERVER['down'] = True
    try:
        visit(page, f'{a}Off/Two/')
        expect(page.locator('main')).to_contain_text('The second page.')
        notice = page.locator('[data-lsg-offline-status]')
        expect(notice).to_have_text(page.evaluate(
            "'Saved copy from ' + new Date().toLocaleDateString(undefined, { day: 'numeric', month: 'long' })"))
        results['server_stopped_visited_page'] = notice.text_content()
        # An unvisited page gets the browser's own offline page, and nothing is stored for it.
        try:
            page.goto(f'{a}Off/Four/', wait_until='load')
            failed = None
        except Exception as error:  # noqa: BLE001 - Playwright raises a plain Error for net:: failures
            failed = str(error).splitlines()[0]
        assert failed and 'net::' in failed, failed
        results['server_stopped_unvisited_page'] = failed
    finally:
        SERVER['down'] = False
    # Let the browser finish showing its error page before the next navigation.
    page.wait_for_timeout(500)
    page.goto(f'{a}Off/One/', wait_until='load')
    assert not has(stored(page, name), f'{a}Off/Four/')

    # The browser reports it is offline: the copy and the notice again.
    context.set_offline(True)
    try:
        visit(page, f'{a}Off/Three/')
        expect(page.locator('main')).to_contain_text('The third page.')
        expect(page.locator('[data-lsg-offline-status]')).to_contain_text('Saved copy from ')
        results['browser_offline_visited_page'] = page.locator('[data-lsg-offline-status]').text_content()
    finally:
        context.set_offline(False)

    # A server that takes 6 seconds: the copy answers after about 4.
    SERVER['slow'] = 6
    try:
        started = time.time()
        page.goto(f'{a}Off/Two/', wait_until='domcontentloaded')
        expect(page.locator('main')).to_contain_text('The second page.')
        waited = time.time() - started
        assert 3.5 <= waited < 5.8, waited
        expect(page.locator('[data-lsg-offline-status]')).to_contain_text('Saved copy from ')
        results['slow_network_seconds_to_copy'] = round(waited, 2)
    finally:
        SERVER['slow'] = 0
    page.wait_for_timeout(2500)

    # A page that now returns 404 shows the site's answer and loses its copy.
    shutil.rmtree(served / 'a' / 'Off' / 'Gone')
    response = page.goto(f'{a}Off/Gone/', wait_until='load')
    assert response.status == 404, response.status
    settle(page, name, lambda s: not has(s, f'{a}Off/Gone/'))
    results['not_found_deletes_copy'] = True

    visit(page, f'{a}Off/One/')
    # Remove from the keyboard: the cache and the registration are gone and Save is back.
    menu_item(page, '[data-lsg-offline-remove]')
    expect(page.locator('[data-lsg-offline-status]')).to_have_text('Offline copies removed.')
    page.wait_for_function("navigator.serviceWorker.getRegistrations().then((r) => r.length === 0)")
    assert page.evaluate('caches.keys()') == []
    page.locator('[data-lsg-more]').click()
    expect(page.locator('[data-lsg-offline-save]')).to_be_visible()
    results['remove_deletes_cache_and_worker'] = True
    assert not errors, errors
    context.close()

    # A page opened from disk shows neither item and raises no error.
    context = browser.new_context()
    page = context.new_page()
    page.on('pageerror', lambda error: errors.append(str(error)))
    page.goto((served / 'a' / 'index.html').as_uri(), wait_until='load')
    page.wait_for_timeout(300)
    page.locator('[data-lsg-more]').click()
    expect(page.locator('[data-lsg-offline-save]')).to_be_hidden()
    expect(page.locator('[data-lsg-offline-remove]')).to_be_hidden()
    assert not errors, errors
    results['from_disk_no_items_no_errors'] = True
    context.close()

    # A site at the root of an origin shows neither item and registers nothing.
    context = browser.new_context()
    page = context.new_page()
    visit(page, f'{root_base}/Off/One/')
    page.locator('[data-lsg-more]').click()
    expect(page.locator('[data-lsg-offline-save]')).to_be_hidden()
    expect(page.locator('[data-lsg-offline-remove]')).to_be_hidden()
    assert storage(page)['workers'] == []
    results['origin_root_no_items_no_worker'] = True
    context.close()


def check_neighbours(browser, base, results):
    """Gardens at /a/ and /ab/: separate scopes, separate caches, and Remove touches only its own."""
    context = browser.new_context()
    page = context.new_page()
    for site in ('a', 'ab'):
        visit(page, f'{base}/{site}/Off/One/')
        save(page)
        visit(page, f'{base}/{site}/Off/Two/')
    assert storage(page)['workers'] == [f'{base}/a/', f'{base}/ab/'], storage(page)
    assert page.evaluate('caches.keys()') == ['lsg-offline:/a/', 'lsg-offline:/ab/']
    # A page of /a/ fetches a page of /ab/: the /a/ worker passes it through and stores nothing.
    visit(page, f'{base}/a/Off/One/')
    page.evaluate("(url) => fetch(url).then((r) => r.text())", f'{base}/ab/Off/Three/')
    page.wait_for_timeout(500)
    a, ab = stored(page, 'lsg-offline:/a/'), stored(page, 'lsg-offline:/ab/')
    assert all(url.startswith(f'{base}/a/') for url in a['keys']), a['keys']
    assert all(url.startswith(f'{base}/ab/') for url in ab['keys']), ab['keys']
    menu_item(page, '[data-lsg-offline-remove]')
    page.wait_for_function("navigator.serviceWorker.getRegistrations().then((r) => r.length === 1)")
    assert page.evaluate('caches.keys()') == ['lsg-offline:/ab/']
    assert stored(page, 'lsg-offline:/ab/')['keys'] == ab['keys']
    assert storage(page)['workers'] == [f'{base}/ab/']
    results['neighbours_isolated'] = {'a': len(a['keys']), 'ab': len(ab['keys'])}
    context.close()


def check_updates(browser, garden, served, base, results):
    """A new publish checks every stored page; 405 keeps a copy; 30-day expiry; turning it off."""
    a = f'{base}/a/'
    name = 'lsg-offline:/a/'
    garden.publish(served / 'a')
    context = browser.new_context()
    page = context.new_page()
    cdp = context.new_cdp_session(page)
    visit(page, f'{a}Off/One/')
    save(page)
    for title in ('One', 'Two', 'Four', 'Gone'):
        visit(page, f'{a}Off/{title}/')
    first = settle(page, name, lambda s: has(s, f'{a}Off/Gone/') and s['record'] and s['record']['build'])
    old_worker = (served / 'a' / 'lsg-sw.js').read_text(encoding='utf-8')

    # A block edit leaves lsg-sw.js unchanged.
    (garden.root / 'pages' / 'Off___One.md').write_text(PAGES['Off___One.md'] + '- A new block.\n', encoding='utf-8')
    garden.publish(served / 'a')
    assert (served / 'a' / 'lsg-sw.js').read_text(encoding='utf-8') == old_worker
    (garden.root / 'pages' / 'Off___One.md').write_text(PAGES['Off___One.md'], encoding='utf-8')

    # The author unpublishes Four, which no page links; the reader opens another page online.
    SERVER['heads'].clear()
    garden.publish(served / 'a', without=('Off___Four.md',))
    assert (served / 'a' / 'lsg-sw.js').read_text(encoding='utf-8') != old_worker
    state = browse_until(page, f'{a}Off/One/', name,
                         lambda s: not has(s, f'{a}Off/Four/') and s['record']['build'] != first['record']['build'])
    assert has(state, f'{a}Off/One/') and has(state, f'{a}Off/Gone/'), state['keys']
    heads = sorted(set(SERVER['heads']))
    assert '/a/Off/Four/' in heads, heads
    results['unpublished_page_deleted_after_new_publish'] = {'head_requests': len(SERVER['heads'])}

    # A host that answers HEAD with 405 keeps the copy of a page the author unpublished.
    SERVER['head'] = 405
    try:
        garden.publish(served / 'a', without=('Off___Four.md', 'Off___Gone.md'))
        before, build = len(SERVER['heads']), state['record']['build']
        browse_until(page, f'{a}Off/One/', name, lambda s: len(SERVER['heads']) > before and s['record']['build'] != build)
        page.wait_for_timeout(500)
        assert has(stored(page, name), f'{a}Off/Gone/'), stored(page, name)['keys']
        results['head_405_keeps_copy'] = True
    finally:
        SERVER['head'] = None

    # A copy saved 31 days ago is gone the next time the worker starts.
    page.evaluate('''async () => {
      const cache = await caches.open('lsg-offline:/a/'), key = (await cache.keys()).find((r) => r.url.endsWith('?lsg-offline-record'));
      const record = await (await cache.match(key)).json();
      record.pages[new URL('Off/Two/', location.href.split('Off/')[0]).href][1] -= 31 * 864e5;
      await cache.put(key, new Response(JSON.stringify(record)));
    }''')
    cdp.send('ServiceWorker.enable')
    cdp.send('ServiceWorker.stopAllWorkers')
    visit(page, f'{a}Off/One/')
    settle(page, name, lambda s: not has(s, f'{a}Off/Two/'))
    results['expired_copy_deleted_on_start'] = True

    # The author publishes :lsg/offline false: one online visit removes the cache and the worker.
    garden.publish(served / 'a', key=':lsg/offline false')
    assert not (served / 'a' / 'lsg-offline.js').exists()
    visit(page, f'{a}Off/One/')
    deadline = time.time() + 15
    while storage(page)['workers'] or page.evaluate('caches.keys()'):
        assert time.time() < deadline, storage(page)
        page.wait_for_timeout(200)
    results['removing_worker_clears_reader'] = storage(page)
    context.close()
    garden.publish(served / 'a')


def check_missing_script(browser, served, base, results):
    """Record what Chromium does when a registered worker's script later returns 404."""
    context = browser.new_context()
    page = context.new_page()
    visit(page, f'{base}/ab/Off/One/')
    save(page)
    visit(page, f'{base}/ab/Off/Two/')
    script = served / 'ab' / 'lsg-sw.js'
    kept = script.read_bytes()
    script.unlink()
    try:
        visit(page, f'{base}/ab/Off/One/')
        update = page.evaluate('''async () => {
          const r = await navigator.serviceWorker.getRegistration();
          try { await r.update(); return 'updated'; } catch (e) { return e.name + ': ' + e.message; }
        }''')
        page.wait_for_timeout(500)
        visit(page, f'{base}/ab/Off/Two/')
        results['missing_script_chromium'] = {
            'update': update, 'registrations': storage(page)['workers'],
            'controlled': page.evaluate('navigator.serviceWorker.controller !== null'),
            'cache_kept': page.evaluate("caches.has('lsg-offline:/ab/')")}
    finally:
        script.write_bytes(kept)
    context.close()


def big_site(served, folder, worker, pages, size):
    """`pages` pages of `size` bytes and the worker, under served/folder."""
    root = served / folder
    root.mkdir()
    shutil.copy(worker, root / 'lsg-sw.js')
    (root / 'index.html').write_text('<!doctype html><title>big</title><main>big</main>', encoding='utf-8')
    filler = 'x' * size
    for number in range(pages):
        (root / f'p{number}.html').write_text(f'<!doctype html><title>{number}</title>{filler}'[:size], encoding='utf-8')


def fill(page, base, folder, pages):
    """Register the worker of served/folder, then request every page through it."""
    visit(page, f'{base}/{folder}/index.html')
    page.evaluate("navigator.serviceWorker.register('lsg-sw.js', { scope: './' })")
    page.wait_for_function('navigator.serviceWorker.controller !== null')
    statuses = page.evaluate('''async (count) => {
      const statuses = [];
      for (let n = 0; n < count; n++) statuses.push((await fetch(`p${n}.html`)).status);
      return statuses;
    }''', pages)
    assert set(statuses) == {200}, statuses
    page.wait_for_timeout(1000)


def check_limits(browser, served, base, quota_base, results, limits):
    """300 pages of 100 KB, 200 pages of 200 KB, and a quota error."""
    measured = {}
    for folder, pages, size in (('big', 300, 100_000), ('bigger', 200, 200_000)):
        context = browser.new_context()
        page = context.new_page()
        fill(page, base, folder, pages)
        name = f'lsg-offline:/{folder}/'
        state = settle(page, name, lambda s: s and has(s, f'{base}/{folder}/p{pages - 1}.html')
                       and len(s['record']['pages']) == len(s['keys']))
        total = sum(entry[0] for entry in state['record']['pages'].values())
        count = len(state['keys'])
        assert count <= 250 and total <= 32_000_000, (count, total)
        assert not has(state, f'{base}/{folder}/p0.html'), 'the earliest page is gone'
        measured[folder] = {'copies': count, 'bytes': total}
        context.close()
    results['limits'] = measured
    results['cache_pages'] = max(entry['copies'] for entry in measured.values())
    results['cache_mb'] = max(entry['bytes'] for entry in measured.values()) / 1e6

    # A quota of 3 MB: every page still loads, and the cache stays below the quota.
    context = browser.new_context()
    page = context.new_page()
    cdp = context.new_cdp_session(page)
    cdp.send('Storage.overrideQuotaForOrigin', {'origin': quota_base, 'quotaSize': 3_000_000})
    fill(page, quota_base, 'quota', 60)
    state = settle(page, 'lsg-offline:/quota/', lambda s: s and has(s, f'{quota_base}/quota/p59.html'))
    total = sum(entry[0] for entry in state['record']['pages'].values())
    assert total < 3_000_000 and len(state['keys']) < 15, (len(state['keys']), total)
    results['quota_error'] = {'copies': len(state['keys']), 'bytes': total, 'pages_loaded': 60}
    context.close()


def check_speed(browser, base, results, rounds=15):
    """nav_overhead_ms: online navigations with and without an active worker; memory_growth_mb."""
    url = f'{base}/a/Off/One/'
    plain, saved = browser.new_context(), browser.new_context()
    without, with_worker = plain.new_page(), saved.new_page()
    visit(with_worker, url)
    save(with_worker)
    timing = '''() => { const e = performance.getEntriesByType('navigation')[0]; return e.responseEnd - e.startTime; }'''
    times = {'without': [], 'with': []}
    for _ in range(3):
        for page in (without, with_worker):
            visit(page, url)
    for _ in range(rounds):
        for key, page in (('without', without), ('with', with_worker)):
            page.goto(url, wait_until='load')
            times[key].append(page.evaluate(timing))
    assert with_worker.evaluate('navigator.serviceWorker.controller !== null')
    overhead = statistics.median(times['with']) - statistics.median(times['without'])
    results['nav_overhead_ms'] = round(overhead, 1)
    results['nav_median_ms'] = {key: round(statistics.median(values), 1) for key, values in times.items()}

    def heap(page):
        cdp = page.context.new_cdp_session(page)
        cdp.send('HeapProfiler.enable')
        cdp.send('HeapProfiler.collectGarbage')
        cdp.send('Performance.enable')
        metrics = {m['name']: m['value'] for m in cdp.send('Performance.getMetrics')['metrics']}
        return metrics['JSHeapUsedSize']
    for page in (without, with_worker):
        visit(page, url)
        page.wait_for_timeout(500)
    growth = (heap(with_worker) - heap(without)) / 1e6
    results['memory_growth_mb'] = round(growth, 3)
    plain.close()
    saved.close()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', default=str(REPO / 'target' / 'debug' / 'lsg'))
    args = parser.parse_args()
    limits = tomllib.loads(BUDGETS.read_text(encoding='utf-8'))['offline']
    results = {}
    with tempfile.TemporaryDirectory() as temp:
        temp = Path(temp)
        garden = Garden(Path(args.binary), temp)
        served, root_served, quota_served = temp / 'served', temp / 'root', temp / 'quota-origin'
        served.mkdir()
        quota_served.mkdir()
        for site in ('a', 'ab'):
            garden.publish(served / site)
        garden.publish(root_served)
        big_site(served, 'big', served / 'a' / 'lsg-sw.js', 300, 100_000)
        big_site(served, 'bigger', served / 'a' / 'lsg-sw.js', 200, 200_000)
        big_site(quota_served, 'quota', served / 'a' / 'lsg-sw.js', 60, 200_000)
        servers = [serve(served), serve(root_served), serve(quota_served)]
        (server, base), (_, root_base), (_, quota_base) = servers
        try:
            with sync_playwright() as playwright:
                browser = playwright.chromium.launch()
                results['chromium'] = browser.version
                check_reading(browser, garden, served, base, root_base, results)
                garden.publish(served / 'a')
                check_neighbours(browser, base, results)
                check_updates(browser, garden, served, base, results)
                check_missing_script(browser, served, base, results)
                check_limits(browser, served, base, quota_base, results, limits)
                check_speed(browser, base, results)
                browser.close()
        finally:
            for running, _ in servers:
                running.shutdown()
                running.server_close()
    failures = []
    for row in ('nav_overhead_ms', 'memory_growth_mb', 'cache_pages', 'cache_mb'):
        if results[row] > limits[row]:
            failures.append(f'{row}: {results[row]} over {limits[row]}')
    print(json.dumps({'offline_browser': 'failed' if failures else 'passed', **results}))
    if failures:
        print('\n'.join(failures), file=sys.stderr)
        raise SystemExit(1)


if __name__ == '__main__':
    os.environ.setdefault('PYTHONUNBUFFERED', '1')
    main()
