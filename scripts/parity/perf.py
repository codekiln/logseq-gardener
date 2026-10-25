#!/usr/bin/env python3
# /// script
# requires-python = ">=3.12"
# dependencies = ["playwright==1.60.0"]
# ///
"""Measure load time, data, script, memory and search latency for both builds."""
import argparse
import json
import subprocess
import sys
import tempfile
import time
from datetime import datetime, timezone
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import common  # noqa: E402
import logseq_db  # noqa: E402
import metrics  # noqa: E402
import serve  # noqa: E402

QUERIES = Path(__file__).resolve().parent / 'search-queries.txt'
PERFORMANCE_DOC = common.REPO / 'docs' / 'parity' / 'performance.md'
# Lighthouse's mobile settings: a mid-range phone on a slow 4G network.
PROFILES = {
    'desktop': {'viewport': {'width': 1280, 'height': 800}, 'scale': 1, 'mobile': False,
                'cpu': 1, 'network': None, 'timeout_ms': 120_000},
    'phone': {'viewport': {'width': 412, 'height': 823}, 'scale': 1.75, 'mobile': True,
              'cpu': 4, 'network': {'latency': 150, 'download_kbps': 1638.4, 'upload_kbps': 750},
              'timeout_ms': 900_000},
}
# Search controls per build; None means the build has no search.
SEARCH = {
    'logseq': {'open': '#search-button', 'input': '#search', 'results': '.cp__cmdk'},
    'lsg': None,
}
KEY_GAP_MS = 300
# Journal titles in each build's journals feed, and the number of later batches to load.
FEED_TITLES = {'logseq': '.journal-title', 'lsg': 'a.journal-title'}
FEED_BATCHES = 10
FEED_JOURNALS = 3 + 7 * FEED_BATCHES
# Block zoom: bullets of nested blocks, and the element that shows the zoomed view, per build.
ZOOM_PAGE = 'tmux'
ZOOM = {
    'logseq': ('.block-children .bullet', '.breadcrumb.block-parents'),
    'lsg': ('ul.outline ul .bullet', 'main[data-lsg-zoomed]'),
}
ZOOM_PROBE = '''async ([bullets, done]) => {
  const all = [...document.querySelectorAll(bullets)];
  const bullet = all[Math.floor(all.length / 2)];
  const frame = () => new Promise(resolve => requestAnimationFrame(resolve));
  bullet.scrollIntoView();
  await frame();
  const start = performance.now();
  bullet.click();
  while (!document.querySelector(done)) {
    await frame();
    if (performance.now() - start > 60000) throw new Error('zoomed view did not appear');
  }
  await frame();
  return performance.now() - start;
}'''
# Logseq scrolls #main-content-container; lsg scrolls the window.
SCROLL_TO_BOTTOM = '''() => {
  const box = document.querySelector('#main-content-container');
  if (box) box.scrollTop = box.scrollHeight;
  window.scrollTo(0, document.documentElement.scrollHeight);
}'''

PROBE = '''
(() => {
  performance.setResourceTimingBufferSize(100000);
  const state = window.__parity = {title: null, peak: 0};
  const sample = () => {
    const used = performance.memory && performance.memory.usedJSHeapSize;
    if (used > state.peak) state.peak = used;
  };
  setInterval(sample, 100);
  const check = () => {
    if (state.title !== null) return;
    const element = document.querySelector(%s);
    if (!element || !element.getClientRects().length) return;
    requestAnimationFrame(() => {
      if (state.title !== null) return;
      state.title = performance.now();
      sample();
    });
  };
  new MutationObserver(check).observe(document, {childList: true, subtree: true});
  document.addEventListener('DOMContentLoaded', check);
})();
'''

COLLECT = '''() => {
  const title = window.__parity.title;
  const entries = performance.getEntriesByType('navigation').concat(performance.getEntriesByType('resource'));
  let data = 0, script = 0;
  for (const entry of entries) {
    if (entry.startTime > title) continue;
    data += entry.encodedBodySize;
    if (entry.initiatorType === 'script' || /\\.js(\\?|$)/.test(entry.name)) script += entry.decodedBodySize;
  }
  const paint = performance.getEntriesByName('first-contentful-paint')[0];
  return {title_ms: title, fcp_ms: paint ? paint.startTime : null, data_before_title: data,
          script_before_title: script + [...document.scripts].filter(s => !s.src)
            .reduce((total, s) => total + new Blob([s.textContent]).size, 0), peak_heap: window.__parity.peak};
}'''

