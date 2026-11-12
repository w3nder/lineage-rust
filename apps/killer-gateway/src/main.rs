use clap::Parser;
use futures_util::{SinkExt, StreamExt};
use std::io;
use std::sync::{Arc, Mutex};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64,Ordering};
use std::time::Instant;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Semaphore;
use tokio::time::timeout;
use tokio_tungstenite::tungstenite::{Message, handshake::server::{Request, Response, ErrorResponse}, protocol::WebSocketConfig};

#[derive(Parser)]
struct Args {
    #[arg(long, default_value="127.0.0.1:8081")]
    bind: String,
    #[arg(long, env="KILLER_LOGIN_TARGET")]
    login: String,
    #[arg(long, env="KILLER_GAME_TARGET")]
    game: String,
    #[arg(long, default_value="http://127.0.0.1:8080")]
    origin: String,
    #[arg(long, default_value_t=32)]
    max_connections: usize,
}

struct Trace {
    buffer: Vec<u8>,
    seen: VecDeque<String>,
}

impl Trace {
    fn new() -> Trace {
        Trace {buffer: Vec::new(), seen: VecDeque::new()}
    }

    fn feed(&mut self,arrow: &str,at: f32,bytes: &[u8]) {
        self.buffer.extend_from_slice(bytes);
        while self.buffer.len()>=3 {
            let length=usize::from(u16::from_le_bytes([self.buffer[0],self.buffer[1]]));
            if length<3 || self.buffer.len()<length {break;}
            let opcode=self.buffer[2];
            self.buffer.drain(..length);
            if self.seen.len()==24 {self.seen.pop_front();}
            self.seen.push_back(format!("{at:.2}s {arrow} 0x{opcode:02x} ({length}b)"));
        }
    }
}

fn reject() -> ErrorResponse {
    let mut response=ErrorResponse::new(Some("origem ou rota recusada".into()));
    *response.status_mut()=tokio_tungstenite::tungstenite::http::StatusCode::FORBIDDEN;
    response
}

async fn relay(socket: TcpStream,args: &Args) -> Result<(),Box<dyn std::error::Error + Send + Sync>> {
    let mut target=None;
    let mut route_name=None;
    #[allow(clippy::result_large_err)]
    let callback=|request:&Request,response:Response| {
        if !request.headers().get("origin").and_then(|value|value.to_str().ok()).is_some_and(|origin|args.origin.split(',').any(|allowed|allowed.trim()==origin)) {
            return Err(reject());
        }
        target=match request.uri().path() {"/login"=>Some(args.login.clone()),"/game"=>Some(args.game.clone()),_=>return Err(reject())};
        route_name=Some(request.uri().path().to_string());
        if request.uri().query().is_some() {return Err(reject());}
        Ok(response)
    };
    let config=WebSocketConfig::default().max_message_size(Some(65536)).max_frame_size(Some(65536));
    let websocket=timeout(Duration::from_secs(10),tokio_tungstenite::accept_hdr_async_with_config(socket,callback,Some(config))).await??;
    let target=target.ok_or_else(||io::Error::from(io::ErrorKind::InvalidInput))?;
    let target_route=route_name.unwrap_or_default();
    let tcp=timeout(Duration::from_secs(10),TcpStream::connect(target)).await??;
    tcp.set_nodelay(true)?;
    let (mut ws_out,mut ws_in)=websocket.split();
    let (mut tcp_in,mut tcp_out)=tcp.into_split();
    let route=target_route.clone();
    let started=Instant::now();
    let sent=Arc::new(AtomicU64::new(0));
    let received=Arc::new(AtomicU64::new(0));
    let (sent_up,received_down)=(sent.clone(),received.clone());
    let up_trace=Arc::new(Mutex::new((Trace::new(),Trace::new())));
    let down_trace=up_trace.clone();
    let trace_view=up_trace.clone();
    let upstream=async move {
        let reason=loop {
            match ws_in.next().await {
                None=>break "navegador fechou o transporte sem Close".to_string(),
                Some(Err(error))=>break format!("erro do WebSocket do navegador: {error}"),
                Some(Ok(Message::Binary(bytes)))=>{
                    sent_up.fetch_add(bytes.len() as u64,Ordering::Relaxed);
                    if let Ok(mut trace)=up_trace.lock() {trace.0.feed("C->S",started.elapsed().as_secs_f32(),&bytes);}
                    if let Err(error)=tcp_out.write_all(&bytes).await {break format!("erro ao escrever no servidor: {error}");}
                }
                Some(Ok(Message::Close(frame)))=>break format!("navegador enviou Close {frame:?}"),
                Some(Ok(Message::Ping(_) | Message::Pong(_)))=>{}
                Some(Ok(_))=>break "navegador enviou frame que não é binário".to_string(),
            }
        };
        let _=tcp_out.shutdown().await;
        reason
    };
    let downstream=async move {
        let mut bytes=vec![0;32768];
        loop {
            match tcp_in.read(&mut bytes).await {
                Ok(0)=>{let _=ws_out.close().await;break "servidor L2 fechou a conexão (EOF)".to_string();}
                Ok(count)=>{
                    received_down.fetch_add(count as u64,Ordering::Relaxed);
                    if let Ok(mut trace)=down_trace.lock() {trace.1.feed("S->C",started.elapsed().as_secs_f32(),&bytes[..count]);}
                    if let Err(error)=ws_out.send(Message::Binary(bytes[..count].to_vec().into())).await {break format!("erro ao enviar ao navegador: {error}");}
                }
                Err(error)=>break format!("erro de leitura do servidor L2: {error}"),
            }
        }
    };
    let reason=tokio::select! {reason=upstream=>reason,reason=downstream=>reason};
    eprintln!("[{route}] encerrada após {:.1}s, navegador→servidor {} bytes, servidor→navegador {} bytes: {reason}",started.elapsed().as_secs_f32(),sent.load(Ordering::Relaxed),received.load(Ordering::Relaxed));
    if route=="/game"
        && let Ok(trace)=trace_view.lock()
    {
        let mut lines: Vec<(f32,&String)>=trace.0.seen.iter().chain(trace.1.seen.iter()).map(|line|(line.split('s').next().and_then(|time|time.parse::<f32>().ok()).unwrap_or(0.0),line)).collect();
        lines.sort_by(|a,b|a.0.total_cmp(&b.0));
        for (_,line) in lines {eprintln!("  {line}");}
    }
    Ok(())
}

