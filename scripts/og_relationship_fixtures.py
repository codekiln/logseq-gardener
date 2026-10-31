"""Derive the SDK relationship fixtures from the recorded Logseq OG evidence.

The evidence is `relationships.json` from the current-og-relationships experiment.
`--check` fails when the copied notes or `expected.json` differ from what the
evidence implies; `--update` copies the notes and rewrites `expected.json`.
Both read only committed files, so they need no Logseq checkout.
"""

import argparse
import hashlib
import json
import re
import shutil
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
FIXTURES = ROOT / "crates/logseq-gardener-sdk/tests/fixtures/og-relationships"
GARDEN = FIXTURES / "garden"
EXPECTED = FIXTURES / "expected.json"
# The evidence stays at this path inside the change, whether active or archived.
EVIDENCE = "experiments/file-relationships/relationships.json"
FILENAME_FORMATS = {"default": "legacy", "triple-lowbar": "triple-lowbar"}
UUID = re.compile(r"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$")


def evidence_path() -> Path:
    found = sorted(
        path
        for path in (ROOT / "openspec/changes").glob(f"**/{EVIDENCE}")
        if path.parent.parent.parent.name.endswith("current-og-relationships")
    )
    if len(found) != 1:
        sys.exit(f"expected one current-og-relationships evidence file, found {found}")
    return found[0]


def sha256(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def journal_day(value):
    if value is None:
        return None
    text = str(value)
    return f"{text[0:4]}-{text[4:6]}-{text[6:8]}"


def configuration(name: str, result: dict) -> dict:
    """Project Logseq's pages onto the relationships the SDK answers."""
    pages = result["pages"]
    referenced = {ref for block in result["blocks"] for ref in block["refs"] or []}
    stored = [page for page in pages if page["file"] is not None]
    alias_owners = {}
    for page in stored:
        for alias in page["aliases"]:
            alias_owners.setdefault(alias, []).append(page["file"])
    aliases, fileless = [], []
    for page in pages:
        if page["file"] is not None:
            continue
        if page["name"] in alias_owners:
            aliases.append(
                {
                    "name": page["name"],
                    "title": page["original-name"],
                    "files": sorted(alias_owners[page["name"]]),
                }
            )
        elif page["name"] in referenced and not UUID.match(page["name"]):
            fileless.append(
                {
                    "name": page["name"],
                    "title": page["original-name"],
                    "namespace": page["namespace"],
                }
            )
    return {
        "name": name,
        "filename_format": FILENAME_FORMATS[name],
        "pages": [
            {
                "name": page["name"],
                "title": page["original-name"],
                "file": page["file"],
                "aliases": sorted(page["aliases"]),
                "namespace": page["namespace"],
                "journal_day": journal_day(page["journal-day"]),
            }
            for page in sorted(stored, key=lambda page: page["file"])
        ],
        "aliases": aliases,
        "fileless": fileless,
    }


def expected(evidence: dict) -> dict:
    configurations = []
    for result in evidence["configurations"]:
        if not result["equal"]:
            sys.exit(f"{result['name']}: current and historical results differ")
        configurations.append(configuration(result["name"], result["current"]))
    return {
        "source": evidence["sources"]["current"],
        "runtime": evidence["runtime"],
        "inputs": evidence["inputs"],
        "configurations": configurations,
    }


def render(data: dict) -> str:
    return json.dumps(data, indent=2, ensure_ascii=False) + "\n"


def source_note(relative: str, digest: str) -> Path:
    """The committed fixture note with this relative path and hash."""
    for path in sorted((ROOT / "openspec/changes").glob(f"**/fixtures/**/{relative}")):
        if sha256(path) == digest:
            return path
    sys.exit(f"{relative}: no committed fixture has sha256 {digest}")


def check(data: dict) -> list[str]:
    problems = []
    names = {item["file"] for item in data["inputs"]}
    for item in data["inputs"]:
        note = GARDEN / item["file"]
        if not note.is_file():
            problems.append(f"{note.relative_to(ROOT)}: missing")
        elif sha256(note) != item["sha256"]:
            problems.append(f"{note.relative_to(ROOT)}: differs from the recorded input")
    for note in sorted(GARDEN.rglob("*")):
        relative = note.relative_to(GARDEN).as_posix()
        if note.is_file() and relative not in names:
            problems.append(f"{note.relative_to(ROOT)}: not a recorded input")
    if not EXPECTED.is_file() or EXPECTED.read_text() != render(data):
        problems.append(f"{EXPECTED.relative_to(ROOT)}: differs from the evidence")
    return problems


def update(data: dict) -> None:
    if GARDEN.exists():
        shutil.rmtree(GARDEN)
    for item in data["inputs"]:
        target = GARDEN / item["file"]
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(source_note(item["file"], item["sha256"]), target)
    EXPECTED.write_text(render(data))


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--check", action="store_true", help="fail when fixtures are stale")
    mode.add_argument("--update", action="store_true", help="rewrite the fixtures")
    args = parser.parse_args()
    data = expected(json.loads(evidence_path().read_text()))
    if args.update:
        update(data)
        print(f"updated {FIXTURES.relative_to(ROOT)}")
        return 0
    problems = check(data)
    for problem in problems:
        print(problem, file=sys.stderr)
    if problems:
        print("run `mise run parser:sdk-fixtures-update` and review the diff", file=sys.stderr)
        return 1
    print("SDK relationship fixtures match the recorded Logseq evidence")
    return 0


if __name__ == "__main__":
    sys.exit(main())
