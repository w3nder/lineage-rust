export class BrowserAudio {
    constructor({available,ended,warning}) {
        this.available=available;
        this.ended=ended;
        this.warning=warning;
        this.context=null;
        this.voices=new Map();
    }
    bindGestures(target) {
        const unlock=()=>{
            if (this.context?.state!=='running') void this.unlock();
        };
        for (const type of ['pointerdown','pointerup','keydown','click']) target.addEventListener(type,unlock,{capture:true});
        return ()=>{
            for (const type of ['pointerdown','pointerup','keydown','click']) target.removeEventListener(type,unlock,{capture:true});
        };
    }
    async unlock() {
        try {
            if (!this.context) {
                this.context=new AudioContext();
                this.context.onstatechange=()=>{
                    const running=this.context.state==='running';
                    if (!running) for (const id of this.voices.keys()) this.stop(id);
                    this.available(running);
                };
            }
            await this.context.resume();
            this.available(this.context.state==='running');
        } catch {this.available(false);this.warning('Não foi possível iniciar o áudio neste navegador.');}
    }
    finish(id,voice) {
        if (this.voices.get(id)!==voice) return;
        this.voices.delete(id);
        if (voice.media) {
            voice.media.onended=null;
            voice.media.onerror=null;
            voice.media.pause();
            voice.media.removeAttribute('src');
            voice.media.load();
            URL.revokeObjectURL(voice.url);
        }
        voice.source?.disconnect();
        voice.gainNode?.disconnect();
        voice.panner?.disconnect();
        this.ended(id);
    }
    stop(id) {
        const voice=this.voices.get(id);
        if (!voice) return;
        voice.source?.stop?.();
        this.finish(id,voice);
    }
    position(voice) {
        if (!voice.panner) return;
        const [x,y,z]=voice.spatial.position;
        voice.panner.positionX.value=x;
        voice.panner.positionY.value=y;
        voice.panner.positionZ.value=z;
        voice.source.playbackRate.value=voice.spatial.pitch;
    }
    async command(command,encoded) {
        if (command.kind==='listener') {
            if (!this.context) return;
            const listener=this.context.listener;
            for (const [prefix,values] of [['position',command.position],['forward',command.front],['up',command.up]]) {
                for (let i=0;i<3;i++) listener[`${prefix}${'XYZ'[i]}`].value=values[i];
            }
            return;
        }
        if (command.kind==='position') {
            const voice=this.voices.get(command.id);
            if (voice?.spatial) {
                voice.spatial.position=command.position;
                voice.spatial.pitch=command.pitch;
                this.position(voice);
            }
            return;
        }
        if (command.kind==='stop') {this.stop(command.id);return;}
        if (command.kind==='gain') {
            const voice=this.voices.get(command.id);
            if (voice) {
                voice.gain=command.value;
                if (voice.gainNode) voice.gainNode.gain.value=voice.gain;
            }
            return;
        }
        if (command.kind!=='play') return;
        this.stop(command.id);
        const context=this.context;
        if (!context || context.state!=='running') {this.ended(command.id);return;}
        const voice={gain:command.gain,spatial:command.spatial};
        this.voices.set(command.id,voice);
        try {
            if (command.stream) {
                voice.url=URL.createObjectURL(new Blob([encoded],{type:'audio/ogg'}));
                voice.media=new Audio(voice.url);
                voice.media.loop=Boolean(command.looping);
                voice.source=context.createMediaElementSource(voice.media);
                voice.gainNode=context.createGain();
                voice.gainNode.gain.value=voice.gain;
                voice.source.connect(voice.gainNode).connect(context.destination);
                voice.media.onended=()=>this.finish(command.id,voice);
                voice.media.onerror=()=>{
                    if (this.voices.get(command.id)!==voice) return;
                    this.finish(command.id,voice);
                    this.warning('Não foi possível reproduzir a música do cliente.');
                };
                await voice.media.play();
                return;
            }
            const buffer=await context.decodeAudioData(encoded.buffer.slice(encoded.byteOffset,encoded.byteOffset+encoded.byteLength));
            if (this.voices.get(command.id)!==voice) return;
            voice.source=context.createBufferSource();
            voice.gainNode=context.createGain();
            voice.gainNode.gain.value=voice.gain;
            voice.source.buffer=buffer;
            voice.source.loop=Boolean(command.looping);
            if (voice.spatial) {
                if (buffer.numberOfChannels>2) throw new Error('USound requer um ou dois canais');
                if (buffer.numberOfChannels===1) {
                    voice.panner=context.createPanner();
                    voice.panner.panningModel='equalpower';
                    voice.panner.distanceModel='inverse';
                    voice.panner.refDistance=voice.spatial.reference;
                    voice.panner.maxDistance=3.4028234663852886e38;
                    voice.panner.rolloffFactor=voice.spatial.rolloff;
                    this.position(voice);
                }
            }
            if (voice.panner) voice.source.connect(voice.panner).connect(voice.gainNode);
            else voice.source.connect(voice.gainNode);
            voice.gainNode.connect(context.destination);
            voice.source.onended=()=>this.finish(command.id,voice);
            voice.source.start();
        } catch {
            if (this.voices.get(command.id)===voice) {
                this.finish(command.id,voice);
                this.warning('Não foi possível reproduzir um som do cliente.');
            }
        }
    }
}
