const test = require('node:test');
const assert = require('node:assert/strict');
const {readFileSync} = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const source = readFileSync(path.join(__dirname, '../crates/logseq-gardener-sdk/src/site/dialogue.js'), 'utf8');
class Events {
  constructor() { this.handlers = new Map(); }
  addEventListener(type, handler, capture) {
    if (!this.handlers.has(type)) this.handlers.set(type, []);
    this.handlers.get(type).push({handler, capture});
  }
  fire(type, event={}) { for (const {handler} of this.handlers.get(type)||[]) handler(event); }
}
class CustomEvent {
  constructor(type, init) { this.type=type; Object.assign(this,init); this.defaultPrevented=false; }
  preventDefault() { if (this.cancelable) this.defaultPrevented=true; }
}
function fixture({ready=1,duration=300,host=false,noAudio=false}={}) {
  const document = new Events(); document.baseURI='https://garden.example.org/page/';
  const audio = new Events(); audio.readyState=ready; audio.duration=duration; audio.currentTime=0; audio.calls=0;
  audio.getAttribute = name => name==='src' ? '/recording.mp3?edition=2' : null;
  audio.play = () => { audio.calls++; return Promise.resolve(); };
  document.querySelectorAll = selector => {
    assert.equal(selector,'audio[data-lsg-recording]');
    return noAudio ? [] : [audio];
  };
  const window={}; const requests=[];
  const context=vm.createContext({window, document, URL, CustomEvent});
  const rerun=()=>vm.runInContext(source,context); rerun();
  function click(seconds, options={}) {
    const attrs={href:`/recording.mp3?edition=2#t=${seconds}`, 'data-lsg-dialogue-time':String(seconds), ...options.attrs};
    const anchor={target:options.target||'',hasAttribute:name=>Object.hasOwn(attrs,name),getAttribute:name=>attrs[name]??null,
      dispatchEvent(event) { requests.push(event); if (host) event.preventDefault(); return !event.defaultPrevented; }};
    const event={button:0, target:{closest:selector=>{assert.equal(selector,'a[data-lsg-dialogue-time]');return anchor;}},
      defaultPrevented:false, preventDefault(){this.defaultPrevented=true;}, ...options.event};
    document.fire('click',event); return {event,anchor};
  }
  return {document,window,audio,requests,click,rerun};
}
test('setup is inert and plain anchor activation seeks its original recording', () => {
  const f=fixture(); assert.equal(f.audio.calls,0);
  const {event,anchor}=f.click(65);
  assert.equal(event.defaultPrevented,true); assert.equal(f.audio.currentTime,65); assert.equal(f.audio.calls,1);
  const request=f.requests[0]; assert.equal(request.type,'lsg:media-seek'); assert.equal(request.bubbles,true); assert.equal(request.cancelable,true);
  assert.equal(request.detail.source,'https://garden.example.org/recording.mp3?edition=2'); assert.equal(request.detail.seconds,65); assert.equal(request.detail.anchor,anchor);
});
test('a host canceling the custom request owns playback and suppresses competing click handlers', () => {
  const f=fixture({host:true}); const {event}=f.click(1650);
  assert.equal(event.defaultPrevented,true); assert.equal(f.audio.calls,0); assert.equal(f.audio.currentTime,0);
  assert.equal(f.requests.length,1);
});
test('latest gesture seek waits for metadata, clamps duration, and never replays from metadata', () => {
  const f=fixture({ready:0,duration:NaN}); f.click(10); f.click(400);
  assert.equal(f.audio.currentTime,0); assert.equal(f.audio.calls,2);
  f.audio.readyState=1; f.audio.duration=250; f.audio.fire('loadedmetadata');
  assert.equal(f.audio.currentTime,250); assert.equal(f.audio.calls,2);
  f.audio.currentTime=17; f.audio.fire('loadedmetadata'); assert.equal(f.audio.currentTime,17);
  f.click(0); assert.equal(f.audio.currentTime,0);
});
test('failed early seek retries safely while errors discard stale pending positions', () => {
  const f=fixture(); let locked=true, time=0;
  Object.defineProperty(f.audio,'currentTime',{get:()=>time,set:value=>{if(locked)throw Error('metadata');time=value;}});
  f.click(80); locked=false; f.audio.fire('loadedmetadata'); assert.equal(time,80);
  f.audio.readyState=0; f.click(90); f.audio.fire('error'); f.audio.fire('loadedmetadata'); assert.equal(time,80);
});
test('rejected or throwing playback keeps native controls usable without unhandled rejection', async () => {
  const f=fixture(); f.audio.play=()=>Promise.reject(Error('blocked')); assert.equal(f.click(12).event.defaultPrevented,true);
  await new Promise(resolve=>setImmediate(resolve)); assert.equal(f.audio.currentTime,12);
  f.audio.play=()=>{throw Error('blocked');}; f.click(14); assert.equal(f.audio.currentTime,14);
});
test('modified navigation, missing audio, invalid data and mismatched fragment retain native links', () => {
  const f=fixture();
  for (const modifier of ['metaKey','ctrlKey','shiftKey','altKey']) assert.equal(f.click(10,{event:{[modifier]:true}}).event.defaultPrevented,false);
  assert.equal(f.click(10,{event:{button:1}}).event.defaultPrevented,false);
  assert.equal(f.click(10,{target:'_blank'}).event.defaultPrevented,false);
  assert.equal(f.click(10,{attrs:{download:''}}).event.defaultPrevented,false);
  assert.equal(f.click(10,{attrs:{href:'/recording.mp3?edition=2#t=11'}}).event.defaultPrevented,false);
  assert.equal(f.click(10,{attrs:{'data-lsg-dialogue-time':'9007199254740992'}}).event.defaultPrevented,false);
  assert.equal(f.audio.calls,0); assert.equal(f.requests.length,0);
  assert.equal(fixture({noAudio:true}).click(10).event.defaultPrevented,false);
});
test('repeated helper loading retains one capture handler and supports replacement anchors', () => {
  const f=fixture(); f.rerun(); f.rerun();
  assert.equal(f.document.handlers.get('click').length,1); assert.equal(f.document.handlers.get('click')[0].capture,true);
  f.click(5); f.click(20); assert.equal(f.audio.calls,2); assert.equal(f.audio.currentTime,20);
  assert.equal(f.audio.handlers.get('loadedmetadata').length,1);
});
const startup = readFileSync(path.join(__dirname, '../crates/logseq-gardener-sdk/src/site/dialogue-start.js'), 'utf8');
test('conditional startup waits for load and retains its original asset URL across host navigation', () => {
  const window = new Events(); const scripts=[];
  const document={baseURI:'https://garden.example.org/nested/readable/',querySelector:selector=>{assert.equal(selector,'link[rel="lsg-dialogue"]');return {href:'https://garden.example.org/prefix/dialogue.js'};},readyState:'loading',createElement:()=>({}),head:{append:script=>scripts.push(script)}};
  const context=vm.createContext({window,document,URL});
  vm.runInContext(startup,context); vm.runInContext(startup,context);
  assert.equal(scripts.length,0); assert.equal(window.handlers.get('load').length,1);
  document.baseURI='https://garden.example.org/replacement/'; window.fire('load');
  assert.equal(scripts.length,1); assert.equal(scripts[0].src,'https://garden.example.org/prefix/dialogue.js');
});
test('conditional startup handles already-loaded pages and an existing controller', () => {
  const scripts=[]; const document={querySelector:()=>({href:'https://garden.example.org/dialogue.js'}),readyState:'complete',createElement:()=>({}),head:{append:script=>scripts.push(script)}};
  vm.runInNewContext(startup,{window:{},document,URL}); assert.equal(scripts.length,1);
  vm.runInNewContext(startup,{window:{logseqGardenerDialogueInstalled:true},document,URL}); assert.equal(scripts.length,1);
});
