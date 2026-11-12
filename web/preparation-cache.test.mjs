import test from 'node:test';
import assert from 'node:assert/strict';
import {cacheKey,validMetadata,Preparation,discardUnpublished} from './preparation-cache.mjs';

const files=[['L2/maps/a.unr',{size:123,lastModified:4}],['L2/system/option.ini',{size:12,lastModified:5}]];
test('cleanup removes only unpublished cache entries under the preparation lock',async()=>{
    const removed=[];
    const directory=error=>({kind:'directory',async getFileHandle(name){assert.equal(name,'metadata.json');if(error) throw {name:error};return {};}});
    const root={
        async *entries(){
            yield ['a'.repeat(64),directory('NotFoundError')];
            yield ['b'.repeat(64),directory()];
            yield ['c'.repeat(64),directory('NotAllowedError')];
            yield ['unrelated',directory('NotFoundError')];
            yield ['d'.repeat(64),{kind:'file'}];
        },
        async removeEntry(name,options){assert.deepEqual(options,{recursive:true});removed.push(name);},
    };
    assert.equal(await discardUnpublished(root,false),0);
    assert.deepEqual(removed,[]);
    assert.equal(await discardUnpublished(root,true),1);
    assert.deepEqual(removed,['a'.repeat(64)]);
});
test('cache namespace changes with original assets, options and compiled engine',async()=>{
    const original=await cacheKey(files,'{"b":"2","a":"1"}','a','build');
    assert.equal(original,await cacheKey([...files].reverse(),'{"a":"1","b":"2"}','a','build'));
    for (const [source,settings,tile,build] of [
        [[['L2/maps/a.unr',{size:123,lastModified:6}],files[1]],'{"a":"1","b":"2"}','a','build'],
        [files,'{"a":"2","b":"2"}','a','build'],[files,'{"a":"1","b":"2"}','b','build'],
        [files,'{"a":"1","b":"2"}','a','next'],
    ]) assert.notEqual(original,await cacheKey(source,settings,tile,build));
});

test('incomplete cache metadata never qualifies as a hit',()=>{
    const good={version:1,dependencies:'[]',paths:['/client/maps/a.unr'],sceneHash:'0123456789abcdef',traceHash:'0123456789abcdef',sceneSize:10,traceSize:20};
    assert.equal(validMetadata(good,{size:10},{size:20}),true);
    assert.equal(validMetadata(good,{size:9},{size:20}),false);
    assert.equal(validMetadata({...good,traceHash:null},{size:10},{size:20}),false);
    assert.equal(validMetadata({...good,paths:['/other']},{size:10},{size:20}),false);
});

test('replacement terminates the old worker and rejects stale completion',async()=>{
    const workers=[];
    globalThis.Worker=class {
        constructor(){workers.push(this);}
        postMessage(data){this.data=data;}
        terminate(){this.terminated=true;}
    };
    const statuses=[],profiles=[];
    const preparation=new Preparation(files,'1',text=>statuses.push(text),profile=>profiles.push(profile));
    const first=preparation.run('a','{}');
    const cancelled=assert.rejects(first,{name:'AbortError'});
    const second=preparation.run('b','{}');
    workers[0].onmessage({data:{type:'status',text:'old'}});
    workers[0].onmessage({data:{type:'profile',phase:'old'}});
    workers[1].onmessage({data:{type:'status',text:'current'}});
    workers[1].onmessage({data:{type:'profile',phase:'scene'}});
    workers[0].onmessage({data:{type:'prepared',scene:'old'}});
    workers[1].onmessage({data:{type:'prepared',scene:'new'}});
    await cancelled;
    assert.equal((await second).scene,'new');
    assert.equal(workers[0].terminated,true);
    assert.equal(workers[1].terminated,true);
    assert.equal(preparation.worker,null);
    assert.deepEqual(statuses,['current']);
    assert.deepEqual(profiles,[{type:'profile',phase:'scene'}]);
});

test('cache key does not depend on locale ordering of paths or options',async()=>{
    const accents=[['L2/maps/ä.unr',{size:1,lastModified:1}],['L2/maps/z.unr',{size:2,lastModified:2}],['L2/maps/a.unr',{size:3,lastModified:3}]];
    const forward=await cacheKey(accents,'{"é":"1","z":"2","a":"3"}','a','build');
    assert.equal(forward,await cacheKey([...accents].reverse(),'{"a":"3","z":"2","é":"1"}','a','build'));
});
