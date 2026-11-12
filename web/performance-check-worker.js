onmessage=async()=>{
 try {
  const reports=[];
  for (const [name,path] of [['Base','./data/perf-base/killer_web.js'],['Otimizado','./checkpkg/killer_web.js']]) {
   postMessage({text:`Medindo ${name}…`});
   const module=await import(`${path}?v=${Date.now()}`);
   await module.default();
   module.benchmark_math(50000);
   const samples=[];
   let checksum;
   for(let i=0;i<7;i++) {
    const start=performance.now();
    checksum=Array.from(module.benchmark_math(1000000));
    samples.push(performance.now()-start);
   }
   samples.sort((a,b)=>a-b);
   reports.push({name,simd:module.math_simd(),median:samples[3],samples,checksum,checked:module.check_math?.() || 0});
  }
  const [base,optimized]=reports;
  if(base.simd || !optimized.simd) throw Error('A comparação requer base sem SIMD e otimizado com SIMD.');
  if(base.checksum.some((value,i)=>Math.abs(value-optimized.checksum[i])>Math.max(1,Math.abs(value))*0.00001)) throw Error('Resultado numérico divergente.');
  postMessage({done:true,text:JSON.stringify({reports,speedup:base.median/optimized.median,equivalent:true},null,2)});
 } catch(error) {postMessage({done:true,text:String(error)});}
};
