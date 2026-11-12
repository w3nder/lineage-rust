import {Preparation} from './preparation-cache.mjs';

Error.stackTraceLimit=200;
let client;
let preparation;
let mountedFiles=[];
let preparationArgs=null;
let refreshWaiter=null;
const staleError=error=>error?.name==='NotReadableError' || /state had changed|NotReadable|could not be read/i.test(String(error?.message ?? error));
let loading=false;
let loadEpoch=0;
let pendingPackets=[];
let pendingBytes=0;

function cancelPreparation() {
    loadEpoch++;
    loading=false;
    pendingPackets=[];
    pendingBytes=0;
    preparation?.cancel();
}

let failed = false;
let panicReason;
const reportError = console.error.bind(console);
console.error = (...values) => {
    const message = values.filter(value => typeof value === 'string').join(' ');
    if (!panicReason && message.includes('panicked at')) panicReason = message;
    reportError(...values);
};
let canvas;
let cursorRequest;
let socket;
let gateway;
let gameConnected = false;
const status = text => postMessage({ type: 'status', text });
const stage = (op,key,detail) => postMessage({type:'stage',op,key,detail});
const failure = error => {
    if (failed) return;
    failed = true;
    cancelPreparation();
    const previous = socket;
    socket = null;
    gameConnected = false;
    client = null;
    if (previous) previous.close();
    postMessage({ type: 'error', text: panicReason || String(error?.stack || error?.message || error) });
};

function disconnected(error) {
    cancelPreparation();
    const previous = socket;
    socket = null;
    gameConnected = false;
    if (previous) previous.close();
    if (panicReason || error instanceof WebAssembly.RuntimeError || String(error?.message || error).includes('recursive use')) {
        failure(error);
        return;
    }
    try { client.disconnect(); client.lobby_error(String(error.message || error)); } catch (cleanupError) { failure(panicReason ? cleanupError : error); return; }
    postMessage({type:'disconnected',text:String(error.message || error)});
}

function send(bytes) {
    if (!socket || socket.readyState !== WebSocket.OPEN) throw new Error('Conexão indisponível');
    if (socket.bufferedAmount + bytes.length > 1024 * 1024) {
        socket.close();
        throw new Error('Conexão lenta: fila de envio excedida');
    }
    socket.send(new Uint8Array(bytes));
}

function flush() {
    if (gameConnected && socket?.readyState === WebSocket.OPEN) for (const bytes of JSON.parse(client.outgoing())) send(bytes);
}

function connect(game) {
    if (socket) { const previous = socket; socket = null; previous.close(); }
    gameConnected = game;
    const current = new WebSocket(`${gateway}/${game ? 'game' : 'login'}`);
    current.binaryType = 'arraybuffer';
    socket = current;
    current.onopen = () => {
        if (current !== socket || !game) return;
        try { send(client.game_start()); } catch (error) { disconnected(error); }
    };
    current.onerror = () => { if (current === socket) disconnected(new Error('Não foi possível conectar ao gateway')); };
    current.onclose = () => { if (current === socket) disconnected(new Error('Conexão encerrada')); };
    current.onmessage = ({data}) => {
        if (current !== socket) return;
        try {
            if (!(data instanceof ArrayBuffer)) throw new Error('Gateway enviou mensagem não binária');
            if (game && loading) {
                if (pendingBytes+data.byteLength>8*1024*1024) throw new Error('Fila de entrada excedida durante carregamento');
                pendingPackets.push(data);pendingBytes+=data.byteLength;
                return;
            }
            receive(data,game);
        } catch (error) { disconnected(error); }
    };
}

function lobbyActions() {
    for (const action of JSON.parse(client.lobby_actions())) {
        if (action.kind==='connect') connect(false);
        else if (action.kind==='send') send(action.bytes);
        else if (action.kind==='back' || action.kind==='exit') {
            for (const bytes of action.packets) if (socket?.readyState===WebSocket.OPEN) send(bytes);
            const previous=socket;socket=null;gameConnected=false;
            previous?.close();
            postMessage({type:action.kind==='exit'?'exit':'lobby'});
        }
    }
}

