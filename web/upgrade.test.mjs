import test from 'node:test';
import assert from 'node:assert/strict';
import {upgrade,upgradeOrder} from './upgrade.mjs';

const remote=(name,size)=>[`client/${name}`,{url:`https://cdn.test/${name}`,size,sha256:'ab12'}];
const local=name=>[`client/${name}`,new File([new Uint8Array(1)],name)];

function memoryCache() {
    const map=new Map();
    return {
        async match(key) {const stored=map.get(key);return stored?new Response(stored.slice()):undefined;},
        async put(key,response) {map.set(key,new Uint8Array(await response.arrayBuffer()));},
        async delete(key) {return map.delete(key);},
    };
}

const files=()=>[remote('Maps/22_22.unr',4),remote('Sounds/b.uax',2),local('system/a.ini'),remote('Animations/x.ukx',3),remote('Sounds/a.uax',1),remote('Other/z.bin',1)];
const fetcher=async url=>new Response(new Uint8Array(Number(url.endsWith('22_22.unr')?4:url.includes('b.uax')?2:url.includes('x.ukx')?3:1)));

test('só entradas remotas sobem, por prioridade de pasta e depois por tamanho',()=>{
    assert.deepEqual(upgradeOrder(files()).map(([name])=>name),['client/Sounds/a.uax','client/Sounds/b.uax','client/Animations/x.ukx','client/Maps/22_22.unr','client/Other/z.bin']);
});

test('cada arquivo baixado é entregue ao motor com o caminho relativo',async()=>{
    const sent=[];
    const result=await upgrade(files(),{cache:memoryCache(),fetcher,send:(name,file)=>sent.push([name,file.size])});
    assert.deepEqual(sent.map(([name])=>name).sort(),['Animations/x.ukx','Maps/22_22.unr','Other/z.bin','Sounds/a.uax','Sounds/b.uax']);
    assert.equal(result.files,5);
    assert.equal(result.bytes,11);
});

test('o orçamento de bytes interrompe a subida',async()=>{
    const sent=[];
    const result=await upgrade(files(),{cache:memoryCache(),fetcher,concurrency:1,budgetBytes:3,send:name=>sent.push(name)});
    assert.deepEqual(sent,['Sounds/a.uax','Sounds/b.uax']);
    assert.equal(result.bytes,3);
});

test('falha de rede em um arquivo não impede os outros',async()=>{
    const sent=[];
    const flaky=async url=>url.includes('a.uax')?new Response('',{status:500}):fetcher(url);
    await upgrade(files(),{cache:memoryCache(),fetcher:flaky,send:name=>sent.push(name)});
    assert.ok(!sent.includes('Sounds/a.uax'));
    assert.equal(sent.length,4);
});
