use crate::client::{AssetError, Client};
use crate::timeenv::TimeEnv;

pub const ENV_TYPES: usize = 3;
const DAY_HOURS: f32 = 24.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeEnvError {
    NegativeRamp,
    RampBeyondEnd,
    TargetOutOfRange,
}

impl std::fmt::Display for ChangeEnvError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ChangeEnv recusado: {self:?}")
    }
}

impl std::error::Error for ChangeEnvError {}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EnvState {
    current: u8,
    previous: u8,
    start: f32,
    end: f32,
    ramp: f32,
    target: f32,
    previous_target: f32,
    delta: f32,
}

impl Default for EnvState {
    fn default() -> Self {
        EnvState { current: 0, previous: 0, start: 0.0, end: 0.0, ramp: 0.0, target: 1.0, previous_target: 1.0, delta: 0.0 }
    }
}

impl EnvState {
    pub fn current(&self) -> u8 {
        self.current
    }

    pub fn previous(&self) -> u8 {
        self.previous
    }

    pub fn start(&self) -> f32 {
        self.start
    }

    pub fn end(&self) -> f32 {
        self.end
    }

    pub fn delta(&self) -> f32 {
        self.delta
    }

    pub fn change(&mut self, env: u8, immediate: bool, end_after: f32, ramp_after: f32, target: f32, now: f32) -> Result<(), ChangeEnvError> {
        if immediate {
            self.current = env;
            self.previous = env;
            self.start = now;
            self.end = now;
            self.ramp = now;
            self.target = 1.0;
            self.previous_target = 1.0;
            return Ok(());
        }
        if ramp_after < 0.0 {
            return Err(ChangeEnvError::NegativeRamp);
        }
        if ramp_after > end_after + now {
            return Err(ChangeEnvError::RampBeyondEnd);
        }
        if !(0.0..=1.0).contains(&target) {
            return Err(ChangeEnvError::TargetOutOfRange);
        }
        self.previous = self.current;
        self.current = env;
        self.previous_target = self.target;
        self.start = now;
        self.end = end_after + now;
        if env != 0 {
            self.ramp = ramp_after + now;
            self.target = target;
        } else {
            self.ramp = end_after + now;
            self.target = 1.0;
        }
        Ok(())
    }

    pub fn update(&mut self, time: f32) {
        if self.previous != self.current && self.end < time {
            self.previous = self.current;
            self.previous_target = self.target;
        }
        self.delta = self.calc_delta(time);
    }

    pub fn calc_delta(&self, time: f32) -> f32 {
        let (start, mut end, ramp, target) = (self.start, self.end, self.ramp, self.target);
        if time < start || time > end {
            return target;
        }
        let mut time = time;
        if end < start {
            end += DAY_HOURS;
        }
        if time < start {
            time += DAY_HOURS;
        }
        if time < ramp {
            return (time - start) / (ramp - start);
        }
        if time == ramp || target == 1.0 {
            return 1.0;
        }
        let scale = target / (1.0 - target) + 1.0;
        1.0 - (time - ramp) / ((end - ramp) * scale)
    }

    pub fn mix(&self, time: f32, base: f32, current: f32, previous: f32) -> f32 {
        if self.current == self.previous {
            if self.current == 0 {
                return base;
            }
            return (current - base) * self.delta + base;
        }
        if time < self.ramp {
            let held = (previous - base) * self.previous_target + base;
            return (current - held) * self.delta + held;
        }
        (current - base) * self.delta + base
    }

    pub fn mix3(&self, time: f32, base: [f32; 3], current: [f32; 3], previous: [f32; 3]) -> [f32; 3] {
        std::array::from_fn(|i| self.mix(time, base[i], current[i], previous[i]))
    }
}

#[derive(Debug, Clone)]
pub struct Weather {
    envs: Vec<TimeEnv>,
    pub state: EnvState,
}

impl Weather {
    pub fn new(base: TimeEnv) -> Weather {
        Weather { envs: vec![base], state: EnvState::default() }
    }

    pub fn load(client: &mut Client) -> Result<Weather, AssetError> {
        let mut envs = Vec::with_capacity(ENV_TYPES);
        for env_type in 0..ENV_TYPES as u8 {
            envs.push(TimeEnv::load(client, env_type)?);
        }
        Ok(Weather { envs, state: EnvState::default() })
    }

    pub fn with_types(envs: Vec<TimeEnv>) -> Weather {
        Weather { envs, state: EnvState::default() }
    }

    pub fn base(&self) -> &TimeEnv {
        &self.envs[0]
    }

    pub fn env(&self, env_type: u8) -> Option<&TimeEnv> {
        self.envs.get(usize::from(env_type))
    }

    pub fn has(&self, env_type: u8) -> bool {
        usize::from(env_type) < self.envs.len()
    }

