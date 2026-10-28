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

    def test_lsg_index_reads_the_all_pages_data_file(self):
        with tempfile.TemporaryDirectory() as site:
            (Path(site) / '_pages').mkdir()
            (Path(site) / 'pages.html').write_text(
                '<div class="all-pages" data-list-src="_pages/list.json"><table><tbody>'
                '<tr><td><a href="p-07f89207b4ba08a4.html">tmux</a></td></tr></tbody></table></div>',
                encoding='utf-8')
            (Path(site) / '_pages' / 'list.json').write_text(
                '[["tmux",null,"p",3,null,null],["My Notes",null,"n",0,null,null],'
                '["Renamed","p-0000000000000001.html","p",0,null,null]]', encoding='utf-8')
            self.assertEqual(common.lsg_index(site), {
                'tmux': ('tmux', common.lsg_route('tmux')),
                'my notes': ('My Notes', common.lsg_route('My Notes')),
                'renamed': ('Renamed', 'p-0000000000000001.html'),
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
            for name in ('Tools___tmux.md', 'Tools.md', 'Notes.org', '%20Space.md', 'image.png', 'contents.md'):
                (Path(temp) / name).write_text('- x\n', encoding='utf-8')
            selected, skipped = build.roots(temp)
        self.assertEqual(selected, ['Contents', 'Notes', 'Tools'])
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

    def test_completion_latencies(self):
        # The second key's input was replaced before a render showed it; the render of `tmu` answers it.
        keys, values = [0, 300, 600], ['t', 'tm', 'tmu']
        completions = [(40, 't'), (700, 'tmu')]
        self.assertEqual(metrics.completion_latencies(keys, values, completions), [40, 400, 100])
        self.assertEqual(metrics.completion_latencies([0], ['x'], [(10, 'y')]), [None])

    def test_keystroke_fetches(self):
        resources = [(10, 'lsg-search-shard-a.json', 5000), (20, 'lsg-search-chunk-b.json', 3000),
                     (320, 'lsg-search-chunk-c.json', 1000)]
        self.assertEqual(metrics.keystroke_fetches([0, 300, 600], resources, perf.SHARD_FILE),
                         [{'bytes': 8000, 'shard': True}, {'bytes': 1000, 'shard': False},
                          {'bytes': 0, 'shard': False}])

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
                                       'transclusion', 'references', 'all_pages', 'hierarchy', 'sidebar', 'graph', 'media',
                                       'slides', 'queries', 'flashcards'})
        self.assertEqual(set(loaded['flashcards']), {'script_kb', 'styles_gzip_kb', 'deck_gzip_kb',
                                                     'card_batch_gzip_kb', 'deck_page', 'deck_page_growth_gzip_kb',
                                                     'open_ms_desktop', 'open_ms_phone', 'review_heap_mb',
                                                     'build_slowdown_pct'})
        self.assertEqual(set(loaded['queries']), {'script_kb', 'styles_gzip_kb', 'in_page_gzip_kb',
                                                  'fragment_gzip_kb', 'open_ms_desktop', 'open_ms_phone',
                                                  'open_heap_mb', 'build_slowdown_pct'})
        self.assertEqual(set(loaded['slides']), {'script_gzip_kb', 'open_ms_desktop', 'step_ms_desktop', 'deck_heap_mb'})
        self.assertEqual(loaded['shell']['script_kb'], 6.9)
        self.assertEqual(set(loaded['graph']), {'data_gzip_kb', 'script_kb', 'worker_kb', 'first_frame_ms_desktop',
                                                'first_frame_ms_phone', 'heap_mb', 'frame_ms_phone',
                                                'page_graph_fragment_gzip_kb', 'page_graph_open_ms_desktop',
                                                'page_graph_open_ms_phone', 'build_slowdown_pct'})
        self.assertEqual(set(loaded['all_pages']), {'paging_page_gzip_kb', 'growth_kb', 'script_gzip_kb',
                                                    'list_gzip_kb', 'sort_heap_mb', 'search_ms_desktop',
                                                    'search_ms_phone', 'build_slowdown_pct'})
        self.assertEqual(set(loaded['hierarchy']), {'inline_rows', 'styles_gzip_kb', 'script_baseline_kb',
                                                    'script_growth_kb', 'fragment_gzip_kb', 'root_growth_kb',
                                                    'scroll_heap_mb', 'build_slowdown_pct'})
        self.assertEqual(set(loaded['references']), {'script_kb', 'styles_gzip_kb', 'fragment_gzip_kb',
                                                     'open_ms_desktop', 'open_ms_phone', 'open_heap_mb',
                                                     'filter_ms', 'build_slowdown_pct'})
        self.assertEqual(set(loaded['transclusion']), {'styles_gzip_kb'})
        self.assertEqual(set(loaded['code']), {'styles_gzip_kb', 'largest_page_baseline_gzip_kb',
                                               'page_growth_gzip_kb', 'build_slowdown_pct'})
        self.assertEqual(set(loaded['outline']), {'script_kb', 'styles_gzip_kb', 'zoom_ms_desktop', 'zoom_ms_phone'})
        self.assertEqual(set(loaded['shell']), {'script_kb', 'frame_kb'})
        self.assertEqual(set(loaded['journals']), {'script_kb', 'scroll_heap_mb', 'build_slowdown_pct'})
        self.assertEqual(set(loaded['search']), {
            'bootstrap_kb', 'titles_gzip_kb', 'titles_decoded_mb', 'shard_gzip_kb', 'chunk_gzip_kb',
            'keystroke_gzip_kb', 'dialog_ms_desktop', 'dialog_ms_phone', 'first_results_ms_desktop',
            'first_results_ms_phone', 'cold_keystroke_ms_desktop', 'cold_keystroke_ms_phone', 'heap_growth_mb',
            'warm_open_desktop_ms', 'warm_open_phone_ms', 'build_slowdown_pct'})
        for profile in budgets.profiles(loaded).values():
            self.assertEqual(set(profile), {'data_before_title_kb', 'script_before_title_kb', 'title_ms',
                                            'peak_heap_mb', 'search_p95_ms'})