function applyResult(result) {
    for (const bytes of result.outgoing || []) send(bytes);
    if (result.servers) postMessage({type:'lobby'});
    if (result.characters) postMessage({type:'lobby'});
    if (result.entered) postMessage({type:'entered'});
    if (result.connectGame) connect(true);
    else flush();
}

function receive(data,game) {
    const result=JSON.parse(game?client.game_receive(new Uint8Array(data)):client.login_receive(new Uint8Array(data)));
    applyResult(result);
    if (result.loadWorld) void loadWorld(result.loadWorld);
}

function reportStale() {
    if (!client || failed) return;
    const paths=JSON.parse(client.stale_paths());
    if (paths.length) postMessage({type:'stale',paths});
}

function refreshAll() {
    return new Promise(resolve=>{
        refreshWaiter=resolve;
        postMessage({type:'stale',all:true});
        setTimeout(()=>{if (refreshWaiter===resolve) {refreshWaiter=null;resolve(false);}},60000);
    });
}

function refreshFile(name,file) {
    client.refresh_file(name,file);
    const suffix=`/${name.toLowerCase()}`;
    const index=mountedFiles.findIndex(([path])=>path.toLowerCase().endsWith(suffix));
    if (index>=0) mountedFiles[index]=[mountedFiles[index][0],file];
}

async function loadWorld(tile) {
    const epoch=++loadEpoch;
    loading=true;
    let prepared;
    try {
        stage('begin','world-prepare');
        try {prepared=await preparation.run(tile,client.settings());}
        catch (error) {
            if (epoch!==loadEpoch || failed) return;
            if (staleError(error) && await refreshAll()) {
                try {prepared=await preparation.run(tile,client.settings());}
                catch (retry) {postMessage({type:'warning',text:`Preparação em segundo plano indisponível: ${retry.message}. Carregando pelos arquivos originais.`});}
            }
            else postMessage({type:'warning',text:`Preparação em segundo plano indisponível: ${error.message}. Carregando pelos arquivos originais.`});
        }
        if (epoch!==loadEpoch || failed) return;
        stage('complete','world-prepare',prepared?(prepared.hit?'cache quente':'cache frio'):'sem cache');
        status(prepared?.hit?'Instalando mapa do cache…':'Instalando mapa e interface…');
        applyResult(JSON.parse(client.finish_world(prepared?.scene,prepared?.trace)));
        postMessage({type:'loadingStats',...JSON.parse(client.loading_stats()),cacheHit:prepared?.hit || false,prepareMs:prepared?.prepareMs || 0});
        loading=false;
        const queued=pendingPackets;
        pendingPackets=[];pendingBytes=0;
        for (const packet of queued) receive(packet,true);
    } catch (error) {disconnected(error);}
}

async function response(path) {
    const result = await fetch(path);
    if (!result.ok) throw new Error(`${path}: HTTP ${result.status}`);
    return result;
}

