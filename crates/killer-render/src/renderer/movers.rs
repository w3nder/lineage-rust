use super::{Renderer, TextureSource};
use glam::{Mat3, Mat4, Vec3};
use killer_assets::scene::{Vertex, movers::MovingAsset, transform::placement_matrix};
use ue2_level::{
    Placement,
    movers::{MovementState, MoverState},
};

#[derive(Clone, Copy)]
pub(super) struct DoorBinding {
    door_id: i32,
    status: Option<(bool, i32, bool)>,
}

pub(super) struct MovingState {
    asset: MovingAsset,
    movement: MovementState,
    door: Option<MoverState>,
    pose: Placement,
    grade: Option<usize>,
    object_id: Option<i32>,
    attached: Vec<Placement>,
}

pub(super) struct MovingDraw {
    last_matrix: Option<Mat4>,
    slot: usize,
    grade: Option<usize>,
    vertices: Vec<Vertex>,
    index_count: u32,
    center: Vec3,
}

impl Renderer {
    pub fn set_movers_auto_tick(&mut self, enabled: bool) {
        self.movers_auto_tick = enabled;
    }
    pub(super) fn upload_movers(&mut self, assets: &[MovingAsset], textures: &mut dyn TextureSource, tile: usize) {
        for asset in assets {
            let slot = self.tiles[tile].moving.len();
            self.tiles[tile].moving.push(MovingState {
                asset: asset.clone(),
                movement: MovementState::default(),
                door: asset.mover.as_ref().map(MoverState::new),
                pose: asset.mover.as_ref().map_or(asset.origin, |m| m.key(0)),
                grade: None,
                object_id: None,
                attached: asset.attached.iter().map(|a| a.origin).collect(),
            });
            for (grade, sources) in
                std::iter::once((None, &asset.draws)).chain(asset.grades.iter().enumerate().map(|(i, d)| (Some(i), d)))
            {
                for source in sources {
                    if let Some(mut draw) = self.gpu_draw(source, textures, tile) {
                        draw.moving = Some(MovingDraw {
                            last_matrix: None,
                            slot,
                            grade,
                            vertices: source.vertices.clone(),
                            index_count: draw.index_count,
                            center: draw.center,
                        });
                        if grade.is_some() {
                            draw.index_count = 0;
                        }
                        self.draws.push(draw);
                    }
                }
            }
        }
        let pending: Vec<_> = self.door_bindings.iter().map(|(id, value)| (*id, *value)).collect();
        for (object_id, binding) in pending {
            self.door_info(object_id, binding.door_id);
            if let Some((closed, damage, attackable)) = binding.status {
                self.door_status(object_id, closed, damage, attackable);
            }
        }
    }

    pub fn door_info(&mut self, object_id: i32, door_id: i32) {
        self.door_bindings.entry(object_id).or_insert(DoorBinding { door_id, status: None }).door_id = door_id;
        for state in self.tiles.iter_mut().flat_map(|t| &mut t.moving) {
            if state.asset.mover.as_ref().is_some_and(|m| m.server_id == Some(door_id)) {
                state.object_id = Some(object_id);
                return;
            }
        }
    }

    pub fn door_status(&mut self, object_id: i32, closed: bool, damage: i32, attackable: bool) {
        let Some(binding) = self.door_bindings.get_mut(&object_id) else {
            return;
        };
        binding.status = Some((closed, damage, attackable));
        for state in self.tiles.iter_mut().flat_map(|t| &mut t.moving) {
            if state.object_id != Some(object_id) {
                continue;
            }
            if let (Some(def), Some(door)) = (&state.asset.mover, &mut state.door)
                && door.on_door_status_update(def, closed, damage, attackable)
            {
                let index = damage as usize;
                let current = state.grade.map_or(&state.asset.mesh, |i| &def.meshes[i]);
                if current != &def.meshes[index] {
                    state.grade = if state.asset.mesh == def.meshes[index] { None } else { Some(index) };
                }
            }
        }
    }

    pub fn tick_movers(&mut self, dt: f32) {
        for (tile_index, tile) in self.tiles.iter_mut().enumerate() {
            for state in &mut tile.moving {
                let old = state.pose;
                if let Some(def) = &state.asset.movement {
                    state.pose = state.movement.phys_l2_movement(def, dt, || {
                        self.corona_random = self.corona_random.wrapping_mul(0x343fd).wrapping_add(0x269ec3);
                        ((self.corona_random >> 16) & 0x7fff) as i32
                    });
                }
                if let (Some(def), Some(door)) = (&state.asset.mover, &mut state.door) {
                    state.pose = door.phys_moving_brush(def, dt);
                }
                if old != state.pose {
                    for (pose, attachment) in state.attached.iter_mut().zip(&state.asset.attached) {
                        if let Some(index) = attachment.emitter
                            && let Some(emitter) =
                                self.particle_actors.iter_mut().filter(|a| a.tile == tile_index).nth(index)
                        {
                            *pose = move_attached(
                                Placement {
                                    location: emitter.system.actor.location,
                                    rotation: emitter.system.actor.rotation,
                                },
                                old,
                                state.pose,
                            );
                            emitter.system.actor.location = pose.location;
                            emitter.system.actor.rotation = pose.rotation;
                        } else {
                            *pose = move_attached(*pose, old, state.pose);
                        }
                    }
                }
            }
        }
        for draw in &mut self.draws {
            let Some(moving) = &mut draw.moving else {
                continue;
            };
            let state = &self.tiles[draw.tile].moving[moving.slot];
            draw.index_count = if state.grade == moving.grade { moving.index_count } else { 0 };
            if draw.index_count == 0 {
                continue;
            }
            let matrix = delta_matrix(state.asset.origin, state.pose);
            if moving.last_matrix == Some(matrix) {
                continue;
            }
            moving.last_matrix = Some(matrix);
            let normal = Mat3::from_mat4(matrix).inverse().transpose();
            let vertices: Vec<_> = moving
                .vertices
                .iter()
                .map(|v| Vertex {
                    position: matrix.transform_point3(Vec3::from(v.position)).to_array(),
                    normal: (normal * Vec3::from(v.normal)).normalize_or(Vec3::Z).to_array(),
                    ..*v
                })
                .collect();
            self.queue.write_buffer(&draw.vertices, 0, bytemuck::cast_slice(&vertices));
            draw.center = matrix.transform_point3(moving.center);
            if let Some(actor) = &mut draw.actor {
                actor.xy = [state.pose.location[0], state.pose.location[1]];
            }
        }
    }

