import {savedFolder,allowFolder,folderFiles} from './folder.mjs';
const button=document.querySelector('#run');
const status=document.querySelector('#status');
const stop=document.querySelector('#stop');
const settingsSource=document.querySelector('#settings-source');
const settingsSummary=document.querySelector('#settings-summary');
const savedSettings=localStorage.getItem('killer.settings')||'{}';
const scene=document.querySelector('#scene');
const sceneOnly=document.querySelector('#scene-only');
function showSettings() {
    if(settingsSource.value==='original') {
        settingsSummary.textContent='Usará as opções do arquivo original do cliente, incluindo detalhe de textura e distâncias.';
        return;
    }
    try {
        const options=Object.entries(JSON.parse(savedSettings)).find(([name])=>name.toLowerCase()==='option.ini')?.[1];
        if(!options) {settingsSummary.textContent='Sem Option.ini salvo: usará as opções do cliente original.';return;}
        let section='';
        const lines=[];
        for(const line of options.split(/\r?\n/)) {
            const header=line.trim().match(/^\[([^\]]+)\]$/);
            if(header) section=header[1].toLowerCase();
            if(['video','clippingrange'].includes(section)) lines.push(line);
        }
        settingsSummary.textContent=`Opções salvas usadas no teste:\n${lines.join('\n')}`;
    } catch(error) {settingsSummary.textContent=String(error);}
}
settingsSource.addEventListener('change',showSettings);
showSettings();
let activeWorker;
stop.addEventListener('click',()=>{activeWorker?.terminate();activeWorker=null;stop.disabled=true;status.textContent+='Teste interrompido. Recarregue a página para repetir.\n';});
const write=text=>{status.textContent+=`${text}\n`;};
let handle;
try {handle=await savedFolder();} catch(error) {write(String(error));}
button.disabled=!handle;
button.textContent=handle?`Testar com ${handle.name}`:'Abra a pasta no jogo primeiro';
button.addEventListener('click',async()=>{
    scene.value=scene.value.trim().toLowerCase();
    if (!scene.reportValidity()) return;
    button.disabled=true;
    settingsSource.disabled=true;
    scene.disabled=true;
    sceneOnly.disabled=true;
    try {
        if (!await allowFolder(handle)) throw Error('Permissão de leitura não concedida.');
        write('Lendo a pasta local…');
        const files=await folderFiles(handle);
        const worker=new Worker(`./memory-check-worker.js?v=${Date.now()}`,{type:'module'});
        activeWorker=worker;stop.disabled=false;
        worker.onmessage=({data})=>{write(data.text);if(data.done){worker.terminate();activeWorker=null;stop.disabled=true;}};
        worker.onerror=event=>{write(event.message);worker.terminate();activeWorker=null;stop.disabled=true;};
        const canvas=document.querySelector('#canvas').transferControlToOffscreen();
        write(`Fonte das opções: ${settingsSource.selectedOptions[0].textContent}.`);
        worker.postMessage({canvas,files,tile:scene.value,inspect:sceneOnly.checked,preferences:localStorage.getItem('killer.hud')||'',settings:settingsSource.value==='original'?'{}':savedSettings},[canvas]);
    } catch(error) {write(String(error));}
});
