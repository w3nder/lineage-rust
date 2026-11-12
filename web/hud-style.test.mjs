import test from 'node:test';
import assert from 'node:assert/strict';
import {readStyle,withStyle} from './hud-style.mjs';

test('sem preferência salva o padrão é o clássico, como o desktop', () => {
    assert.equal(readStyle(''),'classic');
    assert.equal(readStyle('bars=2\nlocked=false\n'),'classic');
});

test('lê a escolha gravada', () => {
    assert.equal(readStyle('modern=false\nbars=2\n'),'classic');
    assert.equal(readStyle('bars=2\nmodern=true\n'),'modern');
});

test('grava a escolha sem perder as outras preferências', () => {
    const saved=withStyle('modern=true\nbars=3\nlocked=true\n','classic');
    assert.equal(saved,'modern=false\nbars=3\nlocked=true\n');
    assert.equal(readStyle(saved),'classic');
    assert.equal(withStyle('','modern'),'modern=true\n');
});