class QueriesBudgetTest(unittest.TestCase):
    def test_query_boxes(self):
        one = '<div class="custom-query" data-lsg-query="q1"><div class="th">x</div><div>y</div></div>'
        collapsed = '<details class="custom-query lsg-query-collapsed"><summary>t</summary><div></div></details>'
        self.assertEqual(budgets.query_boxes(f'<li>{one}</li><p>{collapsed}</p>'), [one, collapsed])
        self.assertEqual(budgets.query_boxes('<p>no query</p>'), [])

    def test_query_fragment_names(self):
        self.assertTrue(budgets.QUERY_FRAGMENT.search('p-0123-q2-1.html'))
        self.assertTrue(budgets.QUERY_FRAGMENT.search('p-0123-q2-alt-3.html'))
        self.assertFalse(budgets.QUERY_FRAGMENT.search('p-0123-linked-1.html'))


class FlashcardsBudgetTest(unittest.TestCase):
    def test_without_deck_widgets(self):
        widget = ('<div class="cards-review" data-lsg-deck="d1"><div class="cards-title">x</div>'
                  '<div class="cards-first"><div>y</div></div></div>')
        empty = '<div class="cards-review cards-empty"><code>Cards: z</code></div>'
        page = f'<style>.cards-review{{padding:12px}}</style><style>p{{}}</style><p>a</p>{widget}<p>b</p>{empty}'
        self.assertEqual(budgets.without_deck_widgets(page), '<style>p{}</style><p>a</p><p>b</p>')
        self.assertEqual(budgets.without_deck_widgets('<p>no deck</p>'), '<p>no deck</p>')

    def test_flashcards_checks(self):
        limits = {'script_kb': 1, 'styles_gzip_kb': 1, 'deck_gzip_kb': 1, 'card_batch_gzip_kb': 1,
                  'deck_page': 'Tools/Flashcards', 'deck_page_growth_gzip_kb': 1}
        with tempfile.TemporaryDirectory() as temp:
            site = Path(temp)
            self.assertIn('missing from the build', budgets.check_flashcards(site, [], limits)[0])
            (site / 'flashcards' / 'decks').mkdir(parents=True)
            (site / 'flashcards' / 'cards').mkdir()
            (site / 'lsg-flashcards.js').write_text('x' * 1001)
            (site / 'lsg-flashcards.css').write_text('a{}')
            (site / 'flashcards' / 'decks' / 'all.json').write_text('{}')
            (site / 'flashcards' / 'cards' / '0.html').write_text('<p>card</p>')
            (site / common.lsg_route('Tools/Flashcards')).write_text(
                '<div class="cards-review" data-lsg-deck="d1"><p>x</p></div>')
            (site / 'p-other.html').write_text('<script src="lsg-flashcards.js"></script>')
            failures = budgets.check_flashcards(site, ['p-other.html'], limits)
            self.assertEqual(len(failures), 2)
            self.assertTrue(failures[0].startswith('lsg-flashcards.js: 1,001 exceeds 1,000'))
            self.assertIn('p-other.html: links lsg-flashcards.js', failures[1])
            (site / 'lsg-flashcards.js').write_text('x')
            self.assertEqual(budgets.check_flashcards(site, [], limits), [])


