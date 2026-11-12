import {cacheKey,digest,validMetadata,discardUnpublished} from './preparation-cache.mjs';
import {config} from './config.mjs';
const status=text=>postMessage({type:'status',text});
const read=async(directory,name)=>(await directory.getFileHandle(name)).getFile();
const version=new URL(import.meta.url).searchParams.get('v') || '';

async function wasmBytes() {
    const url=new URL('./pkg/killer_web_bg.wasm',import.meta.url);
    url.searchParams.set('v',version);
    const wasm=await fetch(url);
    if (!wasm.ok) throw new Error(`Motor: HTTP ${wasm.status}`);
    return wasm.arrayBuffer();
}

let loaded=null;
function engine(bytes) {
    loaded??=(async()=>{
        const api=await import(`./pkg/killer_web.js?v=${version}`);
        const instance=await api.default({module_or_path:bytes ?? await wasmBytes()});
        return {...api,memory:instance.memory};
    })();
    return loaded;
}

async function writer(directory,name) {
    const handle=await (await directory.getFileHandle(name,{create:true})).createSyncAccessHandle();
    handle.truncate(0);
    let offset=0;
    return {
        write(bytes) {
            let written=0;
            while (written<bytes.length) {
                const count=handle.write(bytes.subarray(written),{at:offset});
                if (!count) throw new Error('Gravação incompleta do cache');
                written+=count;offset+=count;
            }
        },
        close() {try {handle.flush();} finally {handle.close();}},
    };
}

async function prepare({files,settings,tile,trusted},exclusive=false) {
    const started=performance.now();
    if (!navigator.storage?.getDirectory) throw new Error('Cache local OPFS indisponível');
    const root=await (await navigator.storage.getDirectory()).getDirectoryHandle('killer-prepared-v1',{create:true});
    await discardUnpublished(root,exclusive);
    const stable=Boolean(config.build);
    const bytes=stable?null:await wasmBytes();
    const build=stable?config.build:await digest(bytes);
    const key=await cacheKey(files,settings,tile,build);
    const directory=await root.getDirectoryHandle(key,{create:true});
    try {
        status('Verificando o cache do mapa…');
        const metadata=JSON.parse(await (await read(directory,'metadata.json')).text());
        const scene=await read(directory,'scene');
        const trace=await read(directory,'trace');
        if (validMetadata(metadata,scene,trace)) {
            const quick=trusted && stable && metadata.dependencies==='manifest';
            const {fingerprint_files,fingerprint_blob}=quick?{}:await engine(bytes);
            if (quick || ((trusted ? metadata.dependencies==='manifest' : fingerprint_files(files,JSON.stringify(metadata.paths))===metadata.dependencies) &&
                fingerprint_blob(scene)===metadata.sceneHash && fingerprint_blob(trace)===metadata.traceHash)) {
                return {type:'prepared',scene,trace,hit:true,prepareMs:performance.now()-started,paths:metadata.paths};
            }
        }
    } catch {}
    await directory.removeEntry('metadata.json').catch(()=>{});
    const {prepare_world,fingerprint_files,memory}=await engine(bytes);
    status('Preparando mapa e colisão em segundo plano…');
    let sceneWriter,traceWriter;
    try {
        sceneWriter=await writer(directory,'scene');
        traceWriter=await writer(directory,'trace');
        const labels={read:'Lendo o mapa…',scene:'Preparando o cenário…',encode:'Salvando o cenário para as próximas entradas…',trace:'Preparando a movimentação no mapa…',trace_write:'Salvando os dados de movimentação…',done:'Conferindo os arquivos preparados…'};
        const progress=phase=>{
            status(labels[phase] || 'Preparando mapa…');
            postMessage({type:'profile',tile,phase,elapsedMs:performance.now()-started,wasmCapacityMiB:memory.buffer.byteLength/1048576});
        };
        const metadata=JSON.parse(prepare_world(files,settings,tile,sceneWriter.write,traceWriter.write,progress));
        sceneWriter.close();sceneWriter=null;
        traceWriter.close();traceWriter=null;
        metadata.version=1;
        metadata.dependencies=trusted?'manifest':fingerprint_files(files,JSON.stringify(metadata.paths));
        const scene=await read(directory,'scene');
        const trace=await read(directory,'trace');
        metadata.sceneSize=scene.size;
        metadata.traceSize=trace.size;
        const output=await writer(directory,'metadata.json');
        try {output.write(new TextEncoder().encode(JSON.stringify(metadata)));} finally {output.close();}
        return {type:'prepared',scene,trace,hit:false,prepareMs:performance.now()-started,paths:metadata.paths};
    } catch (error) {
        try {sceneWriter?.close();} catch {}
        try {traceWriter?.close();} catch {}
        await root.removeEntry(key,{recursive:true}).catch(()=>{});
        throw error;
    }
}

onmessage=({data})=>{
    const run=navigator.locks?navigator.locks.request('killer-world-preparation',()=>prepare(data,true)):prepare(data);
    run.then(result=>postMessage(result)).catch(error=>postMessage({type:'error',text:String(error?.message || error)}));
};
