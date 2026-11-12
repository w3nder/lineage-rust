import test from 'node:test';
import assert from 'node:assert/strict';
import {BrowserAudio} from './audio.mjs';

class Context {
    state='running';
    destination={};
    listener=Object.fromEntries(['position','forward','up'].flatMap(prefix=>[...'XYZ'].map(axis=>[prefix+axis,{value:0}])));
    pending=[];
    started=[];
    async resume() {}
    decodeAudioData() {return new Promise((resolve,reject)=>this.pending.push({resolve,reject}));}
    createPanner() {return {positionX:{value:0},positionY:{value:0},positionZ:{value:0},connect(){return this;},disconnect(){}};}
    createGain() {return {gain:{value:1},connect(){return this;},disconnect(){}};}
    createBufferSource() {
        const source={playbackRate:{value:1},connect(){return this;},disconnect(){},stop(){},start:()=>this.started.push(source)};
        return source;
    }
}
function fixture() {
    const ended=[];
    const warnings=[];
    const audio=new BrowserAudio({available(){},ended:id=>ended.push(id),warning:text=>warnings.push(text)});
    audio.context=new Context();
    return {audio,ended,warnings};
}
test('page gestures unlock outside the canvas and retry after suspension',async()=>{
    const {audio}=fixture();
    const target=new EventTarget();
    let resumes=0;
    audio.context.state='suspended';
    audio.context.resume=async()=>{resumes++;audio.context.state='running';};
    const unbind=audio.bindGestures(target);
    target.dispatchEvent(new Event('click'));
    await Promise.resolve();
    assert.equal(resumes,1);
    for (const type of ['pointerdown','pointerup','keydown','click']) target.dispatchEvent(new Event(type));
    assert.equal(resumes,1);
    audio.context.state='suspended';
    target.dispatchEvent(new Event('keydown'));
    await Promise.resolve();
    assert.equal(resumes,2);
    unbind();
    audio.context.state='suspended';
    target.dispatchEvent(new Event('click'));
    assert.equal(resumes,2);
});
test('failed activation can be retried by the next page gesture',async()=>{
    const {audio,warnings}=fixture();
    const target=new EventTarget();
    audio.context.state='suspended';
    audio.context.resume=async()=>{throw new Error('not allowed');};
    const unbind=audio.bindGestures(target);
    target.dispatchEvent(new Event('pointerdown'));
    await Promise.resolve();
    assert.equal(warnings.length,1);
    audio.context.resume=async()=>{audio.context.state='running';};
    target.dispatchEvent(new Event('pointerup'));
    await Promise.resolve();
    assert.equal(audio.context.state,'running');
    unbind();
});
test('stop during decoding never starts the cancelled voice',async()=>{
    const {audio,ended}=fixture();
    const playing=audio.command({kind:'play',id:1,gain:0.25},new Uint8Array([1]));
    await audio.command({kind:'stop',id:1});
    audio.context.pending[0].resolve({});
    await playing;
    assert.equal(audio.context.started.length,0);
    assert.deepEqual(ended,[1]);
    assert.equal(audio.voices.size,0);
});
test('replacement and gain change survive reversed decode completion',async()=>{
    const {audio,ended}=fixture();
    const first=audio.command({kind:'play',id:1,gain:0.25},new Uint8Array([1]));
    const second=audio.command({kind:'play',id:1,gain:0.5},new Uint8Array([2]));
    await audio.command({kind:'gain',id:1,value:0.4});
    audio.context.pending[1].resolve({});
    await second;
    audio.context.pending[0].resolve({});
    await first;
    assert.equal(audio.context.started.length,1);
    assert.equal(audio.voices.get(1).gainNode.gain.value,0.4);
    audio.context.started[0].onended();
    assert.equal(audio.voices.size,0);
    assert.deepEqual(ended,[1,1]);
});
test('decoding failure releases slot and reports once',async()=>{
    const {audio,ended,warnings}=fixture();
    const playing=audio.command({kind:'play',id:8,gain:0.2},new Uint8Array([0]));
    audio.context.pending[0].reject(new Error('invalid wave'));
    await playing;
    assert.deepEqual(ended,[8]);
    assert.equal(warnings.length,1);
    assert.equal(audio.voices.size,0);
});

