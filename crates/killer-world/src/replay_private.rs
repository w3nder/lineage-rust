#[derive(Debug, Clone, PartialEq)]
pub struct RepHeader {
    pub version: i32,
    pub running_time: f32,
    pub start_location: [f32; 3],
    pub start_yaw: f32,
    pub hour: u8,
    pub minute: u8,
    pub second: f32,
    pub camera_mode: i32,
    pub first_person_mode: i32,
    pub observer_mode: i32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RepFile {
    pub header: RepHeader,
    pub packet_ids: Vec<i32>,
}

pub const TITLE: &str = "[Lineage II Replay File]";
pub const CAMERA_TITLE: &str = "[Lineage II Replay CameraMove File]";
pub const LINE_BREAK: &str = "\r\n";

fn atoi(text: &str) -> i32 {
    let text = text.trim_start();
    let (negative, rest) = match text.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, text.strip_prefix('+').unwrap_or(text)),
    };
    let digits: String = rest.chars().take_while(char::is_ascii_digit).collect();
    let value = digits.chars().fold(0_i32, |acc, ch| acc.wrapping_mul(10).wrapping_add(ch.to_digit(10).unwrap_or(0) as i32));
    if negative { value.wrapping_neg() } else { value }
}

fn atof(text: &str) -> f32 {
    let text = text.trim_start();
    let mut end = 0;
    let bytes = text.as_bytes();
    let mut seen_dot = false;
    let mut seen_exp = false;
    while end < bytes.len() {
        let ch = bytes[end];
        let accept = match ch {
            b'0'..=b'9' => true,
            b'+' | b'-' => end == 0 || matches!(bytes[end - 1], b'e' | b'E'),
            b'.' if !seen_dot && !seen_exp => {
                seen_dot = true;
                true
            }
            b'e' | b'E' if !seen_exp && end > 0 => {
                seen_exp = true;
                true
            }
            _ => false,
        };
        if !accept {
            break;
        }
        end += 1;
    }
    let mut candidate = &text[..end];
    while !candidate.is_empty() {
        if let Ok(value) = candidate.parse::<f64>() {
            return value as f32;
        }
        candidate = &candidate[..candidate.len() - 1];
    }
    0.0
}

pub fn split_lines(text: &str) -> Vec<&str> {
    text.split(LINE_BREAK).filter(|line| !line.trim().is_empty()).collect()
}

fn marker(line: &str, name: &str) -> bool {
    line.eq_ignore_ascii_case(name)
}

pub fn packet_line_id(line: &str) -> i32 {
    let text = line.trim_start();
    let digits: String = text.chars().enumerate().take_while(|(index, ch)| ch.is_ascii_digit() || (*index == 0 && matches!(ch, '-' | '+'))).map(|(_, ch)| ch).collect();
    if digits.is_empty() || digits == "-" || digits == "+" { -1 } else { atoi(&digits) }
}

pub fn parse_rep(text: &str) -> Option<RepFile> {
    let mut state = 0;
    let mut version = 0;
    let mut running_time = 0.0;
    let mut location = [0.0; 3];
    let mut yaw = 0.0;
    let mut hour = 0;
    let mut minute = 0;
    let mut second = 0.0;
    let mut camera_mode = 0;
    let mut first_person = 0;
    let mut observer = 0;
    let mut header = None;
    let mut packet_ids = Vec::new();
    for line in split_lines(text) {
        if marker(line, "REPLAY_HEADER_BEGIN") {
            if state == 0 {
                state = 1;
            }
        } else if marker(line, "REPLAY_HEADER_END") {
            if state == 1 {
                state = 2;
            }
            header = Some(RepHeader {
                version,
                running_time,
                start_location: location,
                start_yaw: yaw,
                hour,
                minute,
                second,
                camera_mode,
                first_person_mode: first_person,
                observer_mode: observer,
            });
        } else if marker(line, "REPLAY_DATA_BEGIN") {
            if state == 2 {
                state = 3;
            }
        } else if marker(line, "REPLAY_DATA_END") {
            if state == 3 {
                state = 4;
            }
        } else if state == 1 {
            if line.eq_ignore_ascii_case(TITLE) {
                continue;
            }
            let (key, value) = line.split_once('=')?;
            match key.to_ascii_lowercase().as_str() {
                "version" => version = atoi(value),
                "runningtime" => running_time = atof(value),
                "startloc.x" => location[0] = atof(value),
                "startloc.y" => location[1] = atof(value),
                "startloc.z" => location[2] = atof(value),
                "startrot.yaw" => yaw = atof(value),
                "hour" => hour = atoi(value) as u8,
                "min" => minute = atoi(value) as u8,
                "second" => second = atof(value),
                "iscameramode" => camera_mode = atoi(value),
                "isfpmode" => first_person = atoi(value),
                "isobservermode" => observer = atoi(value),
                _ => return None,
            }
        } else if state == 3 {
            let id = packet_line_id(line);
            if id < 0 {
                return None;
            }
            packet_ids.push(id);
        }
    }
    if state != 4 {
        return None;
    }
    Some(RepFile { header: header?, packet_ids })
}

