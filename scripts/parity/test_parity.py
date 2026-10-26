"""Unit tests for the parity harness; they need no garden, network or browser."""
import gzip
import os
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import budgets  # noqa: E402
import build  # noqa: E402
import common  # noqa: E402
import compare  # noqa: E402
import logseq_db  # noqa: E402
import metrics  # noqa: E402
import perf  # noqa: E402
import serve  # noqa: E402

TESTDATA = HERE / 'testdata'


class LogseqDbTest(unittest.TestCase):
    def setUp(self):
        self.pages = {p.name: p for p in logseq_db.read_pages(TESTDATA / 'db.gz')}

    def test_page_records(self):
        self.assertEqual(set(self.pages), {'tmux & co', '2026-10-23 fri', 'reference only'})
        self.assertTrue(self.pages['2026-10-23 fri'].journal)
        self.assertFalse(self.pages['tmux & co'].journal)

    def test_content_flag(self):
        self.assertTrue(self.pages['tmux & co'].has_content)
        self.assertTrue(self.pages['2026-10-23 fri'].has_content)
        self.assertFalse(self.pages['reference only'].has_content)

    def test_escaped_characters_and_cached_keys(self):
        self.assertEqual(self.pages['tmux & co'].title, 'Tmux & Co <"quoted"> it\'s')
        self.assertEqual(self.pages['2026-10-23 fri'].title, '2026-10-23 Fri')

    def test_missing_prefix(self):
        with self.assertRaisesRegex(logseq_db.FormatError, 'window.logseq_db='):
            logseq_db.decode_assignment('logseq_db="[]"')

    def test_missing_tag(self):
        with self.assertRaisesRegex(logseq_db.FormatError, 'datascript/DB'):
            logseq_db.decode_assignment('window.logseq_db="[1]"')


class TitleMappingTest(unittest.TestCase):
    def test_normalize(self):
        self.assertEqual(common.normalize('Café Notes'), 'café notes')

    def test_logseq_route(self):
        self.assertEqual(common.logseq_route('tmux'), '#/page/tmux')
        self.assertEqual(common.logseq_route('My Notes/Q & A'), '#/page/my%20notes%2Fq%20%26%20a')

    def test_lsg_index(self):
        links = common.lsg_pages((TESTDATA / 'index.html').read_text(encoding='utf-8'))
        self.assertEqual(links, {
            'tmux': 'p-07f89207b4ba08a4.html',
            'My Notes': 'p-0d63d4f0f8744f66.html',
            'My Notes/Q & A': 'p-22ec7f0ad9df5075.html',
        })

    def test_targets(self):
        index = {'tmux': ('tmux', 'p-1.html')}
        self.assertEqual(serve.targets('Tmux', {'tmux'}, index), {'logseq': '#/page/tmux', 'lsg': 'p-1.html'})
        self.assertEqual(serve.targets('2026-10-23 Fri', {'2026-10-23 fri'}, index),
                         {'logseq': '#/page/2026-10-23%20fri', 'lsg': None})
        self.assertEqual(serve.targets('All pages', set(), index), {'logseq': '#/all-pages', 'lsg': 'pages.html'})
        with tempfile.TemporaryDirectory() as site:
            self.assertEqual(serve.targets('All pages', set(), index, site)['lsg'], 'index.html')
            (Path(site) / 'pages.html').write_text('', encoding='utf-8')
            self.assertEqual(serve.targets('All pages', set(), index, site)['lsg'], 'pages.html')


class BuildTest(unittest.TestCase):
    def test_page_title(self):
        self.assertEqual(build.page_title('My___Notes%3A Q.md'), 'My/Notes: Q')

    def test_valid_root(self):
        self.assertTrue(build.valid_root('tmux'))
        self.assertFalse(build.valid_root(' tmux'))
        self.assertFalse(build.valid_root('a/ b'))
        self.assertFalse(build.valid_root('a\tb'))

    def test_roots(self):
        with tempfile.TemporaryDirectory() as temp:
            for name in ('Tools___tmux.md', 'Tools.md', 'Notes.org', '%20Space.md', 'image.png'):
                (Path(temp) / name).write_text('- x\n', encoding='utf-8')
            selected, skipped = build.roots(temp)
        self.assertEqual(selected, ['Notes', 'Tools'])
        self.assertEqual(skipped, ['%20Space.md'])


