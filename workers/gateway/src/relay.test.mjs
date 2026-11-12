import test from 'node:test';
import assert from 'node:assert/strict';
import {parseTarget,originAllowed,checkRequest,ROUTES} from './relay.mjs';

const env={LOGIN_TARGET:'host.test:2106',GAME_TARGET:'host.test:7777',ALLOWED_ORIGINS:'https://play.test, http://127.0.0.1:8787'};
const request=(path,headers={})=>new Request(`https://play.test${path}`,{headers:{Upgrade:'websocket',Origin:'https://play.test',...headers}});

test('o destino precisa ser host:porta',()=>{
    assert.deepEqual(parseTarget('server.l2killer.com:2106'),{hostname:'server.l2killer.com',port:2106});
    for (const bad of ['','host','host:0','host:70000','host:80/x','ho st:80',undefined]) assert.throws(()=>parseTarget(bad),bad);
});

test('só as origens da lista passam',()=>{
    assert.ok(originAllowed('https://play.test',env.ALLOWED_ORIGINS));
    assert.ok(originAllowed('http://127.0.0.1:8787',env.ALLOWED_ORIGINS));
    assert.ok(!originAllowed('https://play.test.evil.invalid',env.ALLOWED_ORIGINS));
    assert.ok(!originAllowed(null,env.ALLOWED_ORIGINS));
    assert.ok(!originAllowed('https://play.test',''));
});

test('login e jogo apontam para o destino de cada um',()=>{
    assert.deepEqual(Object.keys(ROUTES),['/gw/login','/gw/game']);
    assert.deepEqual(checkRequest(request('/gw/login'),env),{target:{hostname:'host.test',port:2106}});
    assert.deepEqual(checkRequest(request('/gw/game'),env),{target:{hostname:'host.test',port:7777}});
});

test('rota, upgrade, origem, query e destino inválidos são recusados',()=>{
    assert.equal(checkRequest(request('/gw/other'),env).status,404);
    assert.equal(checkRequest(request('/gw/login',{Upgrade:'h2c'}),env).status,426);
    assert.equal(checkRequest(request('/gw/login',{Origin:'https://evil.invalid'}),env).status,403);
    assert.equal(checkRequest(request('/gw/login?host=x'),env).status,403);
    assert.equal(checkRequest(request('/gw/login'),{...env,LOGIN_TARGET:''}).status,500);
});
