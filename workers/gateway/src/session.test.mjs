import test from 'node:test';
import assert from 'node:assert/strict';
import {cookieValue,openSession,sessionOf,diagnose,COOKIE} from './session.mjs';

const secret='segredo-de-teste';
const encoder=new TextEncoder();
const base64Url=bytes=>Buffer.from(bytes).toString('base64url');

async function seal(payload,key=secret,seconds=3600) {
    const body=base64Url(encoder.encode(JSON.stringify({...payload,exp:Math.floor(Date.now()/1000)+seconds})));
    const hmac=await crypto.subtle.importKey('raw',encoder.encode(key),{name:'HMAC',hash:'SHA-256'},false,['sign']);
    const signature=await crypto.subtle.sign('HMAC',hmac,encoder.encode(body));
    return `${body}.${base64Url(new Uint8Array(signature))}`;
}

test('o cookie é lido pelo nome exato',()=>{
    assert.equal(cookieValue('a=1; unk_play=xyz; b=2',COOKIE),'xyz');
    assert.equal(cookieValue('xunk_play=1',COOKIE),undefined);
    assert.equal(cookieValue(null,COOKIE),undefined);
});

test('sessão assinada com o mesmo segredo abre',async()=>{
    const session=await openSession(await seal({id:'42',name:'Ana'}),secret);
    assert.equal(session.id,'42');
    assert.equal(session.name,'Ana');
});

test('segredo errado, adulterada, vencida e malformada são recusadas',async()=>{
    const token=await seal({id:'42'});
    assert.equal(await openSession(token,'outro'),null);
    const [body,signature]=token.split('.');
    const forged=base64Url(encoder.encode(JSON.stringify({id:'1',exp:9999999999})));
    assert.equal(await openSession(`${forged}.${signature}`,secret),null);
    assert.equal(await openSession(await seal({id:'42'},secret,-10),secret),null);
    assert.equal(await openSession(await seal({name:'sem id'}),secret),null);
    for (const bad of ['',undefined,'x','a.b.c',`${body}.`]) assert.equal(await openSession(bad,secret),null);
    assert.equal(await openSession(token,''),null);
});

test('a requisição precisa trazer o cookie válido',async()=>{
    const token=await seal({id:'42'});
    const withCookie=new Request('https://play.test/',{headers:{Cookie:`${COOKIE}=${token}`}});
    assert.equal((await sessionOf(withCookie,{SESSION_SECRET:secret})).id,'42');
    assert.equal(await sessionOf(new Request('https://play.test/'),{SESSION_SECRET:secret}),null);
    assert.equal(await sessionOf(withCookie,{}),null);
});

test('o diagnóstico diz se falta cookie, segredo ou se a assinatura não confere',async()=>{
    const token=await seal({id:'42'});
    const request=value=>new Request('https://play.test/',{headers:value?{Cookie:`${COOKIE}=${value}`}:{}});
    assert.equal(await diagnose(request(token),{SESSION_SECRET:secret}),'ok');
    assert.equal(await diagnose(request(),{SESSION_SECRET:secret}),'sem-cookie');
    assert.equal(await diagnose(request(token),{SESSION_SECRET:'outro'}),'invalido');
    assert.equal(await diagnose(request(token),{}),'sem-segredo');
});
