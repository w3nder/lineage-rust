export class PointerDelta {
    last=null;
    update(kind,x,y) {
        const previous=this.last;
        this.last=kind==='leave' || kind==='cancel' ? null : [x,y];
        return kind==='move' && previous ? [x-previous[0],y-previous[1]] : [0,0];
    }
}

export function decodeCursor(bytes) {
    const view=new DataView(bytes.buffer,bytes.byteOffset,bytes.byteLength);
    const width=view.getUint16(0,true),height=view.getUint16(2,true);
    const hotspot=[view.getUint16(4,true),view.getUint16(6,true)];
    if (!width || !height || bytes.length!==8+width*height*4 || hotspot[0]>=width || hotspot[1]>=height) throw new Error('Cursor inválido');
    return {width,height,hotspot,rgba:new Uint8ClampedArray(bytes.slice(8))};
}

export class CursorBank {
    resources=[];
    current='default';
    constructor(render) { this.render=render; }
    load(resources) {
        this.resources=resources.map(bytes=>this.render(decodeCursor(bytes)));
        this.current=this.resources[0] || 'default';
        return this.current;
    }
    select(key,bytes) {
        if (key==='system') this.current='default';
        else if (key.startsWith('@')) {
            if (bytes) this.current=this.render(decodeCursor(bytes));
        } else if (this.resources[Number(key)]) this.current=this.resources[Number(key)];
        return this.current;
    }
}
