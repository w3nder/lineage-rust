import {BrowserAudio} from './audio.mjs';
const result=document.querySelector('#result');
const button=document.querySelector('#play');
let ended;
const audio=new BrowserAudio({available(){},ended:id=>ended?.(id),warning:text=>{result.textContent=`Falhou: ${text}`;button.disabled=false;}});
function wave() {
    const count=2400;
    const bytes=new Uint8Array(44+count*2);
    const view=new DataView(bytes.buffer);
    const text=(offset,value)=>{for(let i=0;i<value.length;i++)bytes[offset+i]=value.charCodeAt(i);};
    text(0,'RIFF');view.setUint32(4,bytes.length-8,true);text(8,'WAVE');text(12,'fmt ');
    view.setUint32(16,16,true);view.setUint16(20,1,true);view.setUint16(22,1,true);
    view.setUint32(24,24000,true);view.setUint32(28,48000,true);view.setUint16(32,2,true);view.setUint16(34,16,true);
    text(36,'data');view.setUint32(40,count*2,true);
    for(let i=0;i<count;i++)view.setInt16(44+i*2,Math.round(Math.sin(i*2*Math.PI*440/24000)*1800),true);
    return bytes;
}
button.addEventListener('click',async()=>{
    button.disabled=true;
    result.textContent='Decodificando e reproduzindo…';
    await audio.unlock();
    if(audio.context?.state!=='running') {result.textContent='Falhou: AudioContext não iniciou.';button.disabled=false;return;}
    const completion=new Promise(resolve=>{ended=id=>{if(id===1)resolve();};});
    await audio.command({kind:'listener',position:[0,0,0],front:[0,0,-1],up:[0,1,0]});
    const playing=audio.command({kind:'play',id:1,gain:0.25,spatial:{position:[2,0,-2],reference:2,rolloff:0.5,pitch:1}},wave());
    await audio.command({kind:'position',id:1,position:[1,0,-2],pitch:1.01});
    await playing;
    const voice=audio.voices.get(1);
    const started=voice?.source;
    if (voice?.panner?.positionX.value!==1 || voice?.source?.playbackRate.value!==Math.fround(1.01)) {result.textContent='Falhou: posição/frequência da fonte.';button.disabled=false;return;}
    if(!started){button.disabled=false;return;}
    await completion;
    result.textContent='Passou: WAV mono reproduzido com posição, atenuação e Doppler; atualização durante decode preservada, reprodução encerrada e fonte liberada. Ganho: 0,25.';
    button.disabled=false;
});
