#!/usr/bin/env python3
# /// script
# requires-python = ">=3.12"
# dependencies = ["playwright==1.60.0"]
# ///
"""Check lsg's plugin host in Chromium against a generated publication.

Builds a small garden with the fixture plugins in tests/fixtures/plugins and three generated
ones (a probe, one whose setup throws, one whose export does not match its manifest), then checks
playback across navigation and Back, on-demand loading, stale responses, fragments, setup
failure, a new build during a visit, fetch failure, collapse and the remembered Properties choice
after navigation, and that listener counts stay flat over 20 navigations. It then builds a garden
that turns on the first-party podcast-episode plugin in its config.edn and checks that a page
without an episode requests no plugin code, that the recording keeps playing across navigation and
Back, that another episode takes over only when started, that chapter headings and dialogue
timestamps seek the player, and that a page whose module fails keeps its own recording. Prints one JSON line of results;
--screenshots DIR also saves the recording playing on each page.
"""
import argparse
import functools
import gzip
import http.server
import json
from pathlib import Path
import subprocess
import tempfile
import struct
import threading
from playwright.sync_api import sync_playwright, expect

REPO = Path(__file__).resolve().parents[2]
FIXTURES = REPO / 'tests' / 'fixtures' / 'plugins'
BLOCK = '6730a1b2-0000-4000-8000-000000000001'

PAGES = {
    'Episode___One.md': 'recording:: one\nsecret:: SECRET_VALUE\n\n- Episode one. See [[Episode/Two]] and [[Episode/Three]].\n',
    'Episode___Two.md': 'badge:: new\n\n- Episode two, without a recording. Back to [[Episode/One]].\n',
    'Episode___Three.md': ('recording:: three\n\n- Episode three.\n- Folded parent\n  collapsed:: true\n'
                           f'  - Hidden child\n    id:: {BLOCK}\n'),
    'Episode___Plain.md': '- A page with no plugin properties.\n',
}
GENERATED = {
    'probe': ('after-load', '''export default { id: 'probe', apiVersion: 1, setup(host) {
  window.__probe = [];
  return host.onPageAdopted(({ page, main, signal, metadata }) => {
    const id = location.hash.length > 1 ? decodeURIComponent(location.hash.slice(1)) : '';
    const target = id && document.getElementById(id);
    window.__probe.push({ url: page.url, title: main.querySelector('h1')?.textContent,
      revealed: target ? !target.closest('details:not([open])') : null,
      active: document.activeElement?.id || '', aborted: signal.aborted, metadata });
  });
} };
'''),
    'broken': ('after-load', '''export default { id: 'broken', apiVersion: 1, setup(host) {
  host.persistentRegion('leftover').element.textContent = 'should be removed';
  throw new Error('fixture setup failure');
} };
'''),
    'mismatch': ('after-load', '''export default { id: 'something-else', apiVersion: 1, setup() { window.__mismatchSetup = true; } };
'''),
}

EPISODE_PAGES = {
    'Show___One.md': ('logseq-entity:: [[Logseq/Entity/Podcast/Episode/Fixture]], [[Logseq/Entity/Podcast/Episode]]\n\n'
                      '- # The first episode\n\t- ![Episode recording](../assets/one.wav)\n'
                      '\t- See [[Show/Plain]] and [[Show/Two]].\n\t- ## Chapters\n'
                      '\t\t- ### 00:00 Opening\n\t\t\t- #### 00:20 Twenty seconds in\n\t- Transcript\n'
                      '\t\t- **Host** · 00:00:12 — Twelve seconds in.\n'),
    'Show___Two.md': ('logseq-entity:: [[Logseq/Entity/Podcast/Episode]]\n\n'
                      '- # The second episode\n\t- ![Episode recording](../assets/two.wav)\n'
                      '\t- Back to [[Show/One]].\n\t- Transcript\n'
                      '\t\t- **Host** · 00:00:05 — Five seconds in.\n'),
    'Show___Plain.md': '- A page that is no episode. [[Show/One]] and [[Show/Two]].\n',
}


def tone(frequency, seconds=40, rate=8000):
    """A sine tone as 8-bit mono WAV bytes."""
    import math
    samples = bytes(128 + round(40 * math.sin(2 * math.pi * frequency * i / rate)) for i in range(rate * seconds))
    header = b'RIFF' + struct.pack('<I', 36 + len(samples)) + b'WAVEfmt ' + struct.pack('<IHHIIHH', 16, 1, 1, rate, rate, 1, 8)
    return header + b'data' + struct.pack('<I', len(samples)) + samples


