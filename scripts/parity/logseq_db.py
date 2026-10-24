"""Read page records from the db.gz of a Logseq publish-spa build."""
import gzip
import json
from dataclasses import dataclass
from pathlib import Path

PREFIX = 'window.logseq_db='
DB_TAG = '~#datascript/DB'
# frontend/publishing.cljs reverses publish-spa's escaping in this order.
UNESCAPES = [
    ('logseq____&amp;', '&'),
    ('logseq____&lt;', '<'),
    ('logseq____&gt;', '>'),
    ('logseq____&quot;', '"'),
    ('logseq____&apos;', "'"),
]
CACHE_DIGITS = 44
CACHE_SIZE = CACHE_DIGITS * CACHE_DIGITS


class FormatError(ValueError):
    """The graph data lacks a piece the decoder needs."""


@dataclass(frozen=True)
class Keyword:
    name: str


@dataclass(frozen=True)
class Tagged:
    tag: str
    value: object


@dataclass(frozen=True)
class Page:
    name: str
    title: str
    journal: bool
    has_content: bool


class TransitReader:
    """Decode Transit JSON, including its cache of repeated keys and keywords."""

    def __init__(self):
        self.cache = []

    def remember(self, text, as_key):
        if len(text) > 3 and (as_key or text[:2] in ('~:', '~$', '~#')):
            if len(self.cache) == CACHE_SIZE:
                self.cache = []
            self.cache.append(text)

    def lookup(self, code):
        if len(code) == 2:
            index = ord(code[1]) - 48
        else:
            index = (ord(code[1]) - 48) * CACHE_DIGITS + ord(code[2]) - 48
        return self.cache[index]

    def text(self, value, as_key=False):
        if value.startswith('^') and value != '^ ':
            value = self.lookup(value)
        else:
            self.remember(value, as_key)
        if value.startswith('~:'):
            return Keyword(value[2:])
        if value.startswith('~#'):
            return Tagged(value[2:], None)
        if value.startswith(('~~', '~^', '~`')):
            return value[1:]
        if value.startswith('~$'):
            return value[2:]
        if value.startswith('~i'):
            return int(value[2:])
        if value.startswith('~m'):
            return int(value[2:])
        if value.startswith('~?'):
            return value[2:] == 't'
        if value.startswith('~_'):
            return None
        return value

    def read(self, value, as_key=False):
        if isinstance(value, str):
            return self.text(value, as_key)
        if isinstance(value, list):
            if value and value[0] == '^ ':
                result = {}
                for index in range(1, len(value), 2):
                    key = self.read(value[index], as_key=True)
                    result[_hashable(key)] = self.read(value[index + 1])
                return result
            if len(value) == 2 and isinstance(value[0], str):
                head = self.read(value[0])
                if isinstance(head, Tagged) and head.value is None:
                    return Tagged(head.tag, self.read(value[1]))
                return [head, self.read(value[1])]
            return [self.read(item) for item in value]
        if isinstance(value, dict):
            return {_hashable(self.read(k, as_key=True)): self.read(v) for k, v in value.items()}
        return value


def _hashable(value):
    if isinstance(value, list):
        return tuple(_hashable(item) for item in value)
    if isinstance(value, dict):
        return tuple(sorted((repr(k), _hashable(v)) for k, v in value.items()))
    return value


def unescape(text):
    for token, character in UNESCAPES:
        text = text.replace(token, character)
    return text


def decode_assignment(raw):
    """Return the DataScript database map from the text of db.gz."""
    if not raw.startswith(PREFIX):
        raise FormatError(f'db.gz does not start with {PREFIX!r}')
    transit = unescape(json.loads(raw[len(PREFIX):]))
    data = json.loads(transit)
    if not (isinstance(data, list) and data and data[0] == DB_TAG):
        raise FormatError(f'graph data is not tagged {DB_TAG!r}')
    db = TransitReader().read(data)
    return db.value


def _untag(value):
    return value.value if isinstance(value, Tagged) else value


def datoms(db):
    for datom in _untag(db[Keyword('datoms')]):
        entity, attribute, value = _untag(datom)[:3]
        yield entity, attribute.name if isinstance(attribute, Keyword) else attribute, value


def pages_from_db(db):
    names, titles, journals, with_content = {}, {}, set(), set()
    for entity, attribute, value in datoms(db):
        if attribute == 'block/name':
            names[entity] = value
        elif attribute == 'block/original-name':
            titles[entity] = value
        elif attribute == 'block/journal?' and value:
            journals.add(entity)
        elif attribute == 'block/page':
            with_content.add(value)
    return [
        Page(name, titles.get(entity, name), entity in journals, entity in with_content)
        for entity, name in names.items()
    ]


def read_pages(path):
    raw = gzip.decompress(Path(path).read_bytes()).decode('utf-8')
    return pages_from_db(decode_assignment(raw))


if __name__ == '__main__':
    import sys

    found = read_pages(sys.argv[1])
    print(f'{len(found)} pages, {sum(p.journal for p in found)} journals, '
          f'{sum(p.has_content for p in found)} with content')
