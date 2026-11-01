"""Tests for the Logseq export comparison.

The browser tests read the fixture pages in testdata/logseq-export/ in Chromium and run only
when Playwright is installed; `mise run parity:logseq-export` runs them before it builds.
"""
import collections
import gzip
import hashlib
import importlib.util
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import logseq_db  # noqa: E402
import logseq_export  # noqa: E402
import serve  # noqa: E402

TESTDATA = HERE / 'testdata'

EXPORT_LOG = '''Failed to copy {:from "/garden/assets/[file].png", :to "/out/assets/[file].png"} [Error: ENOENT: no such file or directory, stat '/garden/assets/[file].png'] {
  errno: -2,
  code: 'ENOENT',
  path: '/garden/assets/[file].png'
}
Failed to copy {:from "/garden/assets/image_1.png", :to "/out/assets/image_1.png"} [Error: ENOENT] {
  errno: -2
}
Failed to copy {:from "/garden/assets/image_1.png", :to "/out/assets/image_1.png"} [Error: ENOENT]
Export public pages and publish assets to /out successfully 🎉
'''


class WordsTest(unittest.TestCase):
    def test_case_punctuation_and_page_reference_brackets(self):
        self.assertEqual(logseq_export.words('See [[lsg/How To]]: 2px, 2PX and snake_case.'),
                         collections.Counter({'see': 1, 'lsg': 1, 'how': 1, 'to': 1, '2px': 2, 'and': 1,
                                              'snake_case': 1}))

    def test_only_in_counts_repeats(self):
        logseq = logseq_export.words('one two two two three')
        lsg = logseq_export.words('one two four')
        self.assertEqual(logseq_export.only_in(logseq, lsg), ['three', 'two×2'])
        self.assertEqual(logseq_export.only_in(lsg, logseq), ['four'])

    def test_a_word_split_by_spaces_differs_from_the_whole_word(self):
        # Joining a highlight span's text nodes with spaces read `2px` as `2 px`.
        self.assertEqual(logseq_export.only_in(logseq_export.words('2 px'), logseq_export.words('2px')), ['2', 'px'])


class PropertyRowsTest(unittest.TestCase):
    def test_brackets_spaces_and_key_case(self):
        rows = [('Alias', 'Logseq Gardener'), ('see-also', '[[lsg/Ref/A]], [[lsg/Ref/B]]'),
                ('tags', ' Diataxis/How To ')]
        self.assertEqual(logseq_export.property_rows(rows),
                         {'alias: Logseq Gardener', 'see-also: lsg/Ref/A, lsg/Ref/B', 'tags: Diataxis/How To'})

    def test_same_rows_from_both_builds(self):
        logseq = [('tags', '[[Diataxis/How To]], [[Fixture]]')]
        lsg = [('tags', 'Diataxis/How To, Fixture')]
        self.assertEqual(logseq_export.property_rows(logseq), logseq_export.property_rows(lsg))


class ComparePageTest(unittest.TestCase):
    def test_same_page(self):
        result = logseq_export.compare_page('tmux', {'rows': [], 'text': 'Prefix key', 'settled': True},
                                            {'rows': [], 'text': 'prefix key'})
        self.assertFalse(result['differs'])
        self.assertEqual((result['logseq_words'], result['lsg_words']), (2, 2))

    def test_property_row_only_in_logseq(self):
        logseq = {'rows': [('alias', 'Logseq Gardener'), ('tags', 'Diataxis/Explanation')], 'text': 'Logseq Gardener'}
        lsg = {'rows': [('tags', 'Diataxis/Explanation')], 'text': 'Logseq Gardener'}
        result = logseq_export.compare_page('lsg', logseq, lsg)
        self.assertTrue(result['differs'])
        self.assertEqual(result['rows_only_in_logseq'], ['alias: Logseq Gardener'])
        self.assertEqual(result['rows_only_in_lsg'], [])
        self.assertEqual(result['words_only_in_logseq'], [])

    def test_page_that_kept_growing_is_noted(self):
        result = logseq_export.compare_page('long', {'rows': [], 'text': 'a b', 'settled': False},
                                            {'rows': [], 'text': 'a b c'})
        self.assertEqual(result['words_only_in_lsg'], ['c'])
        self.assertIn('still growing', result['note'])


