const database='killer.local-client';

async function storage(mode,operation) {
    const db=await new Promise((resolve,reject)=>{
        const request=indexedDB.open(database,1);
        request.onupgradeneeded=()=>request.result.createObjectStore('folders');
        request.onsuccess=()=>resolve(request.result);
        request.onerror=()=>reject(request.error);
    });
    try {
        return await new Promise((resolve,reject)=>{
            const transaction=db.transaction('folders',mode);
            const request=operation(transaction.objectStore('folders'));
            transaction.oncomplete=()=>resolve(request.result);
            transaction.onerror=()=>reject(transaction.error);
            transaction.onabort=()=>reject(transaction.error || new Error('Armazenamento interrompido'));
        });
    } finally {db.close();}
}

export const savedFolder=()=>storage('readonly',store=>store.get('client'));
export const rememberFolder=handle=>storage('readwrite',store=>store.put(handle,'client'));

export async function allowFolder(handle) {
    if (await handle.queryPermission({mode:'read'})==='granted') return true;
    return await handle.requestPermission({mode:'read'})==='granted';
}

export async function folderFiles(handle,progress=()=>{}) {
    const files=[];
    let batch=[];
    async function flush() {
        const entries=await Promise.all(batch.map(async([name,entry])=>[name,await entry.getFile()]));
        files.push(...entries);
        batch=[];
        if (files.length%256===0) progress(files.length);
    }
    async function visit(directory,path) {
        for await (const entry of directory.values()) {
            const name=`${path}/${entry.name}`;
            if (entry.kind==='directory') await visit(entry,name);
            else {
                batch.push([name,entry]);
                if (batch.length===8) await flush();
            }
        }
    }
    await visit(handle,handle.name);
    if (batch.length) await flush();
    progress(files.length);
    const root=`${handle.name}/`.toLowerCase();
    if (!files.some(([name])=>name.toLowerCase().startsWith(`${root}system/`)) ||
        !files.some(([name])=>name.toLowerCase()===`${root}maps/lobby.unr`)) {
        throw new Error('Escolha a pasta completa do cliente C4, que contém system e maps/lobby.unr.');
    }
    return files;
}

export async function folderFile(handle,path) {
    let current=handle;
    const parts=path.split('/').filter(Boolean);
    for (const [index,part] of parts.entries()) {
        let found=null;
        for await (const entry of current.values()) {
            if (entry.name.toLowerCase()===part.toLowerCase()) {found=entry;break;}
        }
        if (!found || (index<parts.length-1)!==(found.kind==='directory')) return null;
        current=found;
    }
    return current.kind==='file'?current.getFile():null;
}

export const staleError=error=>error?.name==='NotReadableError' || /state had changed|NotReadable|could not be read/i.test(String(error?.message ?? error));
