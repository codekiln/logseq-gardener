import { execFileSync } from 'node:child_process';
import { readFileSync, writeFileSync, readdirSync, mkdirSync, cpSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';
import { createHash } from 'node:crypto';
const root = path.dirname(fileURLToPath(import.meta.url));
if (process.argv.length > 3 || (process.argv.length === 3 && process.argv[2] !== '--update')) throw Error('Usage: node compare.mjs [--update]');
process.chdir(root);
const lsdoc = path.join(root, 'node_modules/sources/lsdoc');
const { canon } = await import(pathToFileURL(path.join(lsdoc, 'harness/lib/compare.mjs')));
const garden = path.resolve(root, '../../../parser-comparison-baseline/experiments/mldoc-baseline/fixtures/garden');
function walk(dir) {
  return readdirSync(dir, {withFileTypes:true}).flatMap(e => e.isDirectory() ? walk(path.join(dir,e.name)) : e.name.endsWith('.md') ? [path.join(dir,e.name)] : []).sort();
}
const extra = path.join(root,'fixtures');
const corpus = [ ...walk(garden).map(file => ({id:path.relative(garden,file),input:readFileSync(file,'utf8')})), ...walk(extra).map(file => ({id:path.relative(extra,file),input:readFileSync(file,'utf8')})) ].sort((a,b) => a.id.localeCompare(b.id,'en')); 
mkdirSync('node_modules/results', {recursive:true});
writeFileSync('node_modules/results/corpus.json',JSON.stringify(corpus));
execFileSync(path.join(lsdoc,'target/release/lsdoc-parse'), ['node_modules/results/corpus.json','node_modules/results/lsdoc.json'],{stdio:'inherit'});
const candidate = JSON.parse(readFileSync('node_modules/results/lsdoc.json'));
const syntax = corpus.map((c,i) => {
  writeFileSync('node_modules/results/oracle-input.json',JSON.stringify(c));
  const reference = JSON.parse(execFileSync(process.execPath,['oracle.mjs','node_modules/results/oracle-input.json'],{encoding:'utf8'}));
  const actual = canon(candidate[i].projection);
  return {file:c.id,sha256:createHash('sha256').update(c.input).digest('hex'),equal:JSON.stringify(actual)===JSON.stringify(reference),lsdoc:actual,mldoc:reference};
});
const preservationCorpus = [...corpus, {id:'unicode-crlf',input:'# café\r\n  - TODO [#A] café\r\n- final'}, {id:'literal-fence',input:'- root\n  ```markdown\n  - not an outline header\n  ```\n- end\n'}];
writeFileSync('node_modules/results/preservation.json',JSON.stringify(preservationCorpus));
const preservation = JSON.parse(execFileSync('cargo',['run','--locked','--quiet','--release','--manifest-path','probe/Cargo.toml','--','node_modules/results/preservation.json'], {encoding:'utf8'}));
const mldocExport = preservationCorpus.map(item=>{
  writeFileSync('node_modules/results/oracle-input.json',JSON.stringify(item));
  const output=JSON.parse(execFileSync(process.execPath,['oracle.mjs','node_modules/results/oracle-input.json','--export'],{encoding:'utf8'}));
  return {file:item.id,unchanged:output===item.input,output};
});
execFileSync('node_modules/.bin/nbb-logseq',['graph.cljs',garden,'node_modules/results/corpus.json','node_modules/results/graph.json'], {stdio:'inherit'});
const graph = JSON.parse(readFileSync('node_modules/results/graph.json'));
const configuredGarden = path.join(root,'node_modules/results/configured-garden');
cpSync(garden,configuredGarden,{recursive:true});
cpSync(extra,configuredGarden,{recursive:true});
mkdirSync(path.join(configuredGarden,'logseq'),{recursive:true});
writeFileSync(path.join(configuredGarden,'logseq/config.edn'),'{:file/name-format :triple-lowbar}');
execFileSync('node_modules/.bin/nbb-logseq',['graph.cljs',configuredGarden,'node_modules/results/corpus.json','node_modules/results/configured-graph.json'],{stdio:'inherit'});
const configuredGraph = JSON.parse(readFileSync('node_modules/results/configured-graph.json'));
const regressions = JSON.parse(readFileSync('regressions.json'));
writeFileSync('node_modules/results/regressions.json',JSON.stringify(regressions));
execFileSync(path.join(lsdoc,'target/release/lsdoc-parse'),['node_modules/results/regressions.json','node_modules/results/regression-output.json'],{stdio:'inherit'});
const regressionOutput=JSON.parse(readFileSync('node_modules/results/regression-output.json'));
const regressionResults=regressions.map((item,i)=>{
  writeFileSync('node_modules/results/oracle-input.json',JSON.stringify(item));
  const reference=JSON.parse(execFileSync(process.execPath,['oracle.mjs','node_modules/results/oracle-input.json'],{encoding:'utf8'}));
  const actual=canon(regressionOutput[i].projection);
  return {id:item.id,format:item.format,input:item.input,equal:JSON.stringify(reference)===JSON.stringify(actual),lsdoc:actual,mldoc:reference};
});
const result = JSON.stringify({sources:JSON.parse(readFileSync('sources.json')),mldoc:'1.5.9',syntax,graph,configuredGraph,preservation,mldocExport,regressions:regressionResults},null,2)+'\n';
if (process.argv.slice(2).join(' ') === '--update') {
  writeFileSync('fixtures.json',result);
  console.log('Updated fixture comparison.');
} else if (process.argv.length !== 2) {
  throw Error('Usage: node compare.mjs [--update]');
} else if (result !== readFileSync('fixtures.json','utf8')) {
  console.error('Fixture comparison differs. Run mise run parser:comparison-update and review the diff.');
  process.exitCode=1;
} else console.log('Fixture syntax and graph results match.');