class MediaBudgetTest(unittest.TestCase):
    def test_placeholder_sizes(self):
        one = '<div class="lsg-video" data-provider="youtube"><a class="lsg-video-play" href="x"></a></div>'
        self.assertEqual(budgets.placeholder_sizes(f'<p>{one}</p>{one}'), [len(one), len(one)])
        self.assertEqual(budgets.placeholder_sizes('<p>no video</p>'), [])

    def test_player_hosts(self):
        self.assertTrue(perf.is_player_host('i.ytimg.com'))
        self.assertTrue(perf.is_player_host('www.youtube.com'))
        self.assertFalse(perf.is_player_host('pbs.twimg.com'))
        self.assertFalse(perf.is_player_host('notyoutube.com'))

    def test_media_budget_keys(self):
        loaded = metrics.load_budgets()
        self.assertLessEqual({'script_kb', 'styles_gzip_kb', 'placeholder_kb', 'timestamp_page_growth_gzip_kb',
                              'third_party_requests_before_click'}, set(loaded['media']))


class PerfMergeTest(unittest.TestCase):
    def test_shared_ready_and_completion_are_explicit(self):
        self.assertIn('[open]', perf.SEARCH['lsg']['ready'])
        self.assertEqual(perf.SEARCH['lsg']['complete'], 'data-lsg-search-complete')
        self.assertEqual(perf.SEARCH['lsg']['drawn'], 'data-lsg-search-drawn')

    def test_initial_search_budget_includes_module_and_deferred_scripts(self):
        with tempfile.TemporaryDirectory() as temp:
            site = Path(temp)
            (site / 'lsg-search-start-test.mjs').write_text('x' * 100)
            (site / 'deferred.js').write_text('x' * 100)
            (site / 'index.html').write_text('<script type="module" src="lsg-search-start-test.mjs"></script><script defer src="deferred.js"></script>')
            self.assertEqual(len(budgets.check_search(site, {'bootstrap_kb': 2}, {'desktop': {'script_before_title_kb': .15}})), 1)
            self.assertEqual(budgets.check_search(site, {'bootstrap_kb': 2}, {'desktop': {'script_before_title_kb': .3}}), [])


    def search_row(self, **changes):
        row = {'profile': 'phone', 'build': 'lsg', 'dialog_ms': 50, 'first_results_ms': 900, 'cold_p95_ms': 400,
               'warm_p95_ms': 40, 'keystroke_bytes_max': 90_000, 'warm_open_ms': 30, 'heap_growth': 9_000_000,
               'eager_search_requests': 0, 'warm_open_requests': 0, 'keystrokes': 4, 'unchanged': 0,
               'p50_ms': 20, 'p95_ms': 300, 'cold_keystrokes': 1, 'first_open_transfer': 200_000}
        return {**row, **changes}

    def test_standalone_validation_rejects_missing_completion_and_values_over_budget(self):
        limits = metrics.load_budgets()
        report = {'search': [self.search_row(), {**self.search_row(), 'build': 'logseq', 'cold_p95_ms': 9000}]}
        self.assertEqual(perf.search_failures(report, limits), [])
        for changes in ({'unchanged': 1}, {'cold_p95_ms': None}, {'cold_p95_ms': 1200},
                        {'keystroke_bytes_max': 250_000}, {'heap_growth': 40_000_000}, {'eager_search_requests': 1}):
            report['search'][0] = self.search_row(**changes)
            self.assertTrue(perf.search_failures(report, limits), changes)
        report['search'][0] = {'profile': 'desktop', 'build': 'lsg', 'error': 'not ready'}
        self.assertTrue(perf.search_failures(report, limits))

    def test_search_markdown_marks_lsg_values_over_budget(self):
        results = {'search': [self.search_row(cold_p95_ms=1200),
                              {**self.search_row(cold_p95_ms=9000), 'build': 'logseq'}]}
        text = '\n'.join(perf.search_markdown(results, metrics.load_budgets()))
        self.assertIn('| phone | lsg | 50 | 900 | 200.0 | 1 | **1,200** | 40 | 90.0 | 9.0 |', text)
        self.assertIn('| phone | logseq | 50 | 900 | 200.0 | 1 | 9,000 |', text)

    def test_build_over_budget(self):
        limits = {'journals': {'build_slowdown_pct': 20}, 'search': {'build_slowdown_pct': 25}}
        builds = {(True, True): 13.0, (False, True): 12.0, (True, False): 10.0}
        self.assertTrue(perf.build_over_budget((True, True), builds, limits))
        builds[(True, False)] = 11.0
        self.assertFalse(perf.build_over_budget((True, True), builds, limits))
        self.assertFalse(perf.build_over_budget((True, False), builds, limits))

    def test_keeps_rows_of_builds_not_measured(self):
        kept = [{'page': 'tmux', 'profile': 'desktop', 'build': 'logseq', 'title_ms': 15000},
                {'page': 'tmux', 'profile': 'desktop', 'build': 'lsg', 'title_ms': 40}]
        measured = [{'page': 'tmux', 'profile': 'desktop', 'build': 'lsg', 'title_ms': 30}]
        merged = perf.merge(kept, measured, ['lsg'], ('page', 'profile', 'build'))
        self.assertEqual([(r['build'], r['title_ms']) for r in merged], [('logseq', 15000), ('lsg', 30)])


