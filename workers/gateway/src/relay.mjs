export const MAX_MESSAGE=65536;
export const ROUTES={'/gw/login':'LOGIN_TARGET','/gw/game':'GAME_TARGET'};

export function parseTarget(value) {
    const match=/^([A-Za-z0-9.-]+):(\d{1,5})$/.exec(value || '');
    if (!match) throw new Error('destino inválido');
    const port=Number(match[2]);
    if (port<1 || port>65535) throw new Error('destino inválido');
    return {hostname:match[1],port};
}

export function originAllowed(origin,allowed) {
    if (!origin) return false;
    return (allowed || '').split(',').some(item=>item.trim()===origin);
}

export function checkRequest(request,env) {
    const url=new URL(request.url);
    const key=ROUTES[url.pathname];
    if (!key) return {status:404};
    if (request.headers.get('Upgrade')?.toLowerCase()!=='websocket') return {status:426};
    if (url.search || !originAllowed(request.headers.get('Origin'),env.ALLOWED_ORIGINS)) return {status:403};
    try {return {target:parseTarget(env[key])};}
    catch {return {status:500};}
}
