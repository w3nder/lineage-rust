import test from 'node:test';
import assert from 'node:assert/strict';
import {LoadingModel,LOBBY_STAGES,WORLD_STAGES,preparationDetail} from './loading.mjs';

const clock=()=>{let value=0;const now=()=>value;now.advance=ms=>{value+=ms;};return now;};

test('etapas avançam em ordem e cada uma registra a própria duração',()=>{
    const now=clock();
    const model=new LoadingModel(LOBBY_STAGES,now);
    model.begin('folder','12 arquivos');
    now.advance(300);
    model.complete('folder');
    assert.equal(model.stages[0].ms,300);
    assert.equal(model.active.key,'download');
    now.advance(2000);
    model.complete('download','10 arquivos');
    assert.equal(model.active.key,'engine');
    now.advance(500);
    model.complete('engine');
    now.advance(20);
    model.complete('mount');
    now.advance(1400);
    model.complete('prepare','cache frio');
    assert.deepEqual(model.stages.slice(0,5).map(stage=>stage.ms),[300,2000,500,20,1400]);
    assert.equal(model.stages[4].detail,'cache frio');
    assert.equal(model.active.key,'open');
});

test('concluir uma etapa fecha as anteriores que o motor não anunciou',()=>{
    const now=clock();
    const model=new LoadingModel(LOBBY_STAGES,now);
    now.advance(700);
    model.complete('tables');
    assert.ok(model.stages.slice(0,9).every(stage=>stage.state==='done'));
    assert.deepEqual(model.stages.slice(0,8).map(stage=>stage.ms),[0,0,0,0,0,0,0,0]);
    assert.equal(model.stages[8].ms,700);
    assert.equal(model.active.key,'scene');
    assert.equal(model.fraction,9/LOBBY_STAGES.length);
});

test('terminar fecha tudo e a barra chega a 100%',()=>{
    const now=clock();
    const model=new LoadingModel(WORLD_STAGES,now);
    model.begin('world-prepare');
    now.advance(2500);
    model.finish();
    assert.equal(model.fraction,1);
    assert.equal(model.active,null);
    assert.equal(model.elapsed,2500);
});

test('o resumo lista a duração de cada etapa medida e o total',()=>{
    const now=clock();
    const model=new LoadingModel(LOBBY_STAGES,now);
    model.begin('prepare','cache quente');
    now.advance(1420);
    model.complete('prepare','cache quente');
    now.advance(800);
    model.finish();
    const text=model.summary();
    assert.match(text,/^Abertura em 2,22 s:/);
    assert.match(text,/preparando o lobby 1,42 s \(cache quente\)/);
});

test('etapa desconhecida é ignorada e cache vira texto',()=>{
    const model=new LoadingModel(LOBBY_STAGES,clock());
    assert.equal(model.begin('inexistente'),false);
    assert.equal(model.complete('inexistente'),false);
    assert.equal(preparationDetail({hit:true}),'cache quente');
    assert.equal(preparationDetail({hit:false}),'cache frio');
    assert.equal(preparationDetail(undefined),'');
});
