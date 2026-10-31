// Parse one note with mldoc in a fresh process and print lsdoc's canonical projection of it.
// Usage: node mldoc-worker.mjs <extracted lsdoc dir> <input JSON {input, format}>
import { readFileSync } from 'node:fs';
import { createRequire } from 'node:module';
import path from 'node:path';
import { pathToFileURL } from 'node:url';

const [lsdoc, inputPath] = process.argv.slice(2);
const harness = path.join(lsdoc, 'harness');
const { Mldoc } = createRequire(path.join(harness, 'oracle.mjs'))('mldoc');
const lib = name => import(pathToFileURL(path.join(harness, 'lib', name)).href);
const { normalizeAst } = await lib('normalize.mjs');
const { extractRefs } = await lib('refs.mjs');
const { canon } = await lib('compare.mjs');
const item = JSON.parse(readFileSync(inputPath, 'utf8'));
const config = JSON.stringify({ toc: false, parse_outline_only: false, heading_number: false, keep_line_break: true,
  format: item.format === 'org' ? 'Org' : 'Markdown', heading_to_list: false, export_md_remove_options: [], exporting_keep_properties: false });
const ast = JSON.parse(Mldoc.parseJson(item.input, config));
console.log(JSON.stringify(canon({ blocks: normalizeAst(ast), refs: extractRefs(ast, item.format || 'md') })));
