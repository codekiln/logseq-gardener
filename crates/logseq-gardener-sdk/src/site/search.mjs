// Pure search v1. Import lazily; this module performs no fetch or DOM work.
export const FORMAT_VERSION = 1;
export const LIMITS = Object.freeze({ query: 256, terms: 16, results: 100, snippet: 240 });
export function normalize(value) {
  return value.toLowerCase().normalize('NFC').replace(/\s+/gu, ' ').trim();
}
const compare = (a, b) => a < b ? -1 : a > b ? 1 : 0;
const route = value => typeof value === 'string' && /^p-[a-f0-9]{16}\.html$/u.test(value);
const text = value => typeof value === 'string' && value.trim().length > 0;
function invalid() { throw new TypeError('Invalid or incompatible search publication; reload the page and retry.'); }

// expectedRevision comes from the page's manifest link, not cached index data.
// Validation establishes publication identity/shape, not cryptographic integrity.
export function prepareIndex(manifest, corpus, expectedRevision) {
  if (!manifest || !corpus || !/^[a-f0-9]{16}$/u.test(expectedRevision || '') ||
      manifest.format_version !== FORMAT_VERSION || corpus.format_version !== FORMAT_VERSION ||
      manifest.revision !== expectedRevision || corpus.revision !== expectedRevision ||
      manifest.engine !== `lsg-search-engine-${expectedRevision}.mjs` ||
      manifest.corpus?.href !== `lsg-search-${expectedRevision}.json` ||
      !Number.isSafeInteger(manifest.corpus.bytes) || manifest.corpus.bytes <= 0 ||
      !Array.isArray(corpus.pages) || !Array.isArray(corpus.content) ||
      manifest.corpus.records !== corpus.pages.length + corpus.content.length) invalid();
  const byId = new Map();
  const pages = corpus.pages.map(record => {
    if (!record || !route(record.id) || record.href !== record.id || !text(record.title) ||
        !Array.isArray(record.aliases) || !record.aliases.every(text) || byId.has(record.id)) invalid();
    const page = Object.freeze({ id: record.id, title: record.title, href: record.href,
      aliases: Object.freeze([...record.aliases]), titleKey: normalize(record.title),
      aliasKeys: Object.freeze(record.aliases.map(normalize)) });
    byId.set(page.id, page);
    return page;
  });
  const ids = new Set();
  const destinations = new Set();
  const content = corpus.content.map(record => {
    if (!record || !byId.has(record.page) || !Number.isSafeInteger(record.position) || record.position < 0 ||
        record.id !== `${record.page}#source-block-${record.position}` || ids.has(record.id) ||
        !text(record.text) || !Array.isArray(record.ancestors) || !record.ancestors.every(text) ||
        typeof record.href !== 'string' || destinations.has(record.href)) invalid();
    const fragment = record.href.slice(record.page.length);
    if (!record.href.startsWith(record.page) ||
        !(fragment === `#source-block-${record.position}` || /^#block-[a-f0-9]{8}(?:-[a-f0-9]{4}){3}-[a-f0-9]{12}$/u.test(fragment))) invalid();
    ids.add(record.id);
    destinations.add(record.href);
    return Object.freeze({ id: record.id, page: byId.get(record.page), href: record.href,
      text: record.text, key: normalize(record.text), position: record.position,
      ancestors: Object.freeze([...record.ancestors]) });
  });
  return Object.freeze({ format_version: FORMAT_VERSION, revision: expectedRevision,
    pages: Object.freeze(pages), content: Object.freeze(content) });
}

// Map normalized offsets through grapheme spans before choosing a display window.
// Windows and ellipses respect whole graphemes, including emoji/combining text.
const segmenter = new Intl.Segmenter('und', { granularity: 'grapheme' });
function snippet(value, terms) {
  const graphemes = [...segmenter.segment(value)];
  if ([...value].length <= LIMITS.snippet) return value;
  const key = normalize(value);
  const match = Math.min(...terms.map(term => key.indexOf(term)).filter(at => at >= 0));
  let normalizedOffset = 0;
  let at = 0;
  for (; at < graphemes.length; at++) {
    const folded = graphemes[at].segment.toLowerCase().normalize('NFC');
    if (normalizedOffset + folded.length > match) break;
    normalizedOffset += folded.length;
  }
  // The builder collapses whitespace. This mapping concerns snippet context,
  // never highlight offsets; display strings remain authored Unicode text.
  let start = at;
  let context = 0;
  while (start > 0 && context < 60) context += [...graphemes[--start].segment].length;
  const pieces = [];
  let used = start > 0 ? 1 : 0;
  let end = start;
  for (; end < graphemes.length; end++) {
    const piece = graphemes[end].segment;
    const count = [...piece].length;
    const suffix = end < graphemes.length - 1 ? 1 : 0;
    if (used + count + suffix > LIMITS.snippet) break;
    pieces.push(piece);
    used += count;
  }
  return (start ? '…' : '') + pieces.join('') + (end < graphemes.length ? '…' : '');
}
export function query(index, input, { limit = 20 } = {}) {
  if (typeof input !== 'string' || input.length > LIMITS.query * 2 || [...input].length > LIMITS.query || !input.isWellFormed()) {
    return { error: 'invalid-query', results: [] };
  }
  if (!Number.isInteger(limit) || limit < 1 || limit > LIMITS.results) {
    return { error: 'invalid-limit', results: [] };
  }
  const normalized = normalize(input);
  if (!normalized) return { error: null, results: [] };
  const terms = [...new Set(normalized.split(' '))];
  if (terms.length > LIMITS.terms) return { error: 'too-many-terms', results: [] };
  const matches = [];
  for (const page of index.pages) {
    let rank = null;
    if (page.titleKey === normalized) rank = 0;
    else if (page.aliasKeys.includes(normalized)) rank = 1;
    else if ([page.titleKey, ...page.aliasKeys].some(key => key.startsWith(normalized))) rank = 2;
    else if ([page.titleKey, ...page.aliasKeys].some(key => terms.every(term => key.includes(term)))) rank = 3;
    if (rank !== null) matches.push({ rank, page, position: -1, result: {
      kind: 'page', page: page.id, title: page.title, href: page.href, text: '', ancestors: [] } });
  }
  for (const block of index.content) {
    if (terms.every(term => block.key.includes(term))) matches.push({ rank: 4,
      page: block.page, position: block.position, result: { kind: 'block', page: block.page.id,
        title: block.page.title, href: block.href, text: block.text, ancestors: [...block.ancestors] } });
  }
  matches.sort((a, b) => a.rank - b.rank || compare(a.page.titleKey, b.page.titleKey) ||
    compare(a.page.href, b.page.href) || a.position - b.position);
  return { error: null, results: matches.slice(0, limit).map(match => ({
    ...match.result, text: match.result.kind === 'block' ? snippet(match.result.text, terms) : ''
  })) };
}

// Hosts keep immutable corpus routes intact and translate only navigation.
export function resolveResult(result, resolvePage = (_title, href) => href) {
  const [href, fragment] = result.href.split('#');
  const adapted = resolvePage(result.title, href);
  if (typeof adapted !== 'string' || !adapted || adapted.includes('#')) {
    throw new TypeError('Search page resolver must return a route without a fragment.');
  }
  return adapted + (fragment ? `#${fragment}` : '');
}
