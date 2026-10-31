#!/usr/bin/env python3
"""Time `lsg publish` on representative gardens and check the result against recorded rows.

Each corpus is published once cold and then `--runs` times warm, with `--journals`, `--search`,
`--timings` and every top-level namespace included. The first build is the cold one: it runs
right after the garden is written or checked out, without clearing the operating system's file
cache, which needs administrator rights. The fastest warm build counts, as in parity:build-bench,
because noise on a shared machine only ever adds time.

A row records the lsg commit, the corpus and its digest or commit, the machine, the cold and warm
times, the phase times of the fastest warm build, peak memory and the output's size. `--check`
compares the fastest warm build with the recorded rows for the same corpus; see
docs/parity/publication-speed.md for how the limit is chosen.
"""
import argparse
import json
import os
import platform
import shutil
import statistics
import subprocess
import sys
import tempfile
import time
from datetime import datetime, timezone
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import build  # noqa: E402
import common  # noqa: E402
import metrics  # noqa: E402
import sample_garden  # noqa: E402

ROWS = common.REPO / 'docs' / 'parity' / 'publication-speed.json'
CORPORA = ('gitpa-sized', 'encode-garden')
OPTIONS = ['--filename-format', 'triple-lowbar', '--journals', '--search', '--timings']


def cpu_model():
    """The processor's model name, which decides which recorded rows are comparable."""
    if sys.platform == 'darwin':
        result = subprocess.run(['sysctl', '-n', 'machdep.cpu.brand_string'], capture_output=True, text=True)
        return result.stdout.strip() or platform.machine()
    try:
        for line in Path('/proc/cpuinfo').read_text(encoding='utf-8').splitlines():
            if line.startswith('model name'):
                return line.split(':', 1)[1].strip()
    except OSError:
        pass
    return platform.processor() or platform.machine()


def environment():
    return {
        'runner': 'github-actions' if os.environ.get('GITHUB_ACTIONS') == 'true' else 'local',
        'os': f'{platform.system()} {platform.release()}',
        'machine': platform.machine(),
        'cpu': cpu_model(),
        'cpus': os.cpu_count(),
    }


def lsg_revision():
    commit = common.git_commit(common.REPO)
    dirty = subprocess.run(['git', '-C', str(common.REPO), 'status', '--porcelain', '--untracked-files=no'],
                           capture_output=True, text=True, check=True).stdout.strip()
    return commit + ('-modified' if dirty else '')


def source_size(garden):
    files = [p for d in ('pages', 'journals') for p in (Path(garden) / d).rglob('*.md')]
    return len(files), sum(p.stat().st_size for p in files)


def output_size(site):
    files = [p for p in Path(site).rglob('*') if p.is_file()]
    return len(files), sum(p.stat().st_size for p in files)


def run(command):
    """Run one build; return wall seconds, peak resident memory in bytes and the JSON report."""
    started = time.perf_counter()
    process = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL)
    stdout = process.stdout.read()
    _, status, usage = os.wait4(process.pid, 0)
    elapsed = time.perf_counter() - started
    if os.waitstatus_to_exitcode(status) != 0:
        raise SystemExit(f'build failed: {" ".join(command[:6])} ...')
    # ru_maxrss is kilobytes on Linux and bytes on macOS.
    peak = usage.ru_maxrss * (1 if sys.platform == 'darwin' else 1024)
    return elapsed, peak, json.loads(stdout)


def measure(lsg, garden, runs, temp):
    roots, _ = build.roots(Path(garden) / 'pages')
    selection = [option for root in roots for option in ('--include', root)]
    builds = []
    for attempt in range(runs + 1):
        output = Path(temp) / f'site-{attempt}'
        seconds, peak, report = run([lsg, '--format', 'json', 'publish', '--graph', str(garden),
                                     '--output', str(output), *OPTIONS, *selection])
        builds.append({'seconds': seconds, 'peak': peak, 'report': report})
        if attempt == 0:
            files, size = output_size(output)
        shutil.rmtree(output)
    cold, warm = builds[0], builds[1:]
    fastest = min(warm, key=lambda b: b['seconds'])
    report = fastest['report']
    return {
        'cold_s': round(cold['seconds'], 3),
        'warm_s': round(fastest['seconds'], 3),
        'warm_median_s': round(statistics.median(b['seconds'] for b in warm), 3),
        'runs': runs,
        'phases_ms': {phase['phase']: phase['ms'] for phase in report['timings']},
        'peak_mb': round(max(b['peak'] for b in builds) / 1_000_000),
        'published': {'pages': report['pages'], 'journals': report['journals'],
                      'withheld_pages': report['withheld_pages'], 'assets': report['assets']},
        'output': {'files': files, 'mb': round(size / 1_000_000, 1)},
    }


def corpus_row(name, garden, digest_or_commit):
    files, size = source_size(garden)
    key = 'digest' if name == 'gitpa-sized' else 'commit'
    corpus = {'name': name, key: digest_or_commit, 'source_files': files, 'source_mb': round(size / 1_000_000, 1)}
    if name == 'gitpa-sized':
        corpus['generator_version'] = sample_garden.VERSION
    return corpus


