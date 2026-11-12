use crate::characters::compose;
use glam::{Mat4, Quat, Vec3};
use ue2_assets::{Bone, MeshAnimation, Sequence, Track};

pub const ATTACK_SHOT_NOTIFY: &str = "AnimNotify_AttackShot";

#[derive(Debug, Clone, PartialEq)]
pub struct SequenceTiming {
    pub frames: u32,
    pub rate: f32,
    pub attack_shots: Vec<f32>,
}

pub type LocalPose = Vec<(Quat, Vec3)>;

pub fn sequence_index(anim: &MeshAnimation, name: &str) -> Option<usize> {
    anim.sequences.iter().position(|s| s.name.eq_ignore_ascii_case(name))
}

pub fn find_sequence<'a>(anim: &'a MeshAnimation, name: &str) -> Option<&'a Sequence> {
    anim.sequences.get(sequence_index(anim, name)?)
}

pub fn sequence_timing(sequence: &Sequence) -> SequenceTiming {
    let frames = sequence.frames.max(1) as f32;
    let attack_shots = sequence
        .notifies
        .iter()
        .filter(|notify| notify.class.eq_ignore_ascii_case(ATTACK_SHOT_NOTIFY))
        .map(|notify| if notify.time > 1.0 { notify.time / frames } else { notify.time })
        .collect();
    SequenceTiming { frames: sequence.frames, rate: sequence.rate, attack_shots }
}

pub fn bone_tracks(anim: &MeshAnimation, sequence: &Sequence, bones: &[Bone]) -> Vec<Option<usize>> {
    bones
        .iter()
        .map(|bone| {
            anim.bones
                .iter()
                .position(|b| same_bone(&b.name, &bone.name))
                .and_then(|anim_bone| sequence.tracks.iter().position(|t| t.bone == anim_bone))
        })
        .collect()
}

fn same_bone(a: &str, b: &str) -> bool {
    a.len() == b.len()
        && a.chars()
            .zip(b.chars())
            .all(|(a, b)| a.eq_ignore_ascii_case(&b) || (a == ' ' && b == '_') || (a == '_' && b == ' '))
}

pub fn pose(anim: &MeshAnimation, sequence: &Sequence, bones: &[Bone], frame: f32) -> Vec<Mat4> {
    pose_with_tracks(sequence, bones, &bone_tracks(anim, sequence, bones), frame)
}

pub fn pose_with_tracks(sequence: &Sequence, bones: &[Bone], tracks: &[Option<usize>], frame: f32) -> Vec<Mat4> {
    compose_pose(bones, &local_pose(sequence, bones, tracks, frame, false))
}

pub fn local_pose(
    sequence: &Sequence,
    bones: &[Bone],
    tracks: &[Option<usize>],
    frame: f32,
    looping: bool,
) -> LocalPose {
    let wrap = looping.then_some(sequence.frames as f32);
    bones
        .iter()
        .enumerate()
        .map(|(index, bone)| {
            let track = tracks.get(index).copied().flatten().and_then(|t| sequence.tracks.get(t));
            match track {
                Some(track) => (
                    Quat::from_array(rotation_at(track, frame, wrap).unwrap_or(bone.rotation)),
                    Vec3::from(position_at(track, frame, wrap).unwrap_or(bone.position)),
                ),
                None => (Quat::from_array(bone.rotation), Vec3::from(bone.position)),
            }
        })
        .collect()
}

pub fn bind_local_pose(bones: &[Bone]) -> LocalPose {
    bones.iter().map(|bone| (Quat::from_array(bone.rotation), Vec3::from(bone.position))).collect()
}

