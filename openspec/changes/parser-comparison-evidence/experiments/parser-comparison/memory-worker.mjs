import {readFileSync, writeFileSync} from 'node:fs';
import mldoc from 'mldoc';
import {normalizeAst} from './node_modules/sources/lsdoc/harness/lib/normalize.mjs';
import {extractRefs} from './node_modules/sources/lsdoc/harness/lib/refs.mjs';
const corpus=JSON.parse(readFileSync(process.argv[2]));
const output=corpus.map(item=>{
  const config=JSON.stringify({toc:false,parse_outline_only:false,heading_number:false,keep_line_break:true,format:item.format==='org'?'Org':'Markdown',heading_to_list:false,export_md_remove_options:[]});
  const ast=JSON.parse(mldoc.Mldoc.parseJson(item.input,config));
  return {id:item.id,input:item.input,projection:{blocks:normalizeAst(ast),refs:extractRefs(ast,item.format)}};
});
writeFileSync(process.argv[3],JSON.stringify(output));
