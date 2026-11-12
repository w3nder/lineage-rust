use super::{ParticleSystem, ParticleWorld, curve, curve_time, random_range, random_vector, safe_normal};
use glam::Vec3;
use killer_assets::scene::emitters::EmitterAsset;

impl ParticleSystem {
    pub(super) fn spawn_beam(&mut self, emitter: usize, slot: usize, age: f32, asset: &EmitterAsset, world: &mut impl ParticleWorld) {
        let Some(b) = self.actor.sprites[emitter].beam.clone() else { return; };
        let p = self.states[emitter].particles[slot].clone();
        let mut endpoint = None;
        if (2..=5).contains(&b.endpoint_by) && !b.endpoints.is_empty() {
            let total: f32 = b.endpoints.iter().map(|e| e.weight).sum();
            if total != 0.0 {
                let chosen = crate::coronas::app_frand(&mut self.random) * total;
                let mut sum = 0.0;
                endpoint = b.endpoints.iter().position(|e| { let previous = sum; sum += e.weight; chosen > previous && chosen <= sum });
            }
            endpoint = Some(endpoint.unwrap_or(0));
        }
        let mut end = Vec3::ZERO;
        let mut fixed = matches!(b.endpoint_by, 2..=5).then_some(Vec3::ZERO);
        match b.endpoint_by {
            0 => end = p.velocity * p.lifetime,
            1 => { let dir = safe_normal(p.velocity); self.states[emitter].particles[slot].velocity = dir; end = dir * random_range(&mut self.random, b.distance); }
            2 | 4 => if let Some(i) = endpoint {
                end = random_vector(&mut self.random, b.endpoints[i].offset);
                let mut target = p.location + end;
                if b.endpoint_by == 4 && let Some(hit) = world.trace(p.location, target, 0xdf) { target = hit.location; end = target - p.location; }
                fixed = Some(target);
            },
            3 => if let Some(i) = endpoint && let Some(target) = asset.parts[emitter].targets.get(i).copied().flatten() {
                let target = Vec3::from(target); end = target - p.location; fixed = Some(target);
                if b.trigger_endpoint && let Some(tag) = &b.endpoints[i].actor_tag { world.trigger(tag); }
            },
            5 => if let Some(i) = endpoint { let target = random_vector(&mut self.random, b.endpoints[i].offset); end = target - p.location; fixed = Some(target); },
            _ => {}
        }
        if b.noise_endpoint { fixed = None; }
        if b.low_points < 2 || b.high_points < 2 { return; }
        let mut low = vec![p.location];
        let mut noise = Vec3::ZERO;
        for i in 1..b.low_points {
            if !b.use_low_scale { noise = random_vector(&mut self.random, b.low_noise); }
            else if let Some(scale) = curve(b.low_scale.iter().copied(), curve_time(b.low_scale_repeats, i as f32 / b.low_points as f32), [1.0; 3]) { noise = Vec3::from(scale) * random_vector(&mut self.random, b.high_noise); }
            low.push(p.location + end * (i as f32 / (b.low_points - 1) as f32) + noise);
        }
        if let Some(fixed) = fixed && let Some(last) = low.last_mut() { *last = fixed; }
        let mut high = vec![low[0]];
        noise = Vec3::ZERO;
        for i in 1..b.high_points {
            if !b.use_high_scale { noise = random_vector(&mut self.random, b.high_noise); }
            else if let Some(scale) = curve(b.high_scale.iter().copied(), curve_time(b.high_scale_repeats, i as f32 / b.high_points as f32), [1.0; 3]) { noise = Vec3::from(scale) * random_vector(&mut self.random, b.high_noise); }
            let fraction = i as f32 * (b.low_points - 1) as f32 / (b.high_points - 1) as f32;
            let segment = (fraction as usize).min(low.len() - 2);
            let position = low[segment].lerp(low[segment + 1], fraction - segment as f32) + noise;
            high.push(position);
            if b.use_branching && b.branch_points[0] < i as f32 && (i as f32) < b.branch_points[1] {
                let probability = random_range(&mut self.random, b.branch_probability);
                if crate::coronas::app_frand(&mut self.random) <= probability && b.branch_emitter >= 0 && !self.states.is_empty() {
                    let other = (b.branch_emitter as usize).min(self.states.len() - 1);
                    let count = random_range(&mut self.random, b.branch_amount).max(0.0) as u32;
                    for _ in 0..count {
                        let index = self.states[other].index;
                        self.spawn(other, index, age, 0, Some(position), asset, world);
                        if b.linkup_lifetime && let Some(other) = self.states[other].particles.get_mut(index) { other.time = p.time; other.lifetime = p.lifetime; }
                        self.advance(other);
                    }
                }
            }
        }
        if let Some(fixed) = fixed && let Some(last) = high.last_mut() { *last = fixed; }
        self.states[emitter].particles[slot].beam = high;
    }
}
