"""The parser check fails on a graph where Logseq and the SDK read a page reference differently."""
import subprocess
import sys
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent


class CodeSpanPropertyFixture(unittest.TestCase):
    def test_check_names_the_file_and_the_property(self):
        # fixtures/code-span-property starts a block with the code `status:: [[Done]]`:
        # Logseq reads code, while the SDK reads the property `` `status `` and a reference to
        # `done`. Issue #431 tracks the difference.
        result = subprocess.run([sys.executable, HERE / 'check.py', HERE / 'fixtures' / 'code-span-property'],
                                capture_output=True, text=True)
        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        lines = result.stdout.splitlines()
        self.assertIn('pages/Setup.md:1: property "`status": the SDK reads it, Logseq does not', lines)
        for line in lines[1:]:
            self.assertTrue(('"`status"' in line or '"done"' in line) and 'the SDK reads it, Logseq does not' in line,
                            line)


if __name__ == '__main__':
    unittest.main()
