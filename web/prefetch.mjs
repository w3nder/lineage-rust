import {relative,stamp} from './remote.mjs';

export const CACHE_NAME='killer-client-v1';

const versioned=({url,sha256})=>`${url}?v=${sha256}`;

export async function fetchBlob(entry,cache,fetcher) {
    const key=versioned(entry);
    let response=await cache.match(key);
    if (!response) {
        response=await fetcher(entry.url,{mode:'cors'});
        if (!response.ok) throw new Error(`HTTP ${response.status}`);
        await cache.put(key,response.clone());
    }
    const blob=await response.blob();
    if (blob.size!==entry.size) {
        await cache.delete(key);
        throw new Error('tamanho inesperado');
    }
    return blob;
}

export async function prefetch(entries,wanted,{cache,fetcher=fetch,concurrency=16,progress=()=>{}}) {
    const picked=entries.map((entry,index)=>[index,entry]).filter(([,[name]])=>wanted.has(relative(name)));
    const total=picked.reduce((sum,[,[,entry]])=>sum+entry.size,0);
    const result=entries.slice();
    const failures=[];
    let doneFiles=0;
    let doneBytes=0;
    let next=0;
    async function worker() {
        while (next<picked.length) {
            const [index,[name,entry]]=picked[next++];
            try {
                const blob=await fetchBlob(entry,cache,fetcher);
                result[index]=[name,new File([blob],name.slice(name.lastIndexOf('/')+1),{lastModified:stamp(entry.sha256)})];
            } catch (error) {failures.push([name,String(error?.message || error)]);}
            doneFiles++;
            doneBytes+=entry.size;
            progress({files:doneFiles,total:picked.length,bytes:doneBytes,totalBytes:total});
        }
    }
    progress({files:0,total:picked.length,bytes:0,totalBytes:total});
    await Promise.all(Array.from({length:Math.min(concurrency,picked.length)},worker));
    return {files:result,failures,bytes:total};
}

export async function discardStale(entries,cache) {
    const current=new Map(entries.map(([,entry])=>[entry.url,entry.sha256]));
    const stale=(await cache.keys()).filter(request=>{
        const [url,query]=request.url.split('?v=');
        return current.get(url)!==query;
    });
    await Promise.all(stale.map(request=>cache.delete(request)));
    return stale.length;
}

export async function openCache() {
    try {
        if (!globalThis.caches) return null;
        try {await navigator.storage?.persist?.();} catch {}
        return await caches.open(CACHE_NAME);
    } catch {return null;}
}

export const LEARNED_KEY='killer.learned';

export function learnedPaths(storage) {
    try {
        const value=JSON.parse(storage.getItem(LEARNED_KEY) || '[]');
        return Array.isArray(value)?value.filter(path=>typeof path==='string'):[];
    } catch {return [];}
}

export function learnPaths(storage,paths,limit=20000) {
    const known=new Set(learnedPaths(storage));
    for (const path of paths) {
        if (typeof path!=='string' || !path.startsWith('/client/')) continue;
        known.add(path.slice('/client/'.length));
        if (known.size>=limit) break;
    }
    try {storage.setItem(LEARNED_KEY,JSON.stringify([...known]));} catch {}
    return known.size;
}

export function wantedFiles(preload,storage) {
    return new Set([...(preload || []),...learnedPaths(storage)].map(path=>path.toLowerCase()));
}