class ScrollThroughTest(unittest.TestCase):
    class LazyPage:
        """A page that shows three screens and loads another block each time its end is reached."""

        def __init__(self, later):
            self.blocks, self.later, self.position, self.steps = ['first', 'second', 'third'], list(later), 0, 0

        def step(self):
            self.steps += 1
            self.position = min(self.position + 1, len(self.blocks))
            if self.position == len(self.blocks) and self.later:
                self.blocks.append(self.later.pop(0))
            return self.position == len(self.blocks)

        def length(self):
            return len(' '.join(self.blocks))

    def test_blocks_rendered_after_scrolling_are_read(self):
        page = self.LazyPage(['fourth', 'fifth'])
        self.assertTrue(logseq_export.scroll_through(page.step, page.length, lambda: None))
        self.assertEqual(page.blocks, ['first', 'second', 'third', 'fourth', 'fifth'])

    def test_stops_after_the_text_stays_the_same(self):
        page = self.LazyPage([])
        self.assertTrue(logseq_export.scroll_through(page.step, page.length, lambda: None, stable_reads=3))
        self.assertEqual(page.steps, 4)

    def test_reports_a_page_that_keeps_growing(self):
        page = self.LazyPage([f'block {n}' for n in range(100)])
        self.assertFalse(logseq_export.scroll_through(page.step, page.length, lambda: None, limit=20))


class ExportLogTest(unittest.TestCase):
    def test_failed_copies_relative_to_the_garden(self):
        self.assertEqual(logseq_export.failed_copies(EXPORT_LOG, Path('/garden')),
                         ['assets/[file].png', 'assets/image_1.png'])

    def test_log_without_failures(self):
        self.assertEqual(logseq_export.failed_copies('Export public pages ... successfully\n', Path('/garden')), [])


class PagePlanTest(unittest.TestCase):
    def test_pages_to_compare(self):
        manifest = {'pages': [
            {'title': 'lsg', 'url': 'index.html', 'kind': 'home'},
            {'title': 'lsg', 'url': 'lsg/', 'kind': 'page'},
            {'title': 'lsg/How To', 'url': 'lsg/How%20To/', 'kind': 'namespace'},
            {'title': 'Café Notes', 'url': 'Caf%C3%A9%20Notes/', 'kind': 'page'},
            {'title': '2026-11-01 Sun', 'url': 'journals/2026_11_01/', 'kind': 'journal'},
            {'title': 'Only in lsg', 'url': 'Only%20in%20lsg/', 'kind': 'page'},
        ]}
        logseq = [logseq_db.Page('lsg', 'lsg', False, True), logseq_db.Page('lsg/how to', 'lsg/How To', False, False),
                  logseq_db.Page('café notes', 'Café Notes', False, True),
                  logseq_db.Page('2026-11-01 sun', '2026-11-01 Sun', True, True),
                  logseq_db.Page('contents', 'contents', False, True), logseq_db.Page('todo', 'TODO', False, False)]
        compared, missing, outside = logseq_export.page_plan(manifest, logseq)
        self.assertEqual([entry['url'] for entry in compared], ['lsg/', 'Caf%C3%A9%20Notes/', 'journals/2026_11_01/'])
        self.assertEqual(missing, ['Only in lsg'])
        self.assertEqual(outside, ['contents'])

    def test_pages_from_the_inline_graph_data(self):
        assignment = gzip.decompress((TESTDATA / 'db.gz').read_bytes()).decode('utf-8')
        index_html = f'<!DOCTYPE html><body><div id="root"></div>\n <script>{assignment};</script></body>'
        pages = logseq_export.logseq_pages(index_html)
        self.assertEqual({page.name for page in pages}, {'tmux & co', '2026-10-23 fri', 'reference only'})


class OptionsTest(unittest.TestCase):
    def test_docs_graph_gets_the_docs_site_options(self):
        self.assertEqual(logseq_export.lsg_options(logseq_export.DOCS, []), logseq_export.DOCS_OPTIONS)

    def test_given_options_win(self):
        self.assertEqual(logseq_export.lsg_options(Path('/garden'), ['--include', 'notes']), ['--include', 'notes'])

    def test_another_garden_needs_options(self):
        with self.assertRaisesRegex(SystemExit, 'pass the lsg publish options'):
            logseq_export.lsg_options(Path('/garden'), [])

    def test_sparse_patterns(self):
        self.assertEqual(logseq_export.sparse_pattern('deps/db/src'), '/deps/db/src/')
        self.assertEqual(logseq_export.sparse_pattern('scripts/src/logseq/tasks/dev/publishing.cljs'),
                         '/scripts/src/logseq/tasks/dev/publishing.cljs')


