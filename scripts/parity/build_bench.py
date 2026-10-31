#!/usr/bin/env python3
"""Measure what build plugins add to a whole-garden build, and check the [build_plugins] budgets.

Builds the garden without plugins and with each no-op fixture: the in-process example with blocks,
and the Node process fixture without and with blocks. Each variant is compared with a build of the
same binary without plugins: added time per published page, the lsg process's peak memory, and
identical output.
"""
import argparse
import hashlib
import json
import os
import shutil
import subprocess
import sys
import tempfile
import time
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import build  # noqa: E402
import common  # noqa: E402
import metrics  # noqa: E402

FIXTURES = common.REPO / 'tests' / 'fixtures' / 'plugins'
EXAMPLE = 'build_plugin'


def example_binary():
    target = Path(os.environ.get('CARGO_TARGET_DIR', common.REPO / 'target'))
    return target / 'release' / 'examples' / EXAMPLE


def run(command):
    """Run a build; return wall seconds and the process's peak resident memory in bytes."""
    started = time.perf_counter()
    process = subprocess.Popen(command, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    _, status, usage = os.wait4(process.pid, 0)
    elapsed = time.perf_counter() - started
    if os.waitstatus_to_exitcode(status) != 0:
        raise SystemExit(f'build failed: {" ".join(command)}')
    # ru_maxrss is kilobytes on Linux and bytes on macOS.
    peak = usage.ru_maxrss * (1 if sys.platform == 'darwin' else 1024)
    return elapsed, peak


def tree_digest(site):
    digest = hashlib.sha256()
    for path in sorted(p for p in Path(site).rglob('*') if p.is_file()):
        digest.update(str(path.relative_to(site)).encode('utf-8') + b'\0')
        digest.update(path.read_bytes())
    return digest.hexdigest()


def published_pages(site):
    manifest = json.loads((Path(site) / 'lsg-site.json').read_text(encoding='utf-8'))
    return sum(1 for page in manifest['pages'] if page['kind'] in ('page', 'journal', 'namespace'))


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--garden', help='garden path (default: the encode garden from ghq)')
    parser.add_argument('--runs', type=int, default=3, help='builds per variant; the fastest counts (default: 3)')
    parser.add_argument('--output', help='write the measurements as JSON to this file')
    args = parser.parse_args()
    garden = Path(args.garden) if args.garden else common.encode_garden()
    for command in (['cargo', 'build', '--locked', '--release', '--bin', 'lsg'],
                    ['cargo', 'build', '--locked', '--release', '-p', 'logseq-gardener-sdk', '--example', EXAMPLE]):
        subprocess.run(command, cwd=common.REPO, check=True)
    lsg, example = str(common.lsg_binary()), str(example_binary())
    roots, _ = build.roots(garden / 'pages')
    selection = [option for root in roots for option in ('--include', root)]
    limits = metrics.load_budgets()['build_plugins']

    def lsg_build(output, *extra):
        return [lsg, 'publish', '--graph', str(garden), '--output', str(output),
                '--filename-format', 'triple-lowbar', '--journals', *selection, *extra]

    def example_build(output, *extra):
        return [example, str(garden), str(output), 'triple-lowbar', '--journals', *selection, *extra]

    variants = {
        'lsg without plugins': lambda out: lsg_build(out),
        'process no-op, pages only': lambda out: lsg_build(out, '--plugin', str(FIXTURES / 'noop' / 'lsg-plugin.json')),
        'process no-op with blocks': lambda out: lsg_build(out, '--plugin', str(FIXTURES / 'noop-blocks' / 'lsg-plugin.json')),
        'SDK example without plugins': lambda out: example_build(out, '--no-plugin'),
        'in-process no-op with blocks': lambda out: example_build(out),
    }
    baseline_of = {
        'process no-op, pages only': 'lsg without plugins',
        'process no-op with blocks': 'lsg without plugins',
        'in-process no-op with blocks': 'SDK example without plugins',
    }
    budget_of = {
        'process no-op, pages only': limits['process_pages_ms_per_page'],
        'process no-op with blocks': limits['process_blocks_ms_per_page'],
        'in-process no-op with blocks': limits['in_process_blocks_ms_per_page'],
    }
    results = {name: {'seconds': [], 'peak_bytes': []} for name in variants}
    with tempfile.TemporaryDirectory(prefix='lsg-build-bench-') as temp:
        temp = Path(temp)
        # Interleave the variants so that a slow moment of the machine affects all of them.
        for attempt in range(args.runs):
            for name, command in variants.items():
                output = temp / f'{len(results[name]["seconds"])}-{abs(hash(name))}'
                seconds, peak = run(command(output))
                results[name]['seconds'].append(seconds)
                results[name]['peak_bytes'].append(peak)
                if attempt == 0:
                    results[name]['digest'] = tree_digest(output)
                    results[name]['pages'] = published_pages(output)
                shutil.rmtree(output)
    failures = []
    pages = results['lsg without plugins']['pages']
    print(f'{garden} at {common.git_commit(garden)}: {pages:,} published pages, fastest of {args.runs} builds')
    for name, result in results.items():
        result['best_seconds'] = min(result['seconds'])
        result['peak_mb'] = max(result['peak_bytes']) / 1_000_000
        line = f'{name}: {result["best_seconds"]:.2f} s, peak {result["peak_mb"]:.0f} MB'
        if name in baseline_of:
            base = results[baseline_of[name]]
            added = (result['best_seconds'] - base['best_seconds']) * 1000 / pages
            result['added_ms_per_page'] = added
            line += f', {added:+.4f} ms per page (budget {budget_of[name]})'
            if added > budget_of[name]:
                failures.append(f'{name}: adds {added:.4f} ms per page, over {budget_of[name]}')
            if result['digest'] != base['digest']:
                failures.append(f'{name}: output differs from {baseline_of[name]}')
        print(line)
    growth = (results['process no-op with blocks']['peak_mb'] / results['lsg without plugins']['peak_mb'] - 1) * 100
    print(f'peak memory with the process no-op and blocks: {growth:+.1f}% (budget {limits["peak_memory_growth_pct"]}%)')
    if growth > limits['peak_memory_growth_pct']:
        failures.append(f'peak memory grows {growth:.1f}% with the process no-op and blocks, over {limits["peak_memory_growth_pct"]}%')
    results['peak_memory_growth_pct'] = growth
    if args.output:
        Path(args.output).write_text(json.dumps(results, indent=2) + '\n', encoding='utf-8')
    if failures:
        print('\n'.join(failures), file=sys.stderr)
        raise SystemExit(1)
    print('build plugins within budget, and no-op plugins leave the output unchanged')


if __name__ == '__main__':
    main()