class CompareTest(unittest.TestCase):
    def test_hidden_patterns(self):
        config = ';; :hidden ["/x"]\n :hidden [\n "README.md",\n "pages/secret.md"]\n'
        self.assertEqual(compare.hidden_patterns(config), ['README.md', 'pages/secret.md'])

    def test_is_hidden(self):
        self.assertTrue(compare.is_hidden('pages/secret.md', ['pages/secret']))
        self.assertTrue(compare.is_hidden('pages/secret.md', ['/pages']))
        self.assertFalse(compare.is_hidden('pages/README.md', ['README.md']))

    def test_groups(self):
        page = logseq_db.Page
        logseq_pages = [
            page('both', 'Both', False, True),
            page('2026-10-23 fri', '2026-10-23 Fri', True, True),
            page('ref', 'Ref', False, False),
            page('secret', 'Secret', False, True),
            page('a/ /b', 'A/ /B', False, True),
            page('a/ b', 'A/ B', False, True),
            page('other', 'Other', False, True),
        ]
        groups = compare.group_pages(logseq_pages, {'both': 'Both', 'lsg only': 'LSG Only'}, {'secret'})
        self.assertEqual(groups['journals'], ['2026-10-23 Fri'])
        self.assertEqual(groups['reference_only'], ['Ref'])
        self.assertEqual(groups['hidden'], ['Secret'])
        self.assertEqual(groups['unselectable'], ['A/ /B'])
        self.assertEqual(groups['other'], ['A/ B', 'Other'])
        self.assertEqual(groups['lsg_only'], [{'title': 'LSG Only', 'hidden_in_logseq': False}])

    def test_journal_wins_over_reference_only(self):
        groups = compare.group_pages([logseq_db.Page('2026-01-01 thu', '2026-01-01 Thu', True, False)], {}, set())
        self.assertEqual(groups['journals'], ['2026-01-01 Thu'])
        self.assertEqual(groups['reference_only'], [])


class MetricsTest(unittest.TestCase):
    def test_gzip_size(self):
        data = b'tmux ' * 1000
        self.assertEqual(metrics.gzip_size(data), len(gzip.compress(data, 6)))

    def test_summaries(self):
        self.assertEqual(metrics.median([3, None, 1, 2]), 2)
        self.assertIsNone(metrics.median([None]))
        self.assertEqual(metrics.percentile(list(range(1, 101)), 0.95), 95)
        self.assertEqual(metrics.percentile([5], 0.95), 5)
        loads = [{'title_ms': 30, 'data_before_title': 10}, {'title_ms': 10, 'data_before_title': 30},
                 {'title_ms': 20, 'data_before_title': 20}]
        self.assertEqual(metrics.summarize_loads(loads)['title_ms'], 20)

    def test_keystroke_latencies(self):
        keys = [0, 300, 600]
        changes = [50, 120, 650]
        self.assertEqual(metrics.keystroke_latencies(keys, changes), [120, None, 50])

    def test_over_budget(self):
        limits = {'phone': {'data_before_title_kb': 100, 'title_ms': 1500}}
        self.assertEqual(metrics.over_budget('lsg', 'phone', 'data_before_title', 150_000, limits),
                         'data_before_title_kb = 100')
        self.assertIsNone(metrics.over_budget('lsg', 'phone', 'data_before_title', 90_000, limits))
        self.assertIsNone(metrics.over_budget('logseq', 'phone', 'title_ms', 90_000, limits))
        self.assertIsNone(metrics.over_budget('lsg', 'phone', 'fcp_ms', 90_000, limits))

    def test_committed_budgets(self):
        loaded = metrics.load_budgets()
        self.assertEqual(set(loaded), {'desktop', 'phone', 'shell', 'journals', 'outline', 'inline', 'code', 'search',
                                       'transclusion', 'references', 'all_pages'})
        self.assertEqual(set(loaded['all_pages']), {'paging_page_gzip_kb', 'growth_kb', 'script_gzip_kb',
                                                    'list_gzip_kb', 'sort_heap_mb', 'search_ms_desktop',
                                                    'search_ms_phone', 'build_slowdown_pct'})
        self.assertEqual(set(loaded['references']), {'script_kb', 'styles_gzip_kb', 'fragment_gzip_kb',
                                                     'open_ms_desktop', 'open_ms_phone', 'open_heap_mb',
                                                     'filter_ms', 'build_slowdown_pct'})
        self.assertEqual(set(loaded['transclusion']), {'styles_gzip_kb'})
        self.assertEqual(set(loaded['code']), {'styles_gzip_kb', 'largest_page_baseline_gzip_kb',
                                               'page_growth_gzip_kb', 'build_slowdown_pct'})
        self.assertEqual(set(loaded['outline']), {'script_kb', 'styles_gzip_kb', 'zoom_ms_desktop', 'zoom_ms_phone'})
        self.assertEqual(set(loaded['shell']), {'script_kb', 'frame_kb'})
        self.assertEqual(set(loaded['journals']), {'script_kb', 'scroll_heap_mb', 'build_slowdown_pct'})
        for profile in budgets.profiles(loaded).values():
            self.assertEqual(set(profile), {'data_before_title_kb', 'script_before_title_kb', 'title_ms',
                                            'peak_heap_mb', 'search_p95_ms'})


