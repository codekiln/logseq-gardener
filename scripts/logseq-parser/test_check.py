"""The parser check fails on a graph where Logseq and the SDK read a block id differently."""
import subprocess
import sys
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent


class DuplicateBlockIdFixture(unittest.TestCase):
    def test_check_names_the_file_and_the_block_id(self):
        # fixtures/duplicate-block-id gives a block in JSON5.md and a block in JSONC.md the
        # same `id::`. Logseq keeps the id on the block in JSON5.md, which it reads first,
        # while the SDK keeps it on both. Issue #438 tracks the difference.
        result = subprocess.run([sys.executable, HERE / 'check.py', HERE / 'fixtures' / 'duplicate-block-id'],
                                capture_output=True, text=True)
        self.assertEqual(result.returncode, 1, result.stdout + result.stderr)
        lines = result.stdout.splitlines()
        self.assertEqual(lines[1:], [
            'pages/JSONC.md: block id "67c822b0-3271-41da-a14f-4f023a1f4556": the SDK reads it, Logseq does not',
            'pages/JSONC.md:1: property "id": the SDK reads it, Logseq does not',
        ])


if __name__ == '__main__':
    unittest.main()
