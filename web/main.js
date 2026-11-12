import {savedFolder,rememberFolder,allowFolder,folderFiles,folderFile} from './folder.mjs';
import {loadRemoteClient} from './remote.mjs';
import {prefetch,discardStale,openCache,learnPaths,wantedFiles} from './prefetch.mjs';
import {upgrade} from './upgrade.mjs';
import {config} from './config.mjs';
import {BrowserAudio} from './audio.mjs';
import {PointerDelta,CursorBank} from './pointer.mjs';
import {readStyle,withStyle} from './hud-style.mjs';
import {LoadingModel,LoadingView,LOBBY_STAGES,WORLD_STAGES} from './loading.mjs';
const canvas = document.querySelector('#game');
const status = document.querySelector('#status');
const welcome = document.querySelector('#welcome');
const folder = document.querySelector('#folder');
const gpuState = document.querySelector('#gpu-state');
const gatewayInput = document.querySelector('#gateway');
gatewayInput.value = config.gateway;
if (!config.gatewayEditable) document.querySelector('#gateway-field').hidden = true;
const play = document.querySelector('#play');
if (!config.clientUrl) {play.hidden = true;document.querySelector('#intro').textContent = 'Selecione sua pasta do cliente C4. Os arquivos são lidos no seu computador e não são enviados para um servidor.';}
if (gpuState) { const ok = Boolean(navigator.gpu); gpuState.dataset.state = ok ? 'ok' : 'missing'; gpuState.querySelector('b').textContent = ok ? 'WebGPU disponível' : 'WebGPU indisponível'; }
const styleSelect = document.querySelector('#hud-style');
const styleOptions = [...document.querySelectorAll('#hud-choice .option')];
const showStyle = () => styleOptions.forEach(option => option.setAttribute('aria-pressed', String(option.dataset.style === styleSelect.value)));
try { styleSelect.value = readStyle(localStorage.getItem('killer.hud')); } catch {}
showStyle();
styleOptions.forEach(option => option.addEventListener('click', () => { styleSelect.value = option.dataset.style; showStyle(); }));
const loadingView=new LoadingView(document.querySelector('#loading'));
let loading=null;
let loadingTimer=0;
function showLoading(stages) {
    loading=new LoadingModel(stages);
    loadingView.noteText='';
    loadingView.show(loading);
    clearInterval(loadingTimer);
    loadingTimer=setInterval(()=>loadingView.render(),250);
}
function stageEvent(op,key,detail) {
    if (!loading && key.startsWith('world-')) showLoading(WORLD_STAGES);
    if (!loading) return;
    if (op==='begin') loading.begin(key,detail); else loading.complete(key,detail);
    if (!detail || op==='complete') loadingView.noteText='';
    loadingView.render();
}
function hideLoading(summary=true) {
    clearInterval(loadingTimer);
    if (loading) {
        loading.finish();
        if (summary) console.info(loading.summary());
        loading=null;
    }
    loadingView.hide();
}
const workerUrl=new URL('./worker.js',import.meta.url);
workerUrl.searchParams.set('v',config.build || String(Date.now()));
const worker = new Worker(workerUrl, { type: 'module' });
let started = false;
let ready = false;
let engineReady=false;
let worldVisible=false;
const cursors=new CursorBank(image=>{
    const surface=document.createElement('canvas');
    surface.width=image.width;surface.height=image.height;
    surface.getContext('2d').putImageData(new ImageData(image.rgba,image.width,image.height),0,0);
    return `url("${surface.toDataURL()}") ${image.hotspot[0]} ${image.hotspot[1]}, default`;
});
const mode=new URLSearchParams(location.search).get('mode');
const notice = document.querySelector('#notice');
function warn(text) { notice.querySelector('span').textContent=text; notice.hidden=false; }
const audio=new BrowserAudio({available:value=>worker.postMessage({type:'audioAvailable',value}),ended:id=>worker.postMessage({type:'audioEnded',id}),warning:warn});
audio.bindGestures(document);
notice.querySelector('button').addEventListener('click',()=>{notice.hidden=true;canvas.focus();});