class PerfMergeTest(unittest.TestCase):
    def test_shared_ready_and_completion_are_explicit(self):
        self.assertIn('[open]', perf.SEARCH['lsg']['ready'])
        self.assertEqual(perf.SEARCH['lsg']['complete'], 'data-lsg-search-complete')

    def test_initial_search_budget_includes_module_and_deferred_scripts(self):
        with tempfile.TemporaryDirectory() as temp:
            site = Path(temp)
            (site / 'lsg-search-start-test.mjs').write_text('x' * 100)
            (site / 'deferred.js').write_text('x' * 100)
            (site / 'index.html').write_text('<script type="module" src="lsg-search-start-test.mjs"></script><script defer src="deferred.js"></script>')
            self.assertEqual(len(budgets.check_search(site, {'bootstrap_kb': 2}, {'desktop': {'script_before_title_kb': .15}})), 1)
            self.assertEqual(budgets.check_search(site, {'bootstrap_kb': 2}, {'desktop': {'script_before_title_kb': .3}}), [])


    def test_standalone_validation_rejects_missing_completion_and_nonfinite_latency(self):
        report = {'corpus': {'decoded_bytes': 100, 'gzip_bytes': 50, 'bootstrap_bytes': 100},
                  'search': [{'profile': 'desktop', 'first_open_ms': 100, 'warm_open_ms': 20,
                              'heap_growth': 1000, 'eager_search_requests': 0, 'warm_open_requests': 0,
                              'keystrokes': 4, 'unchanged': 0, 'p95_ms': 5}]}
        limits = metrics.load_budgets()
        self.assertEqual(perf.search_failures(report, limits), [])
        report['search'][0]['unchanged'] = 1
        self.assertTrue(perf.search_failures(report, limits))
        report['search'][0]['unchanged'] = 0
        report['search'][0]['p95_ms'] = None
        self.assertTrue(perf.search_failures(report, limits))
        report['search'][0] = {'profile': 'desktop', 'error': 'not ready'}
        self.assertTrue(perf.search_failures(report, limits))

    def test_keeps_rows_of_builds_not_measured(self):
        kept = [{'page': 'tmux', 'profile': 'desktop', 'build': 'logseq', 'title_ms': 15000},
                {'page': 'tmux', 'profile': 'desktop', 'build': 'lsg', 'title_ms': 40}]
        measured = [{'page': 'tmux', 'profile': 'desktop', 'build': 'lsg', 'title_ms': 30}]
        merged = perf.merge(kept, measured, ['lsg'], ('page', 'profile', 'build'))
        self.assertEqual([(r['build'], r['title_ms']) for r in merged], [('logseq', 15000), ('lsg', 30)])


