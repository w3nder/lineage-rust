use super::VertexSequence;

#[derive(Clone, Debug, Default, PartialEq)]
pub struct VertexAnimation {
    pub sequence: Option<usize>,
    pub frame: f32,
    pub rate: f32,
    pub last: f32,
    pub tween_rate: f32,
    pub old_rate: f32,
    pub original_rate: f32,
    pub looping: bool,
    pub notify: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VertexAnimEvent {
    End,
    Notify(usize),
}

impl VertexAnimation {
    pub fn is_animating(&self) -> bool {
        self.sequence.is_some() && if self.frame < 0.0 { self.tween_rate != 0.0 } else { self.rate != 0.0 }
    }

    pub fn play(&mut self, sequences: &[VertexSequence], name: &str, rate: f32, mut tween: f32, looping: bool, speed: f32) -> bool {
        let Some(index) = sequences.iter().position(|s| s.name.eq_ignore_ascii_case(name)) else { return false };
        let seq = &sequences[index];
        if seq.frames == 0 { return false; }
        let frames = seq.frames as f32;
        if !looping && rate <= 0.0 {
            if rate != 0.0 { return false; }
            self.sequence = Some(index);
            self.last = 0.0;
            self.notify = false;
            self.looping = false;
            self.rate = 0.0;
            self.old_rate = 0.0;
            self.tween_rate = if tween > 0.0 { 1.0 / (tween * frames) } else { 0.0 };
            self.frame = if tween > 0.0 { -1.0 / frames } else { 0.0 };
            return true;
        }
        if !looping && self.sequence.is_none() { tween = 0.0; }
        if looping && self.sequence == Some(index) && self.looping && self.is_animating() {
            self.original_rate = seq.rate / frames;
            self.rate = self.original_rate * rate;
            self.old_rate = self.rate;
            return true;
        }
        self.sequence = Some(index);
        self.original_rate = seq.rate / frames;
        self.rate = self.original_rate * rate;
        self.last = 1.0 - 1.0 / frames;
        self.looping = looping;
        self.notify = !seq.notifies.is_empty();
        if self.last == 0.0 {
            self.notify = false;
            self.old_rate = 0.0;
            self.tween_rate = if tween > 0.0 { 1.0 / tween } else { 10.0 };
            self.rate = 0.0;
            self.frame = -1.0 / frames;
        } else if tween > 0.0 {
            self.tween_rate = 1.0 / (frames * tween);
            self.frame = -1.0 / frames;
        } else if tween != -1.0 {
            self.frame = if looping { 0.0001 } else { 0.001 };
            self.tween_rate = 0.0;
        } else {
            self.frame = -1.0 / frames;
            self.tween_rate = if self.old_rate > 0.0 { self.old_rate }
                else if self.old_rate < 0.0 { (self.rate * 0.5).max(-speed * self.old_rate) }
                else { 1.0 / (frames * 0.025) };
        }
        self.old_rate = self.rate;
        true
    }

    pub fn update(&mut self, sequences: &[VertexSequence], mut delta: f32, speed: f32, notifies_enabled: bool, mut event: impl FnMut(&mut Self, VertexAnimEvent) -> bool) {
        for _ in 0..4 {
            if !self.is_animating() || delta <= 0.0 { break; }
            let previous = self.frame;
            if previous < 0.0 {
                self.frame += delta * self.tween_rate;
                if self.frame < 0.0 { break; }
                let frame = self.frame;
                self.frame = 0.0;
                delta = delta * frame / (frame - previous);
                if self.rate == 0.0 && !event(self, VertexAnimEvent::End) { break; }
                continue;
            }
            let rate = if self.rate < 0.0 { (-speed * self.rate).max(0.3) } else { self.rate };
            self.frame += delta * rate;
            if self.notify && notifies_enabled && let Some(sequence) = self.sequence.and_then(|i| sequences.get(i)) {
                let mut next = None;
                let mut distance = 100000.0;
                for (index, &(time, _, _)) in sequence.notifies.iter().enumerate() {
                    let span = f64::from(time) - f64::from(previous);
                    if time > previous && time <= self.frame && (next.is_none() || span < f64::from(distance)) {
                        next = Some((index, time));
                        distance = span as f32;
                    }
                }
                if let Some((index, time)) = next {
                    delta = (self.frame - time) * delta / (self.frame - previous);
                    self.frame = time;
                    if sequence.notifies[index].2 != ue2_package::ObjectRef::Null && !event(self, VertexAnimEvent::Notify(index)) { break; }
                    continue;
                }
            }
            if self.frame < self.last { break; }
            if !self.looping {
                self.rate = 0.0;
                delta = (self.frame - self.last) * delta / (self.frame - previous);
                self.frame = self.last;
                if !event(self, VertexAnimEvent::End) { break; }
            } else {
                if self.frame >= 1.0 {
                    delta = (self.frame - 1.0) * delta / (self.frame - previous);
                    self.frame = 0.0;
                } else { delta = 0.0; }
                if previous < self.last && !event(self, VertexAnimEvent::End) { break; }
            }
        }
    }
}