async function start(data) {
    status('Carregando o motor WebAssembly…');
    const version=new URL(import.meta.url).searchParams.get('v') || 'lobby-1';
    const {default:init,WebClient}=await import(`./pkg/killer_web.js?v=${encodeURIComponent(version)}`);
    const wasmUrl=new URL('./pkg/killer_web_bg.wasm',import.meta.url);
    wasmUrl.searchParams.set('v',version);
    await init({module_or_path:fetch(wasmUrl)});
    status('Inicializando WebGPU…');
    canvas = data.canvas;
    client = guarded(await WebClient.create(canvas, data.scale));
    client.set_progress((name)=>stage('complete',name));
    const url=new URL(data.gateway);
    if (!['ws:','wss:'].includes(url.protocol) || url.username || url.password || url.search || url.hash) throw new Error('Endereço do gateway inválido');
    gateway=url.href.replace(/\/$/,'');
    beginFrames(data.preferences);
    postMessage({type:'engineReady'});
    stage('complete','engine');
    status('Restaurando preferências…');
    client.restore_preferences(data.preferences);
    try {client.restore_settings(data.settings || '{}');}
    catch {postMessage({type:'warning',text:'As configurações salvas estão inválidas. Usando os arquivos originais nesta sessão.'});}
    if (data.files) {
        status('Lendo os arquivos locais do cliente…');
        stage('begin','mount');
        const mounted=client.mount_files(data.files);
        stage('complete','mount',`${mounted} arquivos`);
        try {
            const bytes=client.loading_image();
            const header=new DataView(bytes.buffer,bytes.byteOffset,8);
            const rgba=bytes.slice(8);
            postMessage({type:'loadingImage',width:header.getUint32(0,true),height:header.getUint32(4,true),rgba},[rgba.buffer]);
        } catch (error) {postMessage({type:'warning',text:`Imagem de loading: ${error.message || error}`});}
        try {postMessage({type:'gameCursors',resources:client.game_cursors()});}
        catch (error) {postMessage({type:'warning',text:`Cursor: ${error.message || error}`});}
        mountedFiles=data.files;
        preparationArgs=[version,status,stats=>postMessage({...stats,type:'preparationStats'}),Boolean(data.trusted)];
        preparation=new Preparation(mountedFiles,...preparationArgs);
        setInterval(reportStale,1000);
        client.enable_background_loading();
        if (data.mode==='demo') {
            status('Carregando prévia de Giran…');
            let prepared;
            try {prepared=await preparation.run('22_22',client.settings());}
            catch (error) {postMessage({type:'warning',text:`Cache indisponível: ${error.message}`});}
            if (prepared) client.load_prepared_tile('22_22',prepared.scene,prepared.trace);
            else client.load_tile('22_22');
        }
        else {
            status('Preparando lobby…');
            stage('begin','prepare');
            let prepared;
            try {prepared=await preparation.run('lobby',client.settings());}
            catch (error) {postMessage({type:'warning',text:`Cache do lobby indisponível: ${error.message}`});}
            stage('complete','prepare',prepared?(prepared.hit?'cache quente':'cache frio'):'sem cache');
            status('Instalando lobby…');
            if (prepared) client.load_prepared_lobby(prepared.scene);
            else client.load_lobby();
            postMessage({type:'loadingStats',cacheHit:prepared?.hit || false,prepareMs:prepared?.prepareMs,stages:JSON.parse(client.loading_stats())});
        }
    } else {
        client.set_environment(await (await response('./data/timeenv.txt')).text(), 12);
        const files = await (await response('./data/manifest.json')).json();
        for (const [index, name] of files.entries()) {
            status(`Carregando Giran… ${index + 1}/${files.length}`);
            const bytes = await (await response(`./data/${name}`)).arrayBuffer();
            const header = new DataView(bytes);
            client.load_scene(name, new Uint8Array(bytes), header.getBigUint64(12, true), header.getBigUint64(20, true));
        }
    }
    postMessage({ type: 'ready', localFiles: Boolean(data.files) });
}

let depth=0;
function guarded(target) {
    return new Proxy(target,{get(object,name) {
        const value=object[name];
        if (typeof value!=='function') return value;
        return (...args)=>{
            depth++;
            try {return value.apply(object,args);}
            finally {depth--;}
        };
    }});
}