function start(files,trusted=false) {
    if (started) return;
    if (!navigator.gpu || !canvas.transferControlToOffscreen) {
        status.hidden = false;
        status.textContent = 'Este navegador precisa oferecer WebGPU e OffscreenCanvas. Use Chrome ou Edge atualizado em HTTPS ou localhost.';
        return;
    }
    started = true;
    welcome.hidden = true;
    status.hidden = true;
    if (!loading) showLoading(LOBBY_STAGES);
    stageEvent('begin','engine');
    canvas.width = Math.max(1, Math.round(innerWidth * devicePixelRatio));
    canvas.height = Math.max(1, Math.round(innerHeight * devicePixelRatio));
    const surface = canvas.transferControlToOffscreen();
    let preferences = '';
    let settings = '{}';
    try { preferences = localStorage.getItem('killer.hud') || ''; settings = localStorage.getItem('killer.settings') || '{}'; } catch {warn('Não foi possível ler as preferências salvas neste navegador.');}
    preferences = withStyle(preferences, document.querySelector('#hud-style').value);
    try { localStorage.setItem('killer.hud', preferences); } catch {}
    worker.postMessage({ type: 'start', canvas: surface, files, trusted, mode, scale: devicePixelRatio, preferences, settings, gateway:document.querySelector('#gateway').value }, [surface]);
}

let remoteStats=null;
let upgraded=null;
let pendingUpgrade=null;
let statsTimer=0;
let openedTimer=0;
function requestOpened(delay) {
    clearTimeout(openedTimer);
    openedTimer=setTimeout(()=>worker.postMessage({type:'openedRequest'}),delay);
}
async function refreshStale({paths=[],all=false}) {
    if (!openedHandle) {
        if (all) worker.postMessage({type:'refreshAll',files:null});
        warn('Arquivos do cliente mudaram no disco depois de abertos. Recarregue a página e escolha a pasta de novo.');
        return;
    }
    try {
        if (all) {worker.postMessage({type:'refreshAll',files:await folderFiles(openedHandle)});return;}
        for (const path of paths) {
            const file=await folderFile(openedHandle,path);
            if (file) worker.postMessage({type:'refresh',name:path,file});
        }
    } catch (error) {
        if (all) worker.postMessage({type:'refreshAll',files:null});
        warn(`Não foi possível reler a pasta do cliente: ${error.message}`);
    }
}

const megabytes=bytes=>(bytes/1048576).toFixed(0);
async function preloadList() {
    try {
        const response=await fetch('./preload.json',{cache:'no-cache'});
        const list=response.ok?await response.json():[];
        return Array.isArray(list)?list.filter(path=>typeof path==='string'):[];
    } catch {return [];}
}

