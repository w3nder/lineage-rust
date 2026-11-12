import {fetchBlob} from './prefetch.mjs';
import {relative,stamp} from './remote.mjs';

export const PRIORITY=['sounds','animations','voice','music','system','systextures','textures','staticmeshes','maps'];

const rank=name=>{
    const folder=relative(name).split('/')[0];
    const index=PRIORITY.indexOf(folder);
    return index<0?PRIORITY.length:index;
};

export function upgradeOrder(files) {
    return files.filter(([,value])=>typeof value.url==='string')
        .sort((a,b)=>rank(a[0])-rank(b[0]) || a[1].size-b[1].size);
}

export async function upgrade(files,{cache,fetcher=fetch,concurrency=3,budgetBytes=Infinity,send,progress=()=>{}}) {
    const queue=upgradeOrder(files);
    let next=0;
    let bytes=0;
    let done=0;
    let stopped=false;
    async function worker() {
        while (!stopped && next<queue.length) {
            const [name,entry]=queue[next++];
            if (bytes+entry.size>budgetBytes) {stopped=true;break;}
            try {
                const blob=await fetchBlob(entry,cache,fetcher);
                send(name.slice(name.indexOf('/')+1),new File([blob],name.slice(name.lastIndexOf('/')+1),{lastModified:stamp(entry.sha256)}));
                bytes+=entry.size;
                done++;
                progress({files:done,total:queue.length,bytes});
            } catch {}
        }
    }
    await Promise.all(Array.from({length:concurrency},worker));
    return {files:done,bytes,total:queue.length};
}
