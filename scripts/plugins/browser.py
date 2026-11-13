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
Back, that another episode takes over only when started, that chapter headings, timestamp-led
bullets under `## Chapters` and dialogue timestamps seek the player, which shows the chapter
trail, and that a page whose module fails keeps its own recording. A third garden turns on
sequence-header: the header shows a page's previous and next links, keyboard focus reaches them, a
phone shows arrows alone, the links follow navigation and Back, and a page outside the sequence
requests no plugin code. Another garden turns on ableton-set: a Set page shows one column per track with
the group's track inside the group, each column links to its track page, keyboard focus reaches the
links, the outline stays below, a phone scrolls the tracks and not the page, the dark theme applies,
and a track page or a page that is not a Set requests no plugin code. A fourth garden checks that a video, View as slides, a flashcard review,
the Page graph card and Shift-click into the right sidebar work on a page the host shows in place.
A fifth garden is installable (:lsg/install), published with --readable-urls under the path prefix
/gitpa/ with the probe plugin: the manifest and icon load from a nested page, Chromium parses the
manifest and resolves its start URL and scope to the site folder, reading adds no service worker,
cache or storage entry, the Install app item stays hidden until a test page dispatches
beforeinstallprompt and then opens the prompt from the keyboard, the manifest link survives plugin
navigation, and a page opened from disk raises no script error.
Prints one JSON line of results;
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
import zlib
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
                      '\t\t- ### 00:00 Opening\n\t\t\t- #### 00:20 Twenty seconds in\n'
                      '\t\t\t\t- Notes\n\t\t\t\t\t- 00:25 - the bass comes in\n'
                      '\t\t\t\t\t- The second voice joins at 00:28\n\t\t\t\t\t- `00:29` in code\n\t- Transcript\n'
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
    (garden / 'assets' / 'two.wav').write_bytes(tone(440, seconds=8))
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


def player_labels(page):
    return page.evaluate('''() => {
      const audio = document.querySelector('.lsg-episode-player audio');
      const full = document.querySelector('.lsg-episode-player-recording');
      return { status: document.querySelector('.lsg-episode-player-status').textContent,
        name: audio.getAttribute('aria-label'), full: full.href, src: audio.src, target: full.target,
        rel: full.rel, text: full.textContent,
        current: [...document.querySelectorAll('.lsg-episode-player-chapter a')].map((a) => a.getAttribute('aria-current')) };
    }''')


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
    # The player names its recording, links to it in a new tab and says that it is playing.
    labels = player_labels(page)
    assert labels['status'] == 'Playing' and labels['name'] == 'Full episode recording', labels
    assert labels['full'] == labels['src'] and labels['full'].endswith('.wav') and labels['text'] == 'Full recording', labels
    assert labels['target'] == '_blank' and labels['rel'] == 'noopener noreferrer', labels
    first_full = labels['full']
    results['podcast_player_names_and_links_recording'] = {k: labels[k] for k in ('status', 'name', 'text', 'target')}
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

    # A timestamp-led bullet under a chapter seeks the player, which shows the bullet's chapter
    # with the chapters that enclose it; timestamps later in a bullet or in code stay text.
    bullet = page.locator('main li', has_text='the bass comes in').last
    seek = bullet.locator('a.lsg-episode-seek')
    expect(seek).to_have_text('00:25')
    assert bullet.text_content() == '00:25 - the bass comes in', bullet.text_content()
    assert page.locator('main li', has_text='The second voice joins at').last.locator('a.lsg-episode-seek').count() == 0
    assert page.locator('main li', has_text='in code').last.locator('a.lsg-episode-seek').count() == 0
    seek.click()
    page.wait_for_function("document.querySelector('.lsg-episode-player audio').currentTime >= 24.5")
    page.wait_for_function(f"({trail})() === 'Opening > Twenty seconds in > the bass comes in'")
    now = player(page)
    assert now['kept'] and not now['paused'] and now['src'] == first_src, now
    if screenshots:
        page.screenshot(path=str(screenshots / 'podcast-chapter-bullet-seek.png'))
    results['podcast_chapter_bullet_seeks_player'] = {'time_s': round(now['time'], 2), 'trail': page.evaluate(trail)}
    # Only the chapter playing now, at the end of the trail, is the current one.
    current = player_labels(page)['current']
    assert current == [None, None, 'true'], current
    results['podcast_trail_marks_current_chapter'] = current

    # The status follows the recording: Paused, then Playing again.
    page.evaluate("document.querySelector('.lsg-episode-player audio').pause()")
    page.wait_for_function("document.querySelector('.lsg-episode-player-status').textContent === 'Paused'")
    page.evaluate("document.querySelector('.lsg-episode-player audio').play()")
    page.wait_for_function("document.querySelector('.lsg-episode-player-status').textContent === 'Playing'")
    wait_player(page)

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
    labels = player_labels(page)
    assert labels['full'] == labels['src'] and labels['full'] != first_full, labels
    if screenshots:
        page.screenshot(path=str(screenshots / 'podcast-3-second-episode-playing.png'))
    results['podcast_second_episode_takes_over_when_started'] = True

    # The second recording is 8 seconds long; playing on to its end says Ended.
    page.wait_for_function("document.querySelector('.lsg-episode-player audio').ended")
    page.wait_for_function("document.querySelector('.lsg-episode-player-status').textContent === 'Ended'")
    results['podcast_status_says_ended'] = True
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
    assert page.locator('main audio').get_attribute('aria-label') == 'Episode recording'
    assert page.locator('main .lsg-episode-seek').count() == 0
    assert page.locator('main h4').text_content() == '00:20 Twenty seconds in'
    assert page.locator('main li', has_text='the bass comes in').last.locator('a.lsg-episode-seek').count() == 0
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