class BudgetCheckTest(unittest.TestCase):
    limits = {'desktop': {'data_before_title_kb': 100, 'script_before_title_kb': 16}}

    def test_within_budget(self):
        sizes = {'p-1.html': (20_000, 5_000, 8_000)}
        self.assertEqual(budgets.check(sizes, dict(sizes), self.limits), [])

    def test_page_grows_with_garden(self):
        failures = budgets.check({'p-1.html': (20_000, 5_000, 8_000)}, {'p-1.html': (20_500, 5_100, 8_000)},
                                 self.limits)
        self.assertEqual(len(failures), 1)
        self.assertIn('grew from 20,000 to 20,500', failures[0])

    def test_over_budget(self):
        failures = budgets.check({'p-1.html': (900_000, 150_000, 20_000)}, {'p-1.html': (900_000, 150_000, 20_000)},
                                 self.limits)
        self.assertEqual(len(failures), 2)

    def test_journals_script_and_home_page(self):
        with tempfile.TemporaryDirectory() as temp:
            small, older = Path(temp) / 'small', Path(temp) / 'older'
            for site, home in ((small, 'x' * 100), (older, 'x' * 120)):
                site.mkdir()
                (site / budgets.JOURNALS_SCRIPT).write_text('y' * 50, encoding='utf-8')
                (site / 'index.html').write_text(home, encoding='utf-8')
            failures = budgets.check_journals(small, older, {'script_kb': 0.04})
            self.assertEqual(len(failures), 2)
            self.assertIn('index.html: size changed from 100 to 120', failures[1])
            self.assertEqual(budgets.check_journals(small, small, {'script_kb': 4}), [])
            garden = Path(temp) / 'garden'
            (garden / 'journals').mkdir(parents=True)
            budgets.add_older_journals(garden, 3)
            self.assertEqual(sorted(p.name for p in (garden / 'journals').iterdir()),
                             ['2000_01_01.md', '2000_01_02.md', '2000_01_03.md'])

    def test_outline_script_and_styles(self):
        with tempfile.TemporaryDirectory() as temp:
            site = Path(temp)
            styles = site / 'outline.css'
            styles.write_text('.outline{}' * 50, encoding='utf-8')
            self.assertEqual(budgets.check_outline(site, {'script_kb': 8, 'styles_gzip_kb': 1.5}, styles),
                             ['lsg-outline.js: missing from the build'])
            (site / budgets.OUTLINE_SCRIPT).write_text('x' * 100, encoding='utf-8')
            self.assertEqual(budgets.check_outline(site, {'script_kb': 8, 'styles_gzip_kb': 1.5}, styles), [])
            failures = budgets.check_outline(site, {'script_kb': 0.05, 'styles_gzip_kb': 0.001}, styles)
            self.assertEqual(len(failures), 2)

    def test_inline_styles_and_largest_page_growth(self):
        with tempfile.TemporaryDirectory() as temp:
            styles = Path(temp) / 'inline.css'
            styles.write_text('.tag{}' * 20, encoding='utf-8')
            limits = {'styles_gzip_kb': 1, 'largest_page_baseline_gzip_kb': 7, 'largest_page_growth_kb': 2}
            pages = {'p-1.html': (100, 8_000, 0), 'p-2.html': (100, 6_000, 0)}
            self.assertEqual(budgets.check_inline(pages, limits, styles), [])
            pages['p-2.html'] = (100, 9_500, 0)
            failures = budgets.check_inline(pages, {**limits, 'styles_gzip_kb': 0.001}, styles)
            self.assertEqual(len(failures), 2)
            self.assertIn('p-2.html', failures[1])

    def test_code_styles_and_largest_code_page_growth(self):
        with tempfile.TemporaryDirectory() as temp:
            site = Path(temp)
            styles = site / 'code.css'
            styles.write_text('.cm-keyword{}' * 20, encoding='utf-8')
            (site / 'p-1.html').write_text('<div class="cp__fenced-code-block"></div>', encoding='utf-8')
            (site / 'p-2.html').write_text('<p>no code</p>', encoding='utf-8')
            limits = {'styles_gzip_kb': 1.5, 'largest_page_baseline_gzip_kb': 8, 'page_growth_gzip_kb': 10}
            pages = {'p-1.html': (100, 9_000, 0), 'p-2.html': (100, 30_000, 0)}
            self.assertEqual(budgets.check_code(site, pages, limits, styles), [])
            pages['p-1.html'] = (100, 18_500, 0)
            failures = budgets.check_code(site, pages, {**limits, 'styles_gzip_kb': 0.001}, styles)
            self.assertEqual(len(failures), 2)
            self.assertIn('p-1.html', failures[1])
            self.assertEqual(budgets.check_code(site, {'p-2.html': pages['p-2.html']}, limits, styles),
                             ['no fixture page has fenced code'])

    def test_transclusion_styles_and_an_embedding_fixture(self):
        with tempfile.TemporaryDirectory() as temp:
            site = Path(temp)
            styles = site / 'transclusion.css'
            styles.write_text('.block-ref{}' * 20, encoding='utf-8')
            (site / 'p-1.html').write_text('<div class="embed-page" data-level="1"></div>', encoding='utf-8')
            (site / 'p-2.html').write_text('<p>no embed</p>', encoding='utf-8')
            self.assertEqual(budgets.check_transclusion(site, ['p-1.html', 'p-2.html'], {'styles_gzip_kb': 1}, styles), [])
            failures = budgets.check_transclusion(site, ['p-2.html'], {'styles_gzip_kb': 0.001}, styles)
            self.assertEqual(len(failures), 2)
            self.assertEqual(failures[1], 'no fixture page has an embed')

    def test_references_script_styles_and_fragments(self):
        with tempfile.TemporaryDirectory() as temp:
            site = Path(temp)
            styles = site / 'references.css'
            styles.write_text('.references{margin-top:24px}', encoding='utf-8')
            limits = {'script_kb': 1, 'styles_gzip_kb': 1, 'fragment_gzip_kb': 1}
            self.assertEqual(budgets.check_references(site, limits, styles),
                             ['lsg-references.js: missing from the build'])
            (site / 'lsg-references.js').write_text('x' * 500, encoding='utf-8')
            (site / '_refs').mkdir()
            self.assertEqual(budgets.check_references(site, limits, styles),
                             ['no fixture page has a reference fragment'])
            (site / '_refs' / 'p-1-linked-1.html').write_text('<p>small</p>', encoding='utf-8')
            self.assertEqual(budgets.check_references(site, limits, styles), [])
            (site / '_refs' / 'p-2-linked-1.html').write_bytes(os.urandom(3000))
            failures = budgets.check_references(site, limits, styles)
            self.assertEqual(len(failures), 1)
            self.assertIn('_refs/p-2-linked-1.html', failures[0])

    def test_all_pages_paging_growth_script_and_list(self):
        limits = {'desktop': {'data_before_title_kb': 100, 'script_before_title_kb': 16},
                  'phone': {'data_before_title_kb': 100, 'script_before_title_kb': 16},
                  'all_pages': {'paging_page_gzip_kb': 1, 'growth_kb': 1, 'script_gzip_kb': 1, 'list_gzip_kb': 1}}
        with tempfile.TemporaryDirectory() as temp:
            small, large = Path(temp) / 'small', Path(temp) / 'large'
            for site, rows in ((small, 40), (large, 40)):
                (site / '_pages').mkdir(parents=True)
                (site / 'pages.html').write_text('<tr></tr>' * rows, encoding='utf-8')
                (site / budgets.ALL_PAGES_SCRIPT).write_text('x' * 100, encoding='utf-8')
                (site / budgets.ALL_PAGES_LIST).write_text('[]', encoding='utf-8')
            (small / 'pages-2.html').write_text('<p>small</p>', encoding='utf-8')
            (small / 'p-1.html').write_text('<p>page</p>', encoding='utf-8')
            self.assertEqual(budgets.check_all_pages(small, large, limits), [])
            self.assertEqual(budgets.site_pages(small), ['p-1.html'])
            (small / 'pages-3.html').write_bytes(os.urandom(3000))
            (large / 'pages.html').write_text('<tr></tr>' * 400, encoding='utf-8')
            (small / 'p-1.html').write_text(f'<script src="{budgets.ALL_PAGES_SCRIPT}"></script>', encoding='utf-8')
            failures = budgets.check_all_pages(small, large, limits)
            self.assertEqual(len(failures), 3, failures)
            self.assertIn('pages-3.html', failures[0])
            self.assertIn('grew', failures[1])
            self.assertIn('loaded outside All pages', failures[2])

    def test_shell_script_and_frame(self):
        with tempfile.TemporaryDirectory() as temp:
            small, large = Path(temp) / 'small', Path(temp) / 'large'
            for site, script in ((small, 'x' * 100), (large, 'x' * 101)):
                site.mkdir()
                (site / budgets.SHELL_SCRIPT).write_text(script, encoding='utf-8')
                (site / 'p-1.html').write_text('<body>' + 'y' * 200 + '<main>body</main>', encoding='utf-8')
            self.assertEqual(budgets.frame_size(small, 'p-1.html'), 206)
            failures = budgets.check_shell(small, large, ['p-1.html'], {'script_kb': 0.05, 'frame_kb': 0.1})
            self.assertEqual(len(failures), 3)
            self.assertEqual(budgets.check_shell(small, small, ['p-1.html'], {'script_kb': 6, 'frame_kb': 6}), [])

    def test_page_sizes_include_blocking_files(self):
        with tempfile.TemporaryDirectory() as temp:
            site = Path(temp)
            (site / 'custom.css').write_text('body{}', encoding='utf-8')
            (site / 'app.js').write_text('let a=1;', encoding='utf-8')
            (site / 'late.js').write_text('let b=2;', encoding='utf-8')
            page = ('<link rel="stylesheet" href="custom.css"><script>let c=3;</script>'
                    '<script src="app.js"></script><script defer src="late.js"></script><h1>T</h1>')
            (site / 'p-1.html').write_text(page, encoding='utf-8')
            raw, _, script = budgets.page_sizes(site, 'p-1.html')
        self.assertEqual(raw, len(page) + len('body{}') + len('let a=1;'))
        self.assertEqual(script, len('let c=3;') + len('let a=1;'))


if __name__ == '__main__':
    unittest.main()
