import { execFileSync, spawnSync } from 'node:child_process';
import { readFileSync, writeFileSync, mkdirSync, readdirSync, rmSync } from 'node:fs';
import { createHash } from 'node:crypto';
import path from 'node:path';
import { fileURLToPath } from 'node:url';
const root=path.dirname(fileURLToPath(import.meta.url));
process.chdir(root);
const name=process.argv[2];
const corpus=JSON.parse(readFileSync('corpora.json'))[name];
if (!corpus || process.argv.length!==3) throw Error('Usage: node garden.mjs <name from corpora.json>');
const repo=execFileSync('ghq',['list','--full-path','--exact',corpus.repo],{encoding:'utf8'}).trim();
if(!repo) throw Error(`Missing ghq checkout: ${corpus.repo}`);
const dest=path.join(root,'node_modules/corpora',name);
rmSync(dest,{recursive:true,force:true});mkdirSync(dest,{recursive:true});
const paths=execFileSync('git',['-C',repo,'ls-tree','--name-only',corpus.revision,'pages','journals','logseq/config.edn'],{encoding:'utf8'}).trim().split('\n').filter(Boolean);
const archive=execFileSync('git',['-C',repo,'archive',corpus.revision,...paths],{maxBuffer:128*1024*1024});
if(spawnSync('tar',['-x','-C',dest],{input:archive}).status!==0)throw Error('Archive extraction failed');
function walk(dir){return readdirSync(dir,{withFileTypes:true}).flatMap(e=>e.isDirectory()?walk(path.join(dir,e.name)):/\.(md|org)$/i.test(e.name)?[path.join(dir,e.name)]:[]).sort();}
const files=paths.filter(p=>p==='pages'||p==='journals').flatMap(p=>walk(path.join(dest,p))).sort();
const hash=createHash('sha256');
for(const file of files){hash.update(path.relative(dest,file));hash.update('\0');hash.update(createHash('sha256').update(readFileSync(file)).digest());}
const report=`${name}-report.md`;
const runner='node_modules/sources/lsdoc/tools/graph-check.mjs';
const result=spawnSync(process.execPath,[runner,dest,'--mode','both','--no-pathological','--jobs','4','--out',report],{stdio:'inherit'});
if(result.status!==0) throw Error(`Garden comparison failed: ${result.status}`);
const sources=JSON.parse(readFileSync('sources.json'));
// The archived source has no .git; upstream otherwise reports the enclosing gardener revision.
let text=readFileSync(report,'utf8').replace(/^lsdoc version:.*$/m,`lsdoc source revision: \`${sources.lsdoc.revision}\``).replace(/^Graph:.*$/m,`Graph: \`${corpus.repo}\` at \`${corpus.revision}\``);
writeFileSync(report,text);
writeFileSync(`${name}-corpus.json`,JSON.stringify({...corpus,files:files.length,sha256:hash.digest('hex'),selection:'Tracked Markdown and Org files under pages/ and journals/; immutable git archive; upstream 8 MiB limit'},null,2)+'\n');
