import { readFileSync } from 'node:fs';
import mldoc from 'mldoc';
import { normalizeAst } from './node_modules/sources/lsdoc/harness/lib/normalize.mjs';
import { extractRefs } from './node_modules/sources/lsdoc/harness/lib/refs.mjs';
import { canon } from './node_modules/sources/lsdoc/harness/lib/compare.mjs';
const item = JSON.parse(readFileSync(process.argv[2]));
const config = JSON.stringify({toc:false,parse_outline_only:false,heading_number:false,keep_line_break:true,format:item.format==='org'?'Org':'Markdown',heading_to_list:false,export_md_remove_options:[],exporting_keep_properties:process.argv[3]==='--export'});
if (process.argv[3] === '--export') {
  console.log(JSON.stringify(mldoc.Mldoc.parseAndExportMarkdown(item.input,config,JSON.stringify({embed_blocks:[],embed_pages:[]}))));
  process.exit(0);
}
const ast=JSON.parse(mldoc.Mldoc.parseJson(item.input,config));
console.log(JSON.stringify(canon({blocks:normalizeAst(ast),refs:extractRefs(ast,item.format||'md')})));
