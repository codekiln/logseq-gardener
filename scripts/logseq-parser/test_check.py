"""The parser check fails on a graph where Logseq and the SDK read a page title differently."""
import subprocess
import sys
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent


class AliasTitleCaseFixture(unittest.TestCase):
    def test_check_names_the_file_and_both_titles(self):
        # fixtures/alias-title-case holds Solvespace.md, which lists its own name as the
        # alias `SolveSpace`. Logseq titles the page with the alias's spelling, while the
        # SDK keeps the file name's. Issue #444 tracks the difference.
        result = subprocess.run([sys.executable, HERE / 'check.py', HERE / 'fixtures' / 'alias-title-case'],
                                capture_output=True, text=True)
        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        lines = result.stdout.splitlines()
        self.assertEqual(lines[1:], [
            'pages/Solvespace.md: title: Logseq reads "SolveSpace", the SDK reads "Solvespace"',
        ])


if __name__ == '__main__':
    unittest.main()