pub fn compose_scaled_pose(bones: &[Bone], local: &[(Quat, Vec3)], factors: &[f32]) -> Vec<Mat4> {
    let mut world: Vec<(Quat, Vec3, f32)> = Vec::with_capacity(bones.len());
    for (index, bone) in bones.iter().enumerate() {
        let (rotation, position) =
            local.get(index).copied().unwrap_or((Quat::from_array(bone.rotation), Vec3::from(bone.position)));
        let scale = factors.get(index).copied().unwrap_or(1.0);
        let composed = match world.get(bone.parent).filter(|_| index != 0) {
            Some(&(parent_rotation, parent_origin, parent_scale)) => {
                (parent_rotation * rotation.conjugate(), parent_origin + parent_scale * (parent_rotation * position), parent_scale * scale)
            }
            None => (rotation, position, scale),
        };
        world.push(composed);
    }
    world.iter().map(|&(rotation, origin, scale)| Mat4::from_scale_rotation_translation(Vec3::splat(scale), rotation, origin)).collect()
}

pub fn compose_pose(bones: &[Bone], local: &[(Quat, Vec3)]) -> Vec<Mat4> {
    let mut world: Vec<(Quat, Vec3)> = Vec::with_capacity(bones.len());
    for (index, bone) in bones.iter().enumerate() {
        let (rotation, position) =
            local.get(index).copied().unwrap_or((Quat::from_array(bone.rotation), Vec3::from(bone.position)));
        world.push(compose(&world, index, bone.parent, rotation.to_array(), position.to_array()));
    }
    world.iter().map(|&(rotation, origin)| Mat4::from_rotation_translation(rotation, origin)).collect()
}

pub fn pose_blend(from: &[(Quat, Vec3)], to: &[(Quat, Vec3)], alpha: f32) -> LocalPose {
    let alpha = alpha.clamp(0.0, 1.0);
    from.iter()
        .zip(to)
        .map(|(&(rotation_a, position_a), &(rotation_b, position_b))| {
            let rotation_b = if rotation_a.dot(rotation_b) < 0.0 { -rotation_b } else { rotation_b };
            (rotation_a.lerp(rotation_b, alpha).normalize(), position_a.lerp(position_b, alpha))
        })
        .collect()
}

pub fn tween_pose(
    cached: &[(Quat, Vec3)],
    sequence: &Sequence,
    bones: &[Bone],
    tracks: &[Option<usize>],
    alpha: f32,
) -> LocalPose {
    pose_blend(cached, &local_pose(sequence, bones, tracks, 0.0, false), alpha)
}

pub fn palette(pose: &[Mat4], inverse_bind: &[Mat4]) -> Vec<Mat4> {
    pose.iter().zip(inverse_bind).map(|(p, b)| *p * *b).collect()
}

fn rotation_at(track: &Track, frame: f32, wrap: Option<f32>) -> Option<[f32; 4]> {
    let (a, b, t) = segment(&track.times, track.rotations.len(), frame, wrap)?;
    let from = Quat::from_array(track.rotations[a]);
    let to = Quat::from_array(track.rotations[b]);
    let to = if from.dot(to) < 0.0 { -to } else { to };
    Some(from.lerp(to, t).normalize().to_array())
}

fn position_at(track: &Track, frame: f32, wrap: Option<f32>) -> Option<[f32; 3]> {
    let (a, b, t) = segment(&track.times, track.positions.len(), frame, wrap)?;
    Some(Vec3::from(track.positions[a]).lerp(Vec3::from(track.positions[b]), t).to_array())
}

