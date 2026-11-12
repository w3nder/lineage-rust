import test from 'node:test';
import assert from 'node:assert/strict';
import {encodePath,remoteEntries,loadRemoteClient} from './remote.mjs';

const manifest={files:[
    {path:'system/L2.ini',size:10,sha256:'aa'},
    {path:'maps/lobby.unr',size:20,sha256:'bb'},
    {path:'Sounds/a b#c.ogg',size:5,sha256:'cc'},
]};

test('o caminho é codificado por segmento',()=>{
    assert.equal(encodePath('Sounds/a b#c.ogg'),'Sounds/a%20b%23c.ogg');
});

test('as entradas remotas carregam endereço, tamanho e hash',()=>{
    const entries=remoteEntries('https://cdn.test/base/',manifest);
    assert.deepEqual(entries[0],['client/system/L2.ini',{url:'https://cdn.test/base/system/L2.ini',size:10,lastModified:0xaa,sha256:'aa'}]);
    assert.equal(entries[2][1].url,'https://cdn.test/base/Sounds/a%20b%23c.ogg');
});

test('manifestos inválidos são recusados',()=>{
    assert.throws(()=>remoteEntries('https://x',{}));
    assert.throws(()=>remoteEntries('https://x',{files:[{path:'../x',size:1,sha256:'a'}]}));
    assert.throws(()=>remoteEntries('https://x',{files:[{path:'/x',size:1,sha256:'a'}]}));
    assert.throws(()=>remoteEntries('https://x',{files:[{path:'x',size:-1,sha256:'a'}]}));
    assert.throws(()=>remoteEntries('https://x',{files:[{path:'x',size:1.5,sha256:'a'}]}));
});

test('o cliente remoto exige system e o mapa do lobby',async()=>{
    const ok=async()=>({ok:true,status:200,json:async()=>manifest});
    assert.equal((await loadRemoteClient('https://cdn.test',ok)).length,3);
    const partial=async()=>({ok:true,status:200,json:async()=>({files:[manifest.files[0]]})});
    await assert.rejects(loadRemoteClient('https://cdn.test',partial));
    const missing=async()=>({ok:false,status:403});
    await assert.rejects(loadRemoteClient('https://cdn.test',missing),/HTTP 403/);
});
