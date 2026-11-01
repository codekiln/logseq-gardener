"""The parser check fails on a graph where Logseq and the SDK read an alias differently."""
import subprocess
import sys
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent


class TagAliasFixture(unittest.TestCase):
    def test_check_names_the_file_and_the_alias(self):
        # fixtures/tag-alias writes `alias:: #[[Changelog]]`: Logseq reads the alias
        # `changelog`, while the SDK reads the whole value as the alias name.
        result = subprocess.run([sys.executable, HERE / 'check.py', HERE / 'fixtures' / 'tag-alias'],
                                capture_output=True, text=True)
        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        lines = result.stdout.splitlines()
        self.assertIn('pages/Release Notes.md: alias "changelog": Logseq reads it, the SDK does not', lines)
        for line in lines[1:]:
            self.assertTrue(line.startswith('pages/Release Notes.md: ') and 'changelog' in line, line)


if __name__ == '__main__':
    unittest.main()