SEARCH_PROBE = '''(results) => {
  const state = window.__search = {keys: [], changes: []};
  document.addEventListener('keydown', event => state.keys.push(event.timeStamp), true);
  new MutationObserver(() => state.changes.push(performance.now()))
    .observe(document.querySelector(results), {childList: true, subtree: true, characterData: true});
}'''


def open_page(browser, profile, selector):
    settings = PROFILES[profile]
    context = browser.new_context(viewport=settings['viewport'], device_scale_factor=settings['scale'],
                                  is_mobile=settings['mobile'], has_touch=settings['mobile'], color_scheme='light')
    context.add_init_script(PROBE % json.dumps(selector))
    page = context.new_page()
    cdp = context.new_cdp_session(page)
    cdp.send('Network.enable')
    cdp.send('Network.setCacheDisabled', {'cacheDisabled': True})
    cdp.send('Performance.enable')
    if settings['cpu'] > 1:
        cdp.send('Emulation.setCPUThrottlingRate', {'rate': settings['cpu']})
    network = settings['network']
    if network:
        cdp.send('Network.emulateNetworkConditions', {
            'offline': False, 'latency': network['latency'],
            'downloadThroughput': network['download_kbps'] * 1000 / 8,
            'uploadThroughput': network['upload_kbps'] * 1000 / 8,
        })
    return context, page, cdp


def wait_loaded(page, profile):
    page.wait_for_function('window.__parity && window.__parity.title !== null',
                           timeout=PROFILES[profile]['timeout_ms'], polling=100)
    try:
        page.wait_for_load_state('networkidle', timeout=PROFILES[profile]['timeout_ms'])
    except Exception:  # A page that keeps polling still has a title time worth recording.
        pass
    page.wait_for_timeout(2000)


def heap_now(cdp):
    found = {m['name']: m['value'] for m in cdp.send('Performance.getMetrics')['metrics']}
    return found.get('JSHeapUsedSize', 0)


def measure_load(browser, url, build, profile):
    context, page, cdp = open_page(browser, profile, serve.TITLE_SELECTORS[build])
    try:
        page.goto(url, wait_until='commit', timeout=PROFILES[profile]['timeout_ms'])
        wait_loaded(page, profile)
        result = page.evaluate(COLLECT)
        result['peak_heap'] = max(result['peak_heap'] or 0, heap_now(cdp))
        return result
    except Exception as error:  # Playwright raises its own TimeoutError type.
        return {'error': f'{type(error).__name__}: {str(error).splitlines()[0]}'}
    finally:
        context.close()


def measure_search(browser, url, build, profile, queries):
    controls = SEARCH[build]
    if controls is None:
        return {'note': 'no search'}
    context, page, _ = open_page(browser, profile, serve.TITLE_SELECTORS[build])
    try:
        page.goto(url, wait_until='commit', timeout=PROFILES[profile]['timeout_ms'])
        wait_loaded(page, profile)
        if page.query_selector(controls['open']) is None:
            return {'note': 'no search'}
        page.click(controls['open'])
        page.wait_for_selector(controls['input'], timeout=30_000)
        page.wait_for_selector(controls['results'], timeout=30_000)
        latencies = []
        for query in queries:
            page.fill(controls['input'], '')
            page.wait_for_timeout(1000)
            page.evaluate(SEARCH_PROBE, controls['results'])
            for character in query:
                page.keyboard.type(character)
                page.wait_for_timeout(KEY_GAP_MS)
            page.wait_for_timeout(2000)
            state = page.evaluate('window.__search')
            latencies += metrics.keystroke_latencies(state['keys'], state['changes'])
        measured = [v for v in latencies if v is not None]
        return {'keystrokes': len(latencies), 'unchanged': len(latencies) - len(measured),
                'p50_ms': metrics.percentile(measured, 0.5), 'p95_ms': metrics.percentile(measured, 0.95)}
    except Exception as error:
        return {'error': f'{type(error).__name__}: {str(error).splitlines()[0]}'}
    finally:
        context.close()


