use killer_world::{NetCommand, NetEvent, NetHandle, PacketRecord, spawn_tapped};
use std::collections::VecDeque;

pub trait Transport {
    fn send(&mut self, command: NetCommand);
    fn poll(&mut self) -> Vec<NetEvent>;
    fn packets(&mut self) -> Vec<PacketRecord> {
        Vec::new()
    }
}

pub struct NetTransport {
    handle: NetHandle,
}

impl NetTransport {
    pub fn connect(host: String, port: u16, account: String, password: String) -> Self {
        Self { handle: spawn_tapped(host, port, account, password) }
    }
}

impl Transport for NetTransport {
    fn send(&mut self, command: NetCommand) {
        let _ = self.handle.commands.send(command);
    }

    fn poll(&mut self) -> Vec<NetEvent> {
        self.handle.events.try_iter().collect()
    }

    fn packets(&mut self) -> Vec<PacketRecord> {
        self.handle.packets.as_ref().map(|packets| packets.try_iter().collect()).unwrap_or_default()
    }
}

#[derive(Default)]
pub struct FakeTransport {
    pub sent: Vec<NetCommand>,
    inbox: VecDeque<NetEvent>,
    tapped: Vec<PacketRecord>,
}

impl FakeTransport {
    pub fn push(&mut self, event: NetEvent) {
        self.inbox.push_back(event);
    }

    pub fn push_packet(&mut self, outgoing: bool, body: &[u8]) {
        self.tapped.push(PacketRecord { outgoing, body: body.to_vec() });
    }
}

impl Transport for FakeTransport {
    fn send(&mut self, command: NetCommand) {
        self.sent.push(command);
    }

    fn poll(&mut self) -> Vec<NetEvent> {
        self.inbox.drain(..).collect()
    }

    fn packets(&mut self) -> Vec<PacketRecord> {
        std::mem::take(&mut self.tapped)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fake_transport_replays_pushed_events_once_and_records_sent_commands() {
        let mut transport = FakeTransport::default();
        transport.push(NetEvent::Connecting);
        assert_eq!(transport.poll().len(), 1);
        assert!(transport.poll().is_empty());
        transport.send(NetCommand::Appearing);
        assert_eq!(transport.sent, vec![NetCommand::Appearing]);
    }
}
