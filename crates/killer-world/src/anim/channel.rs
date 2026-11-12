use super::notify::{NotifyKind, SeqNotify};

#[derive(Debug, Clone, PartialEq, Default)]
pub struct SeqInfo {
    pub frames: u32,
    pub rate: f32,
    pub shots: Vec<f32>,
    pub notifies: Vec<SeqNotify>,
    pub attack_eff: f32,
    pub attack_end_eff: f32,
}

impl SeqInfo {
    pub fn new(frames: u32, rate: f32) -> Self {
        SeqInfo { frames, rate, shots: Vec::new(), notifies: Vec::new(), attack_eff: 0.0, attack_end_eff: 0.0 }
    }

    pub fn with_attack_frames(mut self, eff: f32, end: f32) -> Self {
        self.attack_eff = eff;
        self.attack_end_eff = end;
        self
    }

    pub fn attack_eff_frame(&self) -> f32 {
        self.attack_eff
    }

    pub fn attack_end_eff_frame(&self) -> f32 {
        self.attack_end_eff
    }

    pub fn notify_time(&self, pre_shot: bool) -> f32 {
        let wanted = if pre_shot { NotifyKind::AttackPreShot } else { NotifyKind::AttackShot };
        self.notifies.iter().filter(|notify| notify.kind == wanted).map(|notify| notify.time).next_back().unwrap_or(0.0)
    }

    pub fn with_shots(frames: u32, rate: f32, shots: &[f32]) -> Self {
        let notifies = shots.iter().map(|&time| SeqNotify { time, kind: NotifyKind::AttackShot }).collect();
        SeqInfo { frames, rate, shots: shots.to_vec(), notifies, attack_eff: 0.0, attack_end_eff: 0.0 }
    }

    pub fn with_notifies(frames: u32, rate: f32, notifies: Vec<SeqNotify>) -> Self {
        let shots = notifies.iter().filter(|notify| notify.kind == NotifyKind::AttackShot).map(|notify| notify.time).collect();
        SeqInfo { frames, rate, shots, notifies, attack_eff: 0.0, attack_end_eff: 0.0 }
    }

    pub fn seconds(&self) -> f32 {
        if self.rate > 0.0 { self.frames as f32 / self.rate } else { 0.0 }
    }

    pub fn first_shot(&self) -> Option<f32> {
        self.shots.first().copied()
    }

    pub fn last_shot(&self) -> Option<f32> {
        self.shots.last().copied()
    }
}

pub trait Sequences {
    fn seq(&self, name: &str) -> Option<SeqInfo>;
}