test('mono keeps position and Doppler changes received during decoding',async()=>{
    const {audio}=fixture();
    await audio.command({kind:'listener',position:[1,2,3],front:[0,0,-1],up:[0,1,0]});
    assert.equal(audio.context.listener.positionY.value,2);
    const playing=audio.command({kind:'play',id:2,gain:0.4,spatial:{position:[0,0,0],reference:2,rolloff:0.5,pitch:1}},new Uint8Array([1]));
    await audio.command({kind:'position',id:2,position:[4,5,6],pitch:1.1});
    audio.context.pending[0].resolve({numberOfChannels:1});
    await playing;
    const voice=audio.voices.get(2);
    assert.deepEqual([voice.panner.positionX.value,voice.panner.positionY.value,voice.panner.positionZ.value],[4,5,6]);
    assert.equal(voice.panner.distanceModel,'inverse');
    assert.equal(voice.panner.refDistance,2);
    assert.equal(voice.panner.rolloffFactor,0.5);
    assert.equal(voice.source.playbackRate.value,1.1);
});
test('stereo preserves channels without OpenAL mono positioning or Doppler',async()=>{
    const {audio}=fixture();
    const playing=audio.command({kind:'play',id:3,gain:0.4,spatial:{position:[4,0,0],reference:2,rolloff:0.5,pitch:1.1}},new Uint8Array([1]));
    audio.context.pending[0].resolve({numberOfChannels:2});
    await playing;
    const voice=audio.voices.get(3);
    assert.equal(voice.panner,undefined);
    assert.equal(voice.source.playbackRate.value,1);
});

test('music streams without PCM decoding and stops pending playback on replacement',async()=>{
    const {audio,ended,warnings}=fixture();
    const OriginalAudio=globalThis.Audio;
    const revoked=[];
    const revoke=URL.revokeObjectURL;
    const media=[];
    URL.revokeObjectURL=url=>{revoked.push(url);revoke(url);};
    globalThis.Audio=class {
        constructor(url){this.src=url;media.push(this);}
        play(){return new Promise((resolve,reject)=>{this.resolve=resolve;this.reject=reject;});}
        pause(){this.paused=true;}
        removeAttribute(){this.src='';}
        load(){}
    };
    audio.context.createMediaElementSource=()=>({connect(){return this;},disconnect(){this.disconnected=true;}});
    try {
        const first=audio.command({kind:'play',id:1,gain:0,stream:true,looping:true},new Uint8Array([1]));
        assert.equal(media[0].loop,true);
        assert.equal(audio.context.pending.length,0);
        await audio.command({kind:'gain',id:1,value:0.8});
        assert.equal(audio.voices.get(1).gainNode.gain.value,0.8);
        const second=audio.command({kind:'play',id:1,gain:0.5,stream:true},new Uint8Array([2]));
        assert.equal(media[0].paused,true);
        assert.equal(media[0].src,'');
        media[0].reject(new Error('play interrupted'));
        media[1].resolve();
        await Promise.all([first,second]);
        assert.equal(warnings.length,0);
        assert.equal(audio.voices.size,1);
        media[1].onended();
        assert.equal(audio.voices.size,0);
        assert.equal(revoked.length,2);
        assert.deepEqual(ended,[1,1]);
        const failed=audio.command({kind:'play',id:2,gain:1,stream:true},new Uint8Array([3]));
        media[2].reject(new Error('unsupported codec'));
        await failed;
        assert.equal(warnings.length,1);
        assert.equal(audio.voices.size,0);
        assert.equal(revoked.length,3);
    } finally {
        globalThis.Audio=OriginalAudio;
        URL.revokeObjectURL=revoke;
    }
});
