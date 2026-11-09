"""Compare shard sizes under other block names and number encodings; see README.md."""
import json, sys, zlib, hashlib, random, math
from pathlib import Path
site = Path(sys.argv[1])
m = json.loads((site/'lsg-search-manifest.json').read_text())
titles = json.loads((site/f"lsg-search-titles-{m['titles']}.json").read_text())['pages']
counts = m['counts']
N = len(counts)
def fnv(s):
    h = 0xcbf29ce484222325
    for b in s.encode():
        h ^= b; h = (h * 0x100000001b3) & 0xffffffffffffffff
    return h
def digest(t): return f"p-{fnv(t):016x}.html"
routes = [e[2] if len(e) > 2 else digest(e[0]) for e in titles]
shards = {}
for key, d in m['shards'].items():
    s = json.loads((site/f'lsg-search-shard-{d}.json').read_text())
    words = []
    for w, pairs in s['words']:
        refs = []; page = 0; block = 0
        for i in range(0, len(pairs), 2):
            st, v = pairs[i], pairs[i+1]
            if i == 0 or st: page += st; block = v
            else: block += v
            refs.append((page, block))
        words.append((w, refs))
    shards[key] = words
def gz(t): 
    c = zlib.compressobj(6, zlib.DEFLATED, 31); return len(c.compress(t.encode()) + c.flush())
def enc_pairs(refs):
    out = []; prev = None
    for p, b in refs:
        if prev is None: out += [p, b]
        elif prev[0] == p: out += [0, b - prev[1]]
        else: out += [p - prev[0], b]
        prev = (p, b)
    return out
def enc_group(refs):
    # [pageStep, n, b1, d2.., ...] per page
    out = []; prevp = 0; i = 0
    while i < len(refs):
        p = refs[i][0]; j = i
        while j < len(refs) and refs[j][0] == p: j += 1
        bs = [b for _, b in refs[i:j]]
        out += [p - prevp, len(bs) - 1, bs[0]] + [bs[k] - bs[k-1] for k in range(1, len(bs))]
        prevp = p; i = j
    return out
B64 = '0123456789abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ-_'
def vlq(nums):
    # base-32 continuation digits: low 5 bits per char, high-bit chars are uppercase set
    s = []
    for n in nums:
        digs = []
        while True:
            digs.append(n & 31); n >>= 5
            if not n: break
        digs.reverse()
        for k, d in enumerate(digs):
            s.append(B64[d + (32 if k < len(digs)-1 else 0)])
    return ''.join(s)
def total(idmap, enc='pairs', fmt='json'):
    tot = 0; mx = 0
    for key, words in shards.items():
        ws = []
        for w, refs in words:
            r = sorted((idmap[p], b) for p, b in refs)
            nums = enc_pairs(r) if enc == 'pairs' else enc_group(r)
            ws.append([w, nums if fmt == 'json' else vlq(nums)])
        g = gz(json.dumps({"format_version": 4, "words": ws}, separators=(',', ':')))
        tot += g; mx = max(mx, g)
    return tot, mx
ident = list(range(N))
rng = random.Random(1); perm = list(range(N)); rng.shuffle(perm)
def hashslots(mult, salt=''):
    M = 1
    while M < mult * N: M *= 2
    return [fnv(salt + r) % M for r in routes], M
schemes = {'title': ident, 'dense-random': perm}
for mult in (1, 2, 4):
    ids, M = hashslots(mult)
    coll = len(ids) - len(set(ids))
    schemes[f'hash M={M} coll={coll}'] = ids
for name, ids in schemes.items():
    for enc in ('pairs', 'group'):
        for fmt in ('json', 'vlq'):
            t, mx = total(ids, enc, fmt)
            print(f'{name:28} {enc:6} {fmt:5} total {t/1e6:.3f} MB  max {mx/1e3:.1f} KB', flush=True)
