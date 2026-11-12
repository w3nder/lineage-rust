import {connect} from 'cloudflare:sockets';
import {MAX_MESSAGE,checkRequest} from './relay.mjs';
import {sessionOf,loginRedirect,diagnose} from './session.mjs';

const CONNECT_TIMEOUT=10000;
const reasons={403:'origem ou rota recusada',404:'não encontrado',426:'Upgrade necessário',500:'destino não configurado',401:'entre com o Discord'};

function bridge(server,socket) {
    const writer=socket.writable.getWriter();
    let queue=Promise.resolve();
    let closed=false;
    const close=(code,reason)=>{
        if (closed) return;
        closed=true;
        try {server.close(code,reason);} catch {}
        try {socket.close();} catch {}
    };
    server.addEventListener('message',event=>{
        if (typeof event.data==='string' || event.data.byteLength>MAX_MESSAGE) {close(1009,'mensagem recusada');return;}
        const bytes=new Uint8Array(event.data);
        queue=queue.then(()=>writer.write(bytes)).catch(()=>close(1011,'destino indisponível'));
    });
    server.addEventListener('close',()=>close(1000,''));
    server.addEventListener('error',()=>close(1011,'erro'));
    (async()=>{
        const reader=socket.readable.getReader();
        try {
            for (;;) {
                const {value,done}=await reader.read();
                if (done) break;
                server.send(value);
            }
            close(1000,'destino encerrou');
        } catch {close(1011,'destino indisponível');}
    })();
}

export default {
    async fetch(request,env) {
        const path=new URL(request.url).pathname;
        if (path==='/auth-status') return Response.json({estado:await diagnose(request,env)},{headers:{'Cache-Control':'no-store'}});
        const required=env.REQUIRE_LOGIN!=='false';
        if (required && !await sessionOf(request,env)) {
            if (path.startsWith('/gw/')) return new Response(reasons[401],{status:401});
            return loginRedirect(env);
        }
        if (!path.startsWith('/gw/')) return env.ASSETS.fetch(request);
        const checked=checkRequest(request,env);
        if (!checked.target) return new Response(reasons[checked.status],{status:checked.status});
        const socket=connect(checked.target);
        const timer=new Promise((_,reject)=>setTimeout(()=>reject(new Error('tempo esgotado')),CONNECT_TIMEOUT));
        try {await Promise.race([socket.opened,timer]);}
        catch {
            try {socket.close();} catch {}
            return new Response('destino indisponível',{status:502});
        }
        const pair=new WebSocketPair();
        const [client,server]=Object.values(pair);
        server.binaryType='arraybuffer';
        server.accept();
        bridge(server,socket);
        return new Response(null,{status:101,webSocket:client});
    },
};
