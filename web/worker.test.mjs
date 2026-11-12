import test from 'node:test';
import assert from 'node:assert/strict';
import {readFileSync} from 'node:fs';
import vm from 'node:vm';

const source = readFileSync(new URL('./worker.js', import.meta.url), 'utf8').replace(/^import .*;\n/,'').replaceAll('import.meta.url', '"http://localhost/worker.js"');
function worker() {
    const messages = [];
    const state = {disconnects:0,closed:0,inputs:0};
    const context = vm.createContext({postMessage:message=>messages.push(message),console:{error(){}},WebAssembly,state,performance:{now:()=>0},requestAnimationFrame:callback=>{state.nextFrame=callback;}});
    vm.runInContext(source+`\nclient={lobby_error(text){state.lobbyError=text;},disconnect(){state.disconnects++;},pointer(){state.inputs++;}};socket={close(){state.closed++;}};`,context);
    return {messages,state,run:code=>vm.runInContext(code,context)};
}

test('a WASM trap stops access and retains the original error',()=>{
    const {messages,state,run}=worker();
    run('disconnected(new WebAssembly.RuntimeError("unreachable"));onmessage({data:{type:"pointer",values:[]}});failure(new Error("recursive use"));');
    assert.equal(state.disconnects,0);
    assert.equal(state.inputs,0);
    assert.equal(state.closed,1);
    assert.equal(messages.length,1);
    assert.match(messages[0].text,/unreachable/);
});

test('the Rust panic message takes precedence over the follow-up trap',()=>{
    const {messages,state,run}=worker();
    run('console.error("panicked at render.rs:10: validation failed");disconnected(new WebAssembly.RuntimeError("unreachable"));');
    assert.equal(messages[0].text,'panicked at render.rs:10: validation failed');
    assert.equal(state.disconnects,0);
});

test('an ordinary network disconnect still resets the client',()=>{
    const {messages,state,run}=worker();
    run('disconnected(new Error("Conexão encerrada"));');
    assert.equal(state.disconnects,1);
    assert.equal(state.closed,1);
    assert.equal(messages[0].type,'disconnected');
    assert.equal(state.lobbyError,'Conexão encerrada');
});

test('disconnect cancels preparation and discards queued packets',()=>{
    const {state,run}=worker();
    run('preparation={cancel(){state.cancelled=true;}};loading=true;pendingPackets=[new ArrayBuffer(4)];pendingBytes=4;disconnected(new Error("closed"));');
    assert.equal(state.cancelled,true);
    assert.equal(run('pendingPackets.length'),0);
    assert.equal(run('pendingBytes'),0);
    assert.equal(run('loading'),false);
});

test('late preparation cannot install a world after disconnect',async()=>{
    const {state,run}=worker();
    const task=run('client.settings=()=>"{}";client.finish_world=()=>{state.installed=true;};preparation={run:()=>new Promise(resolve=>{state.resolve=resolve;}),cancel(){}};loadWorld("23_24");');
    run('disconnected(new Error("closed"));state.resolve({scene:{},trace:{}});');
    await task;
    assert.equal(state.installed,undefined);
});

test('prepared entry installs before draining packets in arrival order',async()=>{
    const {state,run}=worker();
    const task=run(`
        state.order=[];
        client.settings=()=>"{}";
        client.finish_world=(scene,trace)=>{state.order.push(["install",scene,trace]);return '{"entered":true}';};
        client.loading_stats=()=>"{}";
        client.game_receive=bytes=>{state.order.push(["packet",bytes[0]]);return "{}";};
        preparation={run:()=>new Promise(resolve=>{state.resolve=resolve;}),cancel(){}};
        loadWorld("23_24");
    `);
    run('pendingPackets=[new Uint8Array([1]).buffer,new Uint8Array([2]).buffer];pendingBytes=2;state.resolve({scene:"scene",trace:"trace",hit:true,prepareMs:123});');
    await task;
    assert.deepEqual(JSON.parse(JSON.stringify(state.order)),[['install','scene','trace'],['packet',1],['packet',2]]);
    assert.equal(run('loading'),false);
});

test('cache failure keeps the original load path available',async()=>{
    const {state,messages,run}=worker();
    await run(`
        client.settings=()=>"{}";
        client.finish_world=(scene,trace)=>{state.original=scene===undefined && trace===undefined;return '{"entered":true}';};
        client.loading_stats=()=>"{}";
        preparation={run:()=>Promise.reject(new Error("QuotaExceededError")),cancel(){}};
        loadWorld("23_24");
    `);
    assert.equal(state.original,true);
    assert.equal(messages.some(message=>message.type==='warning'),true);
    assert.equal(messages.some(message=>message.type==='entered'),true);
});

test('lobby transport sends native packets and closes only after logout is queued',()=>{
    const {state,messages,run}=worker();
    run(`
        var WebSocket={OPEN:1};
        state.order=[];
        socket={readyState:1,bufferedAmount:0,send:bytes=>state.order.push([...bytes]),close:()=>state.order.push('close')};
        client.lobby_actions=()=>JSON.stringify([{kind:'send',bytes:[1,2]},{kind:'back',packets:[[3,4]]}]);
        lobbyActions();
    `);
    assert.deepEqual(JSON.parse(JSON.stringify(state.order)),[[1,2],[3,4],'close']);
    assert.equal(messages.at(-1).type,'lobby');
});
test('lobby frames keep rendering during background preparation without reentering WASM',()=>{
    const {state,run}=worker();
    run(`
        client.frame=()=>{state.frames=(state.frames||0)+1;state.inFrame=false;};
        client.settings_revision=()=>0;
        client.game_cursor_request=()=>'0';
        client.audio_commands=()=>[];
        client.preferences=()=>'';
        client.lobby_actions=()=>{if(state.inFrame) throw new Error('reentrant');return '[]';};
        loading=true;
        beginFrames('');
        state.inFrame=true;
        state.nextFrame(1000);
    `);
    assert.equal(state.frames,1);
    assert.equal(run('failed'),false);
    assert.equal(run('loading'),true);
});
