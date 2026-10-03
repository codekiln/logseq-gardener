#!/usr/bin/env python3
"""Record Cargo dependency metadata and preserve upstream license/NOTICE text."""
import argparse
import json
from pathlib import Path
import subprocess
import tomllib

parser = argparse.ArgumentParser()
parser.add_argument('--check', action='store_true')
args = parser.parse_args()
metadata = json.loads(subprocess.check_output(['cargo', 'metadata', '--locked', '--format-version', '1']))
lock = tomllib.loads(Path('Cargo.lock').read_text())
checksums = {(p['name'], p['version']): p.get('checksum', '') for p in lock['package']}
expected = {}
source_records = []
for package in sorted(metadata['packages'], key=lambda p: (p['name'], p['version'])):
    if package['id'] in metadata['workspace_members']:
        continue
    name, version = package['name'], package['version']
    license_id = package['license']
    if not license_id:
        raise SystemExit(f'{name}: review missing license metadata')
    source = f'https://crates.io/crates/{name}/{version}'
    source_records.append({'package': name, 'version': version, 'source': source, 'checksum': checksums[(name, version)], 'license': license_id})
    root = Path(package['manifest_path']).parent
    files = [p for p in root.rglob('*') if p.is_file() and p.name.lower().startswith(('license', 'licence', 'copying', 'notice', 'copyright'))]
    if package.get('license_file'):
        files.append(root / package['license_file'])
    if not files:
        raise SystemExit(f'{name}: no license text found; review before distribution')
    for file in files:
        destination = Path('licenses/dependencies') / f'{name}-{version}' / file.relative_to(root)
        expected[destination] = file.read_bytes()
expected[Path('docs/dependency-sources.json')] = (json.dumps(source_records, indent=2) + '\n').encode()
existing = set(p for p in Path('licenses/dependencies').rglob('*') if p.is_file())
if args.check:
    failures = [str(p) for p, content in expected.items() if not p.exists() or p.read_bytes() != content]
    failures += [str(p) for p in existing - set(expected)]
    if failures:
        raise SystemExit('Dependency notices are stale; run mise run notices:update:\n' + '\n'.join(failures))
else:
    for p in existing - set(expected):
        p.unlink()
    for p, content in expected.items():
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_bytes(content)
print('Dependency notices verified' if args.check else 'Dependency notices updated')
