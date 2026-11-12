import init,{check_platform,check_options} from './checkpkg/killer_web.js';
try {
    await init();
    const bytes=Uint8Array.from({length:65537},(_,i)=>i%251);
    const files=[['Fixture/System/Fixture.bin',new File([bytes],'Fixture.bin')],['Fixture/System/empty.bin',new File([],'empty.bin')],['Fixture/System/Option.ini',new File(['[Game]\r\nMyName=True\r\n'],'Option.ini')]];
    const platform=check_platform(files);
    postMessage(`${platform}\n${await check_options()}`);
} catch (error) {postMessage(`Falhou: ${String(error.message || error)}`);}
