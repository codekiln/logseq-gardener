#!/usr/bin/env python3
"""Write a generated garden the size and shape of Gitpa's for publication benchmarks.

The garden is the same on every machine: a seeded random generator chooses every title, word,
link and property, and `digest` identifies the result. Its shape follows Gitpa's garden on
2026-10-31: about 2,400 pages and 300,000 words, deep namespaces, about seven page links, seven
blocks and eight properties per page, page embeds, a few code blocks and a few journals. Pages
must say `public:: true` to be published, as in Gitpa; a few say `public:: false`.
"""
import argparse
import hashlib
import os
import random
import subprocess
from pathlib import Path

NAME = 'gitpa-sized'
VERSION = 1
SEED = 259
PAGES = 2400
JOURNALS = 10
# Namespace depth of Gitpa's pages: depth -> share of pages, in percent.
DEPTHS = {0: 1, 1: 1, 2: 19, 3: 11, 4: 22, 5: 27, 6: 6, 7: 5, 8: 4, 9: 2, 10: 2}
CONFIG = '''{:meta/version 1
 :file/name-format :triple-lowbar
 :journal/page-title-format "MMM do, yyyy"
 :publishing/all-pages-public? false}
'''
SYLLABLES = ['ka', 'lo', 'mi', 'ne', 'ru', 'sa', 'ti', 'vo', 'pe', 'da', 'go', 'hu', 'ji', 'be',
             'fa', 'ze', 'qu', 'xo', 'wi', 'yo', 'an', 'el', 'or', 'um', 'is', 'en', 'ar', 'ol']


def vocabulary(rng, size=6000):
    words = set()
    while len(words) < size:
        words.add(''.join(rng.choice(SYLLABLES) for _ in range(rng.randint(1, 4))))
    return sorted(words)


def zipf_picker(rng, words):
    # Common words appear far more often than rare ones, as in prose.
    weights = [1 / (rank + 1) for rank in range(len(words))]
    order = words[:]
    rng.shuffle(order)

    def pick(count):
        return rng.choices(order, weights=weights, k=count)
    return pick


def titles(rng, words):
    """Page titles with Gitpa's namespace depths; every namespace parent is also a page."""
    depth_choices = [depth for depth, share in DEPTHS.items() for _ in range(share)]
    roots = [w.capitalize() for w in rng.sample(words, 12)]
    chosen, order = set(), []
    while len(order) < PAGES:
        depth = rng.choice(depth_choices)
        parts = [rng.choice(roots)] + [rng.choice(words).capitalize() for _ in range(depth)]
        for end in range(1, len(parts) + 1):
            title = '/'.join(parts[:end])
            if title not in chosen and len(order) < PAGES:
                chosen.add(title)
                order.append(title)
    return order


def sentence(pick, rng):
    words = pick(rng.randint(5, 14))
    return ' '.join(words).capitalize() + '.'


def page(rng, pick, title, all_titles, index):
    links = rng.sample(all_titles, 7)
    lines = [f'type:: {rng.choice(["note", "episode", "person", "topic"])}',
             'public:: false' if index % 97 == 0 else 'public:: true',
             f'tags:: {rng.choice(all_titles)}',
             f'source:: https://example.com/{index}',
             f'created:: {2020 + index % 6}-{1 + index % 12:02d}-{1 + index % 28:02d}',
             f'rating:: {index % 5}',
             f'status:: {rng.choice(["draft", "done", "open"])}',
             f'related:: [[{links[0]}]]',
             '']
    for block in range(rng.randint(4, 10)):
        indent = '\t' * (0 if block == 0 else rng.randint(0, 2))
        text = ' '.join(sentence(pick, rng) for _ in range(rng.randint(1, 2)))
        if block < len(links) - 1:
            text += f' See [[{links[block + 1]}]].'
        if rng.random() < 0.08:
            text += f' #{rng.choice(pick(1)).capitalize()}'
        lines.append(f'{indent}- {text}')
    if index % 2 == 0:
        lines.append(f'- {{{{embed [[{rng.choice(all_titles)}]]}}}}')
    if index % 400 == 0:
        lines += ['- ```python', '  print("lsg")', '  ```']
    return '\n'.join(lines) + '\n'


def file_name(title):
    return title.replace('/', '___') + '.md'


def generate(destination):
    """Write the garden into `destination`, which must not exist; return its digest."""
    rng = random.Random(SEED)
    words = vocabulary(rng)
    pick = zipf_picker(rng, words)
    order = titles(rng, words)
    root = Path(destination)
    (root / 'pages').mkdir(parents=True)
    (root / 'journals').mkdir()
    (root / 'logseq').mkdir()
    (root / 'logseq' / 'config.edn').write_text(CONFIG, encoding='utf-8')
    for index, title in enumerate(order):
        (root / 'pages' / file_name(title)).write_text(page(rng, pick, title, order, index), encoding='utf-8')
    for day in range(1, JOURNALS + 1):
        text = f'public:: true\n\n- {sentence(pick, rng)} [[{rng.choice(order)}]]\n'
        (root / 'journals' / f'2026_10_{day:02d}.md').write_text(text, encoding='utf-8')
    commit(root)
    return digest(root)


def commit(root):
    """Commit the garden at a fixed date, so its git history is the same on every machine."""
    environment = {'GIT_AUTHOR_DATE': '2026-10-31T00:00:00Z', 'GIT_COMMITTER_DATE': '2026-10-31T00:00:00Z',
                   'GIT_AUTHOR_NAME': 'lsg', 'GIT_AUTHOR_EMAIL': 'lsg@example.com',
                   'GIT_COMMITTER_NAME': 'lsg', 'GIT_COMMITTER_EMAIL': 'lsg@example.com',
                   'GIT_CONFIG_GLOBAL': '/dev/null', 'GIT_CONFIG_NOSYSTEM': '1', 'PATH': os.environ.get('PATH', '/usr/bin:/bin')}
    for command in (['init', '--quiet'], ['add', '--all'], ['commit', '--quiet', '-m', 'generated garden']):
        subprocess.run(['git', '-C', str(root), *command], env=environment, check=True)


def digest(root):
    """SHA-256 of every source path and its contents, outside `.git`."""
    sha = hashlib.sha256()
    root = Path(root)
    for path in sorted(p for p in root.rglob('*') if p.is_file() and '.git' not in p.relative_to(root).parts):
        sha.update(str(path.relative_to(root)).encode('utf-8') + b'\0')
        sha.update(path.read_bytes())
    return sha.hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('destination', help='directory to create')
    args = parser.parse_args()
    print(generate(args.destination))


if __name__ == '__main__':
    main()
