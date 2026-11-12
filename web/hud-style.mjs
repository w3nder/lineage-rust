export const HUD_STYLES=['classic','modern'];

export function readStyle(text) {
    for (const line of String(text||'').split('\n')) {
        const [key,value]=line.split('=');
        if (key?.trim()==='modern') return value?.trim()==='false'?'classic':'modern';
    }
    return 'classic';
}

export function withStyle(text,style) {
    const enabled=style==='classic'?'false':'true';
    const lines=String(text||'').split('\n').filter(line=>line.trim()!==''&&line.split('=')[0].trim()!=='modern');
    return [`modern=${enabled}`,...lines].join('\n')+'\n';
}
