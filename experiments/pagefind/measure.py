#!/usr/bin/env python3
# /// script
# requires-python = ">=3.12"
# dependencies = ["playwright==1.60.0"]
# ///
"""Index an lsg parity build with Pagefind, then measure its index size, bytes per query and keystroke latency.

Uses the parity harness's desktop and phone profiles and query list, so the numbers sit beside
`mise run parity:perf --search-site` for lsg's own search index.
"""
import argparse
import json
import shutil
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[1]
sys.path.insert(0, str(REPO / 'scripts' / 'parity'))
import common  # noqa: E402
import metrics  # noqa: E402
import perf  # noqa: E402
import serve  # noqa: E402

PAGEFIND = 'npm:pagefind@1.5.2'
# Rows drawn per keystroke: lsg's dialog shows five pages and five blocks before Show more.
ROWS = 10
HARNESS = '<!doctype html><html lang="en"><meta charset="utf-8"><title>Pagefind</title><h1>Pagefind</h1></html>\n'
# Load Pagefind, then for each keystroke of each query run a search and load the data of the rows
# drawn. Each keystroke waits for the previous one, then for the parity harness's key gap.
# Pagefind fetches its files from a worker, out of the page's resource timing, so the script
# records each keystroke's wall-clock span and the requests are matched to it from the browser's
# network events.
PROBE = '''async ({queries, gap, rows}) => {
  let start = performance.now(), wall = Date.now();
  const pagefind = await import('/pagefind/pagefind.js');
  await pagefind.init();
  await pagefind.search('a');
  const init = {ms: performance.now() - start, from: wall, to: Date.now()};
  const keys = [];
  for (const query of queries) {
    for (let length = 1; length <= query.length; length++) {
      const value = query.slice(0, length);
      start = performance.now(); wall = Date.now();
      const search = await pagefind.search(value);
      await Promise.all(search.results.slice(0, rows).map(result => result.data()));
      keys.push({value, ms: performance.now() - start, from: wall, to: Date.now(), results: search.results.length});
      await new Promise(resolve => setTimeout(resolve, gap));
    }
  }
  return {init, keys};
}'''


def index(site, output):
    """Run Pagefind over the site's HTML, writing the bundle to output/pagefind; return seconds taken."""
    if output.exists():
        shutil.rmtree(output)
    output.mkdir(parents=True)
    (output / 'index.html').write_text(HARNESS, encoding='utf-8')
    command = ['mise', 'exec', PAGEFIND, '--', 'pagefind', '--site', str(site.resolve()),
               '--output-path', str((output / 'pagefind').resolve())]
    start = time.monotonic()
    subprocess.run(command, cwd=HERE, check=True)
    return time.monotonic() - start


def sizes(bundle):
    """Total bytes of the Pagefind bundle by kind, and the largest index chunk and fragment."""
    kinds = {}
    for path in bundle.rglob('*'):  # Pagefind's fragment and index folders hold one kind each.
        if not path.is_file():
            continue
        kind = path.suffix.lstrip('.') or path.name
        if path.parent.name == 'wasm' or kind == 'wasm':
            kind = 'wasm'
        entry = kinds.setdefault(kind, {'files': 0, 'bytes': 0, 'largest': 0})
        entry['files'] += 1
        entry['bytes'] += path.stat().st_size
        entry['largest'] = max(entry['largest'], path.stat().st_size)
    return {'total_bytes': sum(k['bytes'] for k in kinds.values()), 'kinds': kinds}


def measure(browser, url, profile, queries):
    context, page, cdp = perf.open_page(browser, profile, 'h1')
    try:
        page.goto(url, wait_until='commit', timeout=perf.PROFILES[profile]['timeout_ms'])
        perf.wait_loaded(page, profile)
        before = perf.collect_garbage(cdp)
        requests = []

        def finished(request):
            if '/pagefind/' in request.url:
                requests.append((time.time() * 1000, request.url, request.sizes()['responseBodySize']))
        context.on('requestfinished', finished)
        found = page.evaluate(PROBE, {'queries': queries, 'gap': perf.KEY_GAP_MS, 'rows': ROWS})
        found['heap_growth'] = max(0, perf.collect_garbage(cdp) - before)
        for span in [found['init'], *found['keys']]:
            mine = [size for at, _, size in requests if span['from'] <= at <= span['to'] + 5]
            span.update({'bytes': sum(mine), 'requests': len(mine)})
        times = [key['ms'] for key in found['keys']]
        cold = [key['ms'] for key in found['keys'] if key['requests']]
        warm = [key['ms'] for key in found['keys'] if not key['requests']]
        found.update({'p50_ms': metrics.percentile(times, 0.5), 'p95_ms': metrics.percentile(times, 0.95),
                      'cold_keystrokes': len(cold), 'cold_p95_ms': metrics.percentile(cold, 0.95),
                      'warm_p95_ms': metrics.percentile(warm, 0.95),
                      'keystroke_bytes_max': max(key['bytes'] for key in found['keys']),
                      'keystroke_bytes_p95': metrics.percentile([key['bytes'] for key in found['keys']], 0.95)})
        return found
    finally:
        context.close()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--site', type=Path, help='lsg site to index (default: the latest parity run\'s site)')
    parser.add_argument('--output', type=Path, default=REPO / 'target' / 'pagefind',
                        help='directory for the Pagefind bundle and results (default: target/pagefind)')
    parser.add_argument('--profiles', default='desktop,phone', help='comma-separated parity profiles')
    args = parser.parse_args()
    site = args.site or common.latest_run() / 'site'
    seconds = index(site, args.output)
    queries = serve.fixtures(perf.QUERIES)
    rows = []
    from playwright.sync_api import sync_playwright
    with serve.Server(args.output) as server, sync_playwright() as playwright:
        browser = playwright.chromium.launch(args=['--enable-precise-memory-info'])
        for profile in [p.strip() for p in args.profiles.split(',') if p.strip()]:
            row = {'profile': profile, **measure(browser, server.url + 'index.html', profile, queries)}
            rows.append(row)
            print(json.dumps({k: v for k, v in row.items() if k != 'keys'}), flush=True)
        browser.close()
    report = {'measured': datetime.now(timezone.utc).strftime('%Y-%m-%d'), 'pagefind': PAGEFIND,
              'site': str(site), 'pages': len(list(site.glob('*.html'))), 'index_seconds': seconds,
              'bundle': sizes(args.output / 'pagefind'), 'profiles': rows}
    (args.output / 'results.json').write_text(json.dumps(report, indent=2) + '\n', encoding='utf-8')
    print(json.dumps({k: v for k, v in report.items() if k != 'profiles'}, indent=2))
    print(f"Results: {args.output / 'results.json'}")


if __name__ == '__main__':
    main()
