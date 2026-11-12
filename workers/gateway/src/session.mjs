export const COOKIE='unk_play';

const encoder=new TextEncoder();
const decoder=new TextDecoder();

function fromBase64Url(text) {
    try {
        const padded=text.replace(/-/g,'+').replace(/_/g,'/').padEnd(Math.ceil(text.length/4)*4,'=');
        return Uint8Array.from(atob(padded),char=>char.charCodeAt(0));
    } catch {return null;}
}

export function cookieValue(header,name) {
    for (const part of (header||'').split(';')) {
        const index=part.indexOf('=');
        if (index>0 && part.slice(0,index).trim()===name) return part.slice(index+1).trim();
    }
    return undefined;
}

export async function openSession(token,secret,now=Date.now()) {
    if (!token || !secret) return null;
    const [body,signature,extra]=token.split('.');
    if (!body || !signature || extra!==undefined) return null;
    const signatureBytes=fromBase64Url(signature);
    const bodyBytes=fromBase64Url(body);
    if (!signatureBytes || !bodyBytes) return null;
    const key=await crypto.subtle.importKey('raw',encoder.encode(secret),{name:'HMAC',hash:'SHA-256'},false,['verify']);
    if (!await crypto.subtle.verify('HMAC',key,signatureBytes,encoder.encode(body))) return null;
    try {
        const parsed=JSON.parse(decoder.decode(bodyBytes));
        if (typeof parsed.id!=='string' || !(parsed.exp>Math.floor(now/1000))) return null;
        return parsed;
    } catch {return null;}
}

export async function sessionOf(request,env) {
    return openSession(cookieValue(request.headers.get('Cookie'),COOKIE),env.SESSION_SECRET);
}

export function loginRedirect(env) {
    return Response.redirect(env.LOGIN_URL,302);
}

export async function diagnose(request,env) {
    if (!env.SESSION_SECRET) return 'sem-segredo';
    const token=cookieValue(request.headers.get('Cookie'),COOKIE);
    if (!token) return 'sem-cookie';
    return await openSession(token,env.SESSION_SECRET) ? 'ok' : 'invalido';
}
