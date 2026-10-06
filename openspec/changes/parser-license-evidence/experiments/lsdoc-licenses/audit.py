#!/usr/bin/env python3
"""Compare the pinned parser's package licenses with saved evidence."""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess
import tempfile
import tomllib

ROOT = Path(__file__).resolve().parent
REVISION = "32e63ef095c711d6d9947257bf5fd07d540fa59d"
REPOSITORY = "https://github.com/martinkoutecky/lsdoc"


def run(*args, **kwargs):
    return subprocess.check_output(args, **kwargs)


def digest(data):
    return hashlib.sha256(data).hexdigest()


def audit(online):
    upstream = run("ghq", "list", "--full-path", "--exact", "github.com/martinkoutecky/lsdoc").decode().strip()
    if not upstream:
        raise SystemExit("The existing martinkoutecky/lsdoc ghq checkout is required.")
    archive = run("git", "-C", upstream, "archive", REVISION)
    with tempfile.TemporaryDirectory(prefix="lsdoc-licenses-") as directory:
        subprocess.run(["tar", "-xf", "-", "-C", directory], input=archive, check=True)
        source = Path(directory)
        lock_bytes = (source / "Cargo.lock").read_bytes()
        locked = {(p["name"], p["version"]): p for p in tomllib.loads(lock_bytes.decode())["package"]}
        command = ["cargo", "metadata", "--locked", "--format-version", "1"]
        if not online:
            command.append("--offline")
        metadata = json.loads(run(*command, cwd=directory))
        packages = []
        texts = []
        for package in sorted(metadata["packages"], key=lambda p: (p["name"], p["version"])):
            identity = (package["name"], package["version"])
            entry = locked[identity]
            package_root = Path(package["manifest_path"]).parent
            files = []
            for file in sorted(package_root.rglob("*")):
                if file.is_file() and file.name.upper().startswith(("LICENSE", "LICENCE", "COPYING", "COPYRIGHT", "NOTICE")):
                    relative = file.relative_to(package_root).as_posix()
                    data = file.read_bytes()
                    files.append({"path": relative, "sha256": digest(data)})
                    texts.append(f"===== {package['name']} {package['version']} / {relative} =====\n" + data.decode() + "\n")
            if not package["license"] or not files:
                raise SystemExit(f"Missing declaration or license files: {identity}")
            packages.append({"name": identity[0], "version": identity[1], "license": package["license"],
                             "repository": package["repository"], "source": entry.get("source", f"git+{REPOSITORY}#{REVISION}"),
                             "checksum": entry.get("checksum"), "licenseFiles": files})
        inventory = {"repository": REPOSITORY, "revision": REVISION, "lockfileSha256": digest(lock_bytes), "packages": packages}
        return json.dumps(inventory, indent=2, ensure_ascii=False) + "\n", "\n".join(texts)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--update", action="store_true", help="write evidence for review")
    parser.add_argument("--online", action="store_true", help="allow Cargo to fetch locked registry dependencies")
    args = parser.parse_args()
    inventory, texts = audit(args.online)
    for name, content in [("inventory.json", inventory), ("license-texts.txt", texts)]:
        file = ROOT / name
        if args.update:
            file.write_text(content)
        elif not file.exists() or file.read_text() != content:
            raise SystemExit(f"License evidence differs: {name}. Run the audit with --update and review the changes.")
    print("Parser license evidence updated." if args.update else "Parser license evidence matches.")


if __name__ == "__main__":
    main()