SEQUENCE_PAGES = {
    'Seq___One.md': 'next:: [[Seq/Two]]\n\n- The first page. [[Seq/Plain]]\n',
    'Seq___Two.md': 'prev:: [[Seq/One]]\nnext:: [[Seq/Three]]\n\n- The second page. [[Seq/Plain]]\n',
    'Seq___Three.md': 'title:: Seq/Three & <b>bold</b>\nalias:: [[Seq/Three]]\nprev:: [[Seq/Two]]\n\n- The last page.\n',
    'Seq___Plain.md': '- A page outside the sequence. [[Seq/Two]]\n',
}


def build_sequence(binary, temp):
    """A garden that turns on sequence-header in its config.edn, published to temp/site-sequence."""
    garden = temp / 'sequence'
    (garden / 'pages').mkdir(parents=True)
    (garden / 'logseq').mkdir()
    (garden / 'logseq' / 'config.edn').write_text(
        '{:publishing/all-pages-public? true :lsg/plugins [:sequence-header]}\n', encoding='utf-8')
    for name, text in SEQUENCE_PAGES.items():
        (garden / 'pages' / name).write_text(text, encoding='utf-8')
    output = temp / 'site-sequence'
    subprocess.run([str(binary), 'publish', '--graph', str(garden), '--output', str(output),
                    '--filename-format', 'triple-lowbar', '--include', 'Seq'],
                   check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    return output


def header_links(page):
    return page.evaluate('''() => [...document.querySelectorAll('.lsg-header .lsg-sequence')].map((a) => ({
      rel: a.getAttribute('rel'), href: a.getAttribute('href'), label: a.getAttribute('aria-label'),
      title: a.querySelector('.lsg-sequence-title').textContent, markup: a.querySelectorAll('b').length,
      width: Math.round(a.getBoundingClientRect().width),
      shown: getComputedStyle(a.querySelector('.lsg-sequence-title')).clipPath === 'none',
      side: a.compareDocumentPosition(document.querySelector('.lsg-header-space')) & Node.DOCUMENT_POSITION_FOLLOWING ? 'before' : 'after' }))''')


def check_sequence_header(browser, site, prefix, screenshots, results):
    """The first-party sequence-header plugin: header links, keyboard focus, narrow screens and navigation."""
    one, two, three, plain = (route(t) for t in ('Seq/One', 'Seq/Two', 'Seq/Three & <b>bold</b>', 'Seq/Plain'))
    module = 'plugins/sequence-header/sequence-header.mjs'
    loaded = "performance.getEntriesByType('resource').some((r) => r.name.includes('lsg-host-'))"

    # A page outside the sequence, opened directly, requests no plugin file and shows no links.
    context = browser.new_context()
    page = context.new_page()
    requests = []
    page.on('request', lambda request: requests.append(request.url))
    page.goto(prefix + plain, wait_until='load')
    page.wait_for_function(loaded)
    page.wait_for_timeout(300)
    assert not [u for u in requests if '/plugins/sequence-header/' in u], requests
    assert header_links(page) == []
    context.close()
    results['sequence_no_plugin_request_without_entry'] = True

    context = browser.new_context(viewport={'width': 1280, 'height': 800})
    page = context.new_page()
    errors, requests = [], []
    page.on('pageerror', lambda error: errors.append(str(error)))
    page.on('request', lambda request: requests.append(request.url))
    page.goto(prefix + two, wait_until='load')
    page.wait_for_function("document.querySelectorAll('.lsg-header .lsg-sequence').length === 2")
    links = header_links(page)
    assert [(l['rel'], l['href'].rsplit('/', 1)[-1], l['title'], l['side']) for l in links] == [
        ('prev', one, 'Seq/One', 'before'), ('next', three, 'Seq/Three & <b>bold</b>', 'after')], links
    assert links[1]['label'] == 'Next: Seq/Three & <b>bold</b>' and links[1]['markup'] == 0, links
    assert all(l['shown'] for l in links), links
    results['sequence_both_links'] = [l['title'] for l in links]

    # Keyboard focus reaches each link and shows the focus ring.
    focused = set()
    for _ in range(20):
        page.keyboard.press('Tab')
        state = page.evaluate('''() => { const a = document.activeElement;
          return a?.matches('.lsg-sequence') ? [a.rel, a.matches(':focus-visible'), getComputedStyle(a).outlineStyle] : null; }''')
        if state:
            assert state[1] and state[2] == 'solid', state
            focused.add(state[0])
        if focused == {'prev', 'next'}:
            break
    assert focused == {'prev', 'next'}, focused
    results['sequence_keyboard_focus'] = sorted(focused)
    if screenshots:
        screenshots.mkdir(parents=True, exist_ok=True)
        page.screenshot(path=str(screenshots / 'sequence-1-both-links.png'))

    # Following a link to a page outside the sequence removes the links; Back brings them back.
    follow(page, plain)
    assert title(page) == 'Seq/Plain'
    assert header_links(page) == []
    page.go_back()
    page.wait_for_function("document.querySelector('main h1')?.textContent === 'Seq/Two'")
    page.wait_for_function("document.querySelectorAll('.lsg-header .lsg-sequence').length === 2")
    assert [l['rel'] for l in header_links(page)] == ['prev', 'next']
    results['sequence_navigation_and_back'] = True

    # The last page shows only its previous link.
    follow(page, three)
    links = header_links(page)
    assert [(l['rel'], l['href'].rsplit('/', 1)[-1]) for l in links] == [('prev', two)], links
    assert len([u for u in requests if u.endswith(module)]) == 1, requests
    assert not errors, errors
    results['sequence_one_link'] = True
    context.close()

    # On a phone, each link is its arrow alone and the header does not overflow.
    context = browser.new_context(viewport={'width': 375, 'height': 812})
    page = context.new_page()
    page.goto(prefix + two, wait_until='load')
    page.wait_for_function("document.querySelectorAll('.lsg-header .lsg-sequence').length === 2")
    links = header_links(page)
    assert all(l['width'] == 36 and not l['shown'] for l in links), links
    assert page.evaluate("(() => { const h = document.querySelector('.lsg-header'); return h.scrollWidth <= h.clientWidth; })()")
    if screenshots:
        page.screenshot(path=str(screenshots / 'sequence-2-phone.png'))
    context.close()
    results['sequence_phone_arrows'] = True


SET_PAGES = {
    'Live___Set.md': ('logseq-entity:: [[Logseq/Entity/Ableton/Set]]\n\n- # Evening Set\n  live-version:: 11.3.43\n  tempo:: 96\n'
                      '  time-signature:: 4/4\n\t- How the performance was wired. [[Live/Plain]]\n'
                      '\t- {{embed [[Live/Set/Track/Synths]]}}\n\t- {{embed [[Live/Set/Track/Commentary]]}}\n'
                      '\t- {{embed [[Live/Set/Track/A-Reverb]]}}\n'),
    'Live___Set___Track___Synths.md': ('logseq-entity:: [[Logseq/Entity/Ableton/Track/Group]]\n\n- # Synths\n  type:: group\n'
                                       '  audio-to:: AudioOut/Master\n\t- {{embed [[Live/Set/Track/Synths/MicroFreak MIDI]]}}\n'),
    'Live___Set___Track___Synths___MicroFreak MIDI.md': (
        'logseq-entity:: [[Logseq/Entity/Ableton/Track/Track]]\n\n- # MicroFreak MIDI\n  type:: midi\n'
        '  midi-from:: MidiIn/External.Dev:Launchpad Pro MK3 (LPProMK3 MIDI)/0\n'
        '  midi-to:: MidiOut/External.Dev:Arturia MicroFreak/0\n  midi-channel:: 1\n  monitor:: auto\n  arm:: true\n'
        '\t- Carries the sequencer\'s notes to the synthesizer.\n'),
    'Live___Set___Track___Commentary.md': (
        'logseq-entity:: [[Logseq/Entity/Ableton/Track/Track]]\n\n- # Commentary\n  type:: audio\n'
        '  audio-from:: AudioIn/External/S0\n  audio-to:: [[Live/Set/Track/Synths]] Track In\n  monitor:: off\n  arm:: false\n'
        '  devices:: Compressor, EQ <i>Eight</i>, Limiter\n\t- Records the spoken afterword.\n'),
    'Live___Set___Track___A-Reverb.md': ('logseq-entity:: [[Logseq/Entity/Ableton/Track/Track]]\n\n- # A-Reverb\n  type:: return\n'
                                         '  audio-to:: AudioOut/Master\n  devices:: Reverb\n'),
    'Live___Plain.md': '- A page that is not a Set. [[Live/Set]]\n',
}


def build_set(binary, temp):
    """A garden that turns on ableton-set in its config.edn, published to temp/site-set."""
    garden = temp / 'set'
    (garden / 'pages').mkdir(parents=True)
    (garden / 'logseq').mkdir()
    (garden / 'logseq' / 'config.edn').write_text(
        '{:publishing/all-pages-public? true :lsg/plugins [:ableton-set]}\n', encoding='utf-8')
    for name, text in SET_PAGES.items():
        (garden / 'pages' / name).write_text(text, encoding='utf-8')
    output = temp / 'site-set'
    subprocess.run([str(binary), 'publish', '--graph', str(garden), '--output', str(output),
                    '--filename-format', 'triple-lowbar', '--include', 'Live'],
                   check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    return output


def track_view(page):
    """The track view as a reader finds it: each column with its settings, and where the view sits."""
    return page.evaluate('''() => {
      const view = document.querySelector('main .lsg-ableton-set');
      if (!view) return null;
      const column = (item) => {
        const own = (selector) => [...item.querySelectorAll(selector)].filter((node) => node.closest('li.lsg-as-track, li.lsg-as-group') === item);
        const name = own('.lsg-as-name')[0];
        return { title: name.textContent, href: name.href ?? null, group: item.matches('.lsg-as-group'),
          kind: item.dataset.type ?? null, type: own('.lsg-as-type')[0]?.textContent ?? null,
          settings: own('.lsg-as-settings dt').map((dt) => [dt.textContent, dt.nextElementSibling.matches('.lsg-as-monitor')
            ? dt.nextElementSibling.querySelector('strong').textContent : dt.nextElementSibling.textContent]),
          devices: own('.lsg-as-devices > li').map((li) => li.textContent),
          tracks: own('.lsg-as-tracks')[0] ? [...own('.lsg-as-tracks')[0].children].map(column) : null };
      };
      const scroller = view.querySelector('.lsg-as-scroll');
      return { count: document.querySelectorAll('.lsg-ableton-set').length, label: view.getAttribute('aria-label'),
        caption: view.querySelector('.lsg-as-caption').textContent,
        tracks: [...scroller.querySelector('.lsg-as-tracks').children].map(column),
        outline_after: view.nextElementSibling?.matches('.outline') ?? false,
        outline_text: document.querySelector('main .outline').textContent,
        markup: view.querySelectorAll('i').length,
        page_overflow: document.documentElement.scrollWidth - document.documentElement.clientWidth,
        scroll: [scroller.scrollWidth, scroller.clientWidth],
        colors: [getComputedStyle(view.querySelector('.lsg-as-track')).backgroundColor,
                 getComputedStyle(view.querySelector('.lsg-as-track')).color] };
    }''')


def luminance(color):
    """The relative luminance of a computed `rgb(r, g, b)` colour."""
    channels = [int(part) / 255 for part in color[color.index('(') + 1:color.index(')')].split(',')[:3]]
    red, green, blue = (c / 12.92 if c <= 0.03928 else ((c + 0.055) / 1.055) ** 2.4 for c in channels)
    return 0.2126 * red + 0.7152 * green + 0.0722 * blue


def check_ableton_set(browser, site, prefix, screenshots, results):
    """The first-party ableton-set plugin: the track view of a Set, its links, narrow screens, the dark theme and navigation."""
    set_page, group, midi, commentary, reverb, plain = (route(t) for t in (
        'Live/Set', 'Live/Set/Track/Synths', 'Live/Set/Track/Synths/MicroFreak MIDI', 'Live/Set/Track/Commentary',
        'Live/Set/Track/A-Reverb', 'Live/Plain'))
    module = 'plugins/ableton-set/ableton-set.mjs'
    loaded = "performance.getEntriesByType('resource').some((r) => r.name.includes('lsg-host-'))"

    # A page that is not a Set, and a track page, opened directly: no plugin file and no view.
    for target in (plain, commentary):
        context = browser.new_context()
        page = context.new_page()
        requests = []
        page.on('request', lambda request: requests.append(request.url))
        page.goto(prefix + target, wait_until='load')
        page.wait_for_function(loaded)
        page.wait_for_timeout(300)
        assert not [u for u in requests if '/plugins/ableton-set/' in u], requests
        assert track_view(page) is None
        context.close()
    results['set_no_plugin_request_without_entry'] = True

    context = browser.new_context(viewport={'width': 1280, 'height': 800}, color_scheme='light')
    page = context.new_page()
    errors, requests = [], []
    page.on('pageerror', lambda error: errors.append(str(error)))
    page.on('request', lambda request: requests.append(request.url))
    page.goto(prefix + set_page, wait_until='load')
    page.wait_for_selector('main .lsg-ableton-set')
    view = track_view(page)
    assert view['count'] == 1 and view['label'] == 'Track view of Evening Set', view
    assert view['caption'] == 'Track view · Live 11.3.43 · 96 BPM · 4/4 · 4 tracks', view['caption']
    # One column per track, the group's track inside the group, each linked to its page.
    assert view['tracks'] == [
        {'title': 'Synths', 'href': prefix + group, 'group': True, 'kind': 'group', 'type': 'Group',
         'settings': [['Audio to', 'AudioOut/Master']], 'devices': [], 'tracks': [
             {'title': 'MicroFreak MIDI', 'href': prefix + midi, 'group': False, 'kind': 'midi', 'type': 'MIDI',
              'settings': [['MIDI from', 'MidiIn/External.Dev:Launchpad Pro MK3 (LPProMK3 MIDI)/0'], ['MIDI channel', '1'],
                           ['Monitor', 'Auto'], ['MIDI to', 'MidiOut/External.Dev:Arturia MicroFreak/0'], ['Arm', 'Armed']],
              'devices': [], 'tracks': None}]},
        {'title': 'Commentary', 'href': prefix + commentary, 'group': False, 'kind': 'audio', 'type': 'Audio',
         'settings': [['Audio from', 'AudioIn/External/S0'], ['Monitor', 'Off'], ['Audio to', 'Synths Track In'], ['Arm', 'Off']],
         'devices': ['Compressor', 'EQ <i>Eight</i>', 'Limiter'], 'tracks': None},
        {'title': 'A-Reverb', 'href': prefix + reverb, 'group': False, 'kind': 'return', 'type': 'Return',
         'settings': [['Audio to', 'AudioOut/Master']], 'devices': ['Reverb'], 'tracks': None}], view['tracks']
    assert view['markup'] == 0, view
    # A routing that names another track of the Set links to that track's page.
    targets = page.evaluate('''() => [...document.querySelectorAll('.lsg-ableton-set a.lsg-as-target')].map((a) => [a.textContent, a.href])''')
    assert targets == [['Synths', prefix + group]], targets
    # The outline stays below the view, with the embedded track pages in it.
    assert view['outline_after'] and 'Records the spoken afterword.' in view['outline_text'], view
    assert view['page_overflow'] <= 0, view
    assert luminance(view['colors'][0]) > 0.7 > 0.2 > luminance(view['colors'][1]), view['colors']
    results['set_columns'] = [track['title'] for track in view['tracks']]

    # Keyboard focus reaches each track link and shows the focus ring.
    focused = []
    for _ in range(40):
        page.keyboard.press('Tab')
        state = page.evaluate('''() => { const a = document.activeElement;
          return a?.matches('a.lsg-as-name') ? [a.textContent, a.matches(':focus-visible'), getComputedStyle(a).outlineStyle] : null; }''')
        if state:
            assert state[1] and state[2] == 'solid', state
            focused.append(state[0])
        if len(focused) == 4:
            break
    assert focused == ['Synths', 'MicroFreak MIDI', 'Commentary', 'A-Reverb'], focused
    results['set_keyboard_focus'] = focused
    if screenshots:
        screenshots.mkdir(parents=True, exist_ok=True)
        page.screenshot(path=str(screenshots / 'set-1-desktop-light.png'))

    # A column's link opens the track page, an ordinary page without a view; Back shows the view once.
    page.locator('.lsg-ableton-set a.lsg-as-name', has_text='Commentary').click()
    page.wait_for_function("document.querySelector('main h1')?.textContent === 'Live/Set/Track/Commentary'")
    assert track_view(page) is None
    assert 'Records the spoken afterword.' in page.locator('main').inner_text()
    page.go_back()
    page.wait_for_function("document.querySelector('main h1')?.textContent === 'Live/Set'")
    page.wait_for_selector('main .lsg-ableton-set')
    assert track_view(page)['count'] == 1
    assert len([u for u in requests if u.endswith(module)]) == 1, requests
    assert not errors, errors
    context.close()
    results['set_navigation_and_back'] = True

    # On a phone the page does not scroll sideways; the tracks do. The dark theme darkens the columns.
    context = browser.new_context(viewport={'width': 375, 'height': 812}, color_scheme='dark')
    page = context.new_page()
    page.goto(prefix + set_page, wait_until='load')
    page.wait_for_selector('main .lsg-ableton-set')
    view = track_view(page)
    assert view['page_overflow'] <= 0, view
    assert view['scroll'][0] > view['scroll'][1], view
    assert luminance(view['colors'][0]) < 0.2 < 0.7 < luminance(view['colors'][1]), view['colors']
    if screenshots:
        page.screenshot(path=str(screenshots / 'set-2-phone-dark.png'))
    context.close()
    results['set_phone_and_dark'] = view['scroll']


FEATURE_PAGES = {
    'Feat___Home.md': '- Home. [[Feat/Video]] [[Feat/Slides]] [[Feat/Cards]] [[Feat/Plain]]\n',
    'Feat___Video.md': '- {{video https://www.youtube.com/watch?v=dQw4w9WgXcQ}}\n- At {{youtube-timestamp 30}} it starts. [[Feat/Home]]\n',
    'Feat___Slides.md': '- First slide [[Feat/Home]]\n- Second slide\n\t- Child slide\n',
    'Feat___Cards.md': '- What is 2+2? #card\n\t- 4\n- {{cards}}\n- [[Feat/Home]]\n',
    'Feat___Plain.md': '- Plain page. [[Feat/Home]] [[Feat/Video]]\n',
}


def build_features(binary, temp):
    """A garden of pages that use lsg's page features, with the probe plugin, published to temp/site-features."""
    garden = temp / 'features'
    (garden / 'pages').mkdir(parents=True)
    (garden / 'logseq').mkdir()
    (garden / 'logseq' / 'config.edn').write_text('{:publishing/all-pages-public? true}\n', encoding='utf-8')
    for name, text in FEATURE_PAGES.items():
        (garden / 'pages' / name).write_text(text, encoding='utf-8')
    output = temp / 'site-features'
    subprocess.run([str(binary), 'publish', '--graph', str(garden), '--output', str(output),
                    '--filename-format', 'triple-lowbar', '--include', 'Feat',
                    '--plugin', str(temp / 'plugins' / 'probe' / 'lsg-plugin.json')],
                   check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    return output


def check_page_features(browser, prefix, results):
    """Media, slides, flashcards, the Page graph card and the right sidebar on pages the host shows in place."""
    home, video, slides, cards, plain = (route(f'Feat/{name}') for name in ('Home', 'Video', 'Slides', 'Cards', 'Plain'))
    context = browser.new_context(viewport={'width': 1280, 'height': 800})
    # The video provider stays offline; the player only needs its iframe.
    context.route('**/*', lambda r: r.continue_() if r.request.url.startswith(prefix) else r.abort())
    page = context.new_page()
    errors, requests = [], []
    page.on('pageerror', lambda error: errors.append(str(error)))
    page.on('request', lambda request: requests.append(request.url))
    page.goto(prefix + home, wait_until='load')
    page.wait_for_function('window.__probe && window.__probe.length === 1')
    page.evaluate('window.__featureMarker = true')

    def visit(target):
        follow(page, plain)
        follow(page, target)

    # Media: Play loads the player, and a timestamp loads it at that time.
    visit(video)
    page.locator('main .lsg-video-play').click()
    expect(page.locator('main .lsg-video iframe')).to_have_count(1)
    visit(video)
    page.locator('main a.youtube-timestamp').click()
    assert 'start=30' in page.locator('main .lsg-video iframe').get_attribute('src')
    results['features_media_after_navigation'] = True

    # Slides: View as slides shows this page's blocks, and the arrow keys move through them.
    visit(slides)
    page.locator('[data-lsg-more]').click()
    page.locator('[data-lsg-slides]').click()
    present = page.locator('.lsg-right-sidebar .reveal section.present')
    expect(present).to_have_text('First slide Feat/Home')
    page.keyboard.press('ArrowRight')
    expect(present).to_have_text('Second slide')
    page.locator('.lsg-right-sidebar .lsg-card-close').click()
    results['features_slides_after_navigation'] = True

    # Flashcards: the deck opens a review, and its keys show the answer.
    visit(cards)
    page.locator('main [data-lsg-deck]').first.click()
    back = page.locator('.lsg-review-card .lsg-card-back')
    expect(back).to_have_count(1)
    expect(back).to_be_hidden()
    page.keyboard.press('s')
    expect(back).to_be_visible()
    page.keyboard.press('Escape')
    expect(page.locator('.lsg-review-shade')).to_have_count(0)
    results['features_flashcards_after_navigation'] = True

    # The Page graph card draws each page the host shows.
    visit(home)
    page.locator('[data-lsg-right-toggle]').click()
    page.locator('.lsg-right-bar [data-add=graph]').click()
    neighbourhood = lambda target: f'/_graph/p/{target[2:18]}.json'
    page.wait_for_function('document.querySelector(".lsg-page-graph canvas")')
    page.wait_for_function('(file) => performance.getEntriesByType("resource").some((r) => r.name.endsWith(file))', arg=neighbourhood(home))
    visit(video)
    page.wait_for_function('(file) => performance.getEntriesByType("resource").some((r) => r.name.endsWith(file))', arg=neighbourhood(video))
    expect(page.locator('.lsg-page-graph')).to_have_count(1)
    results['features_page_graph_after_navigation'] = [u.rsplit('/', 1)[-1] for u in requests if '/_graph/p/' in u]

    # The right sidebar: Shift-click on a page link adds that page's card.
    page.locator('main .page-ref', has_text='Feat/Home').first.click(modifiers=['Shift'])
    expect(page.locator('.lsg-right-sidebar .lsg-card-title').first).to_have_text('Feat/Home')
    results['features_sidebar_after_navigation'] = True

    # Every step stayed in the first document.
    assert page.evaluate('window.__featureMarker === true')
    assert not errors, errors
    context.close()


INSTALL_PAGES = {
    'Inst___One.md': '- The first page. See [[Inst/Deep/Down/Two]] and [[Inst/Three]].\n',
    'Inst___Deep___Down___Two.md': '- A nested page. Back to [[Inst/One]].\n',
    'Inst___Three.md': '- A third page. [[Inst/One]]\n',
}
INSTALL_PREFIX = 'gitpa'


def solid_png(side):
    """A solid-colour square RGB PNG, which is small once compressed."""
    def chunk(kind, data):
        body = kind + data
        return struct.pack('>I', len(data)) + body + struct.pack('>I', zlib.crc32(body))
    row = b'\x00' + bytes([0x2f, 0x6f, 0x4f]) * side
    return (b'\x89PNG\r\n\x1a\n' + chunk(b'IHDR', struct.pack('>IIBBBBB', side, side, 8, 2, 0, 0, 0))
            + chunk(b'IDAT', zlib.compress(row * side, 9)) + chunk(b'IEND', b''))


def build_installable(binary, temp):
    """An installable garden with the probe plugin, published readable to temp/served/gitpa, and the
    same garden without the key, published to temp/served/plain, and without the plugin, to temp/served/bare."""
    garden = temp / 'installable'
    (garden / 'pages').mkdir(parents=True)
    (garden / 'assets').mkdir()
    (garden / 'logseq').mkdir()
    (garden / 'logseq' / 'config.edn').write_text(
        '{:publishing/all-pages-public? true :lsg/install {:name "Inst Garden" :icon "assets/app-icon.png"}}\n',
        encoding='utf-8')
    (garden / 'assets' / 'app-icon.png').write_bytes(solid_png(512))
    for name, text in INSTALL_PAGES.items():
        (garden / 'pages' / name).write_text(text, encoding='utf-8')
    served = temp / 'served'
    served.mkdir()
    config = '{:publishing/all-pages-public? true :lsg/install {:name "Inst Garden" :icon "assets/app-icon.png"}}\n'
    probe = ['--plugin', str(temp / 'plugins' / 'probe' / 'lsg-plugin.json')]
    # The installable site with the probe plugin, the same site without the key, and the installable
    # site without a plugin, which a page opened from disk can load without an error.
    for name, key, plugin in ((INSTALL_PREFIX, True, probe), ('plain', False, probe), ('bare', True, [])):
        (garden / 'logseq' / 'config.edn').write_text(
            config if key else '{:publishing/all-pages-public? true}\n', encoding='utf-8')
        subprocess.run([str(binary), 'publish', '--graph', str(garden), '--output', str(served / name),
                        '--filename-format', 'triple-lowbar', '--include', 'Inst', '--readable-urls', *plugin],
                       check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    return served


def storage_state(page):
    """Everything a page's origin holds that a reader's browser keeps: service workers, caches and storage."""
    return page.evaluate('''async () => ({
      workers: (await navigator.serviceWorker.getRegistrations()).length,
      caches: await caches.keys(),
      local: Object.keys(localStorage), session: Object.keys(sessionStorage),
      databases: (await indexedDB.databases()).map((d) => d.name),
      cookies: document.cookie })''')


def check_installable(browser, served, base, root_base, results):
    """An installable garden under a path prefix: manifest, icon, scope, storage, the item and navigation."""
    prefix = f'{base}/{INSTALL_PREFIX}/'
    nested = 'Inst/Deep/Down/Two/'
    context = browser.new_context(viewport={'width': 1280, 'height': 800})
    page = context.new_page()
    cdp = context.new_cdp_session(page)
    errors, requests, responses = [], [], {}
    page.on('pageerror', lambda error: errors.append(str(error)))
    page.on('request', lambda request: requests.append(request.url))
    page.on('response', lambda response: responses.__setitem__(response.url, response.status))
    page.goto(prefix + nested, wait_until='load')
    page.wait_for_function('window.__probe && window.__probe.length === 1')

    # The manifest and the icon load from a nested page, through <base>, under the prefix.
    assert page.locator('link[rel=manifest]').get_attribute('href') == 'lsg.webmanifest'
    page.wait_for_timeout(500)
    manifest_url, icon_url = prefix + 'lsg.webmanifest', prefix + 'lsg-icon.png'
    manifest_response = page.request.get(manifest_url)
    assert manifest_response.status == 200 and page.request.get(icon_url).status == 200
    document = manifest_response.json()
    assert 'id' not in document and document['start_url'] == './' and document['scope'] == './', document
    results['install_manifest_and_icon_status'] = {'manifest': manifest_response.status, 'icon': 200}

    # Chromium's own parse: no errors, and the start URL and scope are the site folder.
    parsed = cdp.send('Page.getAppManifest')
    assert not [e for e in parsed.get('errors', []) if e.get('critical')], parsed
    data = json.loads(parsed['data'])
    assert parsed['url'] == manifest_url, parsed['url']
    assert data['name'] == 'Inst Garden' and data['short_name'] == 'Inst Garden', data
    resolved = {key: str(__import__('urllib.parse').parse.urljoin(parsed['url'], data[key])) for key in ('start_url', 'scope')}
    assert resolved == {'start_url': prefix, 'scope': prefix}, resolved
    assert data['icons'][0]['sizes'] == '512x512', data
    installability = cdp.send('Page.getInstallabilityErrors')['installabilityErrors']
    results['install_manifest_resolves_to'] = prefix
    results['install_chromium_installability_errors'] = [e['errorId'] for e in installability]

    # The same site served at the root of an origin: the same manifest bytes resolve to the root.
    root = context.new_page()
    root_cdp = context.new_cdp_session(root)
    root.goto(f'{root_base}/{nested}', wait_until='load')
    root_manifest = root.request.get(f'{root_base}/lsg.webmanifest')
    assert root_manifest.status == 200 and root.request.get(f'{root_base}/lsg-icon.png').status == 200
    assert root_manifest.body() == manifest_response.body()
    root_parsed = root_cdp.send('Page.getAppManifest')
    root_data = json.loads(root_parsed['data'])
    urljoin = __import__('urllib.parse').parse.urljoin
    assert urljoin(root_parsed['url'], root_data['start_url']) == f'{root_base}/', root_parsed
    assert urljoin(root_parsed['url'], root_data['scope']) == f'{root_base}/', root_parsed
    root.close()
    results['install_root_served_resolves_to'] = f'{root_base}/'

    # Three navigations later: no service worker, cache entry or storage entry.
    follow(page, 'Inst/Three/')
    follow(page, 'Inst/One/')
    follow(page, nested)
    state = storage_state(page)
    assert state == {'workers': 0, 'caches': [], 'local': [], 'session': [], 'databases': [], 'cookies': ''}, state
    # A page of the same garden without the key holds exactly the same.
    plain_page = context.new_page()
    plain_page.goto(f'{base}/plain/{nested}', wait_until='load')
    assert storage_state(plain_page) == state
    assert plain_page.locator('link[rel=manifest]').count() == 0 and plain_page.locator('[data-lsg-install]').count() == 0
    plain_page.close()
    assert not [u for u in requests if 'service-worker' in u or '/sw.js' in u], requests
    results['install_no_worker_cache_or_storage'] = state

    # The manifest link survives navigation through the plugin host, and so does Chromium's manifest.
    assert page.locator('link[rel=manifest]').count() == 1
    assert cdp.send('Page.getAppManifest')['url'] == manifest_url
    page.evaluate("window.__installMarker = true")
    follow(page, 'Inst/Three/')
    assert page.evaluate('window.__installMarker') is True
    assert page.locator('link[rel=manifest]').get_attribute('href') == 'lsg.webmanifest'
    assert cdp.send('Page.getAppManifest')['url'] == manifest_url
    results['install_manifest_link_after_plugin_navigation'] = True

    # The Install app item stays hidden until the browser offers installation.
    page.locator('[data-lsg-more]').click()
    item = page.locator('[data-lsg-install]')
    expect(item).to_be_hidden()
    page.keyboard.press('Escape')
    page.evaluate('''() => {
      window.__prompts = 0;
      const event = Object.assign(new Event('beforeinstallprompt', { cancelable: true }),
        { prompt() { window.__prompts += 1; return Promise.resolve({ outcome: 'dismissed' }); } });
      window.__offerPrevented = !window.dispatchEvent(event);
    }''')
    assert page.evaluate('window.__offerPrevented') is True
    page.locator('[data-lsg-more]').focus()
    page.keyboard.press('Enter')
    expect(item).to_be_visible()
    assert page.evaluate("document.querySelector('#lsg-more-menu [role=menuitem]:not([hidden])') !== null")
    # From the keyboard: arrow to the item, activate it, and focus returns to the More button.
    for _ in range(5):
        if page.evaluate("document.activeElement?.hasAttribute('data-lsg-install')"):
            break
        page.keyboard.press('ArrowDown')
    assert page.evaluate("document.activeElement?.hasAttribute('data-lsg-install')")
    assert page.evaluate("getComputedStyle(document.activeElement).outlineStyle !== 'none' || getComputedStyle(document.activeElement).backgroundColor !== 'rgba(0, 0, 0, 0)'")
    page.keyboard.press('Enter')
    assert page.evaluate('window.__prompts') == 1
    expect(item).to_be_hidden()
    assert page.evaluate("document.activeElement?.hasAttribute('data-lsg-more')")
    page.keyboard.press('Enter')
    expect(item).to_be_hidden()
    assert page.evaluate('window.__prompts') == 1
    results['install_item_from_keyboard'] = True
    assert not errors, errors
    context.close()

    # Opened from disk: renders, a link opens its target, no Install app item, and no more errors than
    # without the key. A page that loads the plugin host from disk reports the browser's refusal to
    # import a module from a file, with or without the key, so the garden without a plugin is the one
    # that must raise none.
    def from_disk(site, follow_link=True):
        context = browser.new_context()
        page = context.new_page()
        errors, console = [], []
        page.on('pageerror', lambda error: errors.append(str(error)))
        page.on('console', lambda message: message.type == 'error' and console.append(message.text))
        page.goto((served / site / 'index.html').as_uri(), wait_until='load')
        page.wait_for_timeout(500)
        expect(page.locator('main h1').first).to_be_visible()
        installable = page.locator('[data-lsg-install]').count()
        page.locator('[data-lsg-more]').click()
        if installable:
            expect(page.locator('[data-lsg-install]')).to_be_hidden()
        page.keyboard.press('Escape')
        if follow_link:
            page.locator('main a.page-ref').first.click()
            page.wait_for_load_state('load')
            expect(page.locator('main h1').first).to_be_visible()
        context.close()
        return installable, errors, console

    installable, errors, console = from_disk('bare')
    assert installable == 1 and not errors, (installable, errors)
    results['install_from_disk'] = {'script_errors': 0, 'console_errors': console}
    keyed, keyed_errors, _ = from_disk(INSTALL_PREFIX, follow_link=False)
    plain, plain_errors, _ = from_disk('plain', follow_link=False)
    assert (keyed, plain) == (1, 0) and [e.replace(INSTALL_PREFIX, 'plain') for e in keyed_errors] == plain_errors, \
        (keyed_errors, plain_errors)
    results['install_from_disk_with_plugin_host_same_errors_as_without_key'] = len(plain_errors)


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
        sequence = build_sequence(args.binary, temp)
        live_set = build_set(args.binary, temp)
        features = build_features(args.binary, temp)
        installable = build_installable(args.binary, temp)
        server, base = serve(temp)
        install_server, install_base = serve(installable)
        root_server, root_base = serve(installable / INSTALL_PREFIX)
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
                check_sequence_header(browser, sequence, f'{base}/{sequence.name}/', args.screenshots, results)
                check_ableton_set(browser, live_set, f'{base}/{live_set.name}/', args.screenshots, results)
                check_page_features(browser, f'{base}/{features.name}/', results)
                check_installable(browser, installable, install_base, root_base, results)
                browser.close()
        finally:
            for running in (server, install_server, root_server):
                running.shutdown()
                running.server_close()
        host = next(site.glob('lsg-host-*.mjs'))
        results['host_bytes'] = {'raw': host.stat().st_size, 'gzip': len(gzip.compress(host.read_bytes(), mtime=0))}
    print(json.dumps({'plugin_browser': 'passed', **results}))


if __name__ == '__main__':
    main()