def measure_feed(browser, url, build, profile):
    """Scroll the journals feed until it shows ten later batches; record the journals shown and peak heap."""
    context, page, cdp = open_page(browser, profile, serve.TITLE_SELECTORS[build])
    selector = FEED_TITLES[build]
    try:
        page.goto(url, wait_until='commit', timeout=PROFILES[profile]['timeout_ms'])
        wait_loaded(page, profile)
        start = time.monotonic()
        shown = page.locator(selector).count()
        while shown < FEED_JOURNALS:
            page.evaluate(SCROLL_TO_BOTTOM)
            page.mouse.wheel(0, 2000)
            try:
                page.wait_for_function(f'document.querySelectorAll({json.dumps(selector)}).length > {shown}',
                                       timeout=30_000, polling=100)
            except Exception:  # The feed has no later journals.
                break
            shown = page.locator(selector).count()
        elapsed = (time.monotonic() - start) * 1000
        page.wait_for_timeout(1000)
        peak = max(page.evaluate('window.__parity.peak') or 0, heap_now(cdp))
        return {'journals': shown, 'peak_heap': peak, 'scroll_ms': elapsed}
    except Exception as error:
        return {'error': f'{type(error).__name__}: {str(error).splitlines()[0]}'}
    finally:
        context.close()


def measure_zoom(browser, url, build, profile):
    """Click the bullet of a nested block halfway down the page; time until the zoomed view is painted."""
    context, page, _ = open_page(browser, profile, serve.TITLE_SELECTORS[build])
    try:
        page.goto(url, wait_until='commit', timeout=PROFILES[profile]['timeout_ms'])
        wait_loaded(page, profile)
        return {'zoom_ms': page.evaluate(ZOOM_PROBE, list(ZOOM[build]))}
    except Exception as error:
        return {'error': f'{type(error).__name__}: {str(error).splitlines()[0]}'}
    finally:
        context.close()