#[derive(Debug, Clone, PartialEq)]
pub struct CameraHeader {
    pub version: i32,
    pub yaw: f32,
    pub pitch: f32,
    pub current_distance: f32,
    pub desired_distance: f32,
    pub fixed_number: i32,
}

pub fn parse_rcd_header(text: &str, packet_mode_blocks_data: bool) -> Option<(CameraHeader, usize)> {
    let mut state = 0;
    let mut header = CameraHeader { version: 0, yaw: 0.0, pitch: 0.0, current_distance: 0.0, desired_distance: 0.0, fixed_number: 0 };
    let mut stored = None;
    let mut entries = 0;
    for line in split_lines(text) {
        if marker(line, "REPLAY_HEADER_BEGIN") {
            if state == 0 {
                state = 1;
            }
        } else if marker(line, "REPLAY_HEADER_END") {
            if state == 1 {
                state = 2;
            }
            stored = Some(header.clone());
        } else if marker(line, "REPLAY_DATA_BEGIN") {
            if state == 2 {
                if packet_mode_blocks_data {
                    return None;
                }
                state = 3;
            }
        } else if marker(line, "REPLAY_DATA_END") {
            if state == 3 {
                state = 4;
            }
        } else if state == 1 {
            if line.eq_ignore_ascii_case(CAMERA_TITLE) {
                continue;
            }
            let (key, value) = line.split_once('=')?;
            match key.to_ascii_lowercase().as_str() {
                "version" => header.version = atoi(value),
                "camerayaw" => header.yaw = atof(value),
                "camerapitch" => header.pitch = atof(value),
                "cameracurdist" => header.current_distance = atof(value),
                "cameradesdist" => header.desired_distance = atof(value),
                "camerafixedno" => header.fixed_number = atoi(value),
                _ => return None,
            }
        } else if state == 3 && packet_line_id(line) >= 0 {
            entries += 1;
        } else if state == 3 {
            return None;
        }
    }
    (state == 4).then(|| (stored.unwrap_or(header), entries))
}

pub const BM_EXTREMES: usize = 30;
pub const BM_DISCARDED_FRAMES: usize = 60;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct BenchmarkData {
    pub frame_times: Vec<f32>,
}

impl BenchmarkData {
    pub fn reset(&mut self) {
        self.frame_times.clear();
    }

    pub fn update_frame_time(&mut self, seconds: f64) {
        self.frame_times.push(seconds as f32);
    }