function beginFrames(initialPreferences) {
    let preferences = initialPreferences;
    let settingsRevision = client.settings_revision();
    let measuredAt=performance.now(),frames=0,cpu=0,worst=0;
    const frame = time => {
        if (failed) return;
        if (depth>0) {requestAnimationFrame(frame);return;}
        try {
            const frameStarted=performance.now();
            client.frame(time / 1000);
            const key=client.game_cursor_request();
            if (key && key!==cursorRequest) {
                cursorRequest=key;
                try {
                    const bytes=key.startsWith('@') ? client.game_cursor_texture(key.slice(1)) : undefined;
                    postMessage({type:'cursorRequest',key,bytes});
                } catch (error) {console.warn('Cursor:',error.message || error);}
            }
            const duration=performance.now()-frameStarted;
            cpu+=duration;worst=Math.max(worst,duration);frames++;
            const now=performance.now();
            if (now-measuredAt>=1000) {
                postMessage({type:'performance',fps:frames*1000/(now-measuredAt),cpu:cpu/frames,worst,...JSON.parse(client.performance_stats())});
                measuredAt=now;frames=0;cpu=0;worst=0;
            }
            const audio=client.audio_commands().map(([metadata,bytes])=>[JSON.parse(metadata),bytes]);
            if (audio.length) postMessage({type:'audio',commands:audio},audio.flatMap(([,bytes])=>bytes?[bytes.buffer]:[]));
            try { lobbyActions(); flush(); } catch (error) { disconnected(error); }
            const current = client.preferences();
            if (current !== preferences) { preferences = current; postMessage({type:'preferences',text:current}); }
            const revision = client.settings_revision();
            if (revision !== settingsRevision) {settingsRevision=revision;postMessage({type:'settings',text:client.settings()});}
            requestAnimationFrame(frame);
        } catch (error) { failure(error); }
    };
    requestAnimationFrame(frame);
}

onmessage = ({ data }) => {
    if (failed) return;
    if (data.type === 'start') { start(data).catch(failure); return; }
    if (!client) return;
    if (data.type === 'refresh') {refreshFile(data.name,data.file);return;}
    if (data.type === 'refreshAll') {
        if (data.files) {
            client.mount_files(data.files);
            mountedFiles=data.files;
            preparation=new Preparation(mountedFiles,...preparationArgs);
        }
        const waiter=refreshWaiter;refreshWaiter=null;
        waiter?.(Boolean(data.files));
        return;
    }
    if (loading && !['resize','audioAvailable','audioEnded'].includes(data.type)) return;
    try {
        if (data.type === 'audioAvailable') client.audio_available(data.value);
        else if (data.type === 'audioEnded') client.audio_ended(data.id);
        else if (data.type === 'statsRequest') postMessage({type:'remoteStats',stats:JSON.parse(client.remote_stats())});
        else if (data.type === 'upgrade') client.replace_file(data.name,data.file);
        else if (data.type === 'openedRequest') postMessage({type:'opened',paths:[...new Set([...JSON.parse(client.opened_paths()),...(preparation?.paths || [])])]});
        else if (data.type === 'resize') {
            canvas.width = data.width;
            canvas.height = data.height;
            client.resize(data.width, data.height);
        } else if (data.type === 'look') client.look(data.dx, data.dy);
        else if (data.type === 'movement') client.movement(...data.values);
        else if (data.type === 'pointer') client.pointer(...data.values);
        else if (data.type === 'wheel') client.wheel(...data.values);
        else if (data.type === 'keyboard') client.keyboard(...data.values);
        else if (data.type === 'paste') client.paste(data.text);
        else if (data.type === 'focus') client.focus(data.focused);
        else if (data.type === 'login') {
            const url = new URL(data.gateway);
            if (!['ws:','wss:'].includes(url.protocol) || url.username || url.password || url.search || url.hash) throw new Error('Endereço do gateway inválido');
            gateway = url.href.replace(/\/$/, '');
            client.login_start(data.account,data.password);
            data.password = '';
            status('Conectando ao servidor…');
            connect(false);
        } else if (data.type === 'previewCharacter') client.preview_character(data.slot);
        else if (data.type === 'server') send(client.choose_server(data.id));
        else if (data.type === 'character') {client.choose_character(data.slot);flush();status('Entrando no mundo…');}
    } catch (error) { failure(error); }
};
