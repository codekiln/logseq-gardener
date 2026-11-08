"""The parser check fails on a graph where Logseq and the SDK read the pages differently."""
import subprocess
import sys
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent


class WrongWeekdayDateNameFixture(unittest.TestCase):
    def test_check_names_the_file_and_the_page(self):
        # fixtures/wrong-weekday-date-name holds Notes.md, whose property links to
        # 1967-09-03 Fri under the journal title format yyyy-MM-dd EEE. Logseq reads the date
        # with its real weekday, 1967-09-03 sun, while the SDK keeps the name as written.
        # Issue #503 tracks the difference.
        result = subprocess.run([sys.executable, HERE / 'check.py', HERE / 'fixtures' / 'wrong-weekday-date-name'],
                                capture_output=True, text=True)
        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        lines = result.stdout.splitlines()
        self.assertEqual(lines[1:], [
            'pages/Notes.md: page without a file "1967-09-03 fri": the SDK reads it, Logseq does not',
            'pages/Notes.md: page without a file "1967-09-03 sun": Logseq reads it, the SDK does not',
            'pages/Notes.md:1: page reference "1967-09-03 fri": the SDK reads it, Logseq does not',
            'pages/Notes.md:1: page reference "1967-09-03 sun": Logseq reads it, the SDK does not',
        ])


class TrailingSlashNameFixture(unittest.TestCase):
    def test_check_passes_when_a_link_has_a_trailing_slash(self):
        # fixtures/trailing-slash-name holds Notes.md, which links to a/b/. Logseq and the SDK
        # both read the page a/b with the namespace parent a (issue #495).
        result = subprocess.run([sys.executable, HERE / 'check.py', HERE / 'fixtures' / 'trailing-slash-name'],
                                capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)


class EmptyPageFileFixture(unittest.TestCase):
    def test_check_passes_when_page_files_are_empty_or_blank(self):
        # fixtures/empty-page-file holds an empty dotenv.md, a whitespace-only blank.md and
        # Notes.md, which links to dotenv. Logseq reads no page from the first two files and
        # makes dotenv from the link alone; the SDK reads the same pages.
        result = subprocess.run([sys.executable, HERE / 'check.py', HERE / 'fixtures' / 'empty-page-file'],
                                capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)


if __name__ == '__main__':
    unittest.main()
