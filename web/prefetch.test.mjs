import test from 'node:test';
import assert from 'node:assert/strict';
import {prefetch,discardStale,learnPaths,learnedPaths,wantedFiles} from './prefetch.mjs';
import {remoteEntries,stamp} from './remote.mjs';

function memoryCache() {
    const map=new Map();
    return {
        map,
        async match(key) {const stored=map.get(typeof key==='string'?key:key.url);return stored?new Response(stored.slice()):undefined;},
        async put(key,response) {map.set(typeof key==='string'?key:key.url,new Uint8Array(await response.arrayBuffer()));},
        async delete(key) {return map.delete(typeof key==='string'?key:key.url);},
        async keys() {return [...map.keys()].map(url=>({url}));},
    };
}

const manifest={files:[
    {path:'system/A.ini',size:3,sha256:'aa11bb22'},
    {path:'maps/lobby.unr',size:4,sha256:'cc33dd44'},
    {path:'Sounds/big.ogg',size:5,sha256:'ee55ff66'},
]};
const bodies={'https://cdn.test/system/A.ini':[1,2,3],'https://cdn.test/maps/lobby.unr':[4,5,6,7],'https://cdn.test/Sounds/big.ogg':[8,9,10,11,12]};
const entries=remoteEntries('https://cdn.test',manifest);

function fetcher(log) {
    return async url=>{log.push(url);return bodies[url]?new Response(new Uint8Array(bodies[url])):new Response('',{status:404});};
}

test('só os arquivos pedidos viram arquivos locais; os outros continuam remotos',async()=>{
    const log=[];
    const {files,failures}=await prefetch(entries,new Set(['system/a.ini','maps/lobby.unr']),{cache:memoryCache(),fetcher:fetcher(log)});
    assert.deepEqual(failures,[]);
    assert.ok(files[0][1] instanceof File);
    assert.equal(files[0][1].size,3);
    assert.equal(files[0][1].lastModified,stamp('aa11bb22'));
    assert.ok(files[1][1] instanceof File);
    assert.equal(files[2][1].url,'https://cdn.test/Sounds/big.ogg');
    assert.deepEqual(new Uint8Array(await files[1][1].arrayBuffer()),new Uint8Array([4,5,6,7]));
    assert.equal(log.length,2);
});

test('a segunda visita lê do cache e não baixa nada',async()=>{
    const cache=memoryCache();
    const wanted=new Set(['system/a.ini','maps/lobby.unr','sounds/big.ogg']);
    await prefetch(entries,wanted,{cache,fetcher:fetcher([])});
    const log=[];
    const {files}=await prefetch(entries,wanted,{cache,fetcher:fetcher(log)});
    assert.equal(log.length,0);
    assert.ok(files.every(([,value])=>value instanceof File));
});

test('falha de rede ou tamanho errado mantém a entrada remota e é relatada',async()=>{
    const cache=memoryCache();
    const broken={...bodies,'https://cdn.test/system/A.ini':[1,2]};
    const failing=async url=>url.endsWith('lobby.unr')?new Response('',{status:500}):new Response(new Uint8Array(broken[url]));
    const {files,failures}=await prefetch(entries,new Set(['system/a.ini','maps/lobby.unr']),{cache,fetcher:failing});
    assert.equal(failures.length,2);
    assert.equal(files[0][1].url,'https://cdn.test/system/A.ini');
    assert.equal(cache.map.size,0);
});

test('o progresso chega ao total e a concorrência é respeitada',async()=>{
    let active=0,peak=0;
    const slow=async url=>{active++;peak=Math.max(peak,active);await new Promise(r=>setTimeout(r,5));active--;return new Response(new Uint8Array(bodies[url]));};
    const seen=[];
    await prefetch(entries,new Set(['system/a.ini','maps/lobby.unr','sounds/big.ogg']),{cache:memoryCache(),fetcher:slow,concurrency:2,progress:p=>seen.push(p)});
    assert.equal(peak,2);
    assert.deepEqual(seen.at(-1),{files:3,total:3,bytes:12,totalBytes:12});
});

test('versões antigas saem do cache',async()=>{
    const cache=memoryCache();
    cache.map.set('https://cdn.test/system/A.ini?v=old',new Uint8Array([1]));
    cache.map.set('https://cdn.test/system/A.ini?v=aa11bb22',new Uint8Array([1,2,3]));
    cache.map.set('https://cdn.test/removed.bin?v=zz',new Uint8Array([1]));
    assert.equal(await discardStale(entries,cache),2);
    assert.deepEqual([...cache.map.keys()],['https://cdn.test/system/A.ini?v=aa11bb22']);
});

test('os caminhos aprendidos persistem sem duplicar e entram nos pedidos',()=>{
    const store=new Map();
    const storage={getItem:key=>store.get(key) ?? null,setItem:(key,value)=>store.set(key,value)};
    assert.equal(learnPaths(storage,['/client/system/a.ini','/client/maps/x.unr','/etc/passwd',7]),2);
    assert.equal(learnPaths(storage,['/client/system/a.ini','/client/sounds/y.ogg']),3);
    assert.deepEqual(learnedPaths(storage).sort(),['maps/x.unr','sounds/y.ogg','system/a.ini']);
    assert.deepEqual([...wantedFiles(['System/Z.ini'],storage)].sort(),['maps/x.unr','sounds/y.ogg','system/a.ini','system/z.ini']);
    assert.deepEqual(learnedPaths({getItem:()=>'não é json'}),[]);
});

test('a chave do cache de preparação é a mesma com arquivos baixados ou remotos',async()=>{
    const {cacheKey}=await import('./preparation-cache.mjs');
    const {files}=await prefetch(entries,new Set(['system/a.ini','maps/lobby.unr']),{cache:memoryCache(),fetcher:fetcher([])});
    assert.equal(await cacheKey(files,'{}','lobby','b'),await cacheKey(entries,'{}','lobby','b'));
});

test('stale cache entries are all removed and counted',async()=>{
    const {discardStale}=await import('./prefetch.mjs');
    const kept='https://x/a?v=1',stale='https://x/b?v=0',gone='https://x/c?v=9';
    const deleted=[];
    const cache={keys:async()=>[{url:kept},{url:stale},{url:gone}],delete:async request=>{deleted.push(request.url);}};
    const removed=await discardStale([['a',{url:'https://x/a',sha256:'1'}],['b',{url:'https://x/b',sha256:'1'}]],cache);
    assert.equal(removed,2);
    assert.deepEqual(deleted.sort(),[gone,stale].sort());
});