class PinsTest(unittest.TestCase):
    def test_tree_digest_is_the_digest_of_a_shasum_listing(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder)
            (root / 'js').mkdir()
            (root / 'js' / 'main.js').write_text('main')
            (root / 'VERSION').write_text('0.10.15\n')
            (root / 'other.txt').write_text('left out')
            listing = ''.join(f'{hashlib.sha256(text.encode()).hexdigest()}  {name}\n'
                              for name, text in (('VERSION', '0.10.15\n'), ('js/main.js', 'main')))
            self.assertEqual(logseq_export.tree_digest(root, ['VERSION', 'js']),
                             hashlib.sha256(listing.encode()).hexdigest())
            self.assertNotEqual(logseq_export.tree_digest(root), logseq_export.tree_digest(root, ['VERSION', 'js']))

    def test_frontend_of_another_version(self):
        with tempfile.TemporaryDirectory() as folder:
            (Path(folder) / 'VERSION').write_text('0.10.9\n')
            self.assertIn('VERSION is 0.10.9', logseq_export.frontend_problem(folder))

    def test_frontend_with_other_files(self):
        with tempfile.TemporaryDirectory() as folder:
            (Path(folder) / 'VERSION').write_text(logseq_export.FRONTEND['version'] + '\n')
            self.assertIn('pins ' + logseq_export.FRONTEND['sha256'], logseq_export.frontend_problem(folder))

    def test_static_dir_that_fails_the_digest_stops_the_task(self):
        with tempfile.TemporaryDirectory() as folder:
            with self.assertRaisesRegex(SystemExit, '--static-dir'):
                logseq_export.frontend_folder(folder)


class MarkdownTest(unittest.TestCase):
    def test_report(self):
        report = {
            'summary': {'Pages compared': 2, 'Pages that differ': 1},
            'pages': [
                {'title': 'tmux', 'differs': False},
                {'title': 'lsg', 'differs': True, 'words_only_in_logseq': [], 'words_only_in_lsg': ['new'],
                 'rows_only_in_logseq': ['alias: Logseq Gardener'], 'rows_only_in_lsg': []},
            ],
            'missing_from_logseq': [],
            'outside_lsg': ['contents'],
            'failed_copies': ['assets/[file].png'],
        }
        text = logseq_export.markdown(report)
        self.assertIn('### lsg\n\n- Words only lsg shows: `new`\n- Property rows only Logseq shows: '
                      '`alias: Logseq Gardener`\n', text)
        self.assertNotIn('### tmux', text)
        self.assertIn("## Pages missing from Logseq's export\n\nNone.", text)
        self.assertIn('- contents', text)
        self.assertIn('- `assets/[file].png`', text)


@unittest.skipUnless(importlib.util.find_spec('playwright'), 'needs Playwright')
class PageReadingTest(unittest.TestCase):
    """Read the fixture pages in Chromium as the comparison reads Logseq's and lsg's pages."""

    @classmethod
    def setUpClass(cls):
        from playwright.sync_api import sync_playwright

        cls.server = serve.Server(TESTDATA / 'logseq-export').__enter__()
        cls.playwright = sync_playwright().start()
        cls.browser = cls.playwright.chromium.launch()
        page = cls.browser.new_page(viewport={'width': 1280, 'height': 800})
        cls.logseq = logseq_export.read_logseq(page, cls.server.url + 'logseq.html', 'Fixture/Page')
        cls.lsg = logseq_export.read_lsg(page, cls.server.url + 'lsg.html')

    @classmethod
    def tearDownClass(cls):
        cls.browser.close()
        cls.playwright.stop()
        cls.server.__exit__(None, None, None)

    def test_reference_count_badge_is_left_out(self):
        self.assertIn('Referenced block\n', self.logseq['text'])
        self.assertNotIn('3', self.logseq['text'])

    def test_word_split_across_highlight_spans_stays_one_word(self):
        for build in (self.logseq, self.lsg):
            self.assertEqual(logseq_export.words(build['text'])['2px'], 1)

    def test_line_numbers_and_properties_block_are_left_out(self):
        self.assertNotIn('1', self.logseq['text'])
        self.assertNotIn('Fixture Alias', self.logseq['text'])

    def test_blocks_rendered_after_scrolling_are_read(self):
        self.assertTrue(self.logseq['settled'])
        self.assertIn('Lazy block three', self.logseq['text'])

    def test_lsg_text_is_the_outline_only(self):
        self.assertNotIn('Words below the blocks', self.lsg['text'])
        self.assertNotIn('Diataxis', self.lsg['text'])

    def test_fixture_pages_differ_only_in_the_alias_row(self):
        result = logseq_export.compare_page('Fixture/Page', self.logseq, self.lsg)
        self.assertEqual(result['words_only_in_logseq'], [])
        self.assertEqual(result['words_only_in_lsg'], [])
        self.assertEqual(result['rows_only_in_logseq'], ['alias: Fixture Alias'])
        self.assertEqual(result['rows_only_in_lsg'], [])


if __name__ == '__main__':
    unittest.main()