impl<F: Fn(&str) -> Option<SeqInfo>> Sequences for F {
    fn seq(&self, name: &str) -> Option<SeqInfo> {
        self(name)
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Channel {
    pub sequence: Option<String>,
    pub frames: u32,
    pub frame: f32,
    pub rate: f32,
    pub last: f32,
    pub tween_rate: f32,
    pub old_rate: f32,
    pub base_rate: f32,
    pub looping: bool,
    pub finished: bool,
    pub shots: Vec<f32>,
    pub notifies: Vec<SeqNotify>,
    pub attack_eff_frame: f32,
    pub attack_end_eff_frame: f32,
    pub pre_shot_time: f32,
    pub shot_time: f32,
    pub notify_flag: bool,
    pub notify_disabled: bool,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Step {
    pub ended: bool,
    pub wrapped: bool,
    pub fired: Vec<SeqNotify>,
}

impl Step {
    pub fn shot(&self) -> bool {
        self.fired.iter().any(|notify| notify.kind == NotifyKind::AttackShot)
    }
}

const MAX_ITERATIONS: u32 = 4;

impl Channel {
    pub fn play(&mut self, name: &str, seq: &SeqInfo, rate: f32, tween: f32, looping: bool) -> bool {
        if seq.frames == 0 {
            return false;
        }
        let frames = seq.frames as f32;
        let cycles = seq.rate / frames;
        let same = self.sequence.as_deref().is_some_and(|current| current.eq_ignore_ascii_case(name));
        if looping {
            if same && self.looping && self.is_animating() {
                self.finished = false;
                self.base_rate = cycles;
                self.rate = cycles * rate;
                self.old_rate = self.rate;
                return true;
            }
            self.start(name, seq, cycles, rate, looping);
            self.tween_into(tween, frames);
            return true;
        }
        if rate <= 0.0 {
            if rate != 0.0 {
                return false;
            }
            self.sequence = Some(name.to_string());
            self.frames = seq.frames;
            self.shots = seq.shots.clone();
            self.notifies = seq.notifies.clone();
        self.attack_eff_frame = seq.attack_eff_frame();
        self.attack_end_eff_frame = seq.attack_end_eff_frame();
        self.pre_shot_time = seq.notify_time(true);
        self.shot_time = seq.notify_time(false);
            self.notify_flag = false;
            self.last = 0.0;
            self.finished = false;
            self.looping = false;
            self.rate = 0.0;
            self.old_rate = 0.0;
            self.base_rate = cycles;
            if tween <= 0.0 {
                self.tween_rate = 0.0;
                self.frame = 0.0;
            } else {
                self.tween_rate = 1.0 / (tween * frames);
                self.frame = -1.0 / frames;
            }
            return true;
        }
        let tween = if self.sequence.is_none() { 0.0 } else { tween };
        self.start(name, seq, cycles, rate, looping);
        self.tween_into(tween, frames);
        true
    }

    fn start(&mut self, name: &str, seq: &SeqInfo, cycles: f32, rate: f32, looping: bool) {
        let frames = seq.frames as f32;
        self.sequence = Some(name.to_string());
        self.frames = seq.frames;
        self.shots = seq.shots.clone();
        self.notifies = seq.notifies.clone();
        self.attack_eff_frame = seq.attack_eff_frame();
        self.attack_end_eff_frame = seq.attack_end_eff_frame();
        self.pre_shot_time = seq.notify_time(true);
        self.shot_time = seq.notify_time(false);
        self.base_rate = cycles;
        self.rate = cycles * rate;
        self.last = 1.0 - 1.0 / frames;
        self.notify_flag = !self.notifies.is_empty() && self.last > 0.0;
        self.finished = false;
        self.looping = looping;
    }

    fn tween_into(&mut self, tween: f32, frames: f32) {
        if self.last == 0.0 {
            self.old_rate = 0.0;
            self.tween_rate = if tween > 0.0 { 1.0 / tween } else { 10.0 };
            self.frame = -1.0 / frames;
            self.rate = 0.0;
        } else if tween > 0.0 {
            self.old_rate = self.rate;
            self.tween_rate = 1.0 / (tween * frames);
            self.frame = -1.0 / frames;
        } else if tween == -1.0 {
            self.frame = -1.0 / frames;
            self.tween_rate = if self.old_rate > 0.0 {
                self.old_rate
            } else if self.old_rate == 0.0 {
                1.0 / (frames * 0.025)
            } else {
                (self.rate * 0.5).max(-self.old_rate)
            };
            self.old_rate = self.rate;
        } else {
            self.old_rate = self.rate;
            self.tween_rate = 0.0;
            self.frame = 0.0001;
        }
    }

    pub fn set_rate(&mut self, rate: f32) {
        self.rate = rate * self.base_rate;
    }

    pub fn is_animating(&self) -> bool {
        self.sequence.is_some() && (if self.frame < 0.0 { self.tween_rate } else { self.rate }) != 0.0
    }

    pub fn is_tweening(&self) -> bool {
        self.sequence.is_some() && self.frame < 0.0
    }

    pub fn tween_alpha(&self) -> Option<f32> {
        (self.sequence.is_some() && self.frame < 0.0).then(|| (1.0 + self.frame * self.frames as f32).clamp(0.0, 1.0))
    }

    pub fn key_frame(&self) -> f32 {
        if self.frame < 0.0 { 0.0 } else { self.frame * self.frames as f32 }
    }

    pub fn advance(&mut self, dt: f32) -> Step {
        let mut result = Step::default();
        let mut dt = dt;
        let mut iterations = 0;
        while self.sequence.is_some() && self.is_animating() && dt > 0.0 {
            iterations += 1;
            if iterations > MAX_ITERATIONS {
                break;
            }
            let old = self.frame;
            if self.frame >= 0.0 {
                self.frame += dt * self.rate;
                if self.notify_flag && !self.notify_disabled {
                    let new = self.frame;
                    let mut shot_seen = false;
                    let fired: Vec<usize> = (0..self.notifies.len())
                        .filter(|&index| {
                            let time = self.notifies[index].time;
                            if time <= old || time > new {
                                return false;
                            }
                            if self.notifies[index].kind == NotifyKind::AttackShot {
                                if shot_seen {
                                    return false;
                                }
                                shot_seen = true;
                            }
                            true
                        })
                        .collect();
                    for index in fired {
                        let time = self.notifies[index].time;
                        dt = ((self.frame - time) * dt) / (self.frame - old);
                        self.frame = time;
                        result.fired.push(self.notifies[index].clone());
                    }
                }
                if self.frame < self.last {
                    break;
                }
                if !self.looping {
                    self.frame = self.last;
                    self.finished = true;
                    if !self.notify_disabled && self.rate > 0.0 {
                        result.ended = true;
                    }
                    self.rate = 0.0;
                    break;
                }
                if self.frame >= 1.0 {
                    let reached = self.frame;
                    dt = (reached - 1.0) * dt / (reached - old);
                    self.frame = 0.0;
                    result.wrapped = true;
                } else {
                    dt = 0.0;
                }
            } else {
                self.frame += dt * self.tween_rate;
                if self.frame < 0.0 {
                    break;
                }
                let reached = self.frame;
                self.frame = 0.0;
                dt = dt * reached / (reached - old);
                if self.rate == 0.0 {
                    result.ended = true;
                    break;
                }
            }
        }
        result
    }
}

pub fn force_anim_rate(channels: &mut [Channel], channel: i32, rate: f32) {
    if let Some(target) = usize::try_from(channel).ok().and_then(|index| channels.get_mut(index)) {
        target.rate = rate;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seq(frames: u32, rate: f32) -> SeqInfo {
        SeqInfo::new(frames, rate)
    }

    #[test]
    fn play_sets_rate_last_and_tween_like_the_engine() {
        let mut channel = Channel::default();
        assert!(channel.play("Walk", &seq(12, 15.0), 1.0, 0.2, true));
        assert_eq!(channel.frames, 12);
        assert!((channel.base_rate - 1.25).abs() < 1e-6);
        assert!((channel.rate - 1.25).abs() < 1e-6);
        assert!((channel.last - (1.0 - 1.0 / 12.0)).abs() < 1e-6);
        assert!((channel.frame + 1.0 / 12.0).abs() < 1e-6);
        assert!((channel.tween_rate - 1.0 / (0.2 * 12.0)).abs() < 1e-6);
        assert!(channel.looping && channel.is_tweening() && channel.is_animating());
        assert_eq!(channel.tween_alpha(), Some(0.0));
        assert_eq!(channel.key_frame(), 0.0);
    }

    #[test]
    fn first_play_ignores_the_tween_and_later_plays_tween_in() {
        let mut channel = Channel::default();
        assert!(channel.play("Sit", &seq(41, 12.0), 1.27451, 0.1, false));
        assert!((channel.frame - 0.0001).abs() < 1e-7);
        assert_eq!(channel.tween_rate, 0.0);
        assert!(channel.play("Stand", &seq(41, 15.0), 1.0, 0.1, false));
        assert!(channel.is_tweening());
        assert!((channel.tween_rate - 1.0 / (0.1 * 41.0)).abs() < 1e-6);
    }

    #[test]
    fn looping_the_same_sequence_only_changes_the_rate() {
        let mut channel = Channel::default();
        channel.play("Wait", &seq(10, 3.0), 1.0, 0.0, true);
        channel.advance(1.0);
        let frame = channel.frame;
        assert!(frame > 0.0);
        assert!(channel.play("wait", &seq(10, 3.0), 2.0, 0.2, true));
        assert_eq!(channel.frame, frame);
        assert!((channel.rate - 0.6).abs() < 1e-6);
        assert!(!channel.is_tweening());
    }

    #[test]
    fn force_anim_rate_106757c0_writes_the_rate_without_the_base_and_ignores_bad_channels() {
        let mut channels = vec![Channel::default(), Channel::default()];
        channels[1].play("Run", &seq(13, 15.0), 1.0, 0.0, true);
        force_anim_rate(&mut channels, 1, 0.75);
        assert_eq!(channels[1].rate, 0.75);
        force_anim_rate(&mut channels, -1, 9.0);
        force_anim_rate(&mut channels, 2, 9.0);
        assert_eq!(channels[0].rate, 0.0);
        assert_eq!(channels[1].rate, 0.75);
    }

    #[test]
    fn set_rate_scales_the_base_rate() {
        let mut channel = Channel::default();
        channel.play("Run", &seq(13, 15.0), 1.0, 0.0, true);
        channel.set_rate(1.42857);
        assert!((channel.rate - 1.42857 * 15.0 / 13.0).abs() < 1e-5);
    }

    #[test]
    fn tween_reaches_frame_zero_after_the_tween_time_and_spends_the_rest() {
        let mut channel = Channel::default();
        channel.play("A", &seq(10, 10.0), 1.0, 0.0, false);
        channel.play("B", &seq(10, 10.0), 1.0, 0.5, false);
        let step = channel.advance(0.25);
        assert_eq!(step, Step::default());
        assert_eq!(channel.tween_alpha(), Some(0.5));
        channel.advance(0.35);
        assert!(!channel.is_tweening());
        assert!((channel.frame - 0.1).abs() < 1e-5, "{}", channel.frame);
    }

    #[test]
    fn one_shot_ends_at_the_last_key_and_stops() {
        let mut channel = Channel::default();
        channel.play("Death", &seq(51, 15.0), 1.0, 0.0, false);
        assert!(!channel.advance(1.0).ended);
        let step = channel.advance(10.0);
        assert!(step.ended && channel.finished);
        assert!((channel.frame - channel.last).abs() < 1e-6);
        assert_eq!(channel.rate, 0.0);
        assert!(!channel.is_animating());
        assert!((channel.key_frame() - 50.0).abs() < 1e-4);
    }

    #[test]
    fn loop_wraps_past_one_and_keeps_the_remainder() {
        let mut channel = Channel::default();
        channel.play("Wait", &seq(10, 10.0), 1.0, 0.0, true);
        let step = channel.advance(1.25);
        assert!(step.wrapped && !step.ended);
        assert!((channel.frame - 0.2501).abs() < 1e-4, "{}", channel.frame);
        let mut long = Channel::default();
        long.play("Wait", &seq(10, 10.0), 1.0, 0.0, true);
        long.advance(0.95);
        assert!(long.frame >= long.last && long.frame < 1.0);
        assert!((long.key_frame() - 9.501).abs() < 1e-3, "{}", long.key_frame());
    }

    #[test]
    fn attack_shot_fires_once_when_the_frame_crosses_it() {
        let mut channel = Channel::default();
        channel.play("Atk01", &SeqInfo::with_shots(20, 20.0, &[0.5]), 1.0, 0.0, false);
        assert!(!channel.advance(0.4).shot());
        assert!(channel.advance(0.2).shot());
        assert!(!channel.advance(0.1).shot());
    }

    #[test]
    fn zero_rate_freezes_the_first_frame_after_the_tween() {
        let mut channel = Channel::default();
        channel.play("Wait", &seq(10, 3.0), 1.0, 0.0, true);
        assert!(channel.play("Land", &seq(10, 3.0), 0.0, 0.5, false));
        assert!(channel.is_tweening());
        let step = channel.advance(0.6);
        assert!(step.ended);
        assert_eq!(channel.frame, 0.0);
        assert!(!channel.is_animating());
        assert!(!channel.play("Land", &seq(10, 3.0), -1.0, 0.0, false));
        assert!(!channel.play("Land", &seq(0, 3.0), 1.0, 0.0, false));
    }

    #[test]
    fn sequences_can_be_a_closure() {
        let table = |name: &str| (name == "wait").then_some(seq(10, 3.0));
        assert_eq!(table.seq("wait"), Some(seq(10, 3.0)));
        assert_eq!(table.seq("run"), None);
    }

    fn marked(frames: u32, rate: f32, times: &[(f32, NotifyKind)]) -> SeqInfo {
        SeqInfo::with_notifies(frames, rate, times.iter().map(|(time, kind)| SeqNotify { time: *time, kind: kind.clone() }).collect())
    }

    #[test]
    fn update_animation_1067a290_stops_the_frame_at_the_last_notify_crossed() {
        let mut channel = Channel::default();
        channel.play("A", &marked(10, 10.0, &[(0.3, NotifyKind::AttackItem), (0.6, NotifyKind::SwimSound)]), 1.0, 0.0, false);
        assert!(channel.notify_flag);
        let step = channel.advance(0.7);
        assert_eq!(step.fired.iter().map(|notify| notify.time).collect::<Vec<_>>(), [0.3, 0.6]);
        assert_eq!(channel.frame, 0.6);
        assert!(!step.ended);
        let step = channel.advance(0.1);
        assert!(step.fired.is_empty());
        assert!(channel.frame > 0.6);
    }

    #[test]
    fn update_animation_1067a290_fires_only_the_first_attack_shot_of_a_step() {
        let mut channel = Channel::default();
        channel.play("A", &marked(10, 10.0, &[(0.3, NotifyKind::AttackShot), (0.4, NotifyKind::AttackShot), (0.5, NotifyKind::AttackItem)]), 1.0, 0.0, false);
        let step = channel.advance(0.8);
        assert_eq!(step.fired.iter().map(|notify| notify.time).collect::<Vec<_>>(), [0.3, 0.5]);
    }

    #[test]
    fn update_animation_1067a290_fires_notifies_in_the_half_open_interval_after_the_old_frame() {
        let mut channel = Channel::default();
        channel.play("A", &marked(10, 10.0, &[(0.4, NotifyKind::AttackItem)]), 1.0, 0.0, true);
        channel.frame = 0.4;
        assert!(channel.advance(0.1).fired.is_empty());
        channel.frame = 0.35;
        assert_eq!(channel.advance(0.1).fired.len(), 1);
    }

    #[test]
    fn enable_channel_notify_106750c0_off_silences_notifies_and_the_anim_end() {
        let mut channel = Channel::default();
        channel.play("A", &marked(10, 10.0, &[(0.3, NotifyKind::AttackItem)]), 1.0, 0.0, false);
        channel.notify_disabled = true;
        let step = channel.advance(5.0);
        assert!(step.fired.is_empty());
        assert!(!step.ended);
        assert!(channel.finished);
    }

    #[test]
    fn play_10673bd0_leaves_notify_off_for_one_frame_sequences_and_sequences_without_notifies() {
        let mut channel = Channel::default();
        channel.play("A", &marked(1, 10.0, &[(0.3, NotifyKind::AttackItem)]), 1.0, 0.0, false);
        assert!(!channel.notify_flag);
        channel.play("B", &seq(10, 10.0), 1.0, 0.0, false);
        assert!(!channel.notify_flag);
    }
}
