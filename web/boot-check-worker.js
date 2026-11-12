onmessage=async({data:canvas})=>{
 try {
  postMessage('Importando o motor atual…');
  const {default:init,WebClient}=await import(`./pkg/killer_web.js?v=${Date.now()}`);
  postMessage('Carregando WebAssembly…');
  await init({module_or_path:fetch('./pkg/killer_web_bg.wasm',{cache:'no-store'})});
  postMessage('Inicializando WebGPU…');
  const client=await WebClient.create(canvas,1);
  client.frame(0);
  const profile=JSON.parse(client.performance_stats());
  postMessage(`Passou: motor atual inicializado no Worker, WebGPU criado e quadro renderizado. SIMD ${profile.simd?'ativo':'inativo'}; métricas disponíveis.`);
 } catch(error) {postMessage(String(error));}
};
