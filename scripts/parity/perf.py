#!/usr/bin/env python3
# /// script
# requires-python = ">=3.12"
# dependencies = ["playwright==1.60.0"]
# ///
"""Measure load time, data, script, memory and search latency for both builds."""
import argparse
import contextlib
import gzip
import json
import math
import re
import shutil
import subprocess
import sys
import tempfile
import time
from datetime import datetime, timezone
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import budgets as budgets_check  # noqa: E402
import common  # noqa: E402
import logseq_db  # noqa: E402
import metrics  # noqa: E402
import serve  # noqa: E402

QUERIES = Path(__file__).resolve().parent / 'search-queries.txt'
PERFORMANCE_DOC = common.REPO / 'docs' / 'parity' / 'performance.md'
SEARCH_DOC = common.REPO / 'docs' / 'parity' / 'search-performance.json'
# Lighthouse's mobile settings: a mid-range phone on a slow 4G network.
PROFILES = {
    'desktop': {'viewport': {'width': 1280, 'height': 800}, 'scale': 1, 'mobile': False,
                'cpu': 1, 'network': None, 'timeout_ms': 120_000},
    'phone': {'viewport': {'width': 412, 'height': 823}, 'scale': 1.75, 'mobile': True,
              'cpu': 4, 'network': {'latency': 150, 'download_kbps': 1638.4, 'upload_kbps': 750},
              'timeout_ms': 900_000},
}
# Search controls per build; None means the build has no search. `visible` is the open dialog;
# `pages` is an expression that is true once Logseq's dialog lists a Pages group.
SEARCH = {
    'logseq': {'open': '#search-button', 'input': '#search', 'results': '.cp__cmdk', 'visible': '.cp__cmdk',
               'pages': "[...document.querySelectorAll('.cp__cmdk .font-bold')]"
                        ".some(e => /^pages$/i.test(e.textContent.trim()))"},
    'lsg': {'open': '[data-lsg-search]', 'input': '#lsg-search-input', 'results': '#lsg-search-results',
            'visible': '#lsg-search-dialog[open]',
            'ready': '#lsg-search-dialog[open][data-lsg-search-state=ready]',
            'complete': 'data-lsg-search-complete', 'drawn': 'data-lsg-search-drawn'},
}
# Files that lsg's search fetches; a keystroke that requests a shard is a cold keystroke.
SEARCH_FILES = re.compile(r'lsg-search-(manifest|titles|shard|chunk|engine|ui)-')
SHARD_FILE = re.compile(r'lsg-search-shard-')
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
# Hierarchy: the page whose long Hierarchy list is scrolled to the end.
HIERARCHY_PAGE = 'Microfreak'
HIERARCHY_ROWS = 'document.querySelectorAll(".page-hierarchy .namespaces > li").length'
# Reference sections: the page whose sections are opened, filtered and scrolled to the end.
REFERENCES_PAGE = 'mise'
REFERENCES_OPEN = '''async (kind) => {
  const section = document.querySelector(`[data-refs=${kind}]`);
  const start = performance.now();
  section.querySelector('summary').click();
  while (!section.querySelector('.references-blocks-item') && !/^0 /.test(section.querySelector('h2').textContent)) {
    if (performance.now() - start > 60000) throw new Error(`${kind} references did not load`);
    await new Promise(resolve => requestAnimationFrame(resolve));
  }
  return performance.now() - start;
}'''
REFERENCES_FILTER = '''async () => {
  const section = document.querySelector('[data-refs=linked]');
  const heading = section.querySelector('h2');
  section.querySelector('.filter').click();
  for (let i = 0; !section.querySelector('dialog[open] .chip'); i++) {
    if (i > 500) throw new Error('the filter dialog did not open');
    await new Promise(resolve => setTimeout(resolve, 20));
  }
  const before = heading.textContent;
  const start = performance.now();
  // Removing a starting filter always changes the count; a new include might not.
  section.querySelector('dialog .chip').click();
  while (heading.textContent === before) {
    if (performance.now() - start > 10000) throw new Error('the filter change did not change the count');
    await new Promise(resolve => requestAnimationFrame(resolve));
  }
  await new Promise(resolve => requestAnimationFrame(resolve));
  const elapsed = performance.now() - start;
  section.querySelector('dialog').close();
  return elapsed;
}'''
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

