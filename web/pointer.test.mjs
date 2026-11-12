import test from 'node:test';
import assert from 'node:assert/strict';
import {PointerDelta,decodeCursor,CursorBank} from './pointer.mjs';

test('drag starts at press and uses canvas coordinates regardless of browser movement units',()=>{
    const pointer=new PointerDelta();
    assert.deepEqual(pointer.update('down',100,80),[0,0]);
    assert.deepEqual(pointer.update('move',110,75),[10,-5]);
    assert.deepEqual(pointer.update('move',110.5,75.25),[0.5,0.25]);
    assert.deepEqual(pointer.update('up',110.5,75.25),[0,0]);
    pointer.update('cancel',0,0);
    assert.deepEqual(pointer.update('move',200,150),[0,0]);
    pointer.update('leave',200,150);
    assert.deepEqual(pointer.update('move',300,250),[0,0]);
});

test('cursor transport preserves RGBA and little-endian hotspot with a sliced buffer',()=>{
    const buffer=new Uint8Array(22);
    const bytes=buffer.subarray(3,19);
    bytes.set([2,0,1,0,1,0,0,0,255,0,0,255,0,255,0,0]);
    const cursor=decodeCursor(bytes);
    assert.equal(cursor.width,2);
    assert.equal(cursor.height,1);
    assert.deepEqual(cursor.hotspot,[1,0]);
    assert.deepEqual([...cursor.rgba],[255,0,0,255,0,255,0,0]);
    assert.throws(()=>decodeCursor(bytes.subarray(0,15)));
    bytes[4]=2;
    assert.throws(()=>decodeCursor(bytes));
    assert.throws(()=>decodeCursor(new Uint8Array()));
});

test('all resources, drag, disabling and restoration preserve the last cursor on missing texture',()=>{
    let renders=0;
    const bank=new CursorBank(image=>`${++renders}:${image.hotspot}`);
    const bytes=new Uint8Array([1,0,1,0,0,0,0,0,1,2,3,255]);
    bank.load(Array.from({length:7},()=>bytes));
    for (let i=0;i<7;i++) assert.equal(bank.select(String(i)),`${i+1}:0,0`);
    assert.equal(bank.select('@icon',bytes),'8:0,0');
    assert.equal(bank.select('@missing'),'8:0,0');
    assert.equal(bank.select('system'),'default');
    assert.equal(bank.select('0'),'1:0,0');
    assert.equal(renders,8);
});