    pub fn change_env(&mut self, env: u8, immediate: bool, end_after: f32, ramp_after: f32, target: f32, now: f32) -> Result<(), ChangeEnvError> {
        if !immediate && !self.has(env) {
            return Ok(());
        }
        self.state.change(env, immediate, end_after, ramp_after, target, now)
    }

    pub fn mix3(&self, hour: f32, get: impl Fn(&TimeEnv) -> [f32; 3]) -> [f32; 3] {
        let base = get(self.base());
        let value = |env: u8| self.env(env).map_or([0.0; 3], &get);
        self.state.mix3(hour, base, value(self.state.current()), value(self.state.previous()))
    }

    pub fn scale(&self, hour: f32, get: impl Fn(&TimeEnv, f32) -> Option<f32>) -> Option<f32> {
        let state = &self.state;
        if state.current() == state.previous() {
            return self.env(state.current()).and_then(|env| get(env, hour));
        }
        let from = self.env(state.previous()).and_then(|env| get(env, state.start())).unwrap_or(0.0);
        let to = self.env(state.current()).and_then(|env| get(env, state.end())).unwrap_or(0.0);
        let mut span = state.end() - state.start();
        if state.end() < state.start() {
            span += DAY_HOURS;
        }
        Some(from + (to - from) * ((hour - state.start()) / span))
    }

    pub fn mix1(&self, hour: f32, get: impl Fn(&TimeEnv) -> f32) -> f32 {
        let base = get(self.base());
        let value = |env: u8| self.env(env).map_or(0.0, &get);
        self.state.mix(hour, base, value(self.state.current()), value(self.state.previous()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn immediate_change_sets_both_types_and_the_delta_is_one_from_the_next_update_104ecec0_104e7860() {
        let mut state = EnvState::default();
        state.change(2, true, 0.02, 0.02, 1.0, 5.0).unwrap();
        state.update(5.0);
        assert_eq!((state.current(), state.previous(), state.delta()), (2, 2, 1.0));
        assert_eq!(state.mix(5.0, 10.0, 200.0, 99.0), 200.0);
        state.change(0, true, 0.0, 0.0, 1.0, 5.0).unwrap();
        state.update(5.0);
        assert_eq!(state.mix(5.0, 10.0, 200.0, 99.0), 10.0);
    }

    #[test]
    fn timed_change_ramps_to_one_then_ends_with_the_previous_type_dropped_104ecec0_10311bda() {
        let mut state = EnvState::default();
        state.change(1, false, 0.2, 0.1, 0.5, 6.0).unwrap();
        state.update(6.0);
        assert_eq!(state.delta(), 0.0);
        state.update(6.05);
        assert!((state.delta() - 0.5).abs() < 1e-3);
        state.update(6.1);
        assert_eq!(state.delta(), 1.0);
        state.update(6.2);
        assert!((state.delta() - 0.5).abs() < 1e-3);
        assert_eq!(state.previous(), 0);
        state.update(6.25);
        assert_eq!((state.previous(), state.delta()), (1, 0.5));
        assert_eq!(state.mix(6.25, 0.0, 100.0, 0.0), 50.0);
    }

    #[test]
    fn scale_follows_the_current_type_or_interpolates_the_transition_ends_104e89e0() {
        let env0 = TimeEnv::parse("[SunScale]\nNUM=2\nScale1=(T=0,Scale=2.0)\nScale2=(T=24,Scale=2.0)\n");
        let env1 = TimeEnv::parse("[SunScale]\nNUM=2\nScale1=(T=0,Scale=4.0)\nScale2=(T=24,Scale=4.0)\n");
        let mut weather = Weather::with_types(vec![env0, env1]);
        let get = |env: &TimeEnv, hour: f32| env.sun_scale(hour);
        assert_eq!(weather.scale(10.0, get), Some(2.0));
        weather.change_env(1, true, 0.0, 0.0, 1.0, 10.0).unwrap();
        assert_eq!(weather.scale(10.0, get), Some(4.0));
        weather.change_env(0, false, 1.0, 0.5, 1.0, 10.0).unwrap();
        weather.change_env(1, false, 1.0, 0.5, 1.0, 10.0).unwrap();
        assert_eq!(weather.state.previous(), 0);
        let middle = weather.scale(10.5, get).unwrap();
        assert!((middle - 3.0).abs() < 1e-5);
    }

    #[test]
    fn invalid_arguments_are_refused_before_any_state_changes_104ecf2e() {
        let mut state = EnvState::default();
        assert_eq!(state.change(1, false, 1.0, -0.1, 0.5, 6.0), Err(ChangeEnvError::NegativeRamp));
        assert_eq!(state.change(1, false, 0.1, 7.0, 0.5, 6.0), Err(ChangeEnvError::RampBeyondEnd));
        assert_eq!(state.change(1, false, 1.0, 0.1, 1.5, 6.0), Err(ChangeEnvError::TargetOutOfRange));
        assert_eq!(state, EnvState::default());
    }
}