    pub fn collision_poses(&self) -> impl Iterator<Item = (&str, &str, killer_assets::collision::world::Pose)> {
        self.tiles.iter().flat_map(|tile| tile.moving.iter().map(move |state| {
            let mesh = state.grade.and_then(|grade| state.asset.mover.as_ref().and_then(|mover| mover.meshes.get(grade))).cloned().flatten()
                .or_else(|| state.asset.mesh.clone());
            (tile.name.as_str(),state.asset.name.as_str(),killer_assets::collision::world::Pose {
                placement:state.pose,mesh,disabled:state.door.as_ref().is_some_and(|door| door.damage == 6),
            })
        }))
    }

    pub fn moving_actors(&self) -> impl Iterator<Item = (&str, &MovingAsset, Mat4, Option<usize>)> {
        self.tiles.iter().flat_map(|tile| {
            tile.moving.iter().map(move |state| {
                (tile.name.as_str(), &state.asset, delta_matrix(state.asset.origin, state.pose), state.grade)
            })
        })
    }
}

fn delta_matrix(origin: Placement, pose: Placement) -> Mat4 {
    placement_matrix(pose.location, pose.rotation, [1.0; 3])
        * placement_matrix(origin.location, origin.rotation, [1.0; 3]).inverse()
}

fn move_attached(child: Placement, old: Placement, new: Placement) -> Placement {
    Placement {
        location: delta_matrix(old, new).transform_point3(Vec3::from(child.location)).to_array(),
        rotation: std::array::from_fn(|i| {
            child.rotation[i].wrapping_add((new.rotation[i] & 0xfffc).wrapping_sub(old.rotation[i] & 0xfffc))
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn attached_emitter_follows_parent_and_survives_scene_cache() {
        use killer_assets::scene::{Scene, emitters::EmitterAsset, movers::AttachedActor};
        use ue2_level::movers::Mover;
        let Ok(mut headless) = crate::headless::Headless::new(32, 32) else {
            return;
        };
        let origin = Placement { location: [0.0; 3], rotation: [0; 3] };
        let mut door = Mover {
            base: origin,
            key_pos: [[0.0; 3]; 8],
            key_rot: [[0; 3]; 8],
            meshes: std::array::from_fn(|_| None),
            server_id: Some(123),
            move_time: 1.0,
            glide: true,
        };
        door.key_rot[1] = [0, 16384, 0];
        let emitter = EmitterAsset {
            definition: ue2_level::Emitter { location: [100.0, 0.0, 0.0], ..Default::default() },
            parts: Vec::new(),
            zone: 0,
            range: f32::INFINITY,
            lighting: None,
        };
        let asset = MovingAsset {
            name: "parent".into(),
            mesh: None,
            origin,
            movement: None,
            mover: Some(door),
            draws: Vec::new(),
            grades: std::array::from_fn(|_| Vec::new()),
            attached: vec![AttachedActor {
                name: "child".into(),
                origin: Placement { location: [100.0, 0.0, 0.0], ..origin },
                emitter: Some(0),
            }],
        };
        let scene = Scene { moving: vec![asset], ..Default::default() };
        let stamp = killer_assets::cache::SourceStamp::of(b"attached");
        let cached = killer_assets::cache::CachedScene::from_bank(scene, &Default::default());
        let bytes = killer_assets::cache::encode_to_vec(&stamp, &cached);
        let mut scene = killer_assets::cache::decode(bytes.as_slice(), &stamp).unwrap().scene;
        scene.emitters.push(emitter);
        struct Empty;
        impl TextureSource for Empty {
            fn texture(&mut self, _: &str) -> std::sync::Arc<killer_assets::textures::TextureData> {
                panic!("sem textura")
            }
        }
        headless.upload(&scene, &mut Empty);
        let renderer = headless.renderer_mut();
        renderer.door_info(1, 123);
        renderer.door_status(1, false, 0, false);
        renderer.tick_movers(1.0);
        let actor = &renderer.particle_actors[0].system.actor;
        assert!((Vec3::from(actor.location) - Vec3::new(0.0, 100.0, 0.0)).length() < 0.001);
        assert_eq!(actor.rotation, [0, 16384, 0]);
    }
    #[test]
    fn move_actor_1058b3c0_rotates_attached_location_and_quantizes_child_rotation() {
        let old = Placement { location: [100.0, 200.0, 0.0], rotation: [0; 3] };
        let new = Placement { rotation: [0, 16385, 0], ..old };
        let child = Placement { location: [200.0, 200.0, 0.0], rotation: [0, 4, 0] };
        let result = move_attached(child, old, new);
        assert!((Vec3::from(result.location) - Vec3::new(100.0, 300.0, 0.0)).length() < 0.02);
        assert_eq!(result.rotation, [0, 16388, 0]);
        let back = move_attached(result, new, old);
        assert!((Vec3::from(back.location) - Vec3::from(child.location)).length() < 0.001);
        assert_eq!(back.rotation, child.rotation);
    }
}
