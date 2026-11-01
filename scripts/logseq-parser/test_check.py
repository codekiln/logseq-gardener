"""The parser check fails on a graph where Logseq and the SDK read a page reference differently."""
import subprocess
import sys
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent


class NamespaceParentSpaceFixture(unittest.TestCase):
    def test_check_names_the_file_and_the_page(self):
        # fixtures/namespace-parent-space writes `[[MacOS /Applications Folder]]`: Logseq
        # reads the namespace parent `macos ` with its trailing space, while the SDK reads
        # `macos`. Issue #412 tracks the difference.
        result = subprocess.run([sys.executable, HERE / 'check.py', HERE / 'fixtures' / 'namespace-parent-space'],
                                capture_output=True, text=True)
        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        lines = result.stdout.splitlines()
        self.assertIn('pages/Setup.md:1: page reference "macos": the SDK reads it, Logseq does not', lines)
        for line in lines[1:]:
            self.assertTrue(line.startswith('pages/Setup.md') and '"macos' in line, line)


if __name__ == '__main__':
    unittest.main()
