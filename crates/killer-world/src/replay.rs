use crate::replay_table::USE_PACKETS;

pub const REVISION: i32 = 656;
pub const HEADER_TEXT: &str = "LineageII Replay File ";
pub const HEADER_TEXT_OLD: &str = "LineageII Replay File";
pub const SAFE_PACKAGE: &str = "SafePackage";
pub const EXTENSION: &str = ".l2r";
pub const FORMAT_MARKER: i32 = 1;
pub const AUTO_STOP_SECONDS: f64 = 1800.0;
pub const NAME_LIMIT: i32 = 0x10000;
pub const OP_EX_PACKET: u8 = 0xfe;
pub const EX_ID_BASE: u32 = 0xff;
pub const PACKET_USER_INFO: u32 = 4;
pub const PACKET_NPC_SAY: u32 = 2;
pub const PACKET_SAY: u32 = 0x4a;
pub const TABLE_SIZE: usize = 0x130;
pub const START_DELAY: f64 = 1.0;
pub const FADE_IN_FIRST_MS: f32 = 2000.0;
pub const FADE_IN_MS: f32 = 1000.0;
pub const CLOCK_MARK: f64 = 0.001;
pub const SPEED_MAX_DOUBLE: f32 = 4.0;
pub const SPEED_MIN_HALVE: f32 = 0.25;
pub const SYSMSG_RECORD_STARTED: i32 = 0x674;
pub const SYSMSG_RECORD_SAVED: i32 = 0x675;
pub const SYSMSG_RECORD_FAILED: i32 = 0x676;
pub const KEY_ESCAPE: u32 = 0x1b;
pub const KEY_SPACE: u32 = 0x20;
pub const KEY_ADD: u32 = 0x6b;
pub const KEY_SUBTRACT: u32 = 0x6d;
pub const ACTION_PRESS: u32 = 1;
pub const ACTION_RELEASE: u32 = 3;
pub const PAWN_CREATE_PACKETS: [u8; 4] = [3, 4, 0x16, 0x8c];

pub fn packet_id(opcode: u8, ext: u16) -> u32 {
    if opcode == OP_EX_PACKET { ext as i16 as i32 as u32 + EX_ID_BASE } else { opcode as u32 }
}

pub fn use_flag(id: u32, chat: bool) -> bool {
    if chat && (id == PACKET_NPC_SAY || id == PACKET_SAY) {
        return true;
    }
    USE_PACKETS.iter().any(|(entry, _)| *entry as u32 == id && *entry > 0)
}

pub fn packet_name(id: u32) -> Option<&'static str> {
    if id == PACKET_NPC_SAY {
        return Some("NPC_SAY");
    }
    if id == PACKET_SAY {
        return Some("SAY");
    }
    USE_PACKETS.iter().find(|(entry, _)| *entry as u32 == id).map(|(_, name)| *name)
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReplayHeader {
    pub version: i32,
    pub start: [f32; 3],
    pub camera: [f32; 4],
    pub hour: u8,
    pub minute: u8,
    pub second: f32,
    pub running_time: f64,
    pub camera_mode: i32,
    pub first_person: i32,
    pub observer: i32,
    pub yaw: f32,
    pub view_mode: i32,
}

