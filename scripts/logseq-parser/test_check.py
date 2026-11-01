"""The parser check fails on a graph where Logseq and the SDK read a property value differently."""
import subprocess
import sys
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent


class NestedPropertyReferenceFixture(unittest.TestCase):
    def test_check_names_the_file_and_the_page(self):
        # fixtures/nested-property-reference writes `type:: [[Notes on [[Mldoc]]]]`: Logseq
        # reads a reference to `Notes on [[Mldoc]]` only, while the SDK also reads one to `Mldoc`.
        # Issue #378 tracks the difference.
        result = subprocess.run([sys.executable, HERE / 'check.py', HERE / 'fixtures' / 'nested-property-reference'],
                                capture_output=True, text=True)
        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        lines = result.stdout.splitlines()
        self.assertIn('pages/Reading List.md:1: page reference "mldoc": the SDK reads it, Logseq does not', lines)
        for line in lines[1:]:
            self.assertTrue(line.startswith('pages/Reading List.md') and '"mldoc"' in line, line)


if __name__ == '__main__':
    unittest.main()