    pub fn score_text(&self) -> String {
        let count = self.frame_times.len() as i32;
        if count == 0 {
            return String::new();
        }
        let mut raw = [0.0_f32; 0x4c];
        for slot in raw.iter_mut().skip(0x1f).take(BM_EXTREMES) {
            *slot = 1000.0;
        }
        let mut min_fps = 1000.0_f32;
        let mut max_fps = 0.0_f32;
        let mut total = 0.0_f32;
        for &dt in &self.frame_times {
            let fps = 1.0 / dt;
            let mut at = 0;
            while at < BM_EXTREMES && fps <= raw[at + 1] {
                at += 1;
            }
            if at < BM_EXTREMES - 1 {
                for index in ((at + 1)..=(BM_EXTREMES - 1)).rev() {
                    raw[index + 1] = raw[index];
                }
            }
            if at < BM_EXTREMES {
                raw[at + 1] = fps;
            }
            if at == BM_EXTREMES && max_fps < fps {
                max_fps = fps;
            }
            let mut low = 0;
            while low < BM_EXTREMES && raw[low + 0x1f] <= fps {
                low += 1;
            }
            if low < BM_EXTREMES - 1 {
                for index in ((low + 1)..=(BM_EXTREMES - 1)).rev() {
                    raw[index + 0x1f] = raw[index + 0x1e];
                }
            }
            if low < BM_EXTREMES {
                raw[low + 0x1f] = fps;
            }
            if low == BM_EXTREMES && min_fps > fps {
                min_fps = fps;
            }
            total += dt;
        }
        for block in 0..3 {
            let base = block * 10;
            let mut sum = f64::from(total);
            for offset in 1..=10 {
                sum -= 1.0 / f64::from(raw[base + offset]);
                sum -= 1.0 / f64::from(raw[base + 30 + offset]);
            }
            total = sum as f32;
        }
        let measured = (count - BM_DISCARDED_FRAMES as i32) as f32;
        format!(
            "MinFPS : {:.2} / AvgFPS : {:.2} / MaxFPS : {:.2} fps -- Score = {:.2}      ( Lower score is better. )",
            min_fps,
            measured / total,
            max_fps,
            (total * 10000.0) / measured
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplayScreen {
    Menu,
    BenchmarkResult,
    Play { desktop: bool },
}

pub const STATE_MENU: i32 = 1;
pub const STATE_BENCHMARK_RESULT: i32 = 5;

pub fn screen_for_state(state: i32, desktop_flag: bool) -> ReplayScreen {
    match state {
        STATE_MENU => ReplayScreen::Menu,
        STATE_BENCHMARK_RESULT => ReplayScreen::BenchmarkResult,
        _ => ReplayScreen::Play { desktop: desktop_flag },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const REP: &str = "REPLAY_HEADER_BEGIN\r\n[Lineage II Replay File]\r\nVersion=3\r\nRunningTime=12.5\r\nStartLoc.X=1.5\r\nStartLoc.Y=-2\r\nStartLoc.Z=3\r\nStartRot.Yaw=1000\r\nHour=7\r\nMin=30\r\nSecond=4.25\r\nIsCameraMode=1\r\nIsFPMode=0\r\nIsObserverMode=1\r\nREPLAY_HEADER_END\r\n\r\nREPLAY_DATA_BEGIN\r\n5\t\t0.5\t\tAction(3)\r\n   \r\n9\t\t1.0\t\tMove(4)\r\nREPLAY_DATA_END\r\n";

    #[test]
    fn the_rep_loader_reads_the_header_and_the_packet_ids_like_0x1048be80() {
        let file = parse_rep(REP).unwrap();
        assert_eq!(file.header.version, 3);
        assert_eq!(file.header.running_time, 12.5);
        assert_eq!(file.header.start_location, [1.5, -2.0, 3.0]);
        assert_eq!((file.header.start_yaw, file.header.hour, file.header.minute, file.header.second), (1000.0, 7, 30, 4.25));
        assert_eq!((file.header.camera_mode, file.header.first_person_mode, file.header.observer_mode), (1, 0, 1));
        assert_eq!(file.packet_ids, vec![5, 9]);
    }

    #[test]
    fn the_rep_loader_refuses_an_unknown_key_a_bad_data_line_and_a_file_without_the_end() {
        assert_eq!(parse_rep(&REP.replace("Version=3", "Colour=3")), None);
        assert_eq!(parse_rep(&REP.replace("9\t\t1.0", "x\t\t1.0")), None);
        assert_eq!(parse_rep(&REP.replace("REPLAY_DATA_END\r\n", "")), None);
        assert_eq!(parse_rep(&REP.replace("\r\n", "\n")), None);
        assert!(parse_rep(&REP.replace("REPLAY_HEADER_BEGIN", "replay_header_begin")).is_some());
    }

    #[test]
    fn the_camera_loader_reads_its_own_keys_0x1048ca40() {
        let text = "REPLAY_HEADER_BEGIN\r\n[Lineage II Replay CameraMove File]\r\nVersion=2\r\nCameraYaw=1.5\r\nCameraPitch=2.5\r\nCameraCurDist=3\r\nCameraDesDist=4\r\nCameraFixedNo=6\r\nREPLAY_HEADER_END\r\nREPLAY_DATA_BEGIN\r\n1\t\t0.5\t\tA (1)_2\t\t0.5\r\nREPLAY_DATA_END\r\n";
        let (header, entries) = parse_rcd_header(text, false).unwrap();
        assert_eq!((header.version, header.yaw, header.pitch, header.current_distance, header.desired_distance, header.fixed_number), (2, 1.5, 2.5, 3.0, 4.0, 6));
        assert_eq!(entries, 1);
        assert_eq!(parse_rcd_header(text, true), None);
    }

    #[test]
    fn the_score_of_no_frames_is_empty_and_of_a_steady_run_is_the_framerate_0x10489b10() {
        let mut data = BenchmarkData::default();
        assert_eq!(data.score_text(), "");
        for _ in 0..200 {
            data.update_frame_time(0.01);
        }
        let text = data.score_text();
        assert!(text.starts_with("MinFPS : "), "{text}");
        assert!(text.ends_with("( Lower score is better. )"));
        data.reset();
        assert!(data.frame_times.is_empty());
    }

    #[test]
    fn the_screen_follows_the_manager_state_0x100f8af0() {
        assert_eq!(screen_for_state(1, false), ReplayScreen::Menu);
        assert_eq!(screen_for_state(5, true), ReplayScreen::BenchmarkResult);
        assert_eq!(screen_for_state(3, true), ReplayScreen::Play { desktop: true });
    }
}