def measure_build(settings):
    """Seconds to publish the whole garden with and without journals, with the run's lsg options."""
    lsg = common.lsg_binary()
    options = settings['lsg_options']
    rows = []
    with tempfile.TemporaryDirectory(prefix='lsg-build-time-') as temp:
        for journals in (False, True):
            chosen = [o for o in options if o != '--journals'] + (['--journals'] if journals else [])
            output = Path(temp) / ('with' if journals else 'without')
            command = [str(lsg), 'publish', '--graph', settings['garden'], '--output', str(output), *chosen]
            start = time.monotonic()
            subprocess.run(command, check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
            rows.append({'build': 'lsg', 'journals': journals, 'seconds': time.monotonic() - start})
    return rows


def merge(kept, measured, builds, key):
    """Keep earlier rows for builds not measured now, in the order of the new rows."""
    rows = {tuple(r[k] for k in key): r for r in kept if r['build'] not in builds}
    rows.update({tuple(r[k] for k in key): r for r in measured})
    order = {name: index for index, name in enumerate(('logseq', 'lsg'))}
    return sorted(rows.values(), key=lambda r: (*[_position(kept + measured, r, key[:-1])], order[r['build']]))


def _position(rows, row, key):
    for index, other in enumerate(rows):
        if all(other[k] == row[k] for k in key):
            return [index]
    return [len(rows)]


def fmt(field, value):
    if value is None:
        return '–'
    _, divisor, _ = metrics.MEASURES[field]
    scaled = value / divisor
    return f'{scaled:,.0f}' if scaled >= 100 else f'{scaled:,.1f}'


def markdown(results, budgets):
    lines = ['# Parity performance', '']
    lines += [f'- {key}: {value}' for key, value in results['summary'].items()]
    loads = {}
    for row in results['pages']:
        if 'loads' in row:
            loads.setdefault(row['build'], set()).add(row['loads'])
    lines += [f"- loads per page, {build}: {', '.join(map(str, sorted(counts)))}" for build, counts in sorted(loads.items())]
    lines += ['', 'Medians of the page loads. **Bold** marks an `lsg` value over its budget in '
              '[`scripts/parity/budgets.toml`](../../scripts/parity/budgets.toml).', '',
              '## Page loads', '',
              '| Page | Profile | Build | ' + ' | '.join(m[2] for m in metrics.MEASURES.values()) + ' |',
              '| --- | --- | --- | ' + ' | '.join('---:' for _ in metrics.MEASURES) + ' |']
    for row in results['pages']:
        if 'missing' in row or 'error' in row:
            note = 'missing from this build' if 'missing' in row else row['error']
            lines.append(f"| {row['page']} | {row['profile']} | {row['build']} | {note} |" + ' |' * (len(metrics.MEASURES) - 1))
            continue
        cells = []
        for field in metrics.MEASURES:
            text = fmt(field, row[field])
            if metrics.over_budget(row['build'], row['profile'], field, row[field], budgets):
                text = f'**{text}**'
            cells.append(text)
        lines.append(f"| {row['page']} | {row['profile']} | {row['build']} | " + ' | '.join(cells) + ' |')
    feed_limit = budgets.get('journals', {}).get('scroll_heap_mb')
    lines += ['', f'## Journals feed after {FEED_BATCHES} later batches', '',
              f'The home page scrolled until it shows {FEED_JOURNALS} journals or runs out. '
              'The phone heap budget applies to `lsg`.', '',
              '| Profile | Build | Journals shown | Peak JS heap (MB) | Time to load the batches (ms) |',
              '| --- | --- | ---: | ---: | ---: |']
    for row in results.get('feed', []):
        if 'error' in row:
            lines.append(f"| {row['profile']} | {row['build']} | {row['error']} | | |")
            continue
        heap = f"{row['peak_heap'] / 1_000_000:,.1f}"
        if (row['build'] == 'lsg' and row['profile'] == 'phone' and feed_limit is not None
                and row['peak_heap'] > feed_limit * 1_000_000):
            heap = f'**{heap}**'
        lines.append(f"| {row['profile']} | {row['build']} | {row['journals']} | {heap} | {row['scroll_ms']:,.0f} |")
    builds = {row['journals']: row['seconds'] for row in results.get('build', [])}
    if builds:
        slowdown = budgets.get('journals', {}).get('build_slowdown_pct')
        lines += ['', '## Whole-garden build time', '',
                  '`lsg publish` with the run\'s options, once without and once with `--journals`.', '',
                  '| Build | Seconds |', '| --- | ---: |']
        for journals, seconds in sorted(builds.items()):
            text = f'{seconds:,.1f}'
            if (journals and False in builds and slowdown is not None
                    and seconds > builds[False] * (1 + slowdown / 100)):
                text = f'**{text}**'
            lines.append(f"| {'with' if journals else 'without'} journals | {text} |")
    zoom_rows = results.get('zoom', [])
    if zoom_rows:
        lines += ['', f'## Block zoom on {ZOOM_PAGE}', '',
                  'Time from clicking the bullet of a nested block halfway down the page to the zoomed view.', '',
                  '| Profile | Build | Zoom (ms) |', '| --- | --- | ---: |']
        for row in zoom_rows:
            if 'error' in row:
                lines.append(f"| {row['profile']} | {row['build']} | {row['error']} |")
                continue
            text = f"{row['zoom_ms']:,.0f}"
            limit = budgets.get('outline', {}).get(f"zoom_ms_{row['profile']}")
            if row['build'] == 'lsg' and limit is not None and row['zoom_ms'] > limit:
                text = f'**{text}**'
            lines.append(f"| {row['profile']} | {row['build']} | {text} |")
    lines += ['', '## Search latency per keystroke', '',
              '| Profile | Build | Keystrokes | Median (ms) | 95th percentile (ms) |',
              '| --- | --- | ---: | ---: | ---: |']
    for row in results['search']:
        if 'p95_ms' not in row:
            lines.append(f"| {row['profile']} | {row['build']} | {row.get('note') or row.get('error')} | | |")
            continue
        p95 = f"{row['p95_ms']:.0f}" if row['p95_ms'] is not None else '–'
        limit = budgets.get(row['profile'], {}).get('search_p95_ms')
        if row['build'] == 'lsg' and row['p95_ms'] is not None and limit is not None and row['p95_ms'] > limit:
            p95 = f'**{p95}**'
        p50 = f"{row['p50_ms']:.0f}" if row['p50_ms'] is not None else '–'
        lines.append(f"| {row['profile']} | {row['build']} | {row['keystrokes']} | {p50} | {p95} |")
    return '\n'.join(lines) + '\n'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--run', help='run directory from parity:build and parity:compare (default: the latest)')
    parser.add_argument('--fixtures', default=str(serve.FIXTURES), help='fixture list, one title per line')
    parser.add_argument('--profiles', default='desktop,phone', help='comma-separated profiles')
    parser.add_argument('--loads', type=int, default=3, help='loads per page, build and profile')
    parser.add_argument('--builds', default='logseq,lsg',
                        help='comma-separated builds to measure; other builds keep their rows from this run')
    parser.add_argument('--record', action='store_true', help='rewrite docs/parity/performance.md')
    args = parser.parse_args()
    from playwright.sync_api import sync_playwright

    run = common.latest_run(args.run)
    settings = json.loads((run / 'settings.json').read_text(encoding='utf-8'))
    logseq_dir = run / 'logseq'
    if not logseq_dir.exists():
        raise SystemExit('Logseq build missing; run `mise run parity:compare` first')
    names = {common.normalize(p.name) for p in logseq_db.read_pages(logseq_dir / 'static/js/db.gz')}
    lsg_index = common.lsg_index(run / 'site')
    budgets = metrics.load_budgets()
    profiles = [p.strip() for p in args.profiles.split(',') if p.strip()]
    queries = serve.fixtures(QUERIES)
    builds = [b.strip() for b in args.builds.split(',') if b.strip()]
    previous = run / 'perf.json'
    kept = json.loads(previous.read_text(encoding='utf-8')) if previous.exists() else {'pages': [], 'search': []}
    pages, search = [], []
    with serve.Server(logseq_dir) as logseq, serve.Server(run / 'site') as lsg, sync_playwright() as playwright:
        browser = playwright.chromium.launch(args=['--enable-precise-memory-info'])
        servers = {name: server for name, server in (('logseq', logseq), ('lsg', lsg)) if name in builds}
        for name in serve.fixtures(args.fixtures):
            target = serve.targets(name, names, lsg_index, run / 'site')
            for profile in profiles:
                for build, server in servers.items():
                    row = {'page': name, 'profile': profile, 'build': build}
                    if target[build] is None:
                        pages.append({**row, 'missing': True})
                        continue
                    loads = [measure_load(browser, server.url + target[build], build, profile)
                             for _ in range(args.loads)]
                    good = [load for load in loads if 'error' not in load]
                    row.update(metrics.summarize_loads(good) if good else {'error': loads[0]['error']})
                    row['loads'] = len(good)
                    pages.append(row)
                    print(json.dumps(row, ensure_ascii=False), flush=True)
        feed, zoom = [], []
        zoom_target = serve.targets(ZOOM_PAGE, names, lsg_index, run / 'site')
        for profile in profiles:
            for build, server in servers.items():
                home = server.url + serve.LOGSEQ_ROUTES['Home'] if build == 'logseq' else server.url + 'index.html'
                row = {'profile': profile, 'build': build, **measure_search(browser, home, build, profile, queries)}
                search.append(row)
                print(json.dumps(row), flush=True)
                row = {'profile': profile, 'build': build, **measure_feed(browser, home, build, profile)}
                feed.append(row)
                print(json.dumps(row), flush=True)
                if zoom_target[build] is not None:
                    row = {'profile': profile, 'build': build,
                           **measure_zoom(browser, server.url + zoom_target[build], build, profile)}
                    zoom.append(row)
                    print(json.dumps(row), flush=True)
        browser.close()
    pages = merge(kept['pages'], pages, builds, ('page', 'profile', 'build'))
    search = merge(kept['search'], search, builds, ('profile', 'build'))
    feed = merge(kept.get('feed', []), feed, builds, ('profile', 'build'))
    zoom = merge(kept.get('zoom', []), zoom, builds, ('profile', 'build'))
    build_rows = measure_build(settings) if 'lsg' in builds else kept.get('build', [])
    for row in build_rows:
        print(json.dumps(row), flush=True)
    garden = Path(settings['garden'])
    results = {
        'summary': {
            'measured': datetime.now(timezone.utc).strftime('%Y-%m-%d'),
            'garden commit': settings['garden_commit'][:7],
            'gh-pages commit': common.git_commit(garden, 'origin/gh-pages')[:7],
            'lsg': f"{settings['lsg_version']} ({settings['lsg_commit'][:7]})",
            'profiles': ', '.join(profiles),
        },
        'pages': pages,
        'search': search,
        'feed': feed,
        'zoom': zoom,
        'build': build_rows,
    }
    (run / 'perf.json').write_text(json.dumps(results, indent=2, ensure_ascii=False) + '\n', encoding='utf-8')
    text = markdown(results, budgets)
    (run / 'perf.md').write_text(text, encoding='utf-8')
    if args.record:
        PERFORMANCE_DOC.write_text(text, encoding='utf-8')
    print(f"Results: {run / 'perf.md'}")


if __name__ == '__main__':
    main()
