import test from 'node:test';
import assert from 'node:assert/strict';
import {allowFolder,folderFiles,folderFile,staleError} from './folder.mjs';
const file=name=>({kind:'file',name,getFile:async()=>({name})});
const directory=(name,children)=>({kind:'directory',name,async *values(){yield* children;}});

test('pasta preserva caminhos, não lê bytes e rejeita raiz incompleta',async()=>{
    const root=directory('Cliente',[directory('System',[file('L2.ini')]),directory('Maps',[file('Lobby.unr')])]);
    const entries=await folderFiles(root);
    assert.deepEqual(entries.map(([path])=>path),['Cliente/System/L2.ini','Cliente/Maps/Lobby.unr']);
    assert.equal(entries[1][1].name,'Lobby.unr');
    await assert.rejects(folderFiles(directory('System',[file('L2.ini')])),/pasta completa/);
});

test('permissão existente é reutilizada e recusa não é contornada',async()=>{
    const requests=[];
    const granted={queryPermission:async value=>{assert.deepEqual(value,{mode:'read'});return 'granted';},requestPermission:async()=>{throw Error('não deveria solicitar');}};
    assert.equal(await allowFolder(granted),true);
    const denied={queryPermission:async()=> 'prompt',requestPermission:async value=>{requests.push(value);return 'denied';}};
    assert.equal(await allowFolder(denied),false);
    assert.deepEqual(requests,[{mode:'read'}]);
});

test('falha de leitura não entrega uma montagem parcial',async()=>{
    const broken={kind:'file',name:'Lobby.unr',getFile:async()=>{throw Error('acesso revogado');}};
    await assert.rejects(folderFiles(directory('Cliente',[directory('System',[file('L2.ini')]),directory('Maps',[broken])])),/acesso revogado/);
});

test('metadados são lidos em paralelo com limite de oito e ordem preservada',async()=>{
    let active=0,peak=0;
    const entries=Array.from({length:33},(_,i)=>({kind:'file',name:`${i}.dat`,async getFile(){active++;peak=Math.max(peak,active);await new Promise(resolve=>setImmediate(resolve));active--;return {name:this.name};}}));
    const root=directory('Cliente',[directory('System',entries),directory('Maps',[file('Lobby.unr')])]);
    const files=await folderFiles(root);
    assert.equal(peak,8);
    assert.equal(active,0);
    assert.deepEqual(files.slice(0,33).map(([,value])=>value.name),entries.map(entry=>entry.name));
});

test('arquivo é reaberto pelo caminho sem distinção de maiúsculas',async()=>{
    const root=directory('Cliente',[directory('System',[file('L2UI.utx')]),directory('Maps',[file('Lobby.unr')])]);
    assert.equal((await folderFile(root,'system/l2ui.utx')).name,'L2UI.utx');
    assert.equal(await folderFile(root,'system/ausente.utx'),null);
    assert.equal(await folderFile(root,'system'),null);
});

test('erro de arquivo alterado no disco é reconhecido',()=>{
    assert.equal(staleError({name:'NotReadableError',message:'x'}),true);
    assert.equal(staleError(new Error('An operation that depends on state cached in an interface object was made but the state had changed since it was read from disk.')),true);
    assert.equal(staleError(new Error('outro')),false);
});
