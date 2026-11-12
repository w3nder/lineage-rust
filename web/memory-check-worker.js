import {Preparation} from './preparation-cache.mjs';

onmessage=async({data})=>{
    const report=text=>postMessage({text});
    const preparation=new Preparation(data.files,String(Date.now()),report,stats=>report(`Preparação: ${JSON.stringify(stats)}`));
    try {
        const {default:init,WebClient,prepare_world}=await import(`./pkg/killer_web.js?v=${Date.now()}`);
        const wasm=await init({module_or_path:fetch('./pkg/killer_web_bg.wasm',{cache:'no-store'})});
        const memory=()=>`${(wasm.memory.buffer.byteLength/1048576).toFixed(1)} MiB de memória linear WASM`;
        if (data.inspect) {
            const stop=new Error('Fim da inspeção da cena');
            const start=performance.now();
            let snapshot;
            try {
                prepare_world(data.files,data.settings,data.tile,()=>{},()=>{},phase=>{
                    const profile={phase,elapsedMs:performance.now()-start,wasmCapacityMiB:wasm.memory.buffer.byteLength/1048576};
                    report(`Inspeção: ${JSON.stringify(profile)}`);
                    if (phase==='encode') {snapshot=profile;throw stop;}
                });
            } catch(error) {if(error!==stop) throw error;}
            if (!snapshot) throw Error('A inspeção não parou antes da gravação do cache');
            postMessage({done:true,text:`INSPEÇÃO CONCLUÍDA: cena construída e liberada antes de cache, colisão e GPU. ${JSON.stringify(snapshot)}. Não valida entrada no jogo.`});
            return;
        }
        const results=[];
        for (let attempt=1;attempt<=2;attempt++) {
            report(`${data.tile}: carga ${attempt}/2 com preparação isolada…`);
            const start=performance.now();
            const prepared=await preparation.run(data.tile,data.settings);
            report(`${prepared.hit?'Cache válido':'Cache criado'} em ${(prepared.prepareMs/1000).toFixed(1)} s. Instalando cena/colisão…`);
            const client=await WebClient.create(data.canvas,1);
            try {
                client.restore_preferences(data.preferences);
                client.restore_settings(data.settings);
                client.mount_files(data.files);
                if(data.tile==='lobby') client.load_prepared_lobby(prepared.scene);
                else client.load_prepared_tile(data.tile,prepared.scene,prepared.trace);
                const stages=JSON.parse(client.loading_stats());
                const elapsed=(performance.now()-start)/1000;
                report(`Carga ${attempt}: ${elapsed.toFixed(1)} s; ${memory()}; etapas: ${JSON.stringify(stages)}`);
                results.push({hit:prepared.hit,totalSeconds:elapsed,prepareMs:prepared.prepareMs,stages});
            } finally {client.free();}
        }
        if (!results[1].hit) throw Error('Segunda carga não reutilizou o cache');
        postMessage({done:true,text:`PASSOU: duas cargas completas de ${data.tile}; segunda com cache válido. ${JSON.stringify(results)}. Entrada online não exercitada.`});
    } catch(error) {postMessage({done:true,text:String(error?.stack||error)});}
    finally {preparation.cancel();}
};