def build_episodes(binary, temp):
    """A garden that turns on podcast-episode in its config.edn, published to temp/site-episodes."""
    garden = temp / 'episodes'
    (garden / 'pages').mkdir(parents=True)
    (garden / 'assets').mkdir()
    (garden / 'logseq').mkdir()
    (garden / 'logseq' / 'config.edn').write_text(
        '{:publishing/all-pages-public? true :lsg/plugins [:podcast-episode]}\n', encoding='utf-8')
    for name, text in EPISODE_PAGES.items():
        (garden / 'pages' / name).write_text(text, encoding='utf-8')
    (garden / 'assets' / 'one.wav').write_bytes(tone(330))
    (garden / 'assets' / 'two.wav').write_bytes(tone(440))
    output = temp / 'site-episodes'
    subprocess.run([str(binary), 'publish', '--graph', str(garden), '--output', str(output),
                    '--filename-format', 'triple-lowbar', '--include', 'Show'],
                   check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    return output


def player(page):
    return page.evaluate('''() => {
      const audio = document.querySelector('.lsg-episode-player audio');
      return audio && { kept: audio.__kept === true, paused: audio.paused, time: audio.currentTime,
        src: audio.getAttribute('src'), title: document.querySelector('.lsg-episode-player-title')?.textContent,
        inline: document.querySelectorAll('main audio').length };
    }''')


def wait_player(page, after=0.0):
    page.wait_for_function('''(after) => {
      const audio = document.querySelector('.lsg-episode-player audio');
      return audio && !audio.paused && audio.currentTime > after;
    }''', arg=after)


def check_podcast_episode(browser, site, prefix, screenshots, results):
    """The first-party podcast-episode plugin: loading, playback across pages, takeover and seeking."""
    one, two, plain = (route(f'Show/{name}') for name in ('One', 'Two', 'Plain'))
    module = 'plugins/podcast-episode/podcast-episode.mjs'

    # A page without an episode, opened directly, requests no plugin file.
    context = browser.new_context()
    page = context.new_page()
    requests = []
    page.on('request', lambda request: requests.append(request.url))
    page.goto(prefix + plain, wait_until='load')
    page.wait_for_function("performance.getEntriesByType('resource').some((r) => r.name.includes('lsg-host-'))")
    page.wait_for_timeout(300)
    assert not [u for u in requests if '/plugins/podcast-episode/' in u], requests
    context.close()
    results['podcast_no_plugin_request_without_episode'] = True

    context = browser.new_context(viewport={'width': 1280, 'height': 800})
    page = context.new_page()
    errors, requests = [], []
    page.on('pageerror', lambda error: errors.append(str(error)))
    page.on('request', lambda request: requests.append(request.url))
    page.goto(prefix + one, wait_until='load')
    page.wait_for_function("document.querySelector('.lsg-episode-player audio')")
    state = player(page)
    assert state['inline'] == 0 and state['src'].endswith('.wav') and state['title'] == 'The first episode', state
    page.evaluate("document.querySelector('.lsg-episode-player audio').__kept = true")
    page.evaluate("document.querySelector('.lsg-episode-player audio').play()")
    wait_player(page, 0.3)
    first_src = player(page)['src']
    if screenshots:
        screenshots.mkdir(parents=True, exist_ok=True)
        page.screenshot(path=str(screenshots / 'podcast-1-episode-one-playing.png'))

    # Following a link and going Back keep the same element playing; the module loads once.
    start = player(page)['time']
    follow(page, plain)
    assert title(page) == 'Show/Plain'
    now = player(page)
    assert now['kept'] and not now['paused'] and now['src'] == first_src, now
    wait_player(page, start + 0.2)
    if screenshots:
        page.screenshot(path=str(screenshots / 'podcast-2-plain-page-still-playing.png'))
    page.go_back()
    page.wait_for_function("document.querySelector('main h1')?.textContent === 'Show/One'")
    back = player(page)
    assert back['kept'] and not back['paused'] and back['time'] > start and back['inline'] == 0, back
    assert len([u for u in requests if u.endswith(module)]) == 1, requests
    results['podcast_playback_across_navigation_and_back'] = {'start_s': round(start, 2), 'after_back_s': round(back['time'], 2)}

    # A dialogue timestamp seeks the player.
    page.locator('main a[data-lsg-dialogue-time="12"]').click()
    page.wait_for_function("document.querySelector('.lsg-episode-player audio').currentTime >= 11.5")
    now = player(page)
    assert now['kept'] and not now['paused'] and now['src'] == first_src, now
    results['podcast_dialogue_seeks_player'] = round(now['time'], 2)

    # A chapter heading's timestamp seeks the player, which shows the chapter and its parent.
    trail = "() => [...document.querySelectorAll('.lsg-episode-player-chapter a')].map((a) => a.textContent).join(' > ')"
    page.wait_for_function(f"({trail})() === 'Opening'")
    seek = page.locator('main h4 a.lsg-episode-seek')
    expect(seek).to_have_text('00:20')
    assert page.locator('main h4').text_content() == '00:20 Twenty seconds in'
    seek.click()
    page.wait_for_function("document.querySelector('.lsg-episode-player audio').currentTime >= 19.5")
    page.wait_for_function(f"({trail})() === 'Opening > Twenty seconds in'")
    now = player(page)
    assert now['kept'] and not now['paused'] and now['src'] == first_src, now
    if screenshots:
        page.screenshot(path=str(screenshots / 'podcast-chapter-heading-seek.png'))
    results['podcast_chapter_heading_seeks_player'] = {'time_s': round(now['time'], 2), 'trail': page.evaluate(trail)}

    # Another episode leaves the playing recording in place until the reader starts it.
    follow(page, two)
    now = player(page)
    assert now['kept'] and not now['paused'] and now['src'] == first_src, now
    button = page.locator('main button.lsg-episode-play')
    expect(button).to_have_count(1)
    button.click()
    page.wait_for_function("(src) => document.querySelector('.lsg-episode-player audio').getAttribute('src') !== src", arg=first_src)
    wait_player(page)
    now = player(page)
    assert now['kept'] and now['time'] < 3 and now['title'] == 'The second episode', now
    assert page.locator('main button.lsg-episode-play').count() == 0
    page.locator('main a[data-lsg-dialogue-time="5"]').click()
    page.wait_for_function("document.querySelector('.lsg-episode-player audio').currentTime >= 4.5")
    if screenshots:
        page.screenshot(path=str(screenshots / 'podcast-3-second-episode-playing.png'))
    results['podcast_second_episode_takes_over_when_started'] = True
    assert not errors, errors
    context.close()

    # When the module fails, the page keeps its own recording and dialogue lines seek it.
    context = browser.new_context()
    page = context.new_page()
    page.route('**/' + module, lambda request: request.abort())
    page.goto(prefix + one, wait_until='load')
    page.wait_for_function("performance.getEntriesByType('resource').some((r) => r.name.includes('lsg-host-'))")
    page.wait_for_timeout(500)
    assert page.locator('.lsg-episode-player').count() == 0
    assert page.locator('main audio').count() == 1
    assert page.locator('main .lsg-episode-seek').count() == 0
    assert page.locator('main h4').text_content() == '00:20 Twenty seconds in'
    page.locator('main a[data-lsg-dialogue-time="12"]').click()
    page.wait_for_function("document.querySelector('main audio').currentTime >= 11.5 && !document.querySelector('main audio').paused")
    context.close()
    results['podcast_module_failure_keeps_inline_recording'] = True


def route(title):
    digest = 0xcbf29ce484222325
    for byte in title.encode('utf-8'):
        digest = ((digest ^ byte) * 0x100000001b3) % (1 << 64)
    return f'p-{digest:016x}.html'


def build(binary, temp, plugins):
    garden = temp / 'garden'
    if not garden.exists():
        (garden / 'pages').mkdir(parents=True)
        (garden / 'logseq').mkdir()
        (garden / 'logseq' / 'config.edn').write_text(
            '{:publishing/all-pages-public? true :publishing/page-properties? true :block-hidden-properties #{:secret}}\n', encoding='utf-8')
        for name, text in PAGES.items():
            (garden / 'pages' / name).write_text(text, encoding='utf-8')
        for name, (activation, module) in GENERATED.items():
            folder = temp / 'plugins' / name
            folder.mkdir(parents=True)
            (folder / f'{name}.mjs').write_text(module, encoding='utf-8')
            (folder / 'lsg-plugin.json').write_text(json.dumps({
                'id': name, 'apiVersion': 1, 'version': '1', 'module': f'{name}.mjs', 'activation': activation}),
                encoding='utf-8')
    output = temp / ('site-' + '-'.join(plugins) if plugins else 'site-plain')
    manifests = []
    for name in plugins:
        folder = FIXTURES / name if (FIXTURES / name).exists() else temp / 'plugins' / name
        manifests += ['--plugin', str(folder / 'lsg-plugin.json')]
    subprocess.run([str(binary), 'publish', '--graph', str(garden), '--output', str(output),
                    '--filename-format', 'triple-lowbar', '--include', 'Episode', *manifests],
                   check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    return output


class Quiet(http.server.SimpleHTTPRequestHandler):
    def log_message(self, *args):
        pass


def serve(root):
    server = http.server.ThreadingHTTPServer(('127.0.0.1', 0), functools.partial(Quiet, directory=str(root)))
    threading.Thread(target=server.serve_forever, daemon=True).start()
    return server, f'http://127.0.0.1:{server.server_address[1]}'


def follow(page, href):
    """Click a same-site link to `href`, as a reader would, and wait for the host to adopt it."""
    before = page.evaluate("performance.getEntriesByName('lsg-host-adopted').length")
    page.evaluate('''(href) => {
      const link = Object.assign(document.createElement('a'), { href, textContent: 'go' });
      document.querySelector('main').append(link);
      link.click();
    }''', href)
    page.wait_for_function("(n) => performance.getEntriesByName('lsg-host-adopted').length > n", arg=before)


def listeners(cdp):
    counts = {}
    for name in ('document', 'window'):
        handle = cdp.send('Runtime.evaluate', {'expression': name})['result']['objectId']
        counts[name] = len(cdp.send('DOMDebugger.getEventListeners', {'objectId': handle})['listeners'])
    return counts


def playing(page):
    return page.evaluate('''() => {
      const audio = document.querySelector('[data-lsg-plugin="fixture-recorder"] audio');
      return audio && { kept: audio.__kept === true, paused: audio.paused, time: audio.currentTime,
        recording: audio.dataset.recording };
    }''')


def wait_playing(page, after=0.0):
    page.wait_for_function('''(after) => {
      const audio = document.querySelector('[data-lsg-plugin="fixture-recorder"] audio');
      return audio && !audio.paused && audio.currentTime > after;
    }''', arg=after)


def title(page):
    return page.locator('main h1').first.inner_text()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', default=str(REPO / 'target' / 'debug' / 'lsg'))
    parser.add_argument('--screenshots', type=Path, help='directory for screenshots of playback across navigation')
    args = parser.parse_args()
    results = {}
    with tempfile.TemporaryDirectory(prefix='lsg-plugins-') as temp:
        temp = Path(temp)
        site = build(args.binary, temp, ['recorder', 'badge', 'probe', 'broken', 'mismatch'])
        plain = build(args.binary, temp, [])
        episodes = build_episodes(args.binary, temp)
        server, base = serve(temp)
        one, two, three, plain_page = (route(f'Episode/{n}') for n in ('One', 'Two', 'Three', 'Plain'))
        prefix = f'{base}/{site.name}/'
        try:
            with sync_playwright() as playwright:
                browser = playwright.chromium.launch(args=['--autoplay-policy=no-user-gesture-required'])

                # Without plugins a page requests no host, plugin or site manifest.
                context = browser.new_context()
                page = context.new_page()
                requests = []
                page.on('request', lambda request: requests.append(request.url))
                page.goto(f'{base}/{plain.name}/{one}', wait_until='load')
                page.wait_for_timeout(300)
                assert not [u for u in requests if 'lsg-host' in u or '/plugins/' in u or 'lsg-site.json' in u], requests
                assert page.locator('[data-lsg-page]').count() == 0
                context.close()
                results['no_plugin_requests'] = 0

                context = browser.new_context(viewport={'width': 1280, 'height': 800})
                page = context.new_page()
                cdp = context.new_cdp_session(page)
                warnings, errors, requests = [], [], []
                page.on('console', lambda message: message.type in ('warning', 'error') and warnings.append(message.text))
                page.on('pageerror', lambda error: errors.append(str(error)))
                page.on('request', lambda request: requests.append(request.url))
                page.goto(prefix + one, wait_until='load')
                page.evaluate("window.__marker = true")

                # Setup failure and a mismatched export: diagnostics, no leftover region, the page works.
                page.wait_for_function("window.__probe && window.__probe.length === 1")
                page.wait_for_function("document.querySelector('[data-lsg-plugin=\"fixture-recorder\"] audio')")
                page.wait_for_timeout(200)
                assert page.locator('[data-lsg-plugin="broken"]').count() == 0
                assert any('lsg plugin broken' in w for w in warnings), warnings
                assert any('lsg plugin mismatch' in w for w in warnings), warnings
                assert not page.evaluate('window.__mismatchSetup === true')
                results['setup_failure_cleanup'] = True
                # Hidden properties stay out of the metadata.
                assert page.evaluate("document.querySelector('[data-lsg-page]').textContent.includes('SECRET')") is False

                # Playback continues across navigation and Back.
                wait_playing(page, 0.3)
                page.evaluate("document.querySelector('[data-lsg-plugin=\"fixture-recorder\"] audio').__kept = true")
                start = playing(page)
                if args.screenshots:
                    wait_playing(page, 1.0)
                    args.screenshots.mkdir(parents=True, exist_ok=True)
                    page.screenshot(path=str(args.screenshots / '1-episode-one-playing.png'))
                # On-demand: the badge plugin is not requested before a page needs it.
                assert not [u for u in requests if 'fixture-badge' in u], requests
                follow(page, two)
                assert title(page) == 'Episode/Two'
                now = playing(page)
                assert now['kept'] and not now['paused'] and now['recording'] == 'one', now
                wait_playing(page, start['time'] + 0.2)
                page.wait_for_function("document.querySelector('main .fixture-badge')")
                if args.screenshots:
                    wait_playing(page, start['time'] + 2.0)
                    page.screenshot(path=str(args.screenshots / '2-episode-two-still-playing.png'))
                assert len([u for u in requests if u.endswith('fixture-badge/badge.mjs')]) == 1, requests
                page.go_back()
                page.wait_for_function("document.querySelector('main h1')?.textContent === 'Episode/One'")
                back = playing(page)
                assert back['kept'] and not back['paused'] and back['time'] > start['time'], (start, back)
                if args.screenshots:
                    wait_playing(page, back['time'] + 1.0)
                    page.screenshot(path=str(args.screenshots / '3-back-to-episode-one-still-playing.png'))
                assert page.evaluate('window.__marker') is True
                results['playback_across_navigation_and_back'] = {'start_s': round(start['time'], 2), 'after_back_s': round(back['time'], 2)}
                follow(page, two)
                assert len([u for u in requests if u.endswith('fixture-badge/badge.mjs')]) == 1, requests
                assert page.locator('main .fixture-badge').count() == 1
                results['on_demand_loaded_once'] = True

                # A response overtaken by a newer navigation is dropped.
                page.evaluate('''(slow) => {
                  const original = window.fetch;
                  window.__restoreFetch = () => { window.fetch = original; };
                  window.fetch = (url, ...rest) => String(url).includes(slow)
                    ? new Promise((done) => setTimeout(done, 600)).then(() => original(url, ...rest))
                    : original(url, ...rest);
                }''', plain_page)
                page.evaluate('''([slow, fast]) => {
                  for (const href of [slow, fast]) {
                    const link = Object.assign(document.createElement('a'), { href });
                    document.querySelector('main').append(link);
                    link.click();
                  }
                }''', [plain_page, one])
                page.wait_for_function("document.querySelector('main h1')?.textContent === 'Episode/One'")
                page.wait_for_timeout(900)
                assert title(page) == 'Episode/One'
                assert page.url.endswith(one)
                page.evaluate('window.__restoreFetch()')
                results['stale_response_dropped'] = True

                # Plugins are notified before the fragment is revealed and focused.
                follow(page, f'{three}#block-{BLOCK}')
                last = page.evaluate('window.__probe.at(-1)')
                assert last['url'] == three and last['revealed'] is False and last['aborted'] is False, last
                assert page.evaluate(f"document.activeElement?.id === 'block-{BLOCK}'")
                assert page.evaluate(f"!document.getElementById('block-{BLOCK}').closest('details:not([open])')")
                results['fragment_after_adoption'] = True
                now = playing(page)
                assert now['kept'] and now['recording'] == 'three' and not now['paused'], now

                # Collapse after navigation: the disclosure arrow works on an adopted page.
                follow(page, one)
                follow(page, three)
                arrow = page.locator('main li.outline-collapsed > .lsg-arrow').first
                expect(arrow).to_have_attribute('aria-expanded', 'false')
                arrow.click()
                expect(arrow).to_have_attribute('aria-expanded', 'true')
                results['collapse_after_navigation'] = True

                # The reader's remembered Properties choice applies to an adopted page. Properties
                # starts collapsed, so open it on one page, then close it, and follow a link each time.
                details = page.locator('main .page-properties > details')
                key = f'lsg:/{site.name}/'
                for choice in (True, False):
                    follow(page, one)
                    assert details.evaluate('(node) => node.open') is not choice
                    page.locator('main #page-properties').click()
                    page.wait_for_function('([key, open]) => JSON.parse(localStorage.getItem(key))?.open === open', arg=[key, choice])
                    follow(page, two)
                    assert details.evaluate('(node) => node.open') is choice, choice
                results['properties_choice_after_navigation'] = True

                # Listener counts stay flat over 20 navigations; page listeners from aborted pages are gone.
                follow(page, one)
                after_one = listeners(cdp)
                adoptions = int(page.evaluate("document.querySelector('[data-lsg-plugin=\"fixture-recorder\"]').dataset.adoptions"))
                for index in range(20):
                    follow(page, [two, three, one, plain_page][index % 4])
                after_twenty = listeners(cdp)
                assert after_twenty == after_one, (after_one, after_twenty)
                assert int(page.evaluate("document.querySelector('[data-lsg-plugin=\"fixture-recorder\"]').dataset.adoptions")) == adoptions + 20
                page.evaluate("document.dispatchEvent(new Event('lsg-fixture-ping'))")
                assert page.evaluate("document.querySelector('[data-lsg-plugin=\"fixture-recorder\"]').dataset.pings") == '1'
                results['listeners_after_20_navigations'] = {'after_1': after_one, 'after_20': after_twenty}
                timings = page.evaluate('''() => {
                  const marks = performance.getEntriesByName('lsg-host-adopted');
                  return marks.map((mark) => {
                    const fetched = performance.getEntriesByType('resource')
                      .filter((r) => r.initiatorType === 'fetch' && new URL(r.name).pathname === mark.detail && r.responseEnd <= mark.startTime)
                      .at(-1);
                    return fetched ? mark.startTime - fetched.responseEnd : null;
                  }).filter((t) => t !== null);
                }''')
                timings.sort()
                results['navigation_ms_desktop'] = {'median': round(timings[len(timings) // 2], 1), 'max': round(timings[-1], 1)}

                # A new build during a visit: keep the page and the recording, offer the new version.
                follow(page, one)
                wait_playing(page)
                page.evaluate('''() => {
                  const original = window.fetch;
                  window.fetch = (url, ...rest) => original(url, ...rest).then((r) => r.text()).then((text) =>
                    new Response(text.replace(/"revision":"[0-9a-f]+"/, '"revision":"0000000000000000"'), { status: 200 }));
                }''')
                page.evaluate('''(href) => {
                  const link = Object.assign(document.createElement('a'), { href });
                  document.querySelector('main').append(link);
                  link.click();
                }''', two)
                expect(page.locator('[data-lsg-host-notice]')).to_be_visible()
                assert title(page) == 'Episode/One'
                now = playing(page)
                assert now['kept'] and not now['paused'], now
                assert page.locator('[data-lsg-host-notice] a').get_attribute('href').endswith(two)
                results['new_build_notice'] = True

                # A failed fetch falls back to an ordinary page load.
                page.evaluate("() => { window.fetch = () => Promise.reject(new TypeError('offline')); }")
                page.evaluate('''(href) => {
                  const link = Object.assign(document.createElement('a'), { href });
                  document.querySelector('main').append(link);
                  link.click();
                }''', three)
                page.wait_for_url('**/' + three)
                page.wait_for_load_state('load')
                assert page.evaluate('window.__marker') is None
                assert title(page) == 'Episode/Three'
                results['fetch_failure_full_load'] = True
                assert not errors, errors
                context.close()
                check_podcast_episode(browser, episodes, f'{base}/{episodes.name}/', args.screenshots, results)
                browser.close()
        finally:
            server.shutdown()
            server.server_close()
        host = next(site.glob('lsg-host-*.mjs'))
        results['host_bytes'] = {'raw': host.stat().st_size, 'gzip': len(gzip.compress(host.read_bytes(), mtime=0))}
    print(json.dumps({'plugin_browser': 'passed', **results}))


if __name__ == '__main__':
    main()
