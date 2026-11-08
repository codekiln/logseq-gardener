"""The parser check fails on a graph where Logseq and the SDK read the pages differently."""
import subprocess
import sys
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent


class PropertyNameReferenceFixture(unittest.TestCase):
    def test_check_passes_when_a_block_writes_a_property_name_and_a_reference_in_another_case(self):
        # fixtures/property-name-reference holds Notes.md, whose block references [[Gadget]]
        # and has the property gadget::. Logseq and the SDK both spell the page with the
        # property name, gadget. The other files vary the order of the blocks and the case
        # of the property name (issue #521).
        result = subprocess.run([sys.executable, HERE / 'check.py', HERE / 'fixtures' / 'property-name-reference'],
                                capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)


class AliasSpellingReferenceFixture(unittest.TestCase):
    def test_check_passes_when_a_block_references_an_alias_in_another_case(self):
        # fixtures/alias-spelling-reference holds Run.md, whose alias property writes Runtime,
        # and Notes.md, whose block references [[runtime]]. Logseq and the SDK both spell the
        # alias with the reference, runtime, because Notes.md is read first (issue #515).
        result = subprocess.run([sys.executable, HERE / 'check.py', HERE / 'fixtures' / 'alias-spelling-reference'],
                                capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)


class AliasSpellingCaseFixture(unittest.TestCase):
    def test_check_passes_when_an_alias_is_written_twice(self):
        # fixtures/alias-spelling-case holds Brew.md, whose alias property writes homebrew and
        # Homebrew. Logseq and the SDK both spell the alias with the last of them, Homebrew
        # (issue #509).
        result = subprocess.run([sys.executable, HERE / 'check.py', HERE / 'fixtures' / 'alias-spelling-case'],
                                capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)


class WrongWeekdayDateNameFixture(unittest.TestCase):
    def test_check_passes_when_a_date_name_has_a_wrong_weekday(self):
        # fixtures/wrong-weekday-date-name holds Notes.md, whose property links to
        # 1967-09-03 Fri under the journal title format yyyy-MM-dd EEE. Logseq and the SDK
        # both read the date with its real weekday, 1967-09-03 sun (issue #503).
        result = subprocess.run([sys.executable, HERE / 'check.py', HERE / 'fixtures' / 'wrong-weekday-date-name'],
                                capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)


class TrailingSlashNameFixture(unittest.TestCase):
    def test_check_passes_when_a_link_has_a_trailing_slash(self):
        # fixtures/trailing-slash-name holds Notes.md, which links to a/b/. Logseq and the SDK
        # both read the page a/b with the namespace parent a (issue #495).
        result = subprocess.run([sys.executable, HERE / 'check.py', HERE / 'fixtures' / 'trailing-slash-name'],
                                capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)


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