impl Default for ReplayHeader {
    fn default() -> Self {
        ReplayHeader {
            version: REVISION,
            start: [0.0; 3],
            camera: [0.0; 4],
            hour: 0,
            minute: 0,
            second: 0.0,
            running_time: 0.0,
            camera_mode: 0,
            first_person: 0,
            observer: 0,
            yaw: 0.0,
            view_mode: 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReplayPacket {
    pub time: f64,
    pub id: u8,
    pub ext: u16,
    pub data: Vec<u8>,
}

impl ReplayPacket {
    pub fn from_body(time: f64, body: &[u8]) -> Option<ReplayPacket> {
        let id = *body.first()?;
        if id == OP_EX_PACKET {
            let ext = u16::from_le_bytes([*body.get(1)?, *body.get(2)?]);
            let mut data = body[3..].to_vec();
            data.push(0);
            Some(ReplayPacket { time, id, ext, data })
        } else {
            let mut data = body[1..].to_vec();
            data.push(0);
            Some(ReplayPacket { time, id, ext: 0, data })
        }
    }

    pub fn len(&self) -> usize {
        self.data.len().saturating_sub(1)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn body(&self) -> Vec<u8> {
        let mut body = vec![self.id];
        if self.id == OP_EX_PACKET {
            body.extend_from_slice(&self.ext.to_le_bytes());
        }
        body.extend_from_slice(&self.data[..self.len()]);
        body
    }

    pub fn packet_id(&self) -> u32 {
        packet_id(self.id, self.ext)
    }

    pub fn pawn_object_id(&self) -> Option<i32> {
        if !PAWN_CREATE_PACKETS.contains(&self.id) {
            return None;
        }
        let at = match self.id {
            3 | 4 => 16,
            _ => 0,
        };
        let raw = self.data.get(at..at + 4)?;
        Some(i32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CameraEntry {
    pub time: f64,
    pub a: i32,
    pub b: i32,
    pub c: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ReplayFile {
    pub marker: i32,
    pub header: ReplayHeader,
    pub packets: Vec<ReplayPacket>,
    pub cameras: Vec<CameraEntry>,
    pub user_ids: Vec<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplayError {
    Truncated,
    BadHeader,
    NotSafe,
    NewerVersion(i32),
    NegativeCount,
}

impl std::fmt::Display for ReplayError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReplayError::Truncated => write!(f, "arquivo de replay truncado"),
            ReplayError::BadHeader => write!(f, "cabeçalho de replay inválido"),
            ReplayError::NotSafe => write!(f, "arquivo de replay sem a marca SafePackage"),
            ReplayError::NewerVersion(version) => write!(f, "replay da revisão {version}, mais nova que {REVISION}"),
            ReplayError::NegativeCount => write!(f, "contagem negativa no replay"),
        }
    }
}

impl std::error::Error for ReplayError {}

struct Reader<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], ReplayError> {
        let end = self.at.checked_add(n).ok_or(ReplayError::Truncated)?;
        let slice = self.bytes.get(self.at..end).ok_or(ReplayError::Truncated)?;
        self.at = end;
        Ok(slice)
    }

    fn i32(&mut self) -> Result<i32, ReplayError> {
        let raw = self.take(4)?;
        Ok(i32::from_le_bytes([raw[0], raw[1], raw[2], raw[3]]))
    }

    fn f32(&mut self) -> Result<f32, ReplayError> {
        Ok(f32::from_bits(self.i32()? as u32))
    }

    fn f64(&mut self) -> Result<f64, ReplayError> {
        let raw = self.take(8)?;
        let mut array = [0u8; 8];
        array.copy_from_slice(raw);
        Ok(f64::from_le_bytes(array))
    }

    fn u8(&mut self) -> Result<u8, ReplayError> {
        Ok(self.take(1)?[0])
    }

    fn u16(&mut self) -> Result<u16, ReplayError> {
        let raw = self.take(2)?;
        Ok(u16::from_le_bytes([raw[0], raw[1]]))
    }

    fn count(&mut self) -> Result<usize, ReplayError> {
        let value = self.i32()?;
        if value < 0 {
            return Err(ReplayError::NegativeCount);
        }
        Ok(value as usize)
    }

    fn string(&mut self) -> Result<String, ReplayError> {
        let length = self.i32()?;
        if length >= 0 {
            let raw = self.take(length as usize)?;
            let text = raw.iter().take_while(|byte| **byte != 0).map(|byte| *byte as char).collect();
            Ok(text)
        } else {
            let units = length.unsigned_abs() as usize;
            let raw = self.take(units * 2)?;
            let words: Vec<u16> = raw.chunks_exact(2).map(|pair| u16::from_le_bytes([pair[0], pair[1]])).take_while(|word| *word != 0).collect();
            Ok(String::from_utf16_lossy(&words))
        }
    }
}

fn write_string(out: &mut Vec<u8>, text: &str) {
    out.extend_from_slice(&(text.len() as i32 + 1).to_le_bytes());
    out.extend_from_slice(text.as_bytes());
    out.push(0);
}

fn read_header(reader: &mut Reader<'_>) -> Result<ReplayHeader, ReplayError> {
    let version = reader.i32()?;
    let start = [reader.f32()?, reader.f32()?, reader.f32()?];
    let camera = [reader.f32()?, reader.f32()?, reader.f32()?, reader.f32()?];
    let hour = reader.u8()?;
    let minute = reader.u8()?;
    let second = reader.f32()?;
    let running_time = reader.f64()?;
    let camera_mode = reader.i32()?;
    let first_person = reader.i32()?;
    let observer = reader.i32()?;
    let yaw = reader.f32()?;
    let view_mode = reader.i32()?;
    Ok(ReplayHeader { version, start, camera, hour, minute, second, running_time, camera_mode, first_person, observer, yaw, view_mode })
}

fn write_header(out: &mut Vec<u8>, header: &ReplayHeader) {
    out.extend_from_slice(&header.version.to_le_bytes());
    for value in header.start.iter().chain(header.camera.iter()) {
        out.extend_from_slice(&value.to_le_bytes());
    }
    out.push(header.hour);
    out.push(header.minute);
    out.extend_from_slice(&header.second.to_le_bytes());
    out.extend_from_slice(&header.running_time.to_le_bytes());
    out.extend_from_slice(&header.camera_mode.to_le_bytes());
    out.extend_from_slice(&header.first_person.to_le_bytes());
    out.extend_from_slice(&header.observer.to_le_bytes());
    out.extend_from_slice(&header.yaw.to_le_bytes());
    out.extend_from_slice(&header.view_mode.to_le_bytes());
}

pub fn is_safe_package(bytes: &[u8]) -> bool {
    let mut tag = [SAFE_PACKAGE.as_bytes(), &[0]].concat();
    tag.truncate(12);
    for back in [12usize, 0x20] {
        if bytes.len() >= back && bytes[bytes.len() - back..bytes.len() - back + 12] == tag[..] {
            return true;
        }
    }
    false
}

impl ReplayFile {
    pub fn parse(bytes: &[u8]) -> Result<ReplayFile, ReplayError> {
        if !is_safe_package(bytes) {
            return Err(ReplayError::NotSafe);
        }
        let mut reader = Reader { bytes, at: 0 };
        let text = reader.string()?;
        let marker = if text == HEADER_TEXT {
            reader.i32()?
        } else if text == HEADER_TEXT_OLD {
            0
        } else {
            return Err(ReplayError::BadHeader);
        };
        let header = read_header(&mut reader)?;
        if header.version > REVISION {
            return Err(ReplayError::NewerVersion(header.version));
        }
        let mut packets = Vec::new();
        for _ in 0..reader.count()? {
            let time = reader.f64()?;
            let id = reader.u8()?;
            let ext = reader.u16()?;
            let len = reader.count()?;
            let data = reader.take(len + 1)?.to_vec();
            packets.push(ReplayPacket { time, id, ext, data });
        }
        let mut cameras = Vec::new();
        for _ in 0..reader.count()? {
            cameras.push(CameraEntry { time: reader.f64()?, a: reader.i32()?, b: reader.i32()?, c: reader.i32()? });
        }
        let mut user_ids = Vec::new();
        if marker >= 1 {
            for _ in 0..reader.count()? {
                user_ids.push(reader.i32()?);
            }
        }
        Ok(ReplayFile { marker, header, packets, cameras, user_ids })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        write_string(&mut out, HEADER_TEXT);
        out.extend_from_slice(&FORMAT_MARKER.to_le_bytes());
        write_header(&mut out, &self.header);
        out.extend_from_slice(&(self.packets.len() as i32).to_le_bytes());
        for packet in &self.packets {
            out.extend_from_slice(&packet.time.to_le_bytes());
            out.push(packet.id);
            out.extend_from_slice(&packet.ext.to_le_bytes());
            out.extend_from_slice(&(packet.len() as i32).to_le_bytes());
            out.extend_from_slice(&packet.data);
        }
        out.extend_from_slice(&(self.cameras.len() as i32).to_le_bytes());
        for camera in &self.cameras {
            out.extend_from_slice(&camera.time.to_le_bytes());
            out.extend_from_slice(&camera.a.to_le_bytes());
            out.extend_from_slice(&camera.b.to_le_bytes());
            out.extend_from_slice(&camera.c.to_le_bytes());
        }
        out.extend_from_slice(&(self.user_ids.len() as i32).to_le_bytes());
        for id in &self.user_ids {
            out.extend_from_slice(&id.to_le_bytes());
        }
        write_string(&mut out, SAFE_PACKAGE);
        out
    }
}

pub fn file_name(index: i32) -> String {
    format!("Rep{index:05}")
}

pub fn free_name(exists: impl Fn(&str) -> bool) -> Option<String> {
    (0..NAME_LIMIT).map(file_name).find(|name| !exists(&format!("{name}{EXTENSION}")))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecorderState {
    Idle,
    Recording,
}

#[derive(Debug, Clone, PartialEq)]
pub enum RecorderNotice {
    Started,
    Saved(String),
    Failed,
}

#[derive(Debug, Clone)]
pub struct RecordStart {
    pub start: [f32; 3],
    pub yaw: f32,
    pub camera: [f32; 4],
    pub camera_mode: bool,
    pub first_person: bool,
    pub observer: i32,
    pub view_mode: i32,
    pub hour: u8,
    pub minute: u8,
    pub second: f32,
    pub user_ids: Vec<i32>,
}

#[derive(Debug)]
pub struct Recorder {
    state: RecorderState,
    name: String,
    header: ReplayHeader,
    packets: Vec<ReplayPacket>,
    cameras: Vec<CameraEntry>,
    user_ids: Vec<i32>,
    origin: f64,
    use_chat: bool,
}

impl Recorder {
    pub fn new() -> Recorder {
        Recorder {
            state: RecorderState::Idle,
            name: String::new(),
            header: ReplayHeader::default(),
            packets: Vec::new(),
            cameras: Vec::new(),
            user_ids: Vec::new(),
            origin: 0.0,
            use_chat: false,
        }
    }

    pub fn state(&self) -> RecorderState {
        self.state
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn packet_count(&self) -> usize {
        self.packets.len()
    }

    pub fn start(&mut self, exists: impl Fn(&str) -> bool, snapshot: RecordStart) -> Option<RecorderNotice> {
        if self.state != RecorderState::Idle {
            return None;
        }
        self.name = free_name(exists)?;
        self.use_chat = true;
        self.packets.clear();
        self.cameras.clear();
        self.origin = 0.0;
        self.user_ids = snapshot.user_ids;
        self.header = ReplayHeader {
            version: REVISION,
            start: snapshot.start,
            camera: snapshot.camera,
            hour: snapshot.hour,
            minute: snapshot.minute,
            second: snapshot.second,
            running_time: 0.0,
            camera_mode: snapshot.camera_mode as i32,
            first_person: snapshot.first_person as i32,
            observer: snapshot.observer,
            yaw: snapshot.yaw,
            view_mode: snapshot.view_mode,
        };
        self.state = RecorderState::Recording;
        Some(RecorderNotice::Started)
    }

    pub fn add_packet(&mut self, now: f64, body: &[u8]) {
        if self.state != RecorderState::Recording {
            return;
        }
        let Some(&opcode) = body.first() else { return };
        if self.origin == 0.0 && opcode as u32 == PACKET_USER_INFO {
            self.origin = now;
        }
        if self.origin == 0.0 {
            return;
        }
        let ext = if opcode == OP_EX_PACKET && body.len() >= 3 { u16::from_le_bytes([body[1], body[2]]) } else { 0 };
        if !use_flag(packet_id(opcode, ext), self.use_chat) {
            return;
        }
        if let Some(packet) = ReplayPacket::from_body(now - self.origin, body) {
            self.packets.push(packet);
        }
    }

    pub fn should_auto_stop(&self, now: f64) -> bool {
        self.state == RecorderState::Recording && self.origin > 0.0 && now - self.origin > AUTO_STOP_SECONDS
    }

    pub fn finish(&mut self, now: f64) -> Option<(String, Vec<u8>)> {
        if self.state != RecorderState::Recording {
            return None;
        }
        self.header.running_time = now - self.origin;
        let file = ReplayFile {
            marker: FORMAT_MARKER,
            header: self.header.clone(),
            packets: std::mem::take(&mut self.packets),
            cameras: std::mem::take(&mut self.cameras),
            user_ids: std::mem::take(&mut self.user_ids),
        };
        let name = std::mem::take(&mut self.name);
        self.state = RecorderState::Idle;
        self.origin = 0.0;
        Some((format!("{name}{EXTENSION}"), file.to_bytes()))
    }

    pub fn stop(&mut self, now: f64, write: impl FnOnce(&str, &[u8]) -> bool) -> Option<RecorderNotice> {
        let name = self.name.clone();
        let (file_name, bytes) = self.finish(now)?;
        if write(&file_name, &bytes) { Some(RecorderNotice::Saved(name)) } else { Some(RecorderNotice::Failed) }
    }

    pub fn toggle(&mut self, now: f64, exists: impl Fn(&str) -> bool, snapshot: RecordStart, write: impl FnOnce(&str, &[u8]) -> bool) -> Option<RecorderNotice> {
        match self.state {
            RecorderState::Recording => self.stop(now, write),
            RecorderState::Idle => self.start(exists, snapshot),
        }
    }
}

impl Default for Recorder {
    fn default() -> Self {
        Recorder::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayState {
    Idle,
    Loading,
    Failed,
    Done,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PlayEvent {
    LoadMap([f32; 3]),
    SetTime { hour: u8, minute: u8, second: f32 },
    Teleport([f32; 3]),
    Packet(Vec<u8>),
    FadeIn(f32),
    FadeOut,
    CameraPosition,
    Camera { a: i32, b: i32, c: i32 },
    Logo(bool),
    Ended,
}

#[derive(Debug)]
pub struct ReplayPlayer {
    file: ReplayFile,
    step: i32,
    clock: f64,
    delay: f64,
    speed: f32,
    paused: bool,
    stop_requested: bool,
    camera_instructions: bool,
    state: PlayState,
    user_ids: Vec<i32>,
}

impl ReplayPlayer {
    pub fn new(file: ReplayFile) -> ReplayPlayer {
        let user_ids = file.user_ids.clone();
        ReplayPlayer {
            file,
            step: 0,
            clock: 0.0,
            delay: 0.0,
            speed: 1.0,
            paused: false,
            stop_requested: false,
            camera_instructions: false,
            state: PlayState::Loading,
            user_ids,
        }
    }

    pub fn open(bytes: &[u8]) -> Result<ReplayPlayer, ReplayError> {
        Ok(ReplayPlayer::new(ReplayFile::parse(bytes)?))
    }

    pub fn step(&self) -> i32 {
        self.step
    }

    pub fn clock(&self) -> f64 {
        self.clock
    }

    pub fn speed(&self) -> f32 {
        self.speed
    }

    pub fn paused(&self) -> bool {
        self.paused
    }

    pub fn state(&self) -> PlayState {
        self.state
    }

    pub fn header(&self) -> &ReplayHeader {
        &self.file.header
    }

    pub fn remaining(&self) -> usize {
        self.file.packets.len()
    }

    pub fn running(&self) -> bool {
        self.step == 6
    }

    pub fn stop_playing(&mut self) {
        if self.state == PlayState::Loading && self.step == 6 {
            self.stop_requested = true;
        }
    }

    pub fn input(&mut self, key: u32, action: u32, level_loaded: bool) -> bool {
        match key {
            KEY_ESCAPE if action == ACTION_RELEASE => {
                self.stop_playing();
                true
            }
            KEY_SPACE if action == ACTION_RELEASE => {
                if level_loaded {
                    self.paused = !self.paused;
                }
                true
            }
            KEY_ADD if action == ACTION_RELEASE => {
                if self.speed < SPEED_MAX_DOUBLE {
                    self.speed += self.speed;
                }
                true
            }
            KEY_SUBTRACT if action == ACTION_RELEASE => {
                if self.speed >= SPEED_MIN_HALVE {
                    self.speed *= 0.5;
                }
                true
            }
            KEY_ESCAPE | KEY_SPACE | KEY_ADD | KEY_SUBTRACT => action == ACTION_PRESS,
            _ => false,
        }
    }

    pub fn tick(&mut self, dt: f32, pawn_ready: bool) -> Vec<PlayEvent> {
        let mut events = Vec::new();
        if self.state != PlayState::Loading {
            return events;
        }
        let dt = if self.paused { 0.0 } else { dt * self.speed };
        if self.delay > 0.0 {
            self.delay = (self.delay as f32 - dt) as f64;
            return events;
        }
        match self.step {
            0 | 1 => self.step += 1,
            2 => {
                let header = &self.file.header;
                events.push(PlayEvent::LoadMap(header.start));
                events.push(PlayEvent::SetTime { hour: header.hour, minute: header.minute, second: header.second });
                self.step += 1;
            }
            3 => {
                events.push(PlayEvent::Teleport(self.file.header.start));
                self.step += 1;
            }
            4 => {
                self.clock = 0.0;
                let mut index = 0;
                while index < self.file.packets.len() && !self.user_ids.is_empty() {
                    let known = self.file.packets[index]
                        .pawn_object_id()
                        .and_then(|object| self.user_ids.iter().position(|id| *id == object));
                    if let Some(position) = known {
                        self.user_ids.remove(position);
                        let packet = self.file.packets.remove(index);
                        events.push(PlayEvent::Packet(packet.body()));
                    }
                    index += 1;
                }
                self.step += 1;
            }
            5 => {
                if self.clock == 0.0 {
                    self.clock = CLOCK_MARK;
                    events.push(PlayEvent::FadeIn(FADE_IN_FIRST_MS));
                } else if pawn_ready {
                    self.delay = START_DELAY;
                    events.push(PlayEvent::FadeIn(FADE_IN_MS));
                    events.push(PlayEvent::CameraPosition);
                    events.push(PlayEvent::Logo(true));
                    self.step += 1;
                }
            }
            6 => self.run(dt, &mut events),
            8 => {
                events.push(PlayEvent::Logo(false));
                self.step += 1;
            }
            9 => {
                self.state = PlayState::Done;
                events.push(PlayEvent::Ended);
            }
            _ => {}
        }
        events
    }

    fn run(&mut self, dt: f32, events: &mut Vec<PlayEvent>) {
        self.clock = (dt + self.clock as f32) as f64;
        if (self.file.header.running_time as f32) < self.clock as f32 || self.stop_requested {
            events.push(PlayEvent::FadeOut);
            self.delay = START_DELAY;
            self.step = 8;
            return;
        }
        let mut index = 0;
        while index < self.file.packets.len() {
            if self.file.packets[index].time > self.clock {
                break;
            }
            if use_flag(self.file.packets[index].packet_id(), true) {
                let packet = self.file.packets.remove(index);
                events.push(PlayEvent::Packet(packet.body()));
            } else {
                index += 1;
            }
        }
        if !self.camera_instructions {
            return;
        }
        while let Some(camera) = self.file.cameras.first().copied() {
            if camera.time > self.clock {
                return;
            }
            events.push(PlayEvent::Camera { a: camera.a, b: camera.b, c: camera.c });
            self.file.cameras.remove(0);
        }
    }
}

pub fn list_names(files: &[String]) -> Vec<String> {
    let mut names: Vec<String> = files
        .iter()
        .filter_map(|file| file.strip_suffix(EXTENSION).or_else(|| file.strip_suffix(&EXTENSION.to_uppercase())))
        .map(str::to_string)
        .collect();
    names.sort();
    names
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> ReplayFile {
        let mut user_info = vec![4u8];
        for value in [100i32, 200, 300, 0, 7] {
            user_info.extend_from_slice(&value.to_le_bytes());
        }
        let mut npc = vec![0x16u8];
        npc.extend_from_slice(&9i32.to_le_bytes());
        ReplayFile {
            marker: 1,
            header: ReplayHeader { start: [1.0, 2.0, 3.0], camera: [4.0, 5.0, 6.0, 7.0], hour: 5, minute: 30, second: 12.5, running_time: 9.0, yaw: 90.0, ..ReplayHeader::default() },
            packets: vec![
                ReplayPacket::from_body(0.0, &user_info).unwrap(),
                ReplayPacket::from_body(1.0, &[1, 1, 2, 3, 4]).unwrap(),
                ReplayPacket::from_body(2.0, &npc).unwrap(),
                ReplayPacket::from_body(3.0, &[0xfe, 0x13, 0x00, 9, 9]).unwrap(),
            ],
            cameras: vec![CameraEntry { time: 1.5, a: 1, b: 2, c: 3 }],
            user_ids: vec![7],
        }
    }

    #[test]
    fn packet_id_uses_extended_offset() {
        assert_eq!(packet_id(4, 0), 4);
        assert_eq!(packet_id(0xfe, 0x13), 274);
        assert_eq!(packet_id(0xfe, 0x16), 0x115);
    }

    #[test]
    fn file_round_trips_and_is_safe() {
        let file = sample();
        let bytes = file.to_bytes();
        assert!(is_safe_package(&bytes));
        assert_eq!(&bytes[..4], &23i32.to_le_bytes());
        assert_eq!(&bytes[4..26], HEADER_TEXT.as_bytes());
        assert_eq!(ReplayFile::parse(&bytes).unwrap(), file);
        assert_eq!(&bytes[bytes.len() - 16..bytes.len() - 12], &12i32.to_le_bytes());
    }

    #[test]
    fn header_layout_is_the_engine_record() {
        let mut out = Vec::new();
        write_header(&mut out, &ReplayHeader::default());
        assert_eq!(out.len(), 4 + 7 * 4 + 2 + 4 + 8 + 5 * 4);
    }

    #[test]
    fn old_header_has_no_marker_and_no_user_ids() {
        let file = sample();
        let mut out = Vec::new();
        write_string(&mut out, HEADER_TEXT_OLD);
        write_header(&mut out, &file.header);
        out.extend_from_slice(&0i32.to_le_bytes());
        out.extend_from_slice(&0i32.to_le_bytes());
        write_string(&mut out, SAFE_PACKAGE);
        let parsed = ReplayFile::parse(&out).unwrap();
        assert_eq!(parsed.marker, 0);
        assert!(parsed.user_ids.is_empty());
    }

    #[test]
    fn rejects_unsafe_and_newer_files() {
        let file = sample();
        let mut bytes = file.to_bytes();
        let last = bytes.len() - 3;
        bytes[last] = b'X';
        assert_eq!(ReplayFile::parse(&bytes), Err(ReplayError::NotSafe));
        let mut newer = sample();
        newer.header.version = REVISION + 1;
        assert_eq!(ReplayFile::parse(&newer.to_bytes()), Err(ReplayError::NewerVersion(REVISION + 1)));
        assert_eq!(ReplayFile::parse(&[0; 4]), Err(ReplayError::NotSafe));
    }

    #[test]
    fn body_round_trips_through_record() {
        for body in [vec![1u8, 1, 2, 3], vec![0xfe, 0x13, 0x00, 5, 6], vec![0x0e]] {
            let packet = ReplayPacket::from_body(0.5, &body).unwrap();
            assert_eq!(packet.body(), body);
            assert_eq!(packet.data.len(), packet.len() + 1);
        }
    }

    #[test]
    fn pawn_object_id_follows_packet_layout() {
        let file = sample();
        assert_eq!(file.packets[0].pawn_object_id(), Some(7));
        assert_eq!(file.packets[1].pawn_object_id(), None);
        assert_eq!(file.packets[2].pawn_object_id(), Some(9));
    }

    #[test]
    fn free_name_takes_first_missing() {
        let taken = ["Rep00000.l2r", "Rep00001.l2r"];
        assert_eq!(free_name(|name| taken.contains(&name)).unwrap(), "Rep00002");
        assert_eq!(list_names(&["Rep00002.l2r".into(), "x.txt".into(), "A.l2r".into()]), vec!["A".to_string(), "Rep00002".to_string()]);
    }

    fn snapshot() -> RecordStart {
        RecordStart {
            start: [1.0, 2.0, 3.0],
            yaw: 5.0,
            camera: [0.0; 4],
            camera_mode: false,
            first_person: true,
            observer: 0,
            view_mode: 1,
            hour: 1,
            minute: 2,
            second: 3.0,
            user_ids: vec![7],
        }
    }

    #[test]
    fn recorder_waits_for_user_info_and_filters_by_table() {
        let mut recorder = Recorder::new();
        assert_eq!(recorder.start(|_| false, snapshot()), Some(RecorderNotice::Started));
        assert_eq!(recorder.name(), "Rep00000");
        recorder.add_packet(10.0, &[1, 0, 0, 0, 0]);
        assert_eq!(recorder.packet_count(), 0);
        let mut user_info = vec![4u8];
        user_info.extend_from_slice(&[0; 20]);
        recorder.add_packet(11.0, &user_info);
        recorder.add_packet(12.0, &[1, 9, 9]);
        recorder.add_packet(12.5, &[0x7e, 1]);
        recorder.add_packet(13.0, &[0x4a, 1]);
        recorder.add_packet(13.5, &[0xfe, 0x13, 0x00, 1]);
        assert_eq!(recorder.packet_count(), 4);
        assert!(!recorder.should_auto_stop(11.0 + 1800.0));
        assert!(recorder.should_auto_stop(11.0 + 1800.5));
        let mut written = None;
        let notice = recorder.stop(21.0, |name, bytes| {
            written = Some((name.to_string(), bytes.to_vec()));
            true
        });
        assert_eq!(notice, Some(RecorderNotice::Saved("Rep00000".into())));
        let (name, bytes) = written.unwrap();
        assert_eq!(name, "Rep00000.l2r");
        let file = ReplayFile::parse(&bytes).unwrap();
        assert_eq!(file.header.running_time, 10.0);
        assert_eq!(file.header.first_person, 1);
        assert_eq!(file.user_ids, vec![7]);
        assert_eq!(file.packets[0].time, 0.0);
        assert_eq!(file.packets[1].time, 1.0);
        assert_eq!(recorder.state(), RecorderState::Idle);
    }

    #[test]
    fn recorder_reports_failed_write() {
        let mut recorder = Recorder::new();
        recorder.start(|_| false, snapshot());
        assert_eq!(recorder.stop(1.0, |_, _| false), Some(RecorderNotice::Failed));
        assert_eq!(recorder.stop(1.0, |_, _| true), None);
    }

    fn run_to_step_six(player: &mut ReplayPlayer) -> Vec<PlayEvent> {
        let mut seen = Vec::new();
        for _ in 0..5 {
            seen.extend(player.tick(0.0, false));
        }
        seen.extend(player.tick(0.0, false));
        seen.extend(player.tick(0.0, true));
        assert_eq!(player.step(), 6);
        seen
    }

    #[test]
    fn player_steps_follow_the_tick_machine() {
        let mut player = ReplayPlayer::new(sample());
        let seen = run_to_step_six(&mut player);
        assert_eq!(seen.iter().filter(|event| **event == PlayEvent::Logo(true)).count(), 1);
        assert_eq!(seen[0], PlayEvent::LoadMap([1.0, 2.0, 3.0]));
        assert_eq!(seen[1], PlayEvent::SetTime { hour: 5, minute: 30, second: 12.5 });
        assert_eq!(seen[2], PlayEvent::Teleport([1.0, 2.0, 3.0]));
        assert!(matches!(&seen[3], PlayEvent::Packet(body) if body[0] == 4));
        assert_eq!(seen[4], PlayEvent::FadeIn(FADE_IN_FIRST_MS));
        assert_eq!(seen[5], PlayEvent::FadeIn(FADE_IN_MS));
        assert_eq!(seen[6], PlayEvent::CameraPosition);
        assert_eq!(player.remaining(), 3);
    }

    #[test]
    fn player_waits_one_second_then_dispatches_by_time() {
        let mut player = ReplayPlayer::new(sample());
        run_to_step_six(&mut player);
        assert!(player.tick(0.5, true).is_empty());
        assert!(player.tick(0.5, true).is_empty());
        assert!(player.tick(0.5, true).is_empty());
        let first = player.tick(1.0, true);
        assert_eq!(first, vec![PlayEvent::Packet(vec![1, 1, 2, 3, 4])]);
        let second = player.tick(1.0, true);
        assert!(matches!(&second[0], PlayEvent::Packet(body) if body[0] == 0x16));
        let third = player.tick(1.0, true);
        assert_eq!(third, vec![PlayEvent::Packet(vec![0xfe, 0x13, 0x00, 9, 9])]);
        assert_eq!(player.remaining(), 0);
    }

    #[test]
    fn player_ends_after_running_time_with_fade_and_return() {
        let mut player = ReplayPlayer::new(sample());
        run_to_step_six(&mut player);
        for _ in 0..3 {
            player.tick(0.5, true);
        }
        let mut ended = false;
        let mut faded = false;
        let mut logo_hidden = false;
        for _ in 0..40 {
            for event in player.tick(1.0, true) {
                faded |= event == PlayEvent::FadeOut;
                ended |= event == PlayEvent::Ended;
                logo_hidden |= event == PlayEvent::Logo(false);
            }
        }
        assert!(faded && ended && logo_hidden);
        assert_eq!(player.state(), PlayState::Done);
        assert!(player.tick(1.0, true).is_empty());
    }

    #[test]
    fn player_escape_pause_and_speed() {
        let mut player = ReplayPlayer::new(sample());
        assert!(player.input(KEY_ESCAPE, ACTION_RELEASE, true));
        assert!(!player.stop_requested);
        run_to_step_six(&mut player);
        assert!(player.input(KEY_SPACE, ACTION_RELEASE, true));
        assert!(player.paused());
        let before = player.clock();
        for _ in 0..3 {
            player.tick(0.5, true);
        }
        player.tick(5.0, true);
        assert_eq!(player.clock(), before);
        assert!(player.input(KEY_SPACE, ACTION_RELEASE, true));
        for _ in 0..4 {
            player.input(KEY_ADD, ACTION_RELEASE, true);
        }
        assert_eq!(player.speed(), 4.0);
        for _ in 0..4 {
            player.input(KEY_SUBTRACT, ACTION_RELEASE, true);
        }
        assert_eq!(player.speed(), 0.25);
        assert!(player.input(KEY_ADD, ACTION_PRESS, true));
        assert!(!player.input(0x41, ACTION_RELEASE, true));
        assert!(player.input(KEY_ESCAPE, ACTION_RELEASE, true));
        let mut events = Vec::new();
        for _ in 0..10 {
            events.extend(player.tick(2.0, true));
        }
        assert!(events.contains(&PlayEvent::FadeOut));
    }

    #[test]
    fn priming_skips_the_element_after_a_removal() {
        let mut file = sample();
        file.user_ids = vec![7, 9];
        file.packets.insert(1, ReplayPacket::from_body(0.1, &file.packets[2].body()).unwrap());
        let mut player = ReplayPlayer::new(file);
        let mut dispatched = Vec::new();
        for _ in 0..5 {
            dispatched.extend(player.tick(0.0, false));
        }
        let packets = dispatched.iter().filter(|event| matches!(event, PlayEvent::Packet(_))).count();
        assert_eq!(packets, 2);
    }

    #[test]
    fn invalid_header_is_rejected() {
        let mut out = Vec::new();
        write_string(&mut out, "Other");
        write_string(&mut out, SAFE_PACKAGE);
        assert_eq!(ReplayFile::parse(&out), Err(ReplayError::BadHeader));
    }
}
