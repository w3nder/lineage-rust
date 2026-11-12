export function encodePath(path) {
    return path.split('/').map(encodeURIComponent).join('/');
}

export const stamp=sha256=>parseInt(sha256.slice(0,13),16) || 0;

export function relative(name) {
    return name.slice(name.indexOf('/')+1).toLowerCase();
}

export function remoteEntries(base,manifest,root='client') {
    const origin=base.replace(/\/+$/,'');
    if (!Array.isArray(manifest?.files)) throw new Error('Manifesto do cliente inválido.');
    return manifest.files.map(({path,size,sha256})=>{
        if (typeof path!=='string' || !Number.isInteger(size) || size<0 || typeof sha256!=='string') throw new Error('Manifesto do cliente inválido.');
        if (path.startsWith('/') || path.split('/').includes('..')) throw new Error('Manifesto do cliente inválido.');
        return [`${root}/${path}`,{url:`${origin}/${encodePath(path)}`,size,lastModified:stamp(sha256),sha256}];
    });
}

export async function loadRemoteClient(base,fetcher=fetch) {
    const response=await fetcher(`${base.replace(/\/+$/,'')}/manifest.json`,{cache:'no-cache'});
    if (!response.ok) throw new Error(`Não foi possível ler a lista de arquivos do cliente (HTTP ${response.status}).`);
    const entries=remoteEntries(base,await response.json());
    if (!entries.some(([name])=>name.toLowerCase().startsWith('client/system/')) ||
        !entries.some(([name])=>name.toLowerCase()==='client/maps/lobby.unr')) {
        throw new Error('A lista de arquivos do cliente não contém system e maps/lobby.unr.');
    }
    return entries;
}
