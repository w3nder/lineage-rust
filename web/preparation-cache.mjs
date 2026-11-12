const compare=(a,b)=>a<b?-1:a>b?1:0;

export function manifest(files) {
    return files.map(([path,file])=>[path.slice(path.indexOf('/')+1).replaceAll('\\','/').toLowerCase(),file.size,file.lastModified]).sort((a,b)=>compare(a[0],b[0]));
}

export async function digest(value) {
    const bytes=typeof value==='string'?new TextEncoder().encode(value):value;
    return [...new Uint8Array(await crypto.subtle.digest('SHA-256',bytes))].map(v=>v.toString(16).padStart(2,'0')).join('');
}

export async function cacheKey(files,settings,tile,build) {
    const options=Object.entries(JSON.parse(settings)).sort(([a],[b])=>compare(a,b));
    return digest(JSON.stringify({format:1,build,tile,options,files:manifest(files)}));
}

export function validMetadata(metadata,scene,trace) {
    return metadata?.version===1 && typeof metadata.dependencies==='string' &&
        Array.isArray(metadata.paths) && metadata.paths.every(path=>typeof path==='string' && path.startsWith('/client/')) &&
        /^[a-f0-9]{16}$/.test(metadata.sceneHash) && /^[a-f0-9]{16}$/.test(metadata.traceHash) &&
        metadata.sceneSize===scene.size && metadata.traceSize===trace.size;
}

export async function discardUnpublished(root,exclusive) {
    if (!exclusive) return 0;
    let removed=0;
    for await (const [name,directory] of root.entries()) {
        if (directory.kind!=='directory' || !/^[a-f0-9]{64}$/.test(name)) continue;
        try {await directory.getFileHandle('metadata.json');}
        catch (error) {
            if (error?.name!=='NotFoundError') continue;
            try {await root.removeEntry(name,{recursive:true});removed++;} catch {}
        }
    }
    return removed;
}

export class Preparation {
    constructor(files,version,report=()=>{},profile=()=>{},trusted=false) {
        this.files=files;
        this.trusted=trusted;
        this.version=version;
        this.report=report;
        this.profile=profile;
        this.worker=null;
        this.reject=null;
        this.paths=new Set();
    }
    cancel() {
        this.worker?.terminate();
        this.worker=null;
        this.reject?.(new DOMException('Preparação cancelada','AbortError'));
        this.reject=null;
    }
    run(tile,settings) {
        this.cancel();
        return new Promise((resolve,reject)=>{
            const url=new URL('./prepare-worker.js',import.meta.url);
            url.searchParams.set('v',this.version);
            const worker=new Worker(url,{type:'module'});
            this.worker=worker;
            this.reject=reject;
            const finish=(error,result)=>{
                if (this.worker!==worker) return;
                worker.terminate();
                this.worker=null;
                this.reject=null;
                if (error) reject(error); else resolve(result);
            };
            worker.onmessage=({data})=>{
                if (this.worker!==worker) return;
                if (data.type==='status') this.report(data.text);
                else if (data.type==='profile') this.profile(data);
                else if (data.type==='prepared') {for (const path of data.paths || []) this.paths.add(path);finish(null,data);}
                else if (data.type==='error') finish(new Error(data.text));
            };
            worker.onerror=event=>{event.preventDefault();finish(new Error(event.message || 'Falha no preparador'));};
            worker.postMessage({files:this.files,settings,tile,trusted:this.trusted});
        });
    }
}
