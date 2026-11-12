export const LOBBY_STAGES=[
    {key:'folder',label:'Lendo a pasta do cliente'},
    {key:'download',label:'Baixando os arquivos do jogo'},
    {key:'engine',label:'Iniciando o motor'},
    {key:'mount',label:'Registrando os arquivos'},
    {key:'prepare',label:'Preparando o lobby'},
    {key:'open',label:'Abrindo o cliente'},
    {key:'level',label:'Lendo o mapa do lobby'},
    {key:'config',label:'Lendo as configurações'},
    {key:'tables',label:'Lendo as tabelas do jogo'},
    {key:'scene',label:'Montando a cena'},
    {key:'ground',label:'Montando o chão'},
    {key:'upload',label:'Enviando para a placa de vídeo'},
];

export const WORLD_STAGES=[
    {key:'world-prepare',label:'Preparando o mapa'},
    {key:'world-read',label:'Lendo o mapa'},
    {key:'world-scene',label:'Montando a cena'},
    {key:'world-upload',label:'Enviando para a placa de vídeo'},
    {key:'world-runtime',label:'Montando o mundo e a interface'},
];

export class LoadingModel {
    constructor(stages,now=()=>performance.now()) {
        this.stages=stages.map(stage=>({...stage,state:'waiting',startedAt:null,ms:null,detail:''}));
        this.now=now;
        this.startedAt=now();
        this.cursor=this.startedAt;
        this.finishedAt=null;
    }

    index(key) {return this.stages.findIndex(stage=>stage.key===key);}

    begin(key,detail='') {
        const at=this.index(key);
        if (at<0) return false;
        const stage=this.stages[at];
        if (stage.state==='running') {stage.detail=detail||stage.detail;return true;}
        for (const earlier of this.stages.slice(0,at)) if (earlier.state!=='done') this.settle(earlier);
        stage.state='running';
        stage.startedAt=this.now();
        stage.detail=detail;
        this.cursor=stage.startedAt;
        return true;
    }

    settle(stage) {
        const at=this.now();
        if (stage.state==='waiting') stage.ms=0;
        else {
            stage.ms=at-(stage.startedAt??this.cursor);
            this.cursor=at;
        }
        stage.state='done';
    }

    complete(key,detail='') {
        const at=this.index(key);
        if (at<0) return false;
        for (const earlier of this.stages.slice(0,at)) if (earlier.state!=='done') this.settle(earlier);
        const stage=this.stages[at];
        if (stage.state==='waiting') {stage.state='running';stage.startedAt=this.cursor;}
        if (stage.state!=='done') this.settle(stage);
        if (detail) stage.detail=detail;
        const next=this.stages[at+1];
        if (next && next.state==='waiting') {next.state='running';next.startedAt=this.now();}
        return true;
    }

    finish() {
        for (const stage of this.stages) if (stage.state!=='done') this.settle(stage);
        this.finishedAt=this.now();
    }

    get active() {return this.stages.find(stage=>stage.state==='running')||null;}
    get completed() {return this.stages.filter(stage=>stage.state==='done').length;}
    get fraction() {return this.finishedAt===null?this.completed/this.stages.length:1;}
    get elapsed() {return (this.finishedAt??this.now())-this.startedAt;}

    summary() {
        const seconds=value=>`${(value/1000).toFixed(2).replace('.',',')} s`;
        const parts=this.stages.filter(stage=>stage.ms!==null).map(stage=>`${stage.label.toLowerCase()} ${seconds(stage.ms)}${stage.detail?` (${stage.detail})`:''}`);
        return `Abertura em ${seconds(this.elapsed)}: ${parts.join(' · ')}`;
    }
}

const PREPARATION_DETAIL=stats=>{
    if (!stats) return '';
    return stats.hit?'cache quente':'cache frio';
};
export {PREPARATION_DETAIL as preparationDetail};

export class LoadingView {
    constructor(root) {
        this.root=root;
        this.art=root.querySelector('.art');
        this.stage=root.querySelector('.stage');
        this.detail=root.querySelector('.detail');
        this.bar=root.querySelector('.bar i');
        this.steps=root.querySelector('.steps');
        this.model=null;
        this.noteText='';
    }

    show(model) {
        this.model=model;
        this.root.hidden=false;
        this.render();
    }

    hide() {
        this.root.hidden=true;
        this.model=null;
    }

    image(width,height,rgba) {
        this.art.width=width;
        this.art.height=height;
        this.art.getContext('2d').putImageData(new ImageData(new Uint8ClampedArray(rgba.buffer,rgba.byteOffset,rgba.byteLength),width,height),0,0);
        this.root.classList.add('with-art');
    }

    render() {
        const model=this.model;
        if (!model || this.root.hidden) return;
        const active=model.active;
        this.stage.textContent=active?`${active.label}…`:'Concluindo…';
        this.detail.textContent=this.noteText||active?.detail||'';
        this.bar.style.width=`${Math.round(model.fraction*100)}%`;
        this.steps.replaceChildren(...model.stages.map(stage=>{
            const item=document.createElement('li');
            item.dataset.state=stage.state;
            const ms=stage.ms===null?'':` ${(stage.ms/1000).toFixed(2).replace('.',',')} s`;
            item.textContent=`${stage.label}${stage.detail&&stage.state==='done'?` · ${stage.detail}`:''}${ms}`;
            return item;
        }));
    }
}