def comparable(row, corpus):
    key = 'digest' if 'digest' in corpus else 'commit'
    return row['corpus']['name'] == corpus['name'] and row['corpus'].get(key) == corpus[key]


def limit_for(rows, corpus, env, budget):
    """The warm time limit in seconds and why, from the recorded rows for this corpus.

    Rows from the same runner and processor model set the limit at the slowest of them plus
    `slowdown_pct`. Without such a row, the slowest row from the same runner sets it plus
    `unknown_cpu_slowdown_pct`, which allows for the spread between processor models.
    """
    same_corpus = [r for r in rows if comparable(r, corpus) and r['environment']['runner'] == env['runner']]
    same_cpu = [r for r in same_corpus if r['environment']['cpu'] == env['cpu']]
    if same_cpu:
        base = max(r['warm_s'] for r in same_cpu)
        return base * (1 + budget['slowdown_pct'] / 100), f'{len(same_cpu)} rows on {env["cpu"]}, slowest {base:.2f} s, plus {budget["slowdown_pct"]}%'
    if same_corpus:
        base = max(r['warm_s'] for r in same_corpus)
        return (base * (1 + budget['unknown_cpu_slowdown_pct'] / 100),
                f'no row on {env["cpu"]}; slowest of {len(same_corpus)} rows {base:.2f} s, plus {budget["unknown_cpu_slowdown_pct"]}%')
    return None, f'no recorded row for {corpus["name"]} on {env["runner"]}'


def describe(row):
    phases = ', '.join(f'{name} {ms / 1000:.2f}' for name, ms in row['phases_ms'].items())
    return (f'{row["corpus"]["name"]}: cold {row["cold_s"]:.2f} s, warm {row["warm_s"]:.2f} s '
            f'(median {row["warm_median_s"]:.2f} s of {row["runs"]}); phases in seconds: {phases}; '
            f'peak {row["peak_mb"]} MB; {row["output"]["files"]:,} files, {row["output"]["mb"]} MB')


def load_rows(path=ROWS):
    if not Path(path).exists():
        return []
    return json.loads(Path(path).read_text(encoding='utf-8'))['rows']


def save_rows(rows, path=ROWS):
    document = {'format_version': 1,
                'about': 'Recorded by mise run parity:publish-bench --record; see publication-speed.md.',
                'rows': rows}
    Path(path).write_text(json.dumps(document, indent=2, ensure_ascii=False) + '\n', encoding='utf-8')


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument('--corpus', choices=(*CORPORA, 'all'), default='gitpa-sized',
                        help='garden to publish (default: gitpa-sized)')
    parser.add_argument('--garden', help='encode garden path (default: the encode garden from ghq)')
    parser.add_argument('--runs', type=int, default=5, help='warm builds after the cold one (default: 5)')
    parser.add_argument('--record', action='store_true', help=f'append the rows to {ROWS.relative_to(common.REPO)}')
    parser.add_argument('--check', action='store_true', help='fail when a warm build exceeds its limit')
    parser.add_argument('--output', help='write the rows as JSON to this file')
    args = parser.parse_args()
    subprocess.run(['cargo', 'build', '--locked', '--release', '--bin', 'lsg'], cwd=common.REPO, check=True)
    lsg = str(common.lsg_binary())
    names = CORPORA if args.corpus == 'all' else (args.corpus,)
    env, revision = environment(), lsg_revision()
    budget = metrics.load_budgets()['publication']
    recorded = load_rows()
    rows, failures = [], []
    with tempfile.TemporaryDirectory(prefix='lsg-publish-bench-') as temp:
        for name in names:
            if name == 'gitpa-sized':
                garden = Path(temp) / 'garden'
                identity = sample_garden.generate(garden)
            else:
                garden = Path(args.garden) if args.garden else common.encode_garden()
                identity = common.git_commit(garden)
            row = {'recorded': datetime.now(timezone.utc).strftime('%Y-%m-%d'), 'lsg': revision,
                   'corpus': corpus_row(name, garden, identity), 'environment': env, 'options': OPTIONS[2:4],
                   **measure(lsg, garden, args.runs, temp)}
            rows.append(row)
            print(describe(row))
            if args.check:
                limit, reason = limit_for(recorded, row['corpus'], env, budget)
                if limit is None:
                    print(f'  not checked: {reason}; record a row with --record')
                elif row['warm_s'] > limit:
                    failures.append(f'{name}: warm build {row["warm_s"]:.2f} s exceeds {limit:.2f} s ({reason})')
                else:
                    print(f'  within {limit:.2f} s ({reason})')
    if args.output:
        Path(args.output).write_text(json.dumps(rows, indent=2) + '\n', encoding='utf-8')
    if args.record:
        save_rows(recorded + rows)
        print(f'recorded {len(rows)} rows in {ROWS.relative_to(common.REPO)}')
    if failures:
        print('\n'.join(failures), file=sys.stderr)
        raise SystemExit(1)


if __name__ == '__main__':
    main()