class GraphTest(unittest.TestCase):
    LIMITS = {'data_gzip_kb': 1, 'script_kb': 1, 'worker_kb': 1, 'page_graph_fragment_gzip_kb': 1}

    def site(self, temp, data=100, neighbourhood=b'{}', page='<html>tmux</html>'):
        site = Path(temp)
        (site / '_graph/p').mkdir(parents=True)
        (site / '_graph/default.bin').write_bytes(os.urandom(data))
        (site / '_graph/p/0000000000000000.json').write_bytes(neighbourhood)
        for name in ('lsg-graph.js', 'lsg-graph.css', 'lsg-graph-worker.js'):
            (site / name).write_text('x')
        (site / 'graph.html').write_text('<script src="lsg-graph.js"></script>')
        (site / 'tmux.html').write_text(page)
        return site

    def test_graph_files_within_budget(self):
        with tempfile.TemporaryDirectory() as temp:
            site = self.site(temp)
            self.assertEqual(budgets.check_graph(site, ['graph.html', 'tmux.html'], self.LIMITS), [])
            self.assertEqual(budgets.graph_sizes(site)['neighbourhoods'], 1)

    def test_graph_files_over_budget_and_pages_that_load_them(self):
        with tempfile.TemporaryDirectory() as temp:
            site = self.site(temp, data=2000, neighbourhood=os.urandom(2000),
                             page='<script src="lsg-graph.js"></script>')
            failures = budgets.check_graph(site, ['graph.html', 'tmux.html'], self.LIMITS)
            self.assertEqual(len(failures), 3, failures)
            self.assertTrue(any('_graph/default.bin' in f for f in failures))
            self.assertTrue(any('tmux.html: loads graph data' in f for f in failures))

    def test_lsg_route_matches_the_publisher(self):
        self.assertEqual(common.lsg_route('tmux'), 'p-424036ef4353ec8f.html')

    def test_graph_markdown_marks_values_over_budget(self):
        report = {
            'graph': [{'profile': 'phone', 'build': 'lsg', 'first_frame_ms': 3500, 'pan_p95_ms': 20,
                       'peak_heap': 10e6, 'worker_loaded': False, 'data_transfer': 400_000}],
            'page_graph': [{'page': 'tmux', 'profile': 'desktop', 'open_ms': 120, 'file': 'p/x.json'}],
            'sizes': {'data': 400_000, 'script': 11_000, 'worker': 2_700, 'largest_neighbourhood': (85_000, 'x.json'),
                      'neighbourhoods': 3},
            'build': {'seconds': 15.0, 'baseline_seconds': 10.0, 'baseline': '/tmp/lsg'},
        }
        text = '\n'.join(perf.graph_markdown(report, metrics.load_budgets()))
        self.assertIn('| phone | lsg | **3,500** | 20 | 10.0 | 400 | no |', text)
        self.assertIn('| desktop | logseq | 2,291,000 |', text)
        self.assertIn('| tmux | desktop | 120 | `p/x.json` |', text)
        self.assertIn('**15.0** seconds', text)

    def test_sidebar_markdown_marks_values_over_budget(self):
        cards = [{'type': 'page', 'title': 'GitHub/Star', 'ms': 400, 'transfer': 11_300},
                 {'type': 'refs', 'title': 'Block references', 'ms': 30, 'transfer': 61_000}]
        row = {'profile': 'desktop', 'cards': cards, 'p95_ms': 400, 'heap': 2.2e6}
        sizes = {'script': 6_200, 'oversized': [(88_100, 'Big|Page')]}
        text = '\n'.join(perf.sidebar_markdown(row, sizes, metrics.load_budgets()))
        self.assertIn('| desktop | 2 | **400** | 2.2 | 6.2 | **61.0** |', text)
        self.assertIn('| GitHub/Star | page | **400** | 11.3 |', text)
        self.assertIn('| Block references | refs | 30 | **61.0** |', text)
        self.assertIn('`Big|Page` (88.1 KB gzip)', text)
        error = '\n'.join(perf.sidebar_markdown({'profile': 'desktop', 'error': 'TimeoutError: x'}, sizes, {}))
        self.assertIn('Not measured: TimeoutError: x', error)

    def test_block_ref_href_prefers_listed_pages_and_names_same_page_links(self):
        with tempfile.TemporaryDirectory() as temp:
            site = Path(temp)
            self.assertIsNone(common.block_ref_href(site))
            (site / 'p-1.html').write_text('<a class="block-ref-link" href="p-9.html#block-a">x</a>', encoding='utf-8')
            (site / 'p-2.html').write_text('<a class="block-ref-link" href="#block-b">y</a>', encoding='utf-8')
            self.assertEqual(common.block_ref_href(site), 'p-9.html#block-a')
            self.assertEqual(common.block_ref_href(site, ['p-2.html']), 'p-2.html#block-b')