worker.addEventListener('message', ({ data }) => {
    if (data.type === 'opened') {
        try {learnPaths(localStorage,data.paths);} catch {}
    } else if (data.type === 'remoteStats') {
        remoteStats=data.stats;
    } else if (data.type === 'performance') {

        if (!worldVisible) {document.querySelector('#performance').hidden=true;return;}
        const panel=document.querySelector('#performance');
        panel.hidden=false;panel.querySelector('summary').textContent=`${Math.round(data.fps)} FPS`;
        const width=panel.offsetWidth,height=panel.offsetHeight;
        const left=innerWidth-width-18;
        const top=Array.from({length:Math.max(0,Math.floor((innerHeight-height-128)/36)+1)},(_,i)=>112+i*36).find(y=>(data.panels||[]).every(([x,py,w,h])=>left+width<=x || left>=x+w || y+height<=py || y>=py+h));
        panel.hidden=top===undefined && !panel.open;
        panel.style.top=`${top??112}px`;
        const names=['Mundo/mapas','Personagens','Carga/efeitos','Interface'];
        panel.querySelector('pre').textContent=`CPU: ${data.cpu.toFixed(1)} ms · pico ${data.worst.toFixed(1)} ms\n${names.map((name,i)=>`${name}: ${data.stages[i].toFixed(1)} ms`).join('\n')}\nRender (CPU): ${data.render.toFixed(1)} ms\nHUD: ${data.hud.toFixed(1)} ms\nEntidades: ${data.entities} · peças: ${data.pieces}\nSIMD: ${data.simd?'ativo':'inativo'}`;
        if (remoteStats) panel.querySelector('pre').textContent+=`\nR2: ${remoteStats.requests} leituras · ${(remoteStats.bytes/1048576).toFixed(0)} MB · ${remoteStats.ms.toFixed(0)} ms${upgraded?` · ${upgraded.files}/${upgraded.total} locais`:''}`;
        if (data.ground) {
            const ground=data.ground;
            const floor=ground.floor===null?'não encontrado':ground.floor.toFixed(1);
            panel.querySelector('pre').textContent+=`\nMapa: ${ground.tile} · ${ground.loaded?'carregado':'aguardando carga'}\nPosição: ${ground.position.map(value=>value.toFixed(1)).join(', ')}\nPés: ${ground.feet.toFixed(1)} · piso: ${floor}\nTriângulos de piso: ${ground.triangles}`;
        }
    } else if (data.type === 'gameCursors') {
        canvas.style.cursor=cursors.load(data.resources);
    } else if (data.type === 'cursorRequest') {
        canvas.style.cursor=cursors.select(data.key,data.bytes);
    } else if (data.type === 'audio') {
        for (const [command,encoded] of data.commands) audio.command(command,encoded);
    } else if (data.type === 'preferences' || data.type === 'settings') {
        try { localStorage.setItem(data.type === 'settings' ? 'killer.settings' : 'killer.hud', data.text); }
        catch {warn('Não foi possível salvar as preferências neste navegador.');}
    } else if (data.type === 'preparationStats') {
        console.info('Preparação',JSON.stringify(data));
    } else if (data.type === 'loadingStats') {
        console.info('Carregamento',JSON.stringify(data));
    } else if (data.type === 'stale') {
        void refreshStale(data);
    } else if (data.type === 'warning') {
        warn(data.text);
    } else if (data.type === 'stage') {
        stageEvent(data.op,data.key,data.detail);
    } else if (data.type === 'loadingImage') {
        loadingView.image(data.width,data.height,data.rgba);
    } else if (data.type === 'engineReady') {
        engineReady=true;
        ready=true;
        status.hidden=true;
        canvas.focus();
    } else if (data.type === 'ready') {
        requestOpened(2500);
        if (pendingUpgrade) {pendingUpgrade();pendingUpgrade=null;}
        clearInterval(statsTimer);
        statsTimer=setInterval(()=>worker.postMessage({type:'statsRequest'}),2000);
        ready=true;
        worldVisible=mode==='demo' || mode==='cache';
        worker.postMessage({type:'audioAvailable',value:audio.context?.state==='running'});
        status.hidden=true;
        hideLoading();
        canvas.focus();
    } else if (data.type === 'lobby' || data.type === 'disconnected') {
        worldVisible=false;
        status.hidden=true;
        hideLoading(data.type==='lobby');
        canvas.focus();
    } else if (data.type === 'entered') {
        requestOpened(6000);
        worldVisible=true;
        status.hidden=true;
        hideLoading();
        canvas.focus();
    } else if (data.type === 'exit') {
        location.reload();
    } else if (data.type === 'status' || data.type === 'error') {
        if (data.type === 'error') hideLoading(false);
        if (loading && data.type==='status') {
            loadingView.noteText=data.text;
            loadingView.render();
            return;
        }
        status.hidden = engineReady && data.type!=='error';
        status.textContent = data.text;
        if (data.type === 'error') {worker.terminate();console.error(data.text);}
    }
});
worker.addEventListener('error', event => {
    worker.terminate();
    hideLoading(false);
    status.hidden = false;
    status.textContent = `Falha no motor: ${event.message}`;
});
folder.addEventListener('change', () => start(Array.from(folder.files, file => [file.webkitRelativePath, file])));
let remembered;
let openedHandle=null;
const reuse=document.querySelector('#reuse');
async function openHandle(handle) {
    const open=document.querySelector('#open');
    open.disabled=true;reuse.disabled=true;
    status.hidden=true;
    showLoading(LOBBY_STAGES);
    stageEvent('begin','folder','0 arquivos');
    try {
        const files=await folderFiles(handle,count=>stageEvent('begin','folder',`${count} arquivos`));
        stageEvent('complete','folder',`${files.length} arquivos`);
        try {await rememberFolder(handle);} catch {warn('A pasta foi aberta, mas não foi possível guardar sua referência.');}
        openedHandle=handle;
        start(files);
    } catch (error) {
        hideLoading(false);
        throw error;
    } finally {open.disabled=false;reuse.disabled=false;}
}
function folderError(error) {
    hideLoading(false);
    if (error.name==='AbortError') {status.hidden=true;return;}
    status.hidden=false;status.textContent=error.message || 'Não foi possível abrir a pasta.';
}
play.addEventListener('click', async () => {
    const open = document.querySelector('#open');
    play.disabled = true;open.disabled = true;reuse.disabled = true;
    status.hidden = true;
    showLoading(LOBBY_STAGES);
    stageEvent('begin','folder','lendo a lista de arquivos');
    try {
        const cacheReady = openCache();
        const [entries, cache, preload] = await Promise.all([loadRemoteClient(config.clientUrl), cacheReady, cacheReady.then(opened => opened ? preloadList() : null)]);
        stageEvent('complete','folder',`${entries.length} arquivos`);
        let files = entries;
        if (cache) {
            const wanted = wantedFiles(preload, localStorage);
            stageEvent('begin','download','preparando');
            const result = await prefetch(entries, wanted, {cache, progress: p => stageEvent('begin','download',`${p.files}/${p.total} arquivos · ${megabytes(p.bytes)}/${megabytes(p.totalBytes)} MB`)});
            files = result.files;
            stageEvent('complete','download',`${wanted.size-result.failures.length} arquivos · ${megabytes(result.bytes)} MB`);
            if (result.failures.length) console.warn('Arquivos que seguem remotos',result.failures);
            discardStale(entries, cache).catch(() => {});
        }
        pendingUpgrade=()=>{
            if (!cache || !config.backgroundMB) return;
            upgrade(files,{cache,budgetBytes:config.backgroundMB*1048576,send:(name,file)=>worker.postMessage({type:'upgrade',name,file}),progress:p=>{upgraded=p;}})
                .then(result=>console.info('Download em segundo plano',JSON.stringify(result)));
        };
        start(files,true);
    } catch (error) {
        hideLoading(false);
        status.hidden = false;
        status.textContent = error.message || 'Não foi possível carregar o cliente.';
    } finally {play.disabled = false;open.disabled = false;reuse.disabled = false;}
});
document.querySelector('#open').addEventListener('click', async () => {
    if (!window.showDirectoryPicker) {folder.click();return;}
    try {await openHandle(await window.showDirectoryPicker({id:'killer-client',mode:'read'}));}
    catch (error) {folderError(error);}
});
reuse.addEventListener('click',async()=>{
    try {
        if (!await allowFolder(remembered)) throw new Error('Acesso à pasta não autorizado. Autorize a leitura ou escolha outra pasta.');
        await openHandle(remembered);
    } catch (error) {folderError(error);}
});
if (window.showDirectoryPicker) savedFolder().then(handle=>{
    if (!handle || started) return;
    remembered=handle;reuse.textContent=`Continuar com ${handle.name}`;reuse.hidden=false;
}).catch(()=>warn('Não foi possível recuperar a pasta anterior. Selecione a pasta novamente.'));
addEventListener('resize', () => {
    if (started) worker.postMessage({ type: 'resize', width: Math.max(1, Math.round(innerWidth * devicePixelRatio)), height: Math.max(1, Math.round(innerHeight * devicePixelRatio)) });
});
const keys = new Set();
const movement = () => worker.postMessage({ type: 'movement', values: [keys.has('KeyW'), keys.has('KeyS'), keys.has('KeyA'), keys.has('KeyD'), keys.has('ShiftLeft') || keys.has('ShiftRight')] });
const keyboard = (event, pressed) => {
    if (!ready || document.activeElement !== canvas) return;
    if (event.altKey || /^F\d+$/.test(event.key) || ['Tab','Backspace',' ','ArrowUp','ArrowDown'].includes(event.key)) event.preventDefault();
    worker.postMessage({type:'keyboard',values:[event.key,event.keyCode,event.altKey,pressed,event.repeat]});
};
addEventListener('keydown', event => { keyboard(event,true); if (ready && document.activeElement === canvas) { keys.add(event.code); movement(); } });
addEventListener('keyup', event => { keyboard(event,false); keys.delete(event.code); if (ready) movement(); });
addEventListener('blur', () => { keys.clear(); if (ready) {movement();worker.postMessage({type:'focus',focused:false});} });
addEventListener('focus', () => {if (ready) worker.postMessage({type:'focus',focused:true});});
canvas.addEventListener('contextmenu', event => event.preventDefault());
const pointerDelta=new PointerDelta();
for (const kind of ['down','move','up','cancel','leave']) canvas.addEventListener(`pointer${kind}`, event => {
    if (kind === 'down') {canvas.focus();canvas.setPointerCapture(event.pointerId);}
    const [dx,dy]=pointerDelta.update(kind,event.offsetX,event.offsetY);
    if (ready) worker.postMessage({type:'pointer',values:[kind,event.offsetX,event.offsetY,dx,dy,event.buttons,event.button]});
});
canvas.addEventListener('wheel', event => {
    event.preventDefault();
    if (ready) worker.postMessage({type:'wheel',values:[Math.round(event.offsetX),Math.round(event.offsetY),-Math.sign(event.deltaY)]});
},{passive:false});
if (mode === 'cache') start(null);

canvas.addEventListener('paste',event=>{
    if (!ready) return;
    event.preventDefault();
    worker.postMessage({type:'paste',text:event.clipboardData?.getData('text/plain') || ''});
});
