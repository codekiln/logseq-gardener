// A build plugin like Gitpa's homepage episode list: under the first block of the namespace
// root page, list every page of the namespace with a link to its first block, and give each
// listed page its position in the list as metadata for the browser module.
const ROOT = 'tools';
let input = '';
process.stdin.setEncoding('utf8');
for await (const chunk of process.stdin) input += chunk;
const { view } = JSON.parse(input);
const listed = view.pages.filter((page) => page.id.startsWith(`${ROOT}/`) && page.kind === 'page');
const first = new Map();
for (const block of view.blocks) if (!first.has(block.pageId)) first.set(block.pageId, block.id);
const rootBlock = first.get(ROOT);
const rows = listed.map((page) => ({
  content: [{ page: page.id }, { text: ` — ${view.blocks.filter((b) => b.pageId === page.id).length} blocks` }],
  children: first.has(page.id) ? [{ content: [{ page: page.id, block: first.get(page.id), label: 'First block' }] }] : [],
}));
const contributions = [];
if (rows.length) contributions.push({ kind: 'rows', pageId: ROOT, ...(rootBlock ? { blockId: rootBlock } : {}), rows });
listed.forEach((page, index) => contributions.push({ kind: 'metadata', pageId: page.id, values: { position: index + 1, of: listed.length } }));
process.stdout.write(JSON.stringify({ apiVersion: 1, contributions }));
