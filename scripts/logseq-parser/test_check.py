"""The parser check fails on a graph where Logseq and the SDK read the pages differently."""
import subprocess
import sys
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent


class EmptyPageFileFixture(unittest.TestCase):
    def test_check_names_the_file_and_the_page(self):
        # fixtures/empty-page-file holds only the empty file dotenv.md. Logseq reads no page
        # from an empty file, while the SDK reads the page dotenv. Issue #468 tracks the
        # difference.
        result = subprocess.run([sys.executable, HERE / 'check.py', HERE / 'fixtures' / 'empty-page-file'],
                                capture_output=True, text=True)
        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        lines = result.stdout.splitlines()
        self.assertEqual(lines[1:], [
            'pages/dotenv.md: page "dotenv": the SDK reads it, Logseq does not',
        ])


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
