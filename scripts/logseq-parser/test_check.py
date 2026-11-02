"""The parser check fails on a graph where Logseq and the SDK read a page's aliases differently."""
import subprocess
import sys
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent


class AliasOfAnotherPageFixture(unittest.TestCase):
    def test_check_names_the_file_and_the_alias(self):
        # fixtures/alias-of-another-page holds brew.md, which lists Homebrew as an alias,
        # and Homebrew.md. Logseq also gives Homebrew the alias brew, while the SDK does
        # not. Issue #453 tracks the difference.
        result = subprocess.run([sys.executable, HERE / 'check.py', HERE / 'fixtures' / 'alias-of-another-page'],
                                capture_output=True, text=True)
        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        lines = result.stdout.splitlines()
        self.assertEqual(lines[1:], [
            'pages/Homebrew.md: alias "brew": Logseq reads it, the SDK does not',
        ])


if __name__ == '__main__':
    unittest.main()
