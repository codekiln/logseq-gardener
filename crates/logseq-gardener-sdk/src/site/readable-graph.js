  // A --readable-urls build's page addresses, in place of graph.js's routeOf.
  // The FNV-1a 64 digest of a title, which names the page's neighbourhood file.
  const digestOf = title => {
    let h = 0xcbf29ce484222325n;
    for (const byte of new TextEncoder().encode(title)) h = ((h ^ BigInt(byte)) * 0x100000001b3n) & 0xffffffffffffffffn;
    return h.toString(16).padStart(16, '0');
  };
  // A page's folder: each segment keeps A-Z a-z 0-9 - . _ ~ and percent-encodes every other UTF-8 byte.
  const routeOf = title => title.split('/').map(segment => encodeURIComponent(segment).replace(/[!'()*]/g, c => '%' + c.charCodeAt(0).toString(16).toUpperCase())).join('/') + '/';
  // The title a folder route was made from.
  const titleAt = route => route.slice(0, -1).split('/').map(decodeURIComponent).join('/');