#[tokio::main(flavor="current_thread")]
async fn main() -> Result<(),Box<dyn std::error::Error>> {
    let args=Arc::new(Args::parse());
    if args.max_connections==0 {return Err("--max-connections deve ser positivo".into());}
    let listener=TcpListener::bind(&args.bind).await?;
    let limit=Arc::new(Semaphore::new(args.max_connections));
    println!("Gateway em {}",listener.local_addr()?);
    loop {
        let (socket,_)=listener.accept().await?;
        let Ok(permit)=limit.clone().try_acquire_owned() else {drop(socket);continue;};
        let args=args.clone();
        tokio::spawn(async move {
            let _permit=permit;
            if relay(socket,&args).await.is_err() {eprintln!("conexão encerrada ou recusada");}
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio_tungstenite::tungstenite::client::IntoClientRequest;

    #[tokio::test]
    async fn relay_preserves_bytes_and_propagates_backend_close() {
        let tcp=TcpListener::bind("127.0.0.1:0").await.unwrap();
        let destination=tcp.local_addr().unwrap().to_string();
        let echo=tokio::spawn(async move {
            let (mut stream,_)=tcp.accept().await.unwrap();
            let mut bytes=[0;9];
            stream.read_exact(&mut bytes).await.unwrap();
            assert_eq!(bytes,[5,0,1,2,3,4,0,8,9]);
            stream.write_all(&bytes[..1]).await.unwrap();
            stream.write_all(&bytes[1..]).await.unwrap();
            stream.shutdown().await.unwrap();
        });
        let listener=TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address=listener.local_addr().unwrap();
        let server=tokio::spawn(async move {
            let args=Args {bind:String::new(),login:destination.clone(),game:destination,origin:"http://localhost:8080".into(),max_connections:1};
            let (stream,_)=listener.accept().await.unwrap();
            relay(stream,&args).await.unwrap();
        });
        let mut request=format!("ws://{address}/login").into_client_request().unwrap();
        request.headers_mut().insert("origin","http://localhost:8080".parse().unwrap());
        let (mut socket,_)=tokio_tungstenite::connect_async(request).await.unwrap();
        socket.send(Message::Binary(vec![5,0,1].into())).await.unwrap();
        socket.send(Message::Binary(vec![2,3,4,0,8,9].into())).await.unwrap();
        let result=timeout(Duration::from_secs(3),async {
            let mut bytes=Vec::new();
            while let Some(message)=socket.next().await {
                match message.unwrap() {Message::Binary(part)=>bytes.extend(part),Message::Close(_)=>break,_=>{}}
            }
            bytes
        }).await.unwrap();
        assert_eq!(result,[5,0,1,2,3,4,0,8,9]);
        echo.await.unwrap();
        server.await.unwrap();
    }

    #[tokio::test]
    async fn relay_accepts_any_origin_of_the_comma_separated_list_and_nothing_else() {
        let listener=TcpListener::bind("127.0.0.1:0").await.unwrap();
        let destination=listener.local_addr().unwrap().to_string();
        let allowed="https://play.example.com, http://localhost:8080";
        for (origin,accepted) in [("https://play.example.com",true),("http://localhost:8080",true),("https://play.example.com.evil.invalid",false),("http://evil.invalid",false)] {
            let gateway=TcpListener::bind("127.0.0.1:0").await.unwrap();
            let address=gateway.local_addr().unwrap();
            let args=Args {bind:String::new(),login:destination.clone(),game:destination.clone(),origin:allowed.into(),max_connections:1};
            let server=tokio::spawn(async move {let (stream,_)=gateway.accept().await.unwrap();relay(stream,&args).await.is_ok()});
            let mut request=format!("ws://{address}/login").into_client_request().unwrap();
            request.headers_mut().insert("origin",origin.parse().unwrap());
            let connected=tokio_tungstenite::connect_async(request).await.is_ok();
            assert_eq!(connected,accepted,"{origin}");
            if accepted {let (_peer,_)=listener.accept().await.unwrap();}
            server.abort();
        }
    }

    #[tokio::test]
    async fn relay_rejects_other_origins_and_unconfigured_routes_before_tcp() {
        for (origin,path) in [("http://evil.invalid","/login"),("http://localhost:8080","/other"),("http://localhost:8080","/game?host=other")] {
            let listener=TcpListener::bind("127.0.0.1:0").await.unwrap();
            let address=listener.local_addr().unwrap();
            let server=tokio::spawn(async move {
                let (stream,_)=listener.accept().await.unwrap();
                relay(stream,&Args {bind:String::new(),login:"127.0.0.1:1".into(),game:"127.0.0.1:1".into(),origin:"http://localhost:8080".into(),max_connections:1}).await.is_err()
            });
            let mut request=format!("ws://{address}{path}").into_client_request().unwrap();
            request.headers_mut().insert("origin",origin.parse().unwrap());
            let result=tokio_tungstenite::connect_async(request).await;
            assert!(matches!(result,Err(tokio_tungstenite::tungstenite::Error::Http(response)) if response.status()==403));
            assert!(server.await.unwrap());
        }
    }
}
