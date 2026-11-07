"""The parser check fails on a graph where Logseq and the SDK read the pages without a file differently."""
import subprocess
import sys
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent


class AliasNamespaceParentsFixture(unittest.TestCase):
    def test_check_names_the_file_and_the_pages(self):
        # fixtures/alias-namespace-parents holds AGI.md, which lists AI/General/Intelligence
        # as an alias. Logseq also creates the namespace parents ai and ai/general as pages
        # without a file, while the SDK does not. Issue #462 tracks the difference.
        result = subprocess.run([sys.executable, HERE / 'check.py', HERE / 'fixtures' / 'alias-namespace-parents'],
                                capture_output=True, text=True)
        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        lines = result.stdout.splitlines()
        self.assertEqual(lines[1:], [
            'pages/AGI.md: page without a file "ai": Logseq reads it, the SDK does not',
            'pages/AGI.md: page without a file "ai/general": Logseq reads it, the SDK does not',
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
