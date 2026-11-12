use crate::texture_modifier::TextureModifyInfo;

pub const RANGE_BAND: f32 = 256.0;
pub const VIEW_DURATION: f32 = 1.0;
pub const LOD_DURATION: f32 = 1.8;
const ALPHA_MAX: f32 = 255.0;
const LEVELS: usize = 3;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ViewType {
    None,
    InSight,
    InSighting,
    OutSight,
    OutSighting,
    Spawning,
    Despawning,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LodBlend {
    None,
    Normal,
    Downing,
    Upping,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LodInput {
    pub ranges: [f32; 2],
    pub blend: bool,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FrameInput {
    pub distance: f32,
    pub range: f32,
    pub ignored_range: bool,
    pub lod: Option<LodInput>,
}

/// What `RenderLevel`/`RenderStaticMesh` decided for one actor this frame: per LOD level, `Some(alpha)` draws it, with `alpha` the
/// `TexModifyInfo` byte when the actor is modified (`None` draws the material untouched).
pub type Levels = [Option<Option<u8>>; LEVELS];

/// `AActor`'s `L2ActorViewtype`/`L2CurrentLod` state machines and the alpha modifier they drive.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ActorView {
    pub view: ViewType,
    view_duration: f32,
    view_elapsed: f32,
    pub lod_blend: LodBlend,
    lod_duration: f32,
    lod_elapsed: f32,
    pub current_lod: i8,
    pub modify: TextureModifyInfo,
    pub delete_now: bool,
}

impl Default for ActorView {
    fn default() -> Self {
        Self {
            view: ViewType::None,
            view_duration: VIEW_DURATION,
            view_elapsed: 0.0,
            lod_blend: LodBlend::None,
            lod_duration: LOD_DURATION,
            lod_elapsed: 0.0,
            current_lod: -1,
            modify: TextureModifyInfo { flags: 0, color: u32::MAX, value_08: 1, value_0c: 1 },
            delete_now: false,
        }
    }
}

/// `lod_for` of `RenderStaticMesh`: which mesh the 2D distance picks.
pub fn lod_for(distance: f32, ranges: [f32; 2]) -> u8 {
    if distance <= ranges[0] {
        0
    } else if distance <= ranges[1] {
        1
    } else {
        2
    }
}

fn ftol_alpha(value: f32) -> u8 {
    value.clamp(0.0, ALPHA_MAX).trunc() as u8
}

impl ActorView {
    /// The alpha `GetModifiedTextureMaterial` would wrap the material with, or `None` when the actor is not modified.
    pub fn alpha(&self) -> Option<u8> {
        self.modify.is_modified_texture().then_some((self.modify.color >> 24) as u8)
    }

    /// `AActor::SetActorViewType(t, 1, 0)` for the sight, spawning and despawning states.
    pub fn set_view_type(&mut self, t: ViewType) {
        let sighting = |t| matches!(t, ViewType::InSighting | ViewType::OutSighting | ViewType::Spawning | ViewType::Despawning);
        match self.view {
            ViewType::None => {
                self.view = t;
                if sighting(t) {
                    self.view_elapsed = 0.0;
                } else {
                    self.modify.clear();
                }
                self.view_duration = VIEW_DURATION;
            }
            ViewType::InSight => {
                if !matches!(t, ViewType::InSight | ViewType::InSighting) {
                    self.view = t;
                    self.view_elapsed = 0.0;
                }
            }
            ViewType::InSighting => {
                if !matches!(t, ViewType::InSight | ViewType::InSighting) {
                    self.view = t;
                    if t == ViewType::OutSight {
                        self.modify.clear_alpha();
                        self.view_elapsed = 0.0;
                    } else {
                        self.view_elapsed = self.view_duration - self.view_elapsed;
                    }
                }
            }
            ViewType::OutSight => {
                if !matches!(t, ViewType::OutSight | ViewType::OutSighting) {
                    if t == ViewType::InSighting {
                        self.modify.set_alpha(0);
                    }
                    self.view = t;
                    self.view_elapsed = 0.0;
                }
            }
            ViewType::OutSighting => {
                if !matches!(t, ViewType::OutSight | ViewType::OutSighting) {
                    self.view = t;
                    if t == ViewType::InSight {
                        self.modify.clear_alpha();
                        self.view_elapsed = 0.0;
                    } else {
                        self.view_elapsed = self.view_duration - self.view_elapsed;
                    }
                }
            }
            ViewType::Spawning => {
                if t == ViewType::Despawning {
                    self.view = t;
                    self.view_elapsed = 0.0;
                }
            }
            ViewType::Despawning => {}
        }
    }

    /// `RenderLevel`/`RenderBspLeaf` with `GL2ActorAppearingType == 0`: the 256 uu band around `range`, and whether the actor draws.
    pub fn in_range(&mut self, distance: f32, range: f32) -> bool {
        if distance < range {
            self.set_view_type(if distance <= range - RANGE_BAND { ViewType::InSight } else { ViewType::InSighting });
            true
        } else if distance >= range + RANGE_BAND {
            self.set_view_type(ViewType::OutSight);
            self.view != ViewType::OutSight
        } else if self.view == ViewType::OutSight {
            false
        } else {
            self.set_view_type(ViewType::OutSighting);
            true
        }
    }

    /// `AActor::SetStaticMeshActorViewType(new_lod, immediate)`. The interrupted-blend branches reset the `L2ActorViewElapsedTime` timers
    /// (the engine's own mix-up), so a blend cut in the middle does not restart from the right point.
    pub fn set_lod(&mut self, new_lod: i8, immediate: bool) {
        if immediate || self.view != ViewType::InSight {
            self.current_lod = new_lod;
            self.lod_elapsed = 0.0;
            self.lod_blend = LodBlend::None;
            self.lod_duration = LOD_DURATION;
            return;
        }
        if !(0..=2).contains(&new_lod) {
            return;
        }
        match self.current_lod {
            -1 => {
                self.lod_blend = LodBlend::Normal;
                self.lod_duration = LOD_DURATION;
                self.current_lod = new_lod;
            }
            0 if new_lod > 0 => self.blend_down(new_lod),
            1 if new_lod > 1 => self.blend_down(new_lod),
            1 if new_lod < 1 => self.blend_up(new_lod),
            2 if new_lod < 2 => self.blend_up(new_lod),
            _ => {}
        }
    }

    fn blend_down(&mut self, new_lod: i8) {
        self.restart_blend(LodBlend::Upping);
        self.lod_blend = LodBlend::Downing;
        self.current_lod = new_lod;
    }

    fn blend_up(&mut self, new_lod: i8) {
        self.restart_blend(LodBlend::Downing);
        self.lod_blend = LodBlend::Upping;
        self.current_lod = new_lod;
    }

    fn restart_blend(&mut self, opposite: LodBlend) {
        if self.lod_blend == opposite {
            self.view_elapsed = self.view_duration - self.view_elapsed;
        } else {
            self.modify.set_alpha(0);
            self.view_elapsed = 0.0;
        }
    }

    fn invert_alpha(&mut self) {
        let current = (self.modify.color >> 24) as u8;
        self.modify.set_alpha(255 - current);
    }

    /// `RenderStaticMesh`'s LOD choice and, with `bStaticMeshLodBlend`, the second pass that draws the other LOD with the inverted alpha.
    fn static_mesh_levels(&mut self, distance: f32, ranges: [f32; 2], blend: bool) -> Levels {
        let mut levels: Levels = [None; LEVELS];
        let chosen = lod_for(distance, ranges);
        let mut first = chosen;
        if blend {
            self.set_lod(chosen as i8, false);
            if chosen < 2 && self.lod_blend == LodBlend::Upping {
                self.invert_alpha();
                first = chosen + 1;
            }
        }
        levels[usize::from(first)] = Some(self.alpha());
        if blend {
            let mut second = self.current_lod;
            match self.lod_blend {
                LodBlend::Downing => second -= 1,
                LodBlend::Upping => {}
                _ => return levels,
            }
            if (0..=2).contains(&second) {
                self.invert_alpha();
                levels[second as usize] = Some(self.alpha());
            }
        }
        levels
    }

    /// One frame of the actor in a visible leaf: the range band, then the LOD passes.
    pub fn frame(&mut self, input: FrameInput) -> Levels {
        if !input.ignored_range && !self.in_range(input.distance, input.range) {
            return [None; LEVELS];
        }
        match input.lod {
            Some(lod) => self.static_mesh_levels(input.distance, lod.ranges, lod.blend),
            None => [Some(self.alpha()), None, None],
        }
    }

    /// Whether `tick` has anything to advance (`AddL2AvtActors` keeps such actors on the ticked list).
    pub fn animating(&self) -> bool {
        matches!(self.view, ViewType::InSighting | ViewType::OutSighting | ViewType::Spawning | ViewType::Despawning) || matches!(self.lod_blend, LodBlend::Downing | LodBlend::Upping)
    }

    /// `AActor::Tick` (0x10595800): the view-type fade has priority over the LOD blend.
    pub fn tick(&mut self, dt: f32) {
        match self.view {
            ViewType::InSighting => {
                self.view_elapsed += dt;
                if self.view_elapsed < self.view_duration {
                    self.modify.set_alpha(ftol_alpha(self.view_elapsed / self.view_duration * ALPHA_MAX));
                } else {
                    self.modify.clear_alpha();
                    self.view = ViewType::InSight;
                    self.view_elapsed = 0.0;
                }
            }
            ViewType::OutSighting => {
                self.view_elapsed += dt;
                if self.view_elapsed < self.view_duration {
                    self.modify.set_alpha(ftol_alpha(ALPHA_MAX - ALPHA_MAX / self.view_duration * self.view_elapsed));
                } else {
                    self.modify.clear_alpha();
                    self.view = ViewType::OutSight;
                    self.view_elapsed = 0.0;
                }
            }
            ViewType::Spawning => {
                self.view_elapsed += dt;
                if self.view_elapsed < self.view_duration {
                    self.modify.set_alpha(ftol_alpha(self.view_elapsed / self.view_duration * ALPHA_MAX));
                } else {
                    self.modify.clear_alpha();
                    self.view = ViewType::None;
                }
            }
            ViewType::Despawning => {
                self.view_elapsed += dt;
                if self.view_elapsed < self.view_duration {
                    self.modify.set_alpha(ftol_alpha(ALPHA_MAX - ALPHA_MAX / self.view_duration * self.view_elapsed));
                } else {
                    self.modify.clear_alpha();
                    self.view = ViewType::None;
                    self.delete_now = true;
                }
            }
            _ if matches!(self.lod_blend, LodBlend::Downing | LodBlend::Upping) => {
                self.lod_elapsed += dt;
                if self.lod_duration <= self.lod_elapsed {
                    self.modify.clear_alpha();
                    self.lod_elapsed = 0.0;
                    self.lod_blend = LodBlend::Normal;
                } else {
                    self.modify.set_alpha(ftol_alpha(self.lod_elapsed / self.lod_duration * ALPHA_MAX));
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const RANGE: f32 = 7168.0;

    fn frame(view: &mut ActorView, distance: f32) -> Levels {
        view.frame(FrameInput { distance, range: RANGE, ignored_range: false, lod: None })
    }

    #[test]
    fn spawning_fades_in_over_a_second_ignores_the_range_states_and_despawning_fades_out() {
        let mut view = ActorView::default();
        view.set_view_type(ViewType::Spawning);
        assert_eq!(view.view, ViewType::Spawning);
        assert!(view.animating());
        assert_eq!(frame(&mut view, 100.0), [Some(None), None, None]);
        assert_eq!(frame(&mut view, RANGE + RANGE_BAND), [Some(None), None, None]);
        assert_eq!(view.view, ViewType::Spawning);
        view.tick(0.25);
        assert_eq!(view.alpha(), Some(63));
        view.tick(0.75);
        assert_eq!(view.view, ViewType::None);
        assert_eq!(view.alpha(), None);
        assert_eq!(frame(&mut view, 100.0), [Some(None), None, None]);
        assert_eq!(view.view, ViewType::InSight);
        view.set_view_type(ViewType::Despawning);
        assert_eq!(view.view, ViewType::Despawning);
        view.set_view_type(ViewType::OutSight);
        assert_eq!(view.view, ViewType::Despawning);
        view.tick(0.5);
        assert_eq!(view.alpha(), Some(127));
        assert!(!view.delete_now);
        view.tick(0.5);
        assert_eq!((view.view, view.alpha(), view.delete_now), (ViewType::None, None, true));
        let mut spawning = ActorView::default();
        spawning.set_view_type(ViewType::Spawning);
        spawning.tick(0.5);
        spawning.set_view_type(ViewType::Despawning);
        spawning.tick(0.25);
        assert_eq!(spawning.alpha(), Some(191));
    }

    #[test]
    fn inside_the_range_minus_the_band_the_actor_is_in_sight_and_unmodified() {
        let mut view = ActorView::default();
        assert_eq!(frame(&mut view, 100.0), [Some(None), None, None]);
        assert_eq!(view.view, ViewType::InSight);
        assert!(!view.animating());
        assert_eq!(frame(&mut view, RANGE - RANGE_BAND), [Some(None), None, None]);
    }

    #[test]
    fn an_actor_first_seen_in_the_band_draws_one_unmodified_frame_then_fades_in_over_a_second() {
        let mut view = ActorView::default();
        assert_eq!(frame(&mut view, RANGE - 100.0), [Some(None), None, None]);
        assert_eq!(view.view, ViewType::InSighting);
        assert!(view.animating());
        view.tick(0.25);
        assert_eq!(view.alpha(), Some(63));
        view.tick(0.25);
        assert_eq!(view.alpha(), Some(127));
        assert_eq!(frame(&mut view, RANGE - 100.0), [Some(Some(127)), None, None]);
        view.tick(0.6);
        assert_eq!(view.view, ViewType::InSight);
        assert_eq!(view.alpha(), None);
    }

    #[test]
    fn crossing_the_edge_slowly_fades_out_and_the_hysteresis_keeps_it_hidden_until_inside_the_range() {
        let mut view = ActorView::default();
        frame(&mut view, 100.0);
        assert_eq!(frame(&mut view, RANGE + 10.0), [Some(None), None, None]);
        assert_eq!(view.view, ViewType::OutSighting);
        view.tick(0.5);
        assert_eq!(view.alpha(), Some(127));
        view.tick(0.6);
        assert_eq!(view.view, ViewType::OutSight);
        assert_eq!(frame(&mut view, RANGE + 10.0), [None; 3]);
        assert_eq!(frame(&mut view, RANGE - 1.0), [Some(Some(0)), None, None]);
        assert_eq!(view.view, ViewType::InSighting);
    }

    #[test]
    fn jumping_past_the_band_hides_the_actor_without_a_fade() {
        let mut view = ActorView::default();
        frame(&mut view, 100.0);
        assert_eq!(frame(&mut view, RANGE + RANGE_BAND), [None; 3]);
        assert_eq!(view.view, ViewType::OutSight);
        assert_eq!(view.alpha(), None);
    }

    #[test]
    fn a_fade_out_in_progress_keeps_drawing_when_the_actor_jumps_further_out_and_reversing_mirrors_the_elapsed_time() {
        let mut view = ActorView::default();
        frame(&mut view, 100.0);
        frame(&mut view, RANGE + 10.0);
        view.tick(0.25);
        assert_eq!(frame(&mut view, RANGE + RANGE_BAND), [Some(Some(191)), None, None]);
        assert_eq!(view.view, ViewType::OutSighting);
        assert_eq!(frame(&mut view, RANGE - 10.0), [Some(Some(191)), None, None]);
        assert_eq!(view.view, ViewType::InSighting);
        view.tick(0.0);
        assert_eq!(view.alpha(), Some(191));
        view.tick(0.1);
        assert_eq!(view.alpha(), Some(216));
    }

    #[test]
    fn an_actor_with_ignored_range_never_changes_state() {
        let mut view = ActorView::default();
        let input = FrameInput { distance: 99999.0, range: RANGE, ignored_range: true, lod: None };
        assert_eq!(view.frame(input), [Some(None), None, None]);
        assert_eq!(view.view, ViewType::None);
    }

    fn lod_input(distance: f32) -> FrameInput {
        FrameInput { distance, range: 100_000.0, ignored_range: false, lod: Some(LodInput { ranges: [4096.0, 8192.0], blend: true }) }
    }

    #[test]
    fn the_first_lod_choice_starts_normal_and_a_change_of_band_starts_a_downing_blend_with_alpha_zero() {
        let mut view = ActorView::default();
        assert_eq!(view.frame(lod_input(1000.0)), [Some(None), None, None]);
        assert_eq!((view.current_lod, view.lod_blend), (0, LodBlend::Normal));
        let levels = view.frame(lod_input(5000.0));
        assert_eq!((view.current_lod, view.lod_blend), (1, LodBlend::Downing));
        assert_eq!(levels, [Some(Some(255)), Some(Some(0)), None]);
    }

    #[test]
    fn a_downing_blend_draws_the_new_lod_with_a_and_the_old_one_with_the_complement_until_it_ends_after_1_8_seconds() {
        let mut view = ActorView::default();
        view.frame(lod_input(1000.0));
        view.frame(lod_input(5000.0));
        view.tick(0.9);
        assert_eq!(view.alpha(), Some(127));
        let levels = view.frame(lod_input(5000.0));
        assert_eq!(levels, [Some(Some(128)), Some(Some(127)), None]);
        view.tick(1.0);
        assert_eq!(view.lod_blend, LodBlend::Normal);
        assert_eq!(view.alpha(), None);
        assert_eq!(view.frame(lod_input(5000.0)), [None, Some(None), None]);
    }

    #[test]
    fn an_upping_blend_draws_the_old_lod_with_the_complement_and_the_new_one_with_a() {
        let mut view = ActorView::default();
        view.frame(lod_input(5000.0));
        let levels = view.frame(lod_input(1000.0));
        assert_eq!((view.current_lod, view.lod_blend), (0, LodBlend::Upping));
        assert_eq!(levels, [Some(Some(0)), Some(Some(255)), None]);
        view.tick(0.45);
        assert_eq!(view.alpha(), Some(63));
        let levels = view.frame(lod_input(1000.0));
        assert_eq!(levels, [Some(Some(63)), Some(Some(192)), None]);
    }

    #[test]
    fn the_range_fade_has_priority_over_the_lod_blend_in_the_tick_and_blend_is_skipped_outside_in_sight() {
        let mut view = ActorView::default();
        view.frame(lod_input(1000.0));
        view.view = ViewType::OutSighting;
        view.set_lod(1, false);
        assert_eq!((view.current_lod, view.lod_blend), (1, LodBlend::None));
    }
}
