use glam::Vec3;

#[derive(Clone, Copy, Debug)]
pub struct Target {
    pub position: Vec3,
    pub actor: Option<u64>,
    pub tick: bool,
}

pub trait Host {
    fn tick(&mut self, actor: u64, delta: f32);
    fn position(&self, actor: u64) -> Option<Vec3>;
    fn ended(&self, actor: u64) -> Option<bool>;
    fn destroy(&mut self, actor: u64);
    fn move_actor(&mut self, position: Vec3, delta: Vec3) -> Vec3;
}

#[derive(Clone, Debug)]
pub struct Mover {
    pub position: Vec3,
    pub targets: Vec<Target>,
    pub index: usize,
    pub flags: u32,
    pub start_delay: f32,
    pub speed: f32,
    pub current_speed: f32,
    pub acceleration: f32,
    pub acceleration_rate: f32,
    pub current_acceleration: f32,
    pub max_speed: f32,
    pub min_speed: f32,
    pub max_acceleration: f32,
    pub min_acceleration: f32,
}

impl Mover {
    pub fn new(position: Vec3) -> Self {
        Self { position, targets: Vec::new(), index: 0, flags: 0, start_delay: 0.0,
            speed: 100.0, current_speed: 0.0, acceleration: 0.0, acceleration_rate: 0.0,
            current_acceleration: 0.0, max_speed: 2000.0, min_speed: 1.0,
            max_acceleration: 1000.0, min_acceleration: -1000.0 }
    }

    pub fn start(&mut self, delay: f32, looping: bool) {
        self.start_delay = delay;
        self.flags = (self.flags & !13) | u32::from(looping) | 2;
        self.current_speed = self.speed;
        self.current_acceleration = self.acceleration;
        self.index = 0;
    }

    pub fn ended(&self) -> bool { self.flags & 4 != 0 }

    pub fn end(&mut self, host: &mut impl Host) {
        for target in &self.targets {
            if let Some(actor) = target.actor.filter(|_| target.tick) { host.destroy(actor); }
        }
        self.targets.clear();
        self.flags |= 4;
    }

    pub fn tick(&mut self, delta: f32, host: &mut impl Host) -> bool {
        if self.index >= self.targets.len() || self.flags & 2 == 0 || self.flags & 12 != 0 || delta <= 0.0 {
            return false;
        }
        for target in &self.targets {
            if let Some(actor) = target.actor.filter(|_| target.tick)
                && host.ended(actor).is_some() { host.tick(actor, delta); }
        }
        let mut remaining = delta;
        while remaining > 0.0 {
            let target = self.targets[self.index];
            let destination = target.actor.and_then(|actor| host.position(actor)).unwrap_or(target.position);
            let step = remaining.min(0.2);
            self.current_acceleration = (step as f64 * self.acceleration_rate as f64 + self.current_acceleration as f64) as f32;
            if self.current_acceleration > self.max_acceleration { self.current_acceleration = self.max_acceleration; }
            if self.current_acceleration < self.min_acceleration { self.current_acceleration = self.min_acceleration; }
            self.current_speed = (step as f64 * self.current_acceleration as f64 + self.current_speed as f64) as f32;
            if self.current_speed > self.max_speed { self.current_speed = self.max_speed; }
            if self.current_speed < self.min_speed { self.current_speed = self.min_speed; }
            let difference = destination - self.position;
            let direction = safe_normal(difference);
            let distance = step as f64 * self.current_speed as f64;
            let movement = Vec3::from(direction.to_array().map(|v| (v as f64 * distance) as f32));
            let length = size(movement);
            if length < size(difference) as f32 as f64 || length < 0.0 {
                self.position = host.move_actor(self.position, movement);
                remaining -= step;
            } else {
                self.position = host.move_actor(self.position, difference);
                remaining = (remaining as f64 - size(destination - self.position) as f32 as f64 / length * step as f64) as f32;
                if target.actor.and_then(|actor| host.ended(actor)) == Some(false) { return true; }
                self.index += 1;
                if self.index >= self.targets.len() {
                    if self.flags & 1 != 0 { self.index = 0; } else { self.end(host); }
                    return true;
                }
            }
        }
        true
    }
}

fn size(v: Vec3) -> f64 {
    let [x,y,z] = v.to_array().map(f64::from);
    (x*x + y*y + z*z).sqrt()
}

fn safe_normal(v: Vec3) -> Vec3 {
    let [x,y,z] = v.to_array().map(f64::from);
    let squared = x*x + y*y + z*z;
    if squared < 1e-8 { return Vec3::ZERO; }
    let inverse = 1.0 / squared.sqrt();
    Vec3::new(v.x * inverse as f32, v.y * inverse as f32, (z * inverse) as f32)
}
