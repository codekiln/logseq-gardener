"""The parser check fails on a graph where Logseq and the SDK read the pages differently."""
import subprocess
import sys
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent


class TrailingSlashNameFixture(unittest.TestCase):
    def test_check_names_the_file_and_the_page(self):
        # fixtures/trailing-slash-name holds Notes.md, which links to a/b/. Logseq reads the
        # page a/b with the namespace parent a, while the SDK keeps the trailing slash in the
        # title and reads no namespace. Issue #495 tracks the difference.
        result = subprocess.run([sys.executable, HERE / 'check.py', HERE / 'fixtures' / 'trailing-slash-name'],
                                capture_output=True, text=True)
        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        lines = result.stdout.splitlines()
        self.assertEqual(lines[1:], [
            'pages/Notes.md: page without a file "a/b" namespace: Logseq reads "a", the SDK reads null',
            'pages/Notes.md: page without a file "a/b" title: Logseq reads "a/b", the SDK reads "a/b/"',
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
