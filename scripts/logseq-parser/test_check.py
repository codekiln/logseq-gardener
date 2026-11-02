"""The parser check fails on a graph where Logseq and the SDK read a page reference differently."""
import subprocess
import sys
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent


class PageReferenceSpaceFixture(unittest.TestCase):
    def test_check_names_the_file_and_the_page(self):
        # fixtures/page-reference-space writes `[[Release Notes ]]`: Logseq reads the page
        # `release notes ` with its trailing space, while the SDK reads `release notes`.
        # Issue #421 tracks the difference.
        result = subprocess.run([sys.executable, HERE / 'check.py', HERE / 'fixtures' / 'page-reference-space'],
                                capture_output=True, text=True)
        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        lines = result.stdout.splitlines()
        self.assertIn('pages/Setup.md:1: page reference "release notes": the SDK reads it, Logseq does not', lines)
        for line in lines[1:]:
            self.assertTrue(line.startswith('pages/Setup.md') and '"release notes' in line, line)


if __name__ == '__main__':
    unittest.main()