fn segment(times: &[f32], keys: usize, frame: f32, wrap: Option<f32>) -> Option<(usize, usize, f32)> {
    let last = keys.checked_sub(1)?;
    let time = |i: usize| times.get(i).copied().unwrap_or(i as f32);
    if last == 0 || frame <= time(0) {
        return Some((0, 0, 0.0));
    }
    if frame >= time(last) {
        return match wrap {
            Some(length) if length > time(last) && frame < length => {
                Some((last, 0, (frame - time(last)) / (length - time(last))))
            }
            _ => Some((last, last, 0.0)),
        };
    }
    let next = (1..=last).find(|&i| time(i) > frame)?;
    let span = time(next) - time(next - 1);
    let t = if span > 0.0 { (frame - time(next - 1)) / span } else { 0.0 };
    Some((next - 1, next, t))
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::{Mat4, Vec3};
    use ue2_assets::{AnimBone, Bone, MeshAnimation, Notify, Sequence, Track};

    fn bone(name: &str, parent: usize, rotation: [f32; 4], position: [f32; 3]) -> Bone {
        Bone { name: name.into(), parent, rotation, position }
    }

    fn two_bones() -> Vec<Bone> {
        vec![
            bone("Bip01", 0, [0.0, 0.0, 0.0, 1.0], [0.0; 3]),
            bone("Bip01_Spine", 0, [0.0, 0.0, 0.0, 1.0], [5.0, 0.0, 0.0]),
        ]
    }

    #[test]
    fn scaled_pose_scales_the_bone_axes_and_its_children_but_not_its_origin() {
        let bones = vec![
            bone("Bip01", 0, [0.0, 0.0, 0.0, 1.0], [1.0, 0.0, 0.0]),
            bone("Bip01_Spine", 0, [0.0, 0.0, 0.0, 1.0], [5.0, 0.0, 0.0]),
            bone("Bip01_Head", 1, [0.0, 0.0, 0.0, 1.0], [0.0, 3.0, 0.0]),
        ];
        let local = bind_local_pose(&bones);
        assert_eq!(compose_scaled_pose(&bones, &local, &[1.0; 3]), compose_pose(&bones, &local));
        let pose = compose_scaled_pose(&bones, &local, &[1.0, 2.0, 1.0]);
        assert_eq!(pose[1].transform_point3(Vec3::ZERO), Vec3::new(6.0, 0.0, 0.0));
        assert_eq!(pose[2].transform_point3(Vec3::ZERO), Vec3::new(6.0, 6.0, 0.0));
        assert_eq!(pose[2].transform_vector3(Vec3::X), Vec3::new(2.0, 0.0, 0.0));
        let root = compose_scaled_pose(&bones, &local, &[1.5, 1.0, 1.0]);
        assert_eq!(root[0].transform_point3(Vec3::ZERO), Vec3::new(1.0, 0.0, 0.0));
        assert_eq!(root[2].transform_point3(Vec3::ZERO), Vec3::new(8.5, 4.5, 0.0));
    }

    fn sequence(name: &str, frames: u32, rate: f32, tracks: Vec<Track>) -> Sequence {
        Sequence { name: name.into(), frames, rate, attack_eff: 0.0, attack_end_eff: 0.0, notifies: vec![], tracks }
    }

    fn walk(track: Track) -> MeshAnimation {
        MeshAnimation {
            bones: vec![
                AnimBone { name: "Bip01".into(), parent: 0 },
                AnimBone { name: "Bip01_Spine".into(), parent: 0 },
            ],
            sequences: vec![sequence("Walk", 2, 30.0, vec![track])],
        }
    }

    #[test]
    fn pose_interpolates_between_keys_and_uses_bind_for_untracked_bones() {
        let bones = two_bones();
        let anim = walk(Track {
            bone: 0,
            rotations: vec![[0.0, 0.0, 0.0, 1.0]; 2],
            positions: vec![[0.0, 0.0, 0.0], [0.0, 0.0, 20.0]],
            times: vec![0.0, 1.0],
        });
        let sequence = find_sequence(&anim, "walk").unwrap();
        let pose = pose(&anim, sequence, &bones, 0.5);
        assert!((pose[0].transform_point3(Vec3::ZERO) - Vec3::new(0.0, 0.0, 10.0)).length() < 1e-5);
        assert!((pose[1].transform_point3(Vec3::ZERO) - Vec3::new(5.0, 0.0, 10.0)).length() < 1e-5);
    }

    #[test]
    fn find_sequence_is_case_insensitive_and_none_when_missing() {
        let anim = walk(Track { bone: 0, rotations: vec![], positions: vec![], times: vec![] });
        assert!(find_sequence(&anim, "WALK").is_some());
        assert!(find_sequence(&anim, "Run").is_none());
    }

    #[test]
    fn bone_tracks_map_each_mesh_bone_to_its_track_once() {
        let bones = two_bones();
        let anim = MeshAnimation {
            bones: vec![
                AnimBone { name: "bip01_spine".into(), parent: 0 },
                AnimBone { name: "BIP01".into(), parent: 0 },
            ],
            sequences: vec![sequence(
                "Walk",
                1,
                30.0,
                vec![Track {
                    bone: 0,
                    rotations: vec![[0.0, 0.0, 0.0, 1.0]],
                    positions: vec![[0.0, 7.0, 0.0]],
                    times: vec![0.0],
                }],
            )],
        };
        assert_eq!(sequence_index(&anim, "WALK"), Some(0));
        assert_eq!(sequence_index(&anim, "Run"), None);
        let sequence = find_sequence(&anim, "walk").unwrap();
        let tracks = bone_tracks(&anim, sequence, &bones);
        assert_eq!(tracks, [None, Some(0)]);
        let posed = pose_with_tracks(sequence, &bones, &tracks, 0.0);
        assert_eq!(posed, pose(&anim, sequence, &bones, 0.0));
        assert!((posed[1].transform_point3(Vec3::ZERO) - Vec3::new(0.0, 7.0, 0.0)).length() < 1e-5);
        let short = pose_with_tracks(sequence, &bones, &[], 0.0);
        assert!((short[1].transform_point3(Vec3::ZERO) - Vec3::new(5.0, 0.0, 0.0)).length() < 1e-5);
    }

    #[test]
    fn bone_tracks_treat_spaces_and_underscores_in_bone_names_alike() {
        let bones = vec![
            bone("Bip01", 0, [0.0, 0.0, 0.0, 1.0], [0.0; 3]),
            bone("Bip01 Spine", 0, [0.0, 0.0, 0.0, 1.0], [5.0, 0.0, 0.0]),
        ];
        let anim =
            walk(Track { bone: 1, rotations: vec![[0.0, 0.0, 0.0, 1.0]], positions: vec![[0.0; 3]], times: vec![0.0] });
        let sequence = find_sequence(&anim, "walk").unwrap();
        assert_eq!(bone_tracks(&anim, sequence, &bones), [None, Some(0)]);
        assert!(same_bone("Bip01 L Hand", "bip01_l_hand"));
        assert!(!same_bone("Bip01 head", "Bip01_headNub"));
    }

    #[test]
    fn pose_maps_animation_bones_to_mesh_bones_by_name() {
        let bones = two_bones();
        let anim = MeshAnimation {
            bones: vec![
                AnimBone { name: "bip01_spine".into(), parent: 0 },
                AnimBone { name: "BIP01".into(), parent: 0 },
            ],
            sequences: vec![sequence(
                "Walk",
                1,
                30.0,
                vec![Track {
                    bone: 0,
                    rotations: vec![[0.0, 0.0, 0.0, 1.0]],
                    positions: vec![[0.0, 7.0, 0.0]],
                    times: vec![0.0],
                }],
            )],
        };
        let sequence = find_sequence(&anim, "Walk").unwrap();
        let pose = pose(&anim, sequence, &bones, 0.0);
        assert!((pose[1].transform_point3(Vec3::ZERO) - Vec3::new(0.0, 7.0, 0.0)).length() < 1e-5);
    }

    #[test]
    fn pose_holds_a_single_key_and_clamps_past_the_last_key() {
        let bones = two_bones();
        let anim = walk(Track {
            bone: 0,
            rotations: vec![[0.0, 0.0, 0.0, 1.0]],
            positions: vec![[0.0, 0.0, 0.0], [0.0, 0.0, 20.0]],
            times: vec![0.0, 1.0],
        });
        let sequence = find_sequence(&anim, "Walk").unwrap();
        let at = |frame: f32| pose(&anim, sequence, &bones, frame)[0].transform_point3(Vec3::ZERO);
        assert!((at(-1.0) - Vec3::ZERO).length() < 1e-5);
        assert!((at(1.0) - Vec3::new(0.0, 0.0, 20.0)).length() < 1e-5);
        assert!((at(5.0) - Vec3::new(0.0, 0.0, 20.0)).length() < 1e-5);
    }

    #[test]
    fn looped_sampling_blends_the_last_key_back_into_the_first() {
        let bones = vec![bone("Bip01", 0, [0.0, 0.0, 0.0, 1.0], [0.0; 3])];
        let anim = MeshAnimation {
            bones: vec![AnimBone { name: "Bip01".into(), parent: 0 }],
            sequences: vec![sequence(
                "Walk",
                4,
                30.0,
                vec![Track {
                    bone: 0,
                    rotations: vec![[0.0, 0.0, 0.0, 1.0]; 4],
                    positions: vec![[0.0; 3], [0.0, 0.0, 10.0], [0.0, 0.0, 20.0], [0.0, 0.0, 40.0]],
                    times: vec![0.0, 1.0, 2.0, 3.0],
                }],
            )],
        };
        let sequence = &anim.sequences[0];
        let tracks = bone_tracks(&anim, sequence, &bones);
        let z = |frame: f32, looping: bool| local_pose(sequence, &bones, &tracks, frame, looping)[0].1.z;
        assert!((z(3.5, false) - 40.0).abs() < 1e-5);
        assert!((z(3.5, true) - 20.0).abs() < 1e-5);
        assert!((z(3.0, true) - 40.0).abs() < 1e-5);
        assert!((z(3.75, true) - 10.0).abs() < 1e-5);
        assert!((z(4.0, true) - 40.0).abs() < 1e-5);
    }

    #[test]
    fn pose_blend_is_the_identity_at_the_ends_and_halfway_in_between() {
        let quarter = std::f32::consts::FRAC_PI_2;
        let a = vec![(Quat::IDENTITY, Vec3::ZERO), (Quat::IDENTITY, Vec3::new(5.0, 0.0, 0.0))];
        let b = vec![
            (Quat::from_rotation_z(quarter), Vec3::new(0.0, 0.0, 10.0)),
            (Quat::IDENTITY, Vec3::new(5.0, 0.0, 0.0)),
        ];
        assert_eq!(pose_blend(&a, &b, 0.0), a);
        let end = pose_blend(&a, &b, 1.0);
        assert!((end[0].0.dot(b[0].0).abs() - 1.0).abs() < 1e-6);
        assert_eq!(end[0].1, b[0].1);
        let half = pose_blend(&a, &b, 0.5);
        let x = half[0].0 * Vec3::X;
        assert!((x - Vec3::new((quarter / 2.0).cos(), (quarter / 2.0).sin(), 0.0)).length() < 1e-4, "{x}");
        assert!((half[0].1 - Vec3::new(0.0, 0.0, 5.0)).length() < 1e-6);
        assert_eq!(half[1], a[1]);
        let flipped = vec![(-Quat::from_rotation_z(quarter), Vec3::ZERO)];
        let shortest = pose_blend(&a[..1], &flipped, 0.5)[0].0 * Vec3::X;
        assert!((shortest - x).length() < 1e-4);
        assert_eq!(pose_blend(&a, &b, 2.0)[0].1, b[0].1);
    }

    #[test]
    fn tween_pose_walks_from_the_cached_pose_to_the_first_frame() {
        let bones = two_bones();
        let anim = walk(Track {
            bone: 0,
            rotations: vec![[0.0, 0.0, 0.0, 1.0]; 2],
            positions: vec![[0.0, 0.0, 20.0], [0.0, 0.0, 40.0]],
            times: vec![0.0, 1.0],
        });
        let sequence = &anim.sequences[0];
        let tracks = bone_tracks(&anim, sequence, &bones);
        let cached = bind_local_pose(&bones);
        let quarter = tween_pose(&cached, sequence, &bones, &tracks, 0.25);
        assert!((quarter[0].1 - Vec3::new(0.0, 0.0, 5.0)).length() < 1e-6);
        let done = tween_pose(&cached, sequence, &bones, &tracks, 1.0);
        assert!((done[0].1 - Vec3::new(0.0, 0.0, 20.0)).length() < 1e-6);
        let composed = compose_pose(&bones, &done);
        assert_eq!(composed, pose(&anim, sequence, &bones, 0.0));
        assert!((composed[1].transform_point3(Vec3::ZERO) - Vec3::new(5.0, 0.0, 20.0)).length() < 1e-5);
    }

    #[test]
    fn sequence_timing_lists_attack_shot_notifies_as_fractions() {
        let mut seq = sequence("Atk01", 20, 20.0, vec![]);
        seq.notifies = vec![
            Notify::new(0.5, "AnimNotify_AttackShot"),
            Notify::new(0.2, "AnimNotify_Sound"),
            Notify::new(15.0, "animnotify_attackshot"),
        ];
        let timing = sequence_timing(&seq);
        assert_eq!((timing.frames, timing.rate), (20, 20.0));
        assert_eq!(timing.attack_shots, [0.5, 0.75]);
        assert!(sequence_timing(&sequence("Wait", 10, 3.0, vec![])).attack_shots.is_empty());
    }

    #[test]
    fn pose_rotation_keys_follow_the_bind_convention() {
        let half = std::f32::consts::FRAC_PI_4;
        let bones = two_bones();
        let anim = MeshAnimation {
            bones: vec![
                AnimBone { name: "Bip01".into(), parent: 0 },
                AnimBone { name: "Bip01_Spine".into(), parent: 0 },
            ],
            sequences: vec![sequence(
                "Turn",
                1,
                30.0,
                vec![Track {
                    bone: 1,
                    rotations: vec![[0.0, 0.0, half.sin(), half.cos()]],
                    positions: vec![[5.0, 0.0, 0.0]],
                    times: vec![0.0],
                }],
            )],
        };
        let sequence = find_sequence(&anim, "Turn").unwrap();
        let pose = pose(&anim, sequence, &bones, 0.0);
        let x = pose[1].transform_vector3(Vec3::X);
        assert!((x - Vec3::new(0.0, -1.0, 0.0)).length() < 1e-5, "{x}");
    }

    #[test]
    fn pose_nlerps_rotations_along_the_shortest_arc() {
        let bones = vec![bone("Bip01", 0, [0.0, 0.0, 0.0, 1.0], [0.0; 3])];
        let quarter = std::f32::consts::FRAC_PI_4;
        let anim = MeshAnimation {
            bones: vec![AnimBone { name: "Bip01".into(), parent: 0 }],
            sequences: vec![sequence(
                "Turn",
                2,
                30.0,
                vec![Track {
                    bone: 0,
                    rotations: vec![[0.0, 0.0, 0.0, 1.0], [0.0, 0.0, -quarter.sin(), -quarter.cos()]],
                    positions: vec![[0.0; 3]],
                    times: vec![0.0, 1.0],
                }],
            )],
        };
        let sequence = find_sequence(&anim, "Turn").unwrap();
        let x = pose(&anim, sequence, &bones, 0.5)[0].transform_vector3(Vec3::X);
        assert!((x - Vec3::new(quarter.cos(), quarter.sin(), 0.0)).length() < 1e-4, "{x}");
    }

    #[test]
    fn palette_maps_bind_space_to_pose_space() {
        let bind = [Mat4::from_translation(Vec3::new(0.0, 0.0, 10.0))];
        let inverse = vec![bind[0].inverse()];
        let posed = vec![Mat4::from_translation(Vec3::new(0.0, 0.0, 12.0))];
        let palette = palette(&posed, &inverse);
        let v = palette[0].transform_point3(Vec3::new(1.0, 0.0, 10.0));
        assert!((v - Vec3::new(1.0, 0.0, 12.0)).length() < 1e-5);
    }
}
