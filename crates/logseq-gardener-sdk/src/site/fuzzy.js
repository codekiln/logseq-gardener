// Logseq's fuzzy page score (frontend/search.cljs), shared by All pages and search and
// ported to Rust as search::fuzzy_score: consecutive matches earn more, then a
// length-similarity term, and an exact substring adds 1000. Spaces, brackets, slashes,
// underscores and parentheses are ignored.
const fuzzyClean = (s) => s.toLowerCase().replace(/[[ \\/_\]()]+/g, '');
function fuzzyScore(query, name) {
  const q = fuzzyClean(query);
  const s = fuzzyClean(name);
  let i = 0, mult = 1, total = 0;
  for (let j = 0; j < s.length && i < q.length; j++) {
    if (q[i] === s[j]) {
      total += mult++;
      i++;
    } else mult = 1;
  }
  if (i < q.length) return 0;
  const max = Math.max(q.length, s.length);
  return total + (max ? 1 - (max - Math.min(q.length, s.length)) / max : 1) + (name.includes(query) ? 1000 : 0);
}