SEARCH_PROBE = '''(controls) => {
  window.__search?.dispose?.();
  const state = window.__search = {keys: [], changes: [], completions: []};
  const keydown = event => state.keys.push(event.timeStamp);
  document.addEventListener('keydown', keydown, true);
  const input = () => document.querySelector(controls.input);
  let observer;
  if (controls.drawn) {
    // lsg advances the drawn attribute each time it draws the result list for the input, with a
    // placeholder line in block rows whose chunk is still loading; record what it showed.
    observer = new MutationObserver(() => state.completions.push([performance.now(), input()?.value ?? '']));
    observer.observe(document, {attributes: true, subtree: true, attributeFilter: [controls.drawn]});
  } else {
    observer = new MutationObserver(() => state.changes.push(performance.now()));
    observer.observe(document.querySelector(controls.results), {childList: true, subtree: true, characterData: true});
  }
  state.dispose = () => { document.removeEventListener('keydown', keydown, true); observer.disconnect(); };
}'''
SEARCH_COLLECT = '''() => {
  const state = window.__search;
  const first = state.keys.length ? state.keys[0] : performance.now();
  const resources = performance.getEntriesByType('resource').filter(e => e.startTime >= first)
    .map(e => [e.startTime, e.name, e.encodedBodySize]);
  return {keys: state.keys, changes: state.changes, completions: state.completions, resources};
}'''
# From the Cmd/Ctrl+K keydown: the time the dialog shows, then the time it lists page results for
# the query typed as soon as the input appears. lsg lists them when it first draws the result list
# for the query, without waiting for block rows' record chunks; Logseq when its Pages group appears.
OPEN_WATCH = '''(controls) => {
  window.__open?.dispose?.();
  const state = window.__open = {key: null, visible: null, pages: null, query: null, completed: false};
  const keydown = event => { if (state.key === null) state.key = event.timeStamp; };
  document.addEventListener('keydown', keydown, true);
  const input = () => document.querySelector(controls.input);
  const shown = () => {
    const element = document.querySelector(controls.visible);
    return element && element.getClientRects().length;
  };
  const pages = controls.pages ? new Function('return ' + controls.pages) : () => state.completed;
  const check = () => {
    if (state.visible === null && shown()) requestAnimationFrame(() => { state.visible ??= performance.now(); });
    if (state.pages === null && state.query !== null && input()?.value === state.query && pages()) {
      state.pages = performance.now();
    }
  };
  const observer = new MutationObserver(records => {
    if (controls.drawn && state.query !== null && input()?.value === state.query
        && records.some(r => r.attributeName === controls.drawn)) state.completed = true;
    check();
  });
  observer.observe(document, {childList: true, subtree: true, attributes: true, characterData: true});
  state.dispose = () => { document.removeEventListener('keydown', keydown, true); observer.disconnect(); };
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


def collect_garbage(cdp):
    cdp.send('HeapProfiler.collectGarbage')
    return heap_now(cdp)


def search_resources(page, before, build):
    """Requests since resource entry `before`: lsg's search files, or every request for Logseq."""
    entries = page.evaluate('(before) => performance.getEntriesByType("resource").slice(before)'
                            '.map(e => [e.name, e.encodedBodySize, e.decodedBodySize])', before)
    if build == 'lsg':
        entries = [e for e in entries if SEARCH_FILES.search(e[0])]
    return {'requests': len(entries), 'transfer': sum(e[1] for e in entries), 'decoded': sum(e[2] for e in entries)}


def close_search(page, controls):
    """Press Escape until the dialog closes; lsg clears the input and a filter first."""
    for _ in range(4):
        if not page.is_visible(controls['visible']):
            return
        page.keyboard.press('Escape')
        page.wait_for_timeout(100)


def measure_search(browser, url, build, profile, queries):
    """Open search with Cmd/Ctrl+K, type the first query at once, then each query a key at a time.

    Records the time to the dialog and to the first query's page results, the bytes the first
    open fetched, each keystroke's latency and bytes, whether it fetched a shard, and the heap
    growth after the whole query list.
    """
    controls = SEARCH[build]
    if controls is None:
        return {'note': 'no search'}
    timeout = PROFILES[profile]['timeout_ms']
    context, page, cdp = open_page(browser, profile, serve.TITLE_SELECTORS[build])
    try:
        page.goto(url, wait_until='commit', timeout=timeout)
        wait_loaded(page, profile)
        if page.query_selector(controls['open']) is None:
            return {'note': 'no search'}
        page.wait_for_selector(controls['open'], state='visible', timeout=30_000)
        page_load = page.evaluate(COLLECT)
        eager = sum(1 for name in page.evaluate('performance.getEntriesByType("resource").map(e => e.name)')
                    if SEARCH_FILES.search(name))
        resources_before = page.evaluate('performance.getEntriesByType("resource").length')
        before_heap = collect_garbage(cdp)
        page.evaluate(OPEN_WATCH, controls)
        page.keyboard.press('ControlOrMeta+k')
        page.wait_for_function('window.__open.visible !== null', timeout=timeout, polling=50)
        page.wait_for_selector(controls['input'], state='visible', timeout=timeout)
        page.evaluate('(query) => { window.__open.query = query; }', queries[0])
        page.keyboard.type(queries[0])
        page.wait_for_function('window.__open.pages !== null', timeout=timeout, polling=50)
        opened = page.evaluate('({key: window.__open.key, visible: window.__open.visible, pages: window.__open.pages})')
        page.evaluate('window.__open.dispose()')
        fetched = search_resources(page, resources_before, build)
        result = {'dialog_ms': opened['visible'] - opened['key'], 'first_results_ms': opened['pages'] - opened['key'],
                  'first_open_requests': fetched['requests'], 'first_open_transfer': fetched['transfer'],
                  'first_open_decoded': fetched['decoded'], 'page_load': page_load,
                  'eager_search_requests': eager}
        if build == 'lsg':
            close_search(page, controls)
            before_warm = page.evaluate('performance.getEntriesByType("resource").length')
            started = page.evaluate('performance.now()')
            page.keyboard.press('ControlOrMeta+k')
            page.wait_for_selector(controls['ready'], timeout=timeout)
            result['warm_open_ms'] = page.evaluate('performance.now()') - started
            result['warm_open_requests'] = search_resources(page, before_warm, build)['requests']
        latencies, fetches = [], []
        for query in queries:
            page.fill(controls['input'], '')
            page.wait_for_timeout(1000)
            page.evaluate(SEARCH_PROBE, controls)
            for character in query:
                page.keyboard.type(character)
                page.wait_for_timeout(KEY_GAP_MS)
            page.wait_for_timeout(2000)
            state = page.evaluate(SEARCH_COLLECT)
            if build == 'lsg':
                values = [query[:index + 1] for index in range(len(state['keys']))]
                latencies += metrics.completion_latencies(state['keys'], values, state['completions'])
                files = [r for r in state['resources'] if SEARCH_FILES.search(r[1])]
            else:
                latencies += metrics.keystroke_latencies(state['keys'], state['changes'])
                files = state['resources']
            fetches += metrics.keystroke_fetches(state['keys'], files, SHARD_FILE)
        result['heap_growth'] = max(0, collect_garbage(cdp) - before_heap)
        return {**result, **keystroke_summary(latencies, fetches)}
    except Exception as error:
        return {'error': f'{type(error).__name__}: {str(error).splitlines()[0]}'}
    finally:
        context.close()


def keystroke_summary(latencies, fetches):
    """Percentiles of every keystroke, of those that fetched a shard (cold) and of the rest (warm)."""
    measured = [v for v in latencies if v is not None]
    cold = [v for v, f in zip(latencies, fetches) if f['shard'] and v is not None]
    warm = [v for v, f in zip(latencies, fetches) if not f['shard'] and v is not None]
    # Warm keystrokes that downloaded nothing: the ones that waited for no record chunk either.
    idle = [v for v, f in zip(latencies, fetches) if not f['bytes'] and v is not None]
    sizes = [f['bytes'] for f in fetches]
    return {'keystrokes': len(latencies), 'unchanged': len(latencies) - len(measured),
            'p50_ms': metrics.percentile(measured, 0.5), 'p95_ms': metrics.percentile(measured, 0.95),
            'cold_keystrokes': sum(f['shard'] for f in fetches), 'cold_p95_ms': metrics.percentile(cold, 0.95),
            'warm_p95_ms': metrics.percentile(warm, 0.95), 'chunk_keystrokes': sum(
                1 for f in fetches if f['bytes'] and not f['shard']), 'idle_p95_ms': metrics.percentile(idle, 0.95),
            'keystroke_bytes_max': max(sizes, default=0), 'keystroke_bytes_p95': metrics.percentile(sizes, 0.95)}


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


def measure_references(browser, url, profile):
    """Open Linked References, change the filter, open Unlinked References and scroll both to the end."""
    context, page, cdp = open_page(browser, profile, serve.TITLE_SELECTORS['lsg'])
    try:
        page.goto(url, wait_until='commit', timeout=PROFILES[profile]['timeout_ms'])
        wait_loaded(page, profile)
        result = {'open_ms': page.evaluate(REFERENCES_OPEN, 'linked')}
        result['filter_ms'] = page.evaluate(REFERENCES_FILTER)
        page.evaluate('() => sessionStorage.clear()')
        result['unlinked_open_ms'] = page.evaluate(REFERENCES_OPEN, 'unlinked')
        # Each scroll to the end loads at most one more file per section; stop when none arrives.
        for _ in range(100):
            before = page.evaluate('document.querySelectorAll(".references-blocks-item").length')
            page.evaluate(SCROLL_TO_BOTTOM)
            try:
                page.wait_for_function(f'document.querySelectorAll(".references-blocks-item").length > {before}',
                                       timeout=5_000, polling=100)
            except Exception:  # Every file has loaded.
                break
        page.wait_for_timeout(1000)
        result['groups'] = page.evaluate('document.querySelectorAll(".references-blocks-item").length')
        result['peak_heap'] = max(page.evaluate('window.__parity.peak') or 0, heap_now(cdp))
        return result
    except Exception as error:
        return {'error': f'{type(error).__name__}: {str(error).splitlines()[0]}'}
    finally:
        context.close()


ALL_PAGES_QUERIES = ['mise', 'logseq', 'tmux', 'microfreak preset', 'cli tool', 'garden', 'python', 'rust',
                     'a', 'zz', 'keyshort', 'gitp', 'week', 'docker', 'aws', 'q', 'entity', 'readwise',
                     'claude code', 'vim']
ALL_PAGES_PROBE = '''async (queries) => {
  const box = document.querySelector('.all-pages');
  const settle = () => new Promise((resolve) => setTimeout(resolve, 0));
  const start = performance.now();
  // As in Logseq, each click flips the shared direction: Page name ascending, then Backlinks descending.
  box.querySelector('th.name .sort').click();
  box.querySelector('th.backlinks .sort').click();
  while (box.querySelector('th.backlinks')?.getAttribute('aria-sort') !== 'descending') {
    if (performance.now() - start > 120000) throw new Error('page list did not load');
    await new Promise((resolve) => setTimeout(resolve, 20));
  }
  const sortMs = performance.now() - start;
  const first = box.querySelector('tbody td.name').textContent;
  const input = box.querySelector('.search');
  const times = [];
  for (const query of queries) {
    input.value = query;
    const t = performance.now();
    input.dispatchEvent(new KeyboardEvent('keydown', {key: 'Enter', bubbles: true}));
    await settle();
    times.push(performance.now() - t);
  }
  times.sort((a, b) => a - b);
  return {sort_ms: sortMs, first, search_p95_ms: times[Math.ceil(times.length * 0.95) - 1],
          search_p50_ms: times[Math.floor(times.length / 2)]};
}'''


def measure_all_pages(browser, url, profile):
    """Sort All pages by Backlinks, descending, which loads the page list, then search on Enter for each query."""
    context, page, cdp = open_page(browser, profile, serve.TITLE_SELECTORS['lsg'])
    try:
        page.goto(url, wait_until='commit', timeout=PROFILES[profile]['timeout_ms'])
        wait_loaded(page, profile)
        before = page.evaluate("performance.getEntriesByType('resource').some(e => e.name.includes('_pages/'))")
        result = page.evaluate(ALL_PAGES_PROBE, ALL_PAGES_QUERIES)
        result['list_before_interaction'] = before
        result['peak_heap'] = max(page.evaluate('window.__parity.peak') or 0, heap_now(cdp))
        return result
    except Exception as error:
        return {'error': f'{type(error).__name__}: {str(error).splitlines()[0]}'}
    finally:
        context.close()


def measure_hierarchy(browser, url, profile):
    """Scroll the Hierarchy list to its end, loading every fragment file, and read the peak heap."""
    context, page, cdp = open_page(browser, profile, serve.TITLE_SELECTORS['lsg'])
    try:
        page.goto(url, wait_until='commit', timeout=PROFILES[profile]['timeout_ms'])
        wait_loaded(page, profile)
        result = {'inline_rows': page.evaluate(HIERARCHY_ROWS)}
        # Each scroll to the end loads at most one more file; stop when none arrives.
        for _ in range(100):
            before = page.evaluate(HIERARCHY_ROWS)
            page.evaluate(SCROLL_TO_BOTTOM)
            try:
                page.wait_for_function(f'{HIERARCHY_ROWS} > {before}', timeout=5_000, polling=100)
            except Exception:  # Every file has loaded.
                break
        page.wait_for_timeout(1000)
        result['rows'] = page.evaluate(HIERARCHY_ROWS)
        result['peak_heap'] = max(page.evaluate('window.__parity.peak') or 0, heap_now(cdp))
        return result
    except Exception as error:
        return {'error': f'{type(error).__name__}: {str(error).splitlines()[0]}'}
    finally:
        context.close()


def measure_build(settings, baseline=None):
    """Seconds to publish the whole garden with the run's lsg options, and without journals or search.

    With a baseline binary, such as an lsg built before a feature, also time it with the run's
    options so the feature's build slowdown can be checked.
    """
    lsg = common.lsg_binary()
    options = settings['lsg_options']
    variants = [options] + [[o for o in options if o != flag] for flag in ('--journals', '--search') if flag in options]
    rows = []
    with tempfile.TemporaryDirectory(prefix='lsg-build-time-') as temp:
        for number, chosen in enumerate(variants):
            rows.append({'build': 'lsg', 'journals': '--journals' in chosen, 'search': '--search' in chosen,
                         'seconds': time_publish(lsg, settings['garden'], Path(temp) / str(number), chosen)})
        if baseline:
            rows.append({'build': 'lsg', 'baseline': str(baseline), 'journals': '--journals' in options,
                         'search': '--search' in options,
                         'seconds': time_publish(baseline, settings['garden'], Path(temp) / 'baseline', options)})
    return rows


def build_variants(rows):
    """Map (journals, search) to seconds for the rows timed without a baseline binary."""
    return {(row['journals'], row.get('search', False)): row['seconds'] for row in rows if 'baseline' not in row}


def build_over_budget(variant, builds, budgets):
    """True when a build is more than its feature's slowdown budget above the build without it."""
    journals, search = variant
    checks = [((False, search), budgets.get('journals', {}).get('build_slowdown_pct'))] if journals else []
    if search:
        checks.append(((journals, False), budgets.get('search', {}).get('build_slowdown_pct')))
    return any(other in builds and limit is not None and builds[variant] > builds[other] * (1 + limit / 100)
               for other, limit in checks)


def build_label(variant):
    journals, search = variant
    return f"{'with' if journals else 'without'} journals, {'with' if search else 'without'} search"


BUILD_RUNS = 3


def time_publish(lsg, garden, output, options, runs=BUILD_RUNS):
    """The fastest of `runs` publishes, so other work on the machine skews the comparison less."""
    command = [str(lsg), 'publish', '--graph', garden, '--output', str(output), *options]
    times = []
    for _ in range(runs):
        shutil.rmtree(output, ignore_errors=True)
        start = time.monotonic()
        subprocess.run(command, check=True, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
        times.append(time.monotonic() - start)
    return min(times)


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
    builds = build_variants(results.get('build', []))
    if builds:
        lines += ['', '## Whole-garden build time', '',
                  '`lsg publish` with the run\'s options, then without `--journals` and without `--search`, '
                  f'each the fastest of {BUILD_RUNS} runs. '
                  '**Bold** marks a build more than the `[journals]` or `[search]` `build_slowdown_pct` above '
                  'the build without that feature.', '',
                  '| Build | Seconds |', '| --- | ---: |']
        for variant, seconds in sorted(builds.items(), key=lambda item: (-item[0][0], -item[0][1])):
            text = f'{seconds:,.1f}'
            if build_over_budget(variant, builds, budgets):
                text = f'**{text}**'
            lines.append(f'| {build_label(variant)} | {text} |')
    for row in results.get('build', []):
        variant = (row['journals'], row.get('search', False))
        if 'baseline' not in row or variant not in builds:
            continue
        # The latest feature's budget: All pages, then Hierarchy, then reference sections, then code.
        slowdown = next((budgets[table]['build_slowdown_pct'] for table in ('all_pages', 'hierarchy', 'references', 'code')
                         if 'build_slowdown_pct' in budgets.get(table, {})), None)
        current = builds[variant]
        text = f'{current:,.1f}'
        if slowdown is not None and current > row['seconds'] * (1 + slowdown / 100):
            text = f'**{text}**'
        lines += ['', f"Against the baseline `{Path(row['baseline']).name}` with the same options: "
                  f"{row['seconds']:,.1f} seconds, this build {text} seconds "
                  f"({(current / row['seconds'] - 1) * 100:+.0f}%)."]
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
    limits = budgets.get('references', {})
    reference_rows = results.get('references', [])
    if reference_rows:
        lines += ['', f'## Reference sections on {REFERENCES_PAGE}', '',
                  'Time from clicking a section header to its first group, time from clicking a filter chip '
                  'to the new count, and the peak JS heap after opening Linked and Unlinked References and '
                  'scrolling both to the end.', '',
                  '| Profile | Open linked (ms) | Filter change (ms) | Open unlinked (ms) | Groups loaded | Peak JS heap (MB) |',
                  '| --- | ---: | ---: | ---: | ---: | ---: |']
        for row in reference_rows:
            if 'error' in row:
                lines.append(f"| {row['profile']} | {row['error']} | | | | |")
                continue
            def mark(text, value, limit):
                return f'**{text}**' if limit is not None and value > limit else text
            open_limit = limits.get(f"open_ms_{row['profile']}")
            cells = [
                mark(f"{row['open_ms']:,.0f}", row['open_ms'], open_limit),
                mark(f"{row['filter_ms']:,.0f}", row['filter_ms'],
                     limits.get('filter_ms') if row['profile'] == 'desktop' else None),
                mark(f"{row['unlinked_open_ms']:,.0f}", row['unlinked_open_ms'], open_limit),
                f"{row['groups']:,}",
                mark(f"{row['peak_heap'] / 1_000_000:,.1f}", row['peak_heap'] / 1_000_000,
                     limits.get('open_heap_mb') if row['profile'] == 'phone' else None),
            ]
            lines.append(f"| {row['profile']} | " + ' | '.join(cells) + ' |')
    all_pages_rows = results.get('all_pages', [])
    if all_pages_rows:
        limits = budgets.get('all_pages', {})
        lines += ['', '## All pages', '',
                  'Time from clicking `Page name` and then `Backlinks` to the table sorted by Backlinks, descending, which loads the page list; the first row; '
                  'search on Enter for each query once the list has loaded, and the peak JS heap. '
                  f"The page list is {all_pages_rows[0].get('list_gzip', 0) / 1000:,.1f} KB gzip.", '',
                  '| Profile | Sort by Backlinks (ms) | First row | Search median (ms) | Search 95th percentile (ms) | Peak JS heap (MB) |',
                  '| --- | ---: | --- | ---: | ---: | ---: |']
        for row in all_pages_rows:
            if 'error' in row:
                lines.append(f"| {row['profile']} | {row['error']} | | | | |")
                continue
            p95 = f"{row['search_p95_ms']:,.0f}"
            if row['search_p95_ms'] > limits.get(f"search_ms_{row['profile']}", math.inf):
                p95 = f'**{p95}**'
            heap = f"{row['peak_heap'] / 1_000_000:,.1f}"
            if row['profile'] == 'phone' and row['peak_heap'] / 1_000_000 > limits.get('sort_heap_mb', math.inf):
                heap = f'**{heap}**'
            lines.append(f"| {row['profile']} | {row['sort_ms']:,.0f} | {row['first']} | "
                         f"{row['search_p50_ms']:,.0f} | {p95} | {heap} |")
    hierarchy_rows = results.get('hierarchy', [])
    if hierarchy_rows:
        heap_limit = budgets.get('hierarchy', {}).get('scroll_heap_mb')
        lines += ['', f'## Hierarchy on {HIERARCHY_PAGE}', '',
                  'Rows in the page before scrolling, rows after scrolling the list to its end, and the peak '
                  'JS heap.', '',
                  '| Profile | Rows in the page | Rows after scrolling | Peak JS heap (MB) |',
                  '| --- | ---: | ---: | ---: |']
        for row in hierarchy_rows:
            if 'error' in row:
                lines.append(f"| {row['profile']} | {row['error']} | | |")
                continue
            heap = f"{row['peak_heap'] / 1_000_000:,.1f}"
            if row['profile'] == 'phone' and heap_limit is not None and row['peak_heap'] > heap_limit * 1_000_000:
                heap = f'**{heap}**'
            lines.append(f"| {row['profile']} | {row['inline_rows']:,} | {row['rows']:,} | {heap} |")
    lines += search_markdown(results, budgets)
    return '\n'.join(lines) + '\n'


def index_summary(site):
    """The search file sizes recorded beside the browser measurements."""
    sizes = budgets_check.search_index_sizes(site)
    site = Path(site)
    largest_shard, largest_chunk = sizes['shards'][0], sizes['chunks'][0]
    return {'pages': sizes['pages'], 'blocks': sizes['blocks'], 'titles': sizes['titles'],
            'titles_decoded': sizes['titles_decoded'], 'titles_gzip': sizes['titles_gzip'],
            'shard_count': len(sizes['shards']), 'chunk_count': len(sizes['chunks']),
            'largest_shard': {'file': largest_shard[0], 'key': sizes['shard_keys'][largest_shard[0]],
                              'gzip': largest_shard[1]},
            'largest_chunk': {'file': largest_chunk[0], 'gzip': largest_chunk[1]},
            'bootstrap_bytes': next(site.glob('lsg-search-start-*.mjs')).stat().st_size,
            'controller_bytes': next(site.glob('lsg-search-ui-*.mjs')).stat().st_size}


def index_checks(index, budgets):
    """(label, value, limit) for each search file size with a budget."""
    limits = budgets.get('search', {})
    return [('Title index (KB, gzip)', index['titles_gzip'] / 1000, limits.get('titles_gzip_kb')),
            ('Title index (MB, decoded)', index['titles_decoded'] / 1_000_000, limits.get('titles_decoded_mb')),
            (f"Largest shard, words starting `{index['largest_shard']['key']}` (KB, gzip)",
             index['largest_shard']['gzip'] / 1000, limits.get('shard_gzip_kb')),
            ('Largest record chunk (KB, gzip)', index['largest_chunk']['gzip'] / 1000, limits.get('chunk_gzip_kb')),
            ('Search starter (KB)', index['bootstrap_bytes'] / 1000, limits.get('bootstrap_kb'))]


def search_checks(row, budgets):
    """(field, value, limit) for each search measure of one lsg row that has a budget."""
    limits, profile = budgets.get('search', {}), row['profile']
    checks = [('dialog_ms', row.get('dialog_ms'), limits.get(f'dialog_ms_{profile}')),
              ('first_results_ms', row.get('first_results_ms'), limits.get(f'first_results_ms_{profile}')),
              ('cold_p95_ms', row.get('cold_p95_ms'), limits.get(f'cold_keystroke_ms_{profile}')),
              ('warm_p95_ms', row.get('warm_p95_ms'), budgets.get(profile, {}).get('search_p95_ms')),
              ('keystroke_bytes_max', row.get('keystroke_bytes_max'),
               limits['keystroke_gzip_kb'] * 1000 if 'keystroke_gzip_kb' in limits else None),
              ('warm_open_ms', row.get('warm_open_ms'), limits.get(f'warm_open_{profile}_ms'))]
    if profile == 'phone' and 'heap_growth_mb' in limits:
        checks.append(('heap_growth', row.get('heap_growth'), limits['heap_growth_mb'] * 1_000_000))
    return checks


def search_markdown(results, budgets):
    """The search sections of the performance report: file sizes, then both builds in the browser."""
    lines = []
    index = results.get('search_index')
    if index:
        lines += ['', '## Search index', '',
                  f"`lsg`'s search files for {index['pages']:,} pages and {index['blocks']:,} blocks: one title index, "
                  f"{index['shard_count']:,} word shards and {index['chunk_count']:,} record chunks. "
                  '**Bold** marks a value over its `[search]` budget.', '',
                  '| File | Size | Budget |', '| --- | ---: | ---: |']
        for label, value, limit in index_checks(index, budgets):
            text = f'{value:,.1f}' if value < 100 else f'{value:,.0f}'
            if limit is not None and value > limit:
                text = f'**{text}**'
            lines.append(f"| {label} | {text} | {'–' if limit is None else f'{limit:,}'} |")
    rows = [row for row in results.get('search', [])]
    lines += ['', '## Search', '',
              'Cmd/Ctrl+K on the home page, then the first query typed at once: the time from the key to the '
              'dialog and to its page results, and what that first open downloaded. Then each query in '
              '[`search-queries.txt`](../../scripts/parity/search-queries.txt) typed a key every '
              f'{KEY_GAP_MS} ms. A cold keystroke fetches a word shard; a warm one does not. The heap growth is '
              'measured after the whole query list, after garbage collection. **Bold** marks an `lsg` value '
              'over its budget.', '',
              '| Profile | Build | Dialog shown (ms) | First page results (ms) | First open (KB, gzip) '
              '| Cold keystrokes | Cold 95th percentile (ms) | Warm 95th percentile (ms) '
              '| Largest keystroke download (KB, gzip) | Heap growth (MB) |',
              '| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |']
    scale = {'keystroke_bytes_max': 1000, 'heap_growth': 1_000_000}
    for row in rows:
        if 'dialog_ms' not in row:
            lines.append(f"| {row['profile']} | {row['build']} | {row.get('note') or row.get('error')} |" + ' |' * 7)
            continue
        over = {field for field, value, limit in search_checks(row, budgets)
                if row['build'] == 'lsg' and limit is not None and value is not None and value > limit}

        def cell(field, digits=0):
            value = row.get(field)
            if value is None:
                return '–'
            value /= scale.get(field, 1)
            text = f'{value:,.{digits}f}'
            return f'**{text}**' if field in over else text
        lines.append(f"| {row['profile']} | {row['build']} | {cell('dialog_ms')} | {cell('first_results_ms')} | "
                     f"{row['first_open_transfer'] / 1000:,.1f} | {row['cold_keystrokes']} | {cell('cold_p95_ms')} | "
                     f"{cell('warm_p95_ms')} | {cell('keystroke_bytes_max', 1)} | {cell('heap_growth', 1)} |")
    lines += ['', '## Search latency per keystroke', '',
              'Every keystroke of the query list. An unanswered keystroke is one whose results never showed. '
              'An `lsg` keystroke ends when its result list is drawn; block rows whose record chunk is still '
              'loading show a placeholder line until it arrives. The keystrokes that fetched nothing show '
              'the cost of the query alone. '
              '`lsg` reopens the dialog from memory after the first open.', '',
              '| Profile | Build | Keystrokes | Unanswered | Median (ms) | 95th percentile (ms) '
              '| Warm keystrokes that fetched a chunk | 95th percentile of keystrokes that fetched nothing (ms) '
              '| Reopen (ms) | Reopen requests |',
              '| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |']
    for row in rows:
        if 'p95_ms' not in row:
            continue
        reopen = f"{row['warm_open_ms']:,.0f}" if 'warm_open_ms' in row else '–'
        p50, p95, idle = ('–' if row.get(k) is None else f'{row[k]:,.0f}' for k in ('p50_ms', 'p95_ms', 'idle_p95_ms'))
        lines.append(f"| {row['profile']} | {row['build']} | {row['keystrokes']} | {row['unchanged']} | "
                     f"{p50} | {p95} | {row.get('chunk_keystrokes', '–')} | {idle} | {reopen} | "
                     f"{row.get('warm_open_requests', '–')} |")
    builds = build_variants(results.get('search_build', []))
    if (True, True) in builds and (True, False) in builds:
        with_search, without = builds[(True, True)], builds[(True, False)]
        text = f'{with_search:,.1f}'
        if build_over_budget((True, True), builds, budgets):
            text = f'**{text}**'
        lines += ['', f'Whole-garden build with journals, the fastest of {BUILD_RUNS} runs: {text} seconds with '
                  f'`--search`, {without:,.1f} seconds without ({(with_search / without - 1) * 100:+.0f}%; budget '
                  f"+{budgets.get('search', {}).get('build_slowdown_pct')}%)."]
    return lines


def search_failures(report, budgets):
    """Failure messages for standalone search measurements over a budget or missing."""
    failures = []
    index = report.get('search_index')
    if index:
        failures += [f'{label}: {value:,.1f} exceeds {limit}' for label, value, limit in index_checks(index, budgets)
                     if limit is not None and value > limit]
    for row in report['search']:
        if row.get('build', 'lsg') != 'lsg':
            continue
        profile = row['profile']
        if 'error' in row or 'dialog_ms' not in row:
            failures.append(f"{profile}: search did not open: {row.get('error') or row.get('note')}")
            continue
        if row['unchanged']:
            failures.append(f"{profile}: {row['unchanged']} keystrokes had no completed results")
        for field, value, limit in search_checks(row, budgets) + [('eager_search_requests', row['eager_search_requests'], 0),
                                                                  ('warm_open_requests', row.get('warm_open_requests'), 0)]:
            if limit is None:
                continue
            if value is None or not math.isfinite(value) or value > limit:
                failures.append(f'{profile}: {field}={value} exceeds {limit} or is unavailable')
    builds = build_variants(report.get('search_build', []))
    if (True, True) in builds and build_over_budget((True, True), builds, budgets):
        failures.append('whole-garden build with --search exceeds its slowdown budget')
    return failures


def search_run(args):
    """The parity run that holds --search-site, if any: its settings and Logseq build sit beside it."""
    if args.run:
        return Path(args.run)
    parent = args.search_site.resolve().parent
    return parent if (parent / 'settings.json').exists() else None


def measure_search_site(args, sync_playwright):
    """Measure search in a built site and, when it is a parity run's site, Logseq's build and build time.

    Writes --search-output (JSON and Markdown); with --record, also docs/parity/search-performance.json
    and the search sections of docs/parity/performance.md. Fails when an lsg value is over budget.
    """
    budgets = metrics.load_budgets()
    profiles = [p.strip() for p in args.profiles.split(',') if p.strip()]
    builds = [b.strip() for b in args.builds.split(',') if b.strip()]
    queries = serve.fixtures(args.search_queries)
    run = search_run(args)
    logseq_dir = run / 'logseq' if run else None
    if 'logseq' in builds and not (logseq_dir and logseq_dir.exists()):
        print('No Logseq build beside the site; measuring lsg only. Run `mise run parity:compare` for one.')
        builds = [b for b in builds if b != 'logseq']
    site = args.search_site / Path(args.search_path).parent
    rows = []
    with serve.Server(args.search_site) as lsg, sync_playwright() as playwright:
        browser = playwright.chromium.launch(args=['--enable-precise-memory-info'])
        servers = {'lsg': (lsg, args.search_path)}
        with contextlib.ExitStack() as stack:
            if 'logseq' in builds:
                servers['logseq'] = (stack.enter_context(serve.Server(logseq_dir)), serve.LOGSEQ_ROUTES['Home'])
            for profile in profiles:
                for build in ('logseq', 'lsg'):
                    if build not in builds:
                        continue
                    server, path = servers[build]
                    row = {'profile': profile, 'build': build,
                           **measure_search(browser, server.url + path, build, profile, queries)}
                    rows.append(row)
                    print(json.dumps(row), flush=True)
        browser.close()
    summary = {'measured': datetime.now(timezone.utc).strftime('%Y-%m-%d'), 'profiles': ', '.join(profiles)}
    build_rows = []
    if run and 'lsg' in builds:
        settings = json.loads((run / 'settings.json').read_text(encoding='utf-8'))
        garden = Path(settings['garden'])
        summary.update({'garden commit': settings['garden_commit'][:7],
                        'gh-pages commit': common.git_commit(garden, 'origin/gh-pages')[:7],
                        'lsg': f"{settings['lsg_version']} ({settings['lsg_commit'][:7]})"})
        if '--search' in settings['lsg_options']:
            build_rows = measure_build(settings)
            for row in build_rows:
                print(json.dumps(row), flush=True)
    report = {'summary': summary, 'search_index': index_summary(site), 'pages': [], 'search': rows,
              'search_build': build_rows}
    text = '\n'.join(['# Search performance', ''] + [f'- {k}: {v}' for k, v in summary.items()]
                     + search_markdown(report, budgets)) + '\n'
    if args.search_output:
        args.search_output.write_text(json.dumps(report, indent=2, ensure_ascii=False) + '\n', encoding='utf-8')
        args.search_output.with_suffix('.md').write_text(text, encoding='utf-8')
    if args.record:
        SEARCH_DOC.write_text(json.dumps(report, indent=2, ensure_ascii=False) + '\n', encoding='utf-8')
        record_search_sections(report, budgets)
    print(text)
    failures = search_failures(report, budgets)
    if failures:
        raise SystemExit('\n'.join(failures))


GRAPH_PAGES = ['Foam', 'tmux', 'Person']
GRAPH_MARK = "performance.getEntriesByName('lsg-graph-first-frame').length > 0"
# Samples frame times while the reader pans; FRAME_COLLECT returns the intervals in ms.
FRAME_RECORDER = '''() => {
  const state = window.__frames = {times: [], on: true};
  const step = time => { state.times.push(time); if (state.on) requestAnimationFrame(step); };
  requestAnimationFrame(step);
}'''
FRAME_COLLECT = '''() => {
  const state = window.__frames; state.on = false;
  return state.times.slice(1).map((time, i) => time - state.times[i]);
}'''


def measure_graph(browser, url, profile):
    """Open graph.html: the first frame, a pan and a zoom with frame times, the peak heap, and
    whether the worker loaded although no force changed."""
    context, page, cdp = open_page(browser, profile, 'canvas')
    timeout = PROFILES[profile]['timeout_ms']
    try:
        page.goto(url, wait_until='commit', timeout=timeout)
        page.wait_for_function(GRAPH_MARK, timeout=timeout, polling=50)
        result = {'first_frame_ms': page.evaluate("performance.getEntriesByName('lsg-graph-first-frame')[0].startTime")}
        page.wait_for_timeout(500)
        box = page.locator('.lsg-graph-canvas').bounding_box()
        # Start in the bottom-left corner, outside the fitted graph, so the drag pans instead of moving a node.
        x, y = box['x'] + 8, box['y'] + box['height'] - 8
        page.evaluate(FRAME_RECORDER)
        page.mouse.move(x, y)
        page.mouse.down()
        for step in range(40):
            page.mouse.move(x + step * 4, y - step * 2)
        page.mouse.up()
        frames = page.evaluate(FRAME_COLLECT)
        page.mouse.move(box['x'] + box['width'] / 2, box['y'] + box['height'] / 2)
        for _ in range(5):
            page.mouse.wheel(0, -200)
            page.wait_for_timeout(50)
        page.wait_for_timeout(500)
        result['pan_p95_ms'] = metrics.percentile(frames, 0.95)
        result['frames'] = len(frames)
        result['peak_heap'] = max(page.evaluate('window.__parity.peak') or 0, heap_now(cdp))
        result['worker_loaded'] = page.evaluate(
            "performance.getEntriesByType('resource').some(e => e.name.includes('worker'))")
        result['renderer'] = page.evaluate(
            "(() => { const gl = document.createElement('canvas').getContext('webgl2'); "
            "const i = gl && gl.getExtension('WEBGL_debug_renderer_info'); "
            "return i ? gl.getParameter(i.UNMASKED_RENDERER_WEBGL) : 'none'; })()")
        result['data_transfer'] = page.evaluate(
            "performance.getEntriesByType('resource').filter(e => e.name.includes('_graph/'))"
            ".reduce((t, e) => t + e.encodedBodySize, 0)")
        return result
    except Exception as error:  # Playwright raises its own TimeoutError type.
        return {'error': f'{type(error).__name__}: {str(error).splitlines()[0]}'}
    finally:
        context.close()


def measure_page_graph(browser, url, profile):
    """Open the right sidebar on a page and time a click on Page graph to the card's first frame."""
    context, page, _ = open_page(browser, profile, 'h1')
    timeout = PROFILES[profile]['timeout_ms']
    try:
        page.goto(url, wait_until='load', timeout=timeout)
        page.click('[data-lsg-right-toggle]')
        page.wait_for_selector('[data-add=graph]', timeout=timeout)
        page.evaluate('window.__start = performance.now()')
        page.click('[data-add=graph]')
        page.wait_for_function(GRAPH_MARK, timeout=timeout, polling=20)
        return {
            'open_ms': page.evaluate("performance.getEntriesByName('lsg-graph-first-frame')[0].startTime - window.__start"),
            'file': page.evaluate("performance.getEntriesByType('resource').filter(e => e.name.includes('_graph/'))"
                                  ".map(e => e.name.split('/').slice(-2).join('/'))[0] || null"),
        }
    except Exception as error:
        return {'error': f'{type(error).__name__}: {str(error).splitlines()[0]}'}
    finally:
        context.close()


def lsg_route(title):
    """An lsg page's file: p-<FNV-1a 64 of the title's UTF-8>.html."""
    digest = 0xcbf29ce484222325
    for byte in title.encode('utf-8'):
        digest = ((digest ^ byte) * 0x100000001b3) % (1 << 64)
    return f'p-{digest:016x}.html'


def _marked(text, value, limit):
    return f'**{text}**' if limit is not None and value is not None and value > limit else text


def graph_markdown(report, budgets):
    """The graph view and Page graph sections of docs/parity/performance.md."""
    limits = budgets.get('graph', {})
    lines = ['', '## Graph view', '',
             'From navigating to `graph.html` (Logseq: `#/graph`) to the first frame with every node of the default '
             'view drawn; the 95th-percentile frame time while dragging the background; and the peak JS heap after '
             'the first frame, the drag and five wheel zooms. Logseq\'s desktop row comes from the '
             '[Graph view analysis](graph-view.md), which measured 38 minutes to its first frame; `parity:perf` '
             'does not wait that long. **Bold** marks an `lsg` value over its `[graph]` budget.', '',
             '| Profile | Build | First frame (ms) | Pan frame time, 95th percentile (ms) | Peak JS heap (MB) '
             '| Graph data (KB, gzip) | Worker loaded |',
             '| --- | --- | ---: | ---: | ---: | ---: | --- |']
    lines.append('| desktop | logseq | 2,291,000 | – | 2,340 | 19,800 (`db.gz`) | – |')
    for row in report['graph']:
        profile = row['profile']
        if 'error' in row:
            lines.append(f"| {profile} | {row['build']} | {row['error']} | | | | |")
        else:
            first = _marked(f"{row['first_frame_ms']:,.0f}", row['first_frame_ms'], limits.get(f'first_frame_ms_{profile}'))
            phone = profile == 'phone'
            pan = _marked(f"{row['pan_p95_ms']:,.0f}", row['pan_p95_ms'], limits.get('frame_ms_phone') if phone else None)
            heap = _marked(f"{row['peak_heap'] / 1e6:,.1f}", row['peak_heap'] / 1e6, limits.get('heap_mb') if phone else None)
            worker = 'yes' if row['worker_loaded'] else 'no'
            lines.append(f"| {profile} | lsg | {first} | {pan} | {heap} | {row['data_transfer'] / 1000:,.0f} | {worker} |")
    renderers = sorted({row['renderer'] for row in report['graph'] if row.get('renderer')})
    if renderers:
        lines += ['', 'WebGL renderer: ' + '; '.join(f'`{r}`' for r in renderers) + '.']
    sizes = report['sizes']
    size, _ = sizes['largest_neighbourhood']
    lines += ['', f"`lsg`'s graph files: `_graph/default.bin` {sizes['data'] / 1000:,.0f} KB as stored (gzip), "
              f"`lsg-graph.js` with its styles {sizes['script'] / 1000:,.1f} KB gzip, the worker "
              f"{sizes['worker'] / 1000:,.1f} KB gzip, {sizes['neighbourhoods']:,} neighbourhood files, the largest "
              f"{size / 1000:,.1f} KB gzip.",
              '', '## Page graph', '',
              "Time from clicking `Page graph` in the right sidebar to the card's first frame, and the file it fetched. "
              'Below 640px there is no right sidebar, so the phone rows use a 1280px window with the phone '
              "profile's CPU and network throttling.", '',
              '| Page | Profile | Open (ms) | File |', '| --- | --- | ---: | --- |']
    for row in report['page_graph']:
        if 'error' in row:
            lines.append(f"| {row['page']} | {row['profile']} | {row['error']} | |")
            continue
        limit = limits.get(f"page_graph_open_ms_{row['profile']}")
        lines.append(f"| {row['page']} | {row['profile']} | {_marked(f'{row['open_ms']:,.0f}', row['open_ms'], limit)} "
                     f"| `{row['file']}` |")
    build = report.get('build')
    if build:
        slow = (build['seconds'] / build['baseline_seconds'] - 1) * 100
        text = _marked(f"{build['seconds']:,.1f}", slow, limits.get('build_slowdown_pct'))
        lines += ['', f"Whole-garden build with the run's options, the fastest of {BUILD_RUNS} runs: {text} seconds "
                  f"with the graph, {build['baseline_seconds']:,.1f} seconds with the baseline "
                  f"`{Path(build['baseline']).name}` ({slow:+.0f}%; budget +{limits.get('build_slowdown_pct')}%)."]
    return lines


def measure_graph_run(args, sync_playwright):
    """Measure the graph view and Page graph in a parity run's site; with --record, replace their
    sections of docs/parity/performance.md."""
    budgets = metrics.load_budgets()
    run = common.latest_run(args.run)
    site = run / 'site'
    profiles = [p.strip() for p in args.profiles.split(',') if p.strip()]
    report = {'graph': [], 'page_graph': [], 'sizes': budgets_check.graph_sizes(site)}
    with serve.Server(site) as lsg, sync_playwright() as playwright:
        # Headless Chromium draws WebGL on the CPU (SwiftShader) unless it is given the GPU; a phone has one.
        gpu = ['--enable-gpu'] + (['--use-angle=metal'] if sys.platform == 'darwin' else [])
        browser = playwright.chromium.launch(args=['--enable-precise-memory-info', *gpu])
        for profile in profiles:
            row = {'profile': profile, 'build': 'lsg', **measure_graph(browser, lsg.url + 'graph.html', profile)}
            report['graph'].append(row)
            print(json.dumps(row), flush=True)
            # The right sidebar needs 640px; the phone rows keep the phone's CPU and network throttling.
            PROFILES['wide-' + profile] = {**PROFILES[profile], 'viewport': {'width': 1280, 'height': 800},
                                           'mobile': False, 'scale': 1}
            for name in GRAPH_PAGES:
                route = lsg_route(name)
                if not (site / route).exists():
                    continue
                row = {'page': name, 'profile': profile,
                       **measure_page_graph(browser, lsg.url + route, 'wide-' + profile)}
                report['page_graph'].append(row)
                print(json.dumps(row, ensure_ascii=False), flush=True)
        browser.close()
    if args.baseline_lsg:
        settings = json.loads((run / 'settings.json').read_text(encoding='utf-8'))
        with tempfile.TemporaryDirectory(prefix='lsg-graph-build-') as temp:
            options = settings['lsg_options']
            report['build'] = {
                'seconds': time_publish(common.lsg_binary(), settings['garden'], Path(temp) / 'new', options),
                'baseline_seconds': time_publish(args.baseline_lsg, settings['garden'], Path(temp) / 'old', options),
                'baseline': str(args.baseline_lsg),
            }
    (run / 'graph-perf.json').write_text(json.dumps(report, indent=2, ensure_ascii=False) + '\n', encoding='utf-8')
    print('\n'.join(graph_markdown(report, budgets)))
    if args.record:
        record_graph_sections(report, budgets)


def record_graph_sections(report, budgets):
    """Replace the graph sections of docs/parity/performance.md; they sit before the search sections."""
    text = PERFORMANCE_DOC.read_text(encoding='utf-8')
    start = text.find('\n## Graph view')
    if start >= 0:
        end = text.find('\n## Search', start)
        text = text[:start] + (text[end:] if end >= 0 else '\n')
    block = '\n'.join(graph_markdown(report, budgets))
    search = text.find('\n## Search')
    text = text[:search] + block + '\n' + text[search:] if search >= 0 else text.rstrip('\n') + '\n' + block + '\n'
    PERFORMANCE_DOC.write_text(text, encoding='utf-8')


def record_search_sections(report, budgets):
    """Replace the search sections at the end of docs/parity/performance.md with this measurement."""
    text = PERFORMANCE_DOC.read_text(encoding='utf-8')
    start = text.find('\n## Search')
    kept = text if start < 0 else text[:start]
    PERFORMANCE_DOC.write_text(kept.rstrip('\n') + '\n' + '\n'.join(search_markdown(report, budgets)) + '\n',
                               encoding='utf-8')


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--run', help='run directory from parity:build and parity:compare (default: the latest)')
    parser.add_argument('--fixtures', default=str(serve.FIXTURES), help='fixture list, one title per line')
    parser.add_argument('--profiles', default='desktop,phone', help='comma-separated profiles')
    parser.add_argument('--loads', type=int, default=3, help='loads per page, build and profile')
    parser.add_argument('--builds', default='logseq,lsg',
                        help='comma-separated builds to measure; other builds keep their rows from this run')
    parser.add_argument('--search-site', type=Path,
                        help='Measure search only, in a built site; a parity run\'s site also measures Logseq beside it')
    parser.add_argument('--search-path', default='index.html', help='Page path inside --search-site (for a host prefix)')
    parser.add_argument('--search-output', type=Path, help='JSON destination for standalone search measurements')
    parser.add_argument('--search-queries', type=Path, default=QUERIES, help='Query fixture list for search measurements')
    parser.add_argument('--record', action='store_true',
                        help='rewrite docs/parity/performance.md; with --search-site, its search sections and '
                             'docs/parity/search-performance.json')
    parser.add_argument('--graph', action='store_true',
                        help="measure only the graph view and Page graph in the run's site; with --record, "
                             'replace their sections of docs/parity/performance.md')
    parser.add_argument('--baseline-lsg', type=Path,
                        help='also time a whole-garden build with this lsg binary, such as one built before a feature')
    args = parser.parse_args()
    from playwright.sync_api import sync_playwright

    if args.search_site:
        measure_search_site(args, sync_playwright)
        return
    if args.graph:
        measure_graph_run(args, sync_playwright)
        return

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
        feed, zoom, references, all_pages, hierarchy = [], [], [], [], []
        all_pages_target = serve.targets('All pages', names, lsg_index, run / 'site')
        list_file = run / 'site' / '_pages' / 'list.json'
        list_gzip = metrics.gzip_size(list_file.read_bytes()) if list_file.exists() else 0
        hierarchy_target = serve.targets(HIERARCHY_PAGE, names, lsg_index, run / 'site')
        zoom_target = serve.targets(ZOOM_PAGE, names, lsg_index, run / 'site')
        references_target = serve.targets(REFERENCES_PAGE, names, lsg_index, run / 'site')
        for profile in profiles:
            for build, server in servers.items():
                home = server.url + serve.LOGSEQ_ROUTES['Home'] if build == 'logseq' else server.url + 'index.html'
                row = {'profile': profile, 'build': build, **measure_search(browser, home, build, profile, queries)}
                search.append(row)
                print(json.dumps(row), flush=True)
                row = {'profile': profile, 'build': build, **measure_feed(browser, home, build, profile)}
                feed.append(row)
                print(json.dumps(row), flush=True)
                if build == 'lsg' and references_target['lsg'] is not None:
                    row = {'profile': profile, 'build': build,
                           **measure_references(browser, server.url + references_target['lsg'], profile)}
                    references.append(row)
                    print(json.dumps(row), flush=True)
                if build == 'lsg' and all_pages_target['lsg'] is not None:
                    row = {'profile': profile, 'build': build, 'list_gzip': list_gzip,
                           **measure_all_pages(browser, server.url + all_pages_target['lsg'], profile)}
                    all_pages.append(row)
                    print(json.dumps(row, ensure_ascii=False), flush=True)
                if build == 'lsg' and hierarchy_target['lsg'] is not None:
                    row = {'profile': profile, 'build': build,
                           **measure_hierarchy(browser, server.url + hierarchy_target['lsg'], profile)}
                    hierarchy.append(row)
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
    references = merge(kept.get('references', []), references, builds, ('profile', 'build'))
    all_pages = merge(kept.get('all_pages', []), all_pages, builds, ('profile', 'build'))
    hierarchy = merge(kept.get('hierarchy', []), hierarchy, builds, ('profile', 'build'))
    build_rows = measure_build(settings, args.baseline_lsg) if 'lsg' in builds else kept.get('build', [])
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
        'search_index': index_summary(run / 'site') if list((run / 'site').glob('lsg-search-manifest-*.json')) else None,
        'pages': pages,
        'search': search,
        'feed': feed,
        'zoom': zoom,
        'references': references,
        'all_pages': all_pages,
        'hierarchy': hierarchy,
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
