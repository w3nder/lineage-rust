use std::net::{SocketAddr, TcpStream};
use std::sync::mpsc;
use std::time::Instant;

pub const NO_PING: i32 = 9999;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PingRequest {
    pub id: u8,
    pub ip: [u8; 4],
    pub port: u16,
}

pub fn measure(request: PingRequest) -> i32 {
    let address = SocketAddr::from((request.ip, request.port));
    let started = Instant::now();
    match TcpStream::connect(address) {
        Ok(_) => (started.elapsed().as_secs_f64() * 1000.0) as i32,
        Err(_) => NO_PING,
    }
}

pub fn spawn(requests: Vec<PingRequest>) -> mpsc::Receiver<(u8, i32)> {
    let (tx, rx) = mpsc::channel();
    std::thread::spawn(move || {
        for request in requests {
            if tx.send((request.id, measure(request))).is_err() {
                return;
            }
        }
    });
    rx
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;
    use std::time::Duration;

    #[test]
    fn a_reachable_port_gives_the_connect_time_in_milliseconds_and_a_closed_one_gives_9999() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let ping = measure(PingRequest { id: 1, ip: [127, 0, 0, 1], port });
        assert!((0..2000).contains(&ping));
        drop(listener);
        assert_eq!(measure(PingRequest { id: 1, ip: [127, 0, 0, 1], port }), NO_PING);
    }

    #[test]
    fn the_worker_answers_each_request_in_order_and_stops_when_nobody_listens() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = listener.local_addr().unwrap().port();
        let rx = spawn(vec![PingRequest { id: 4, ip: [127, 0, 0, 1], port }, PingRequest { id: 2, ip: [127, 0, 0, 1], port }]);
        let first = rx.recv_timeout(Duration::from_secs(5)).unwrap();
        let second = rx.recv_timeout(Duration::from_secs(5)).unwrap();
        assert_eq!((first.0, second.0), (4, 2));
        assert!(rx.recv_timeout(Duration::from_millis(200)).is_err());
    }
}
