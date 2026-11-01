  // A --readable-urls build's page addresses, in place of graph.js's routeOf.
  // The FNV-1a 64 digest of a title, which names the page's neighbourhood file.
  const digestOf = title => {
    let h = 0xcbf29ce484222325n;
    for (const byte of new TextEncoder().encode(title)) h = ((h ^ BigInt(byte)) * 0x100000001b3n) & 0xffffffffffffffffn;
    return h.toString(16).padStart(16, '0');
  };
  // Whether a title can name folders, as page_urls::readable_url decides: every segment is nonempty, is not . or ..,
  // has no surrounding Unicode White_Space and fits 255 UTF-8 bytes, and the title has no backslash or control character.
  // JavaScript's \s is Rust's White_Space and U+FEFF; U+0085, the one White_Space character it lacks, is a control character.
  const EDGE_SPACE = /^[^\S\uFEFF]|[^\S\uFEFF]$/;
  const foldable = title => !/[\\\p{Cc}]/u.test(title) && title.split('/').every(segment => segment && segment !== '.' && segment !== '..'
    && !EDGE_SPACE.test(segment) && new TextEncoder().encode(segment).length <= 255);
  // A page's folder: each segment keeps A-Z a-z 0-9 - . _ ~ and percent-encodes every other UTF-8 byte.
  // A title that cannot name folders keeps its p-<digest>.html file at the root.
  const routeOf = title => foldable(title)
    ? title.split('/').map(segment => encodeURIComponent(segment).replace(/[!'()*]/g, c => '%' + c.charCodeAt(0).toString(16).toUpperCase())).join('/') + '/'
    : 'p-' + digestOf(title) + '.html';
  // The title a folder route was made from.
  const titleAt = route => route.slice(0, -1).split('/').map(decodeURIComponent).join('/');
  // The digest that names a route's neighbourhood file, and whether a title's page is at a route.
  const digestAt = route => route.endsWith('/') ? digestOf(titleAt(route)) : route.slice(2, 18);
  const isAt = (title, route) => route.endsWith('/') ? title === titleAt(route) : routeOf(title) === route;
