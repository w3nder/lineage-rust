use glam::Vec3;
use killer_assets::collision::world::Surface;

pub const MARK_CLASS: &str = "LineageEffect.e_u093_a";
pub const ARRIVAL_CLASS: &str = "LineageEffect.e_u093_b";
const ARRIVAL_DISTANCE: f64 = 30.0;
const IDLE_LIFE: f32 = 29.0;
const DELETE_SHIFT: f32 = 30.0;
const LIFE_END: f32 = 0.0001;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Slot {
    Mark,
    Arrival,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Request {
    Spawn { slot: Slot, class: &'static str, location: Vec3, rotation: [i32; 3] },
    Remove(Slot),
    Release { slot: Slot, life: f32 },
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Effect {
    location: Vec3,
    rotation: [i32; 3],
    life: f32,
}

#[derive(Default)]
pub struct LandMark {
    mark: Option<Effect>,
    arrival: Option<Effect>,
    requests: Vec<Request>,
}

pub fn sine() -> &'static [f32] {
    static SINE: std::sync::OnceLock<[f32; 16384]> = std::sync::OnceLock::new();
    SINE.get_or_init(killer_assets::skeletal_lod::sine_table)
}

pub struct Pawn {
    pub location: Vec3,
    pub waiting: bool,
}

impl LandMark {
    pub fn add(&mut self, location: Vec3, normal: Vec3, surface: Option<Surface>, life_span: f32, sine: &[f32]) {
        if surface.is_none() { return; }
        let slope = ((f64::from(normal.y)+f64::from(normal.x))*0.0+f64::from(normal.z)) as f32;
        if !(0.5 < slope && slope <= 1.0) { return; }
        if self.mark.take().is_some() { self.requests.push(Request::Remove(Slot::Mark)); }
        let rotation = surface_rotation(normal, sine);
        self.mark = Some(Effect { location, rotation, life: life_span });
        self.requests.push(Request::Spawn { slot: Slot::Mark, class: MARK_CLASS, location, rotation });
    }

    pub fn delete(&mut self, force: bool) {
        if force {
            if self.mark.take().is_some() { self.requests.push(Request::Remove(Slot::Mark)); }
            if self.arrival.take().is_some() { self.requests.push(Request::Remove(Slot::Arrival)); }
            return;
        }
        let Some(mut mark) = self.mark.take() else { return };
        let remaining = DELETE_SHIFT-mark.life;
        if remaining <= 0.0 {
            mark.life = -remaining;
            if mark.life <= 0.0 { self.requests.push(Request::Remove(Slot::Mark)); }
            else { self.requests.push(Request::Release { slot: Slot::Mark, life: mark.life }); }
        } else {
            self.requests.push(Request::Remove(Slot::Mark));
        }
    }

    pub fn update(&mut self, dt: f32, pawn: Option<&Pawn>, arrival_life: f32) {
        for (slot,effect) in [(Slot::Mark,&mut self.mark),(Slot::Arrival,&mut self.arrival)] {
            let Some(current) = effect.as_mut() else { continue };
            if current.life != 0.0 {
                current.life = (f64::from(current.life)-f64::from(dt)) as f32;
                if current.life <= LIFE_END {
                    *effect = None;
                    self.requests.push(Request::Remove(slot));
                }
            }
        }
        let (Some(pawn),Some(mark)) = (pawn,self.mark) else { return };
        let delta = mark.location.as_dvec3()-pawn.location.as_dvec3();
        let distance = ((delta.z*delta.z+delta.y*delta.y)+delta.x*delta.x).sqrt();
        if ARRIVAL_DISTANCE <= distance {
            if pawn.waiting && mark.life < IDLE_LIFE { self.delete(false); }
            return;
        }
        if let Some(old) = self.arrival.take() { self.requests.push(Request::Release { slot: Slot::Arrival, life: old.life }); }
        self.arrival = Some(Effect { location: mark.location, rotation: mark.rotation, life: arrival_life });
        self.requests.push(Request::Spawn { slot: Slot::Arrival, class: ARRIVAL_CLASS, location: mark.location, rotation: mark.rotation });
        self.mark = None;
        self.requests.push(Request::Remove(Slot::Mark));
    }

    pub fn take_requests(&mut self) -> Vec<Request> {
        std::mem::take(&mut self.requests)
    }
}

fn surface_rotation(normal: Vec3, sine: &[f32]) -> [i32; 3] {
    let lookup = |angle: i32| sine.get(((angle >> 2) & 0x3fff) as usize).copied().map_or(0.0, f64::from);
    let (sp, cp) = (lookup(0), lookup(0x4000));
    let (sy, cy) = (lookup(0), lookup(0x4000));
    let axis = [(cp*cy) as f32, (cp*sy) as f32, sp as f32].map(f64::from);
    let [nx,ny,nz] = normal.to_array().map(f64::from);
    let dot = (nz*axis[2]+ny*axis[1])+nx*axis[0];
    let projected = [(nx*dot) as f32, (ny*dot) as f32, (nz*dot) as f32].map(f64::from);
    let tangent: [f32; 3] = std::array::from_fn(|k| (axis[k]-projected[k]) as f32);
    let first = killer_assets::scene::transform::vector_rotation(Vec3::from(tangent));
    let [tx,ty,tz] = tangent.map(f64::from);
    let cross = Vec3::new((ty*nz-tz*ny) as f32, (tz*nx-tx*nz) as f32, (tx*ny-ty*nx) as f32);
    let second = killer_assets::scene::transform::vector_rotation(cross);
    [first[0], 0, second[0]]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sine() -> Vec<f32> {
        (0..16384).map(|i| (i as f64 * std::f64::consts::TAU / 16384.0).sin() as f32).collect()
    }

    #[test]
    fn mark_needs_level_terrain_or_static_mesh_and_a_floor_slope() {
        let table = sine();
        let mut landmark = LandMark::default();
        landmark.add(Vec3::ZERO, Vec3::Z, None, 30.0, &table);
        landmark.add(Vec3::ZERO, Vec3::new(0.0, 0.9, 0.43), Some(Surface::Terrain), 30.0, &table);
        assert!(landmark.take_requests().is_empty());
        landmark.add(Vec3::new(1.0, 2.0, 3.0), Vec3::Z, Some(Surface::Level), 30.0, &table);
        assert_eq!(landmark.take_requests(), [Request::Spawn { slot: Slot::Mark, class: MARK_CLASS, location: Vec3::new(1.0, 2.0, 3.0), rotation: [0, 0, 0] }]);
        landmark.add(Vec3::new(5.0, 2.0, 3.0), Vec3::Z, Some(Surface::StaticMeshActor), 30.0, &table);
        assert_eq!(landmark.take_requests()[0], Request::Remove(Slot::Mark));
    }

    #[test]
    fn arriving_within_thirty_units_swaps_the_mark_for_the_arrival_effect() {
        let table = sine();
        let mut landmark = LandMark::default();
        landmark.add(Vec3::new(100.0, 0.0, 0.0), Vec3::Z, Some(Surface::Terrain), 30.0, &table);
        landmark.take_requests();
        landmark.update(0.1, Some(&Pawn { location: Vec3::new(50.0, 0.0, 0.0), waiting: false }), 30.0);
        assert!(landmark.take_requests().is_empty());
        landmark.update(0.1, Some(&Pawn { location: Vec3::new(75.0, 0.0, 0.0), waiting: false }), 30.0);
        assert_eq!(landmark.take_requests(), [
            Request::Spawn { slot: Slot::Arrival, class: ARRIVAL_CLASS, location: Vec3::new(100.0, 0.0, 0.0), rotation: [0, 0, 0] },
            Request::Remove(Slot::Mark),
        ]);
    }

    #[test]
    fn stopping_far_away_after_one_second_removes_the_mark() {
        let table = sine();
        let mut landmark = LandMark::default();
        landmark.add(Vec3::new(500.0, 0.0, 0.0), Vec3::Z, Some(Surface::Terrain), 30.0, &table);
        landmark.take_requests();
        let pawn = Pawn { location: Vec3::ZERO, waiting: true };
        landmark.update(0.5, Some(&pawn), 30.0);
        assert!(landmark.take_requests().is_empty());
        landmark.update(0.6, Some(&pawn), 30.0);
        assert_eq!(landmark.take_requests(), [Request::Remove(Slot::Mark)]);
    }

    #[test]
    fn life_span_runs_out_and_forced_delete_clears_both() {
        let table = sine();
        let mut landmark = LandMark::default();
        landmark.add(Vec3::new(500.0, 0.0, 0.0), Vec3::Z, Some(Surface::Terrain), 30.0, &table);
        landmark.take_requests();
        landmark.update(30.0, None, 30.0);
        assert_eq!(landmark.take_requests(), [Request::Remove(Slot::Mark)]);
        landmark.add(Vec3::new(10.0, 0.0, 0.0), Vec3::Z, Some(Surface::Terrain), 30.0, &table);
        landmark.update(0.1, Some(&Pawn { location: Vec3::ZERO, waiting: false }), 30.0);
        landmark.take_requests();
        landmark.delete(true);
        assert_eq!(landmark.take_requests(), [Request::Remove(Slot::Arrival)]);
    }
}