class BudgetCheckTest(unittest.TestCase):
    limits = {'desktop': {'data_before_title_kb': 100, 'script_before_title_kb': 16}}

    def test_within_budget(self):
        sizes = {'p-1.html': (20_000, 5_000, 8_000)}
        self.assertEqual(budgets.check(sizes, dict(sizes), self.limits), [])

    def test_sidebar(self):
        limits = {'script_gzip_kb': 6, 'card_fetch_gzip_kb': 60}
        with tempfile.TemporaryDirectory() as temp:
            site = Path(temp)
            self.assertEqual(budgets.check_sidebar(site, [], limits), ['lsg-sidebar.js: missing from the build'])
            (site / 'lsg-sidebar.js').write_text('x', encoding='utf-8')
            (site / 'p-1.html').write_text('<main>page</main>', encoding='utf-8')
            (site / 'p-2.html').write_text('<script src="lsg-sidebar.js"></script>', encoding='utf-8')
            failures = budgets.check_sidebar(site, ['p-1.html', 'p-2.html'], limits)
        self.assertEqual(failures, ['p-2.html: requests lsg-sidebar.js before the sidebar is used'])

    def test_slides(self):
        limits = {'script_gzip_kb': 6}
        with tempfile.TemporaryDirectory() as temp:
            site = Path(temp)
            self.assertEqual(budgets.check_slides(site, [], limits), ['lsg-slides.js: missing from the build'])
            (site / 'lsg-slides.js').write_text('x', encoding='utf-8')
            (site / 'p-1.html').write_text('<button data-lsg-slides>View as slides</button>', encoding='utf-8')
            (site / 'p-2.html').write_text('<script src="lsg-slides.js"></script>', encoding='utf-8')
            failures = budgets.check_slides(site, ['p-1.html', 'p-2.html'], limits)
        self.assertEqual(failures, ['p-2.html: requests lsg-slides.js before a Slides card renders'])

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

    def test_hierarchy_rows_styles_script_fragments_and_grown_root(self):
        with tempfile.TemporaryDirectory() as temp:
            temp = Path(temp)
            styles = temp / 'references.css'
            styles.write_text('.references{margin-top:24px}.namespaces{margin:12px 24px}', encoding='utf-8')
            self.assertEqual(budgets.hierarchy_styles(styles), '.namespaces{margin:12px 24px}')
            self.assertEqual(budgets.hierarchy_styles(styles, hierarchy=False), '.references{margin-top:24px}')
            limits = {'inline_rows': 2, 'styles_gzip_kb': 1, 'script_baseline_kb': 0.4, 'script_growth_kb': 0.1,
                      'fragment_gzip_kb': 1, 'root_growth_kb': 0.1}
            sites = {}
            for name, rows, root in (('site', 2, 0), ('smaller', 2, 100), ('larger', 2, 150)):
                site = sites[name] = temp / name
                (site / '_refs').mkdir(parents=True)
                (site / 'p-1.html').write_text('<section class="page-hierarchy references"><ul>'
                                               + '<li>r</li>' * rows + '</ul></section>', encoding='utf-8')
                (site / 'p-00000000000000aa.html').write_text('x' * root, encoding='utf-8')
                (site / 'pages.html').write_text('<main><ul><li><a href="p-00000000000000aa.html">Grown</a></li></ul></main>',
                                                 encoding='utf-8')
                (site / '_refs' / 'p-grown-hierarchy-1.html').write_text('<li>r</li>', encoding='utf-8')
            (sites['site'] / 'lsg-references.js').write_text('x' * 450, encoding='utf-8')
            self.assertEqual(budgets.check_hierarchy(sites['site'], sites['smaller'], sites['larger'], limits, styles), [])
            (sites['site'] / 'p-2.html').write_text('<section class="page-hierarchy"><li>' * 3, encoding='utf-8')
            (sites['larger'] / 'p-00000000000000aa.html').write_text('x' * 300, encoding='utf-8')
            (sites['site'] / 'lsg-references.js').write_text('x' * 600, encoding='utf-8')
            (sites['smaller'] / '_refs' / 'p-big-hierarchy-1.html').write_bytes(os.urandom(3000))
            failures = budgets.check_hierarchy(sites['site'], sites['smaller'], sites['larger'], limits, styles)
            self.assertEqual(len(failures), 4, failures)

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

    def search_site(self, temp, shard_text):
        """A format version 2 search publication with one title, one shard and one chunk."""
        site = Path(temp)
        files = {'titles': '{"format_version": 2, "pages": [["tmux", "p-1.html", [], 1]]}',
                 'shard': shard_text,
                 'chunk': '{"format_version": 2, "blocks": [["p-1.html", "source-block-1", "tmux", []]]}'}
        for kind, text in files.items():
            (site / f'lsg-search-{kind}-{kind}0000.json').write_text(text, encoding='utf-8')
        (site / 'lsg-search-manifest-r.json').write_text(
            '{"format_version": 2, "revision": "r", "titles": "titles0000", "blocks": 1, "chunk_starts": [0], '
            '"chunks": ["chunk0000"], "shards": {"tm": "shard0000"}}', encoding='utf-8')
        return site

    def test_search_index_within_budget(self):
        limits = metrics.load_budgets()['search']
        with tempfile.TemporaryDirectory() as temp:
            site = self.search_site(temp, '{"format_version": 2, "words": [["tmux", [0]]]}')
            self.assertEqual(budgets.check_search_index(budgets.search_index_sizes(site), limits), [])

    def test_oversized_shard_fails_naming_the_shard(self):
        limits = metrics.load_budgets()['search']
        # Random words compress poorly, so the shard is well over the gzip budget.
        words = ', '.join(f'["tm{os.urandom(12).hex()}", [0]]' for _ in range(8000))
        with tempfile.TemporaryDirectory() as temp:
            site = self.search_site(temp, '{"format_version": 2, "words": [' + words + ']}')
            failures = budgets.check_search_index(budgets.search_index_sizes(site), limits)
        self.assertEqual(len(failures), 1)
        self.assertIn("lsg-search-shard-shard0000.json (words starting 'tm'): largest shard", failures[0])
        self.assertIn('exceeds 64,000', failures[0])

    def test_version_1_search_is_rejected(self):
        with tempfile.TemporaryDirectory() as temp:
            (Path(temp) / 'lsg-search-manifest-r.json').write_text('{"format_version": 1}', encoding='utf-8')
            with self.assertRaisesRegex(ValueError, 'version 1'):
                budgets.search_index_sizes(temp)

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
