import {execFileSync, spawnSync} from 'node:child_process';
import {readFileSync, writeFileSync, readdirSync, mkdirSync} from 'node:fs';
import path from 'node:path';
import {fileURLToPath} from 'node:url';
import os from 'node:os';
process.chdir(path.dirname(fileURLToPath(import.meta.url)));
if(process.platform!=='darwin') throw Error('This peak-memory probe uses macOS /usr/bin/time -l.');
const root='node_modules/corpora/docs';
function walk(dir){return readdirSync(dir,{withFileTypes:true}).flatMap(e=>e.isDirectory()?walk(path.join(dir,e.name)):/\.(md|org)$/i.test(e.name)?[path.join(dir,e.name)]:[]).sort();}
const corpus=['pages','journals'].flatMap(p=>walk(path.join(root,p))).map(file=>({id:path.relative(root,file),input:readFileSync(file,'utf8'),format:file.endsWith('.org')?'org':'md'}));
mkdirSync('node_modules/results',{recursive:true});
writeFileSync('node_modules/results/memory-corpus.json',JSON.stringify(corpus));
const results={platform:process.platform,architecture:process.arch,cpu:os.cpus()[0].model,node:process.version,corpus:JSON.parse(readFileSync('docs-corpus.json')),method:'Peak resident memory of separate whole-corpus processes, including runtime, input, normalized ASTs, and output serialization. One run per parser; no warmup.',parsers:{}};
for(const [name,args] of Object.entries({lsdoc:['node_modules/sources/lsdoc/target/release/lsdoc-parse','node_modules/results/memory-corpus.json','node_modules/results/memory-lsdoc.json'],mldoc:[process.execPath,'memory-worker.mjs','node_modules/results/memory-corpus.json','node_modules/results/memory-mldoc.json']})){
 const run=spawnSync('/usr/bin/time',['-l',...args],{encoding:'utf8',maxBuffer:1024*1024});
 if(run.status!==0)throw Error(run.stderr);
 const peak=run.stderr.match(/(\d+)\s+maximum resident set size/);
 if(!peak)throw Error('Missing time peak memory output');
 results.parsers[name]={peakResidentBytes:Number(peak[1])};
}
writeFileSync('memory.json',JSON.stringify(results,null,2)+'\n');
console.log(JSON.stringify(results.parsers));
