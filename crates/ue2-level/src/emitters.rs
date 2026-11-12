use ue2_assets::{Property, class_default_properties, find, object_properties};
use ue2_package::{Export, ObjectRef, Package};
use crate::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum DrawStyle {
    Regular, AlphaBlend, Modulated, Translucent, AlphaModulate, Darken, Brighten,
}

pub type Range = [f32; 2];

#[derive(Debug, Clone, PartialEq)]
pub struct SpriteEmitter {
    pub class: String,
    pub disabled: bool,
    pub coordinate_system: u8,
    pub max_particles: u32,
    pub lifetime: Range,
    pub initial_time: Range,
    pub initial_delay: Range,
    pub forced_life_time: bool,
    pub forced_fade: bool,
    pub forced_max_particles: bool,
    pub automatic_initial_spawning: bool,
    pub respawn_dead_particles: bool,
    pub initial_particles_per_second: f32,
    pub particles_per_second: f32,
    pub auto_destroy: bool,
    pub auto_reset: bool,
    pub auto_reset_time: Range,
    pub seconds_before_inactive: f32,
    pub warmup_ticks_per_second: f32,
    pub relative_warmup_time: f32,
    pub start_offset: [f32; 3],
    pub start_location: [Range; 3],
    pub start_location_shape: u8,
    pub sphere_radius: Range,
    pub start_location_polar: [Range; 3],
    pub start_velocity: [Range; 3],
    pub acceleration: [f32; 3],
    pub independent_spray_accel: bool,
    pub max_abs_velocity: [f32; 3],
    pub velocity_loss: [Range; 3],
    pub use_velocity_scale: bool,
    pub velocity_scale_repeats: f32,
    pub use_rotation_from: u8,
    pub rotation_offset: [i32; 3],
    pub rotation_normal: [f32; 3],
    pub effect_axis: u8,
    pub velocity_direction: u8,
    pub start_velocity_radial: Range,
    pub add_location_from_other: i32,
    pub add_velocity_from_other: i32,
    pub add_velocity_multiplier: [Range; 3],
    pub start_mass: Range,
    pub start_size: [Range; 3],
    pub uniform_size: bool,
    pub use_size_scale: bool,
    pub use_regular_size_scale: bool,
    pub size_scale_repeats: f32,
    pub use_color_scale: bool,
    pub color_scale_repeats: f32,
    pub color_multiplier: [Range; 3],
    pub opacity: f32,
    pub fade_in: bool,
    pub fade_in_end: f32,
    pub fade_in_factor: [f32; 4],
    pub fade_out: bool,
    pub fade_out_start: f32,
    pub fade_out_factor: [f32; 4],
    pub spin_particles: bool,
    pub start_spin: [Range; 3],
    pub spins_per_second: [Range; 3],
    pub spin_ccw_or_cw: [f32; 3],
    pub use_revolution: bool,
    pub revolution_center: [Range; 3],
    pub revolutions_per_second: [Range; 3],
    pub use_revolution_scale: bool,
    pub revolution_scale_repeats: f32,
    pub alpha_test: bool,
    pub alpha_ref: u32,
    pub z_test: bool,
    pub z_write: bool,
    pub disable_fogging: bool,
    pub accepts_projectors: bool,
    pub direction: u8,
    pub projection_normal: [f32; 3],
    pub blend_subdivisions: bool,
    pub use_subdivision_scale: bool,
    pub random_subdivision: bool,
    pub refraction: i32,
    pub refr_u_scale: f32,
    pub refr_v_scale: f32,
    pub use_collision: bool,
    pub extent_multiplier: [f32; 3],
    pub damping_factor: [Range; 3],
    pub use_collision_planes: bool,
    pub use_max_collisions: bool,
    pub max_collisions: Range,
    pub damp_rotation: bool,
    pub rotation_damping_factor: [Range; 3],
    pub min_squared_velocity: f32,
    pub spawn_from_other: i32,
    pub spawn_amount: u32,
    pub use_spawned_velocity_scale: bool,
    pub spawned_velocity_scale: [Range; 3],
    pub use_actor_forces: bool,
    pub mesh_spawning: u8,
    pub velocity_from_mesh: bool,
    pub velocity_scale_range: [Range; 3],
    pub mesh_scale_range: [Range; 3],
    pub uniform_mesh_scale: bool,
    pub uniform_velocity_scale: bool,
    pub use_color_from_mesh: bool,
    pub spawn_only_normal: bool,
    pub mesh_normal: [f32; 3],
    pub mesh_normal_threshold: Range,
    pub use_skeletal_location: u8,
    pub skeletal_scale: [f32; 3],
    pub relative_bone_index: Range,
    pub trigger_disabled: bool,
    pub reset_on_trigger: bool,
    pub spawn_on_trigger: Range,
    pub spawn_on_trigger_pps: f32,
    pub size_scale: Vec<[f32; 2]>,
    pub color_scale: Vec<(f32, [u8; 4])>,
    pub velocity_scale: Vec<(f32, [f32; 3])>,
    pub revolution_scale: Vec<(f32, [f32; 3])>,
    pub texture: Option<String>,
    pub mesh_spawning_mesh: Option<String>,
    pub skeletal_actor: Option<String>,
    pub subdivisions: [u32; 2],
    pub subdivision_range: [u32; 2],
    pub subdivision_scale: Vec<f32>,
    pub collision_planes: Vec<[f32; 4]>,
    pub draw_style: DrawStyle,
    pub mesh: Option<MeshShape>,
    pub vertex_mesh: Option<MeshShape>,
    pub beam: Option<BeamShape>,
    pub ribbon: Option<RibbonShape>,
}

impl Default for SpriteEmitter {
    fn default() -> Self {
        Self {
            class: "SpriteEmitter".to_owned(),
            disabled: false,
            coordinate_system: 1,
            max_particles: 10,
            lifetime: [4.0; 2],
            initial_time: [0.0; 2],
            initial_delay: [0.0; 2],
            forced_life_time: false,
            forced_fade: false,
            forced_max_particles: false,
            automatic_initial_spawning: true,
            respawn_dead_particles: true,
            initial_particles_per_second: 0.0,
            particles_per_second: 0.0,
            auto_destroy: false,
            auto_reset: false,
            auto_reset_time: [0.0; 2],
            seconds_before_inactive: 0.0,
            warmup_ticks_per_second: 0.0,
            relative_warmup_time: 0.0,
            start_offset: [0.0; 3],
            start_location: [[0.0; 2]; 3],
            start_location_shape: 0,
            sphere_radius: [0.0; 2],
            start_location_polar: [[0.0; 2]; 3],
            start_velocity: [[0.0; 2]; 3],
            acceleration: [0.0; 3],
            independent_spray_accel: false,
            max_abs_velocity: [10000.0; 3],
            velocity_loss: [[0.0; 2]; 3],
            use_velocity_scale: false,
            velocity_scale_repeats: 0.0,
            use_rotation_from: 0,
            rotation_offset: [0; 3],
            rotation_normal: [0.0; 3],
            effect_axis: 0,
            velocity_direction: 0,
            start_velocity_radial: [0.0; 2],
            add_location_from_other: -1,
            add_velocity_from_other: -1,
            add_velocity_multiplier: [[1.0; 2]; 3],
            start_mass: [1.0; 2],
            start_size: [[100.0; 2]; 3],
            uniform_size: false,
            use_size_scale: false,
            use_regular_size_scale: true,
            size_scale_repeats: 0.0,
            use_color_scale: false,
            color_scale_repeats: 0.0,
            color_multiplier: [[1.0; 2]; 3],
            opacity: 1.0,
            fade_in: false,
            fade_in_end: 0.0,
            fade_in_factor: [1.0; 4],
            fade_out: false,
            fade_out_start: 0.0,
            fade_out_factor: [1.0; 4],
            spin_particles: false,
            start_spin: [[0.0; 2]; 3],
            spins_per_second: [[0.0; 2]; 3],
            spin_ccw_or_cw: [0.5; 3],
            use_revolution: false,
            revolution_center: [[0.0; 2]; 3],
            revolutions_per_second: [[0.0; 2]; 3],
            use_revolution_scale: false,
            revolution_scale_repeats: 0.0,
            alpha_test: true,
            alpha_ref: 0,
            z_test: true,
            z_write: false,
            disable_fogging: false,
            accepts_projectors: false,
            direction: 0,
            projection_normal: [0.0, 0.0, 1.0],
            blend_subdivisions: false,
            use_subdivision_scale: false,
            random_subdivision: false,
            refraction: 0,
            refr_u_scale: 0.06,
            refr_v_scale: 0.06,
            use_collision: false,
            extent_multiplier: [1.0; 3],
            damping_factor: [[1.0; 2]; 3],
            use_collision_planes: false,
            use_max_collisions: false,
            max_collisions: [0.0; 2],
            damp_rotation: false,
            rotation_damping_factor: [[0.0; 2]; 3],
            min_squared_velocity: 0.0,
            spawn_from_other: -1,
            spawn_amount: 0,
            use_spawned_velocity_scale: false,
            spawned_velocity_scale: [[0.0; 2]; 3],
            use_actor_forces: false,
            mesh_spawning: 0,
            velocity_from_mesh: false,
            velocity_scale_range: [[1.0; 2]; 3],
            mesh_scale_range: [[1.0; 2]; 3],
            uniform_mesh_scale: true,
            uniform_velocity_scale: true,
            use_color_from_mesh: false,
            spawn_only_normal: false,
            mesh_normal: [0.0, 0.0, 1.0],
            mesh_normal_threshold: [0.0; 2],
            use_skeletal_location: 0,
            skeletal_scale: [1.0; 3],
            relative_bone_index: [0.0, 1.0],
            trigger_disabled: true,
            reset_on_trigger: false,
            spawn_on_trigger: [0.0; 2],
            spawn_on_trigger_pps: 0.0,
            size_scale: Vec::new(),
            color_scale: Vec::new(),
            velocity_scale: Vec::new(),
            revolution_scale: Vec::new(),
            texture: Some("Engine.S_Emitter".to_owned()),
            mesh_spawning_mesh: None,
            skeletal_actor: None,
            subdivisions: [0; 2],
            subdivision_range: [0; 2],
            subdivision_scale: Vec::new(),
            collision_planes: Vec::new(),
            draw_style: DrawStyle::Translucent,
            mesh: None,
            vertex_mesh: None,
            beam: None,
            ribbon: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RibbonShape {
    pub sample_rate: f32,
    pub decay_rate: f32,
    pub num_points: i32,
    pub acc_drop: u8,
    pub drop_rate: i32,
    pub threshold: f32,
    pub min_points: i32,
    pub threshold2: f32,
    pub width: f32,
    pub axis: u8,
    pub axis_normal: [f32; 3],
    pub min_sample_dist: f32,
    pub min_sample_dot: f32,
    pub origin_offset: f32,
    pub uv_scale: [f32; 2],
    pub sheets: i32,
    pub sheet_scale: Vec<f32>,
    pub bone_offsets: [[f32; 3]; 2],
    pub bone_names: [Option<String>; 2],
    pub interpolate: bool,
    pub control_points: i32,
    pub scale_ratio: f32,
    pub flags: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MeshShape {
    pub mesh: Option<String>,
    pub own_blend: bool,
    pub two_sided: bool,
    pub use_particle_color: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BeamEndpoint {
    pub actor_tag: Option<String>,
    pub offset: [Range; 3],
    pub weight: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct BeamShape {
    pub distance: Range,
    pub endpoint_by: u8,
    pub u_scale: f32,
    pub v_scale: f32,
    pub rotating_sheets: u32,
    pub trigger_endpoint: bool,
    pub low_noise: [Range; 3],
    pub low_points: u32,
    pub high_noise: [Range; 3],
    pub high_points: u32,
    pub low_scale_repeats: f32,
    pub high_scale_repeats: f32,
    pub use_low_scale: bool,
    pub use_high_scale: bool,
    pub noise_endpoint: bool,
    pub dynamic_noise: [Range; 3],
    pub dynamic_points: Range,
    pub dynamic_time: Range,
    pub use_branching: bool,
    pub branch_probability: Range,
    pub branch_points: Range,
    pub branch_emitter: i32,
    pub branch_amount: Range,
    pub linkup_lifetime: bool,
    pub endpoints: Vec<BeamEndpoint>,
    pub low_scale: Vec<(f32, [f32; 3])>,
    pub high_scale: Vec<(f32, [f32; 3])>,
}

impl Default for BeamShape {
    fn default() -> Self {
        Self {
            distance: [0.0; 2],
            endpoint_by: 0,
            u_scale: 1.0,
            v_scale: 1.0,
            rotating_sheets: 0,
            trigger_endpoint: false,
            low_noise: [[0.0; 2]; 3],
            low_points: 3,
            high_noise: [[0.0; 2]; 3],
            high_points: 10,
            low_scale_repeats: 0.0,
            high_scale_repeats: 0.0,
            use_low_scale: false,
            use_high_scale: false,
            noise_endpoint: false,
            dynamic_noise: [[0.0; 2]; 3],
            dynamic_points: [0.0; 2],
            dynamic_time: [0.0; 2],
            use_branching: false,
            branch_probability: [0.0; 2],
            branch_points: [0.0; 2],
            branch_emitter: -1,
            branch_amount: [0.0; 2],
            linkup_lifetime: false,
            endpoints: Vec::new(),
            low_scale: Vec::new(),
            high_scale: Vec::new(),
        }
    }
}


#[derive(Debug, Clone, PartialEq)]
pub struct Emitter {
    pub tag: Option<String>,
    pub sun_affect: bool,
    pub location: [f32; 3],
    pub rotation: [i32; 3],
    pub velocity: [f32; 3],
    pub pre_pivot: [f32; 3],
    pub zone: Option<String>,
    pub sprites: Vec<SpriteEmitter>,
    pub skins: Vec<Option<String>>,
    pub speed_rate: f32,
    pub rotate: bool,
    pub rotation_per_second: [i32; 3],
    pub auto_destroy: bool,
    pub auto_reset: bool,
    pub disable_fogging: bool,
    pub global_offset: [Range; 3],
    pub time_till_reset: Range,
    pub l2_events: Vec<crate::L2Event>,
}

impl Default for Emitter {
    fn default() -> Self {
        Self {
            tag: None, sun_affect: false,
            location: [0.0; 3], rotation: [0; 3], velocity: [0.0; 3], pre_pivot: [0.0; 3],
            zone: None, sprites: Vec::new(), skins: Vec::new(), speed_rate: 1.0,
            rotate: false, rotation_per_second: [0; 3], auto_destroy: false, auto_reset: false,
            disable_fogging: false, global_offset: [[0.0; 2]; 3], time_till_reset: [0.0; 2],
            l2_events: Vec::new(),
        }
    }
}

pub fn class_emitter(package: &Package, file: &[u8], class: &Export, package_name: Option<&str>) -> Result<Emitter, Error> {
    let properties = class_default_properties(package, file, class)?;
    read(package, file, &properties, None, package_name)
}

pub(crate) fn read(package: &Package, file: &[u8], properties: &[Property<'_>], zone: Option<String>, package_name: Option<&str>) -> Result<Emitter, Error> {
    let r = Read { package, properties, package_name };
    let mut sprites = Vec::new();
    for object in r.get("Emitters").and_then(|p| p.objects(package)).unwrap_or_default() {
        let ObjectRef::Export(index) = object else {
            sprites.push(SpriteEmitter { class: String::new(), disabled: true, ..Default::default() });
            continue;
        };
        let Some(export) = package.exports().get(index) else {
            sprites.push(SpriteEmitter { class: String::new(), disabled: true, ..Default::default() });
            continue;
        };
        let properties = object_properties(package, file, export)?;
        let source = Read { package, properties: &properties, package_name };
        let mut emitter = sprite(&source);
        emitter.class = package.class_name(export).to_owned();
        if emitter.class.eq_ignore_ascii_case("MeshEmitter") {
            emitter.mesh = Some(MeshShape {
                mesh: source.path("StaticMesh"), own_blend: source.flag("UseMeshBlendMode", false),
                two_sided: source.flag("RenderTwoSided", false), use_particle_color: source.flag("UseParticleColor", false),
            });
        } else if emitter.class.eq_ignore_ascii_case("VertMeshEmitter") {
            emitter.vertex_mesh = Some(MeshShape {
                mesh: source.path("VertexMesh"), own_blend: source.flag("UseMeshBlendMode", true),
                two_sided: source.flag("RenderTwoSided", true), use_particle_color: source.flag("UseParticleColor", false),
            });
        } else if emitter.class.eq_ignore_ascii_case("RibbonEmitter") {
            emitter.ribbon = Some(ribbon(&source));
            emitter.max_particles = 1;
            emitter.automatic_initial_spawning = source.flag("AutomaticInitialSpawning", false);
            emitter.initial_particles_per_second = source.float("InitialParticlesPerSecond", 10000.0);
            emitter.use_regular_size_scale = source.flag("UseRegularSizeScale", false);
            emitter.start_size = source.ranges("StartSizeRange", [[1.0; 2]; 3]);
        } else if emitter.class.eq_ignore_ascii_case("BeamEmitter") {
            emitter.beam = Some(beam(&source));
        }
        sprites.push(emitter);
    }
    Ok(Emitter {
        tag: r.get("Tag").and_then(|p| p.name_value(r.package)).filter(|name| !name.eq_ignore_ascii_case("None")).map(str::to_owned),
        sun_affect: r.flag("bSunAffect", false),
        location: r.vector("Location", [0.0; 3]), rotation: r.rotator("Rotation", [0; 3]),
        velocity: r.vector("Velocity", [0.0; 3]), pre_pivot: r.vector("PrePivot", [0.0; 3]),
        zone, sprites,
        skins: properties.iter().filter(|p| p.name.eq_ignore_ascii_case("Skins")).fold(Vec::new(), |mut skins, p| {
            let index = p.index as usize;
            if index < 65536 { skins.resize(skins.len().max(index + 1), None); skins[index] = p.object(package).and_then(|o| r.object_path(o)); }
            skins
        }),
        speed_rate: r.float("SpeedRate", 1.0), rotate: r.flag("bRotEmitter", false),
        rotation_per_second: r.rotator("RotPerSecond", [0; 3]), auto_destroy: r.flag("AutoDestroy", false),
        auto_reset: r.flag("AutoReset", false), disable_fogging: r.flag("DisableFogging", false),
        global_offset: r.ranges("GlobalOffsetRange", [[0.0; 2]; 3]), time_till_reset: r.range("TimeTillResetRange", [0.0; 2]),
        l2_events: crate::l2_events(package, properties),
    })
}

fn sprite(r: &Read<'_, '_>) -> SpriteEmitter {
    SpriteEmitter {
        disabled: r.flag("Disabled", false),
        coordinate_system: r.byte("CoordinateSystem", 1),
        max_particles: r.unsigned("MaxParticles", 10),
        lifetime: r.range("LifetimeRange", [4.0; 2]),
        initial_time: r.range("InitialTimeRange", [0.0; 2]),
        initial_delay: r.range("InitialDelayRange", [0.0; 2]),
        forced_life_time: r.flag("ForcedLifeTime", false),
        forced_fade: r.flag("ForcedFade", false),
        forced_max_particles: r.flag("ForcedMaxParticles", false),
        automatic_initial_spawning: r.flag("AutomaticInitialSpawning", true),
        respawn_dead_particles: r.flag("RespawnDeadParticles", true),
        initial_particles_per_second: r.float("InitialParticlesPerSecond", 0.0),
        particles_per_second: r.float("ParticlesPerSecond", 0.0),
        auto_destroy: r.flag("AutoDestroy", false),
        auto_reset: r.flag("AutoReset", false),
        auto_reset_time: r.range("AutoResetTimeRange", [0.0; 2]),
        seconds_before_inactive: r.float("SecondsBeforeInactive", 0.0),
        warmup_ticks_per_second: r.float("WarmupTicksPerSecond", 0.0),
        relative_warmup_time: r.float("RelativeWarmupTime", 0.0),
        start_offset: r.vector("StartLocationOffset", [0.0; 3]),
        start_location: r.ranges("StartLocationRange", [[0.0; 2]; 3]),
        start_location_shape: r.byte("StartLocationShape", 0),
        sphere_radius: r.range("SphereRadiusRange", [0.0; 2]),
        start_location_polar: r.ranges("StartLocationPolarRange", [[0.0; 2]; 3]),
        start_velocity: r.ranges("StartVelocityRange", [[0.0; 2]; 3]),
        acceleration: r.vector("Acceleration", [0.0; 3]),
        independent_spray_accel: r.flag("IndependentSprayAccel", false),
        max_abs_velocity: r.vector("MaxAbsVelocity", [10000.0; 3]),
        velocity_loss: r.ranges("VelocityLossRange", [[0.0; 2]; 3]),
        use_velocity_scale: r.flag("UseVelocityScale", false),
        velocity_scale_repeats: r.float("VelocityScaleRepeats", 0.0),
        use_rotation_from: r.byte("UseRotationFrom", 0),
        rotation_offset: r.rotator("RotationOffset", [0; 3]),
        rotation_normal: r.vector("RotationNormal", [0.0; 3]),
        effect_axis: r.byte("EffectAxis", 0),
        velocity_direction: r.byte("GetVelocityDirectionFrom", 0),
        start_velocity_radial: r.range("StartVelocityRadialRange", [0.0; 2]),
        add_location_from_other: r.int("AddLocationFromOtherEmitter", -1),
        add_velocity_from_other: r.int("AddVelocityFromOtherEmitter", -1),
        add_velocity_multiplier: r.ranges("AddVelocityMultiplierRange", [[1.0; 2]; 3]),
        start_mass: r.range("StartMassRange", [1.0; 2]),
        start_size: r.ranges("StartSizeRange", [[100.0; 2]; 3]),
        uniform_size: r.flag("UniformSize", false),
        use_size_scale: r.flag("UseSizeScale", false),
        use_regular_size_scale: r.flag("UseRegularSizeScale", true),
        size_scale_repeats: r.float("SizeScaleRepeats", 0.0),
        use_color_scale: r.flag("UseColorScale", false),
        color_scale_repeats: r.float("ColorScaleRepeats", 0.0),
        color_multiplier: r.ranges("ColorMultiplierRange", [[1.0; 2]; 3]),
        opacity: r.float("Opacity", 1.0),
        fade_in: r.flag("FadeIn", false),
        fade_in_end: r.float("FadeInEndTime", 0.0),
        fade_in_factor: r.plane("FadeInFactor", [1.0; 4]),
        fade_out: r.flag("FadeOut", false),
        fade_out_start: r.float("FadeOutStartTime", 0.0),
        fade_out_factor: r.plane("FadeOutFactor", [1.0; 4]),
        spin_particles: r.flag("SpinParticles", false),
        start_spin: r.ranges("StartSpinRange", [[0.0; 2]; 3]),
        spins_per_second: r.ranges("SpinsPerSecondRange", [[0.0; 2]; 3]),
        spin_ccw_or_cw: r.vector("SpinCCWorCW", [0.5; 3]),
        use_revolution: r.flag("UseRevolution", false),
        revolution_center: r.ranges("RevolutionCenterOffsetRange", [[0.0; 2]; 3]),
        revolutions_per_second: r.ranges("RevolutionsPerSecondRange", [[0.0; 2]; 3]),
        use_revolution_scale: r.flag("UseRevolutionScale", false),
        revolution_scale_repeats: r.float("RevolutionScaleRepeats", 0.0),
        alpha_test: r.flag("AlphaTest", true),
        alpha_ref: r.unsigned("AlphaRef", 0),
        z_test: r.flag("ZTest", true),
        z_write: r.flag("ZWrite", false),
        disable_fogging: r.flag("DisableFogging", false),
        accepts_projectors: r.flag("AcceptsProjectors", false),
        direction: r.byte("UseDirectionAs", 0),
        projection_normal: r.vector("ProjectionNormal", [0.0, 0.0, 1.0]),
        blend_subdivisions: r.flag("BlendBetweenSubdivisions", false),
        use_subdivision_scale: r.flag("UseSubdivisionScale", false),
        random_subdivision: r.flag("UseRandomSubdivision", false),
        refraction: r.int("Refraction", 0),
        refr_u_scale: r.float("RefrUScale", 0.06),
        refr_v_scale: r.float("RefrVScale", 0.06),
        use_collision: r.flag("UseCollision", false),
        extent_multiplier: r.vector("ExtentMultiplier", [1.0; 3]),
        damping_factor: r.ranges("DampingFactorRange", [[1.0; 2]; 3]),
        use_collision_planes: r.flag("UseCollisionPlanes", false),
        use_max_collisions: r.flag("UseMaxCollisions", false),
        max_collisions: r.range("MaxCollisions", [0.0; 2]),
        damp_rotation: r.flag("DampRotation", false),
        rotation_damping_factor: r.ranges("RotationDampingFactorRange", [[0.0; 2]; 3]),
        min_squared_velocity: r.float("MinSquaredVelocity", 0.0),
        spawn_from_other: r.int("SpawnFromOtherEmitter", -1),
        spawn_amount: r.unsigned("SpawnAmount", 0),
        use_spawned_velocity_scale: r.flag("UseSpawnedVelocityScale", false),
        spawned_velocity_scale: r.ranges("SpawnedVelocityScaleRange", [[0.0; 2]; 3]),
        use_actor_forces: r.flag("UseActorForces", false),
        mesh_spawning: r.byte("MeshSpawning", 0),
        velocity_from_mesh: r.flag("VelocityFromMesh", false),
        velocity_scale_range: r.ranges("VelocityScaleRange", [[1.0; 2]; 3]),
        mesh_scale_range: r.ranges("MeshScaleRange", [[1.0; 2]; 3]),
        uniform_mesh_scale: r.flag("UniformMeshScale", true),
        uniform_velocity_scale: r.flag("UniformVelocityScale", true),
        use_color_from_mesh: r.flag("UseColorFromMesh", false),
        spawn_only_normal: r.flag("SpawnOnlyInDirectionOfNormal", false),
        mesh_normal: r.vector("MeshNormal", [0.0, 0.0, 1.0]),
        mesh_normal_threshold: r.range("MeshNormalThresholdRange", [0.0; 2]),
        use_skeletal_location: r.byte("UseSkeletalLocationAs", 0),
        skeletal_scale: r.vector("SkeletalScale", [1.0; 3]),
        relative_bone_index: r.range("RelativeBoneIndexRange", [0.0, 1.0]),
        trigger_disabled: r.flag("TriggerDisabled", true),
        reset_on_trigger: r.flag("ResetOnTrigger", false),
        spawn_on_trigger: r.range("SpawnOnTriggerRange", [0.0; 2]),
        spawn_on_trigger_pps: r.float("SpawnOnTriggerPPS", 0.0),
        size_scale: r.curve("SizeScale", "RelativeTime", "RelativeSize").into_iter().map(|(t, p)| [t, p.and_then(|p| p.float()).unwrap_or(0.0)]).collect(),
        color_scale: r.curve("ColorScale", "RelativeTime", "Color").into_iter().map(|(t, p)| (t, p.and_then(|p| p.color()).unwrap_or([0; 4]))).collect(),
        velocity_scale: r.vector_curve("VelocityScale", "RelativeTime", "RelativeVelocity"),
        revolution_scale: r.vector_curve("RevolutionScale", "RelativeTime", "RelativeRevolution"),
        texture: if r.get("Texture").is_some() { r.path("Texture") } else { Some("Engine.S_Emitter".to_owned()) },
        mesh_spawning_mesh: r.path("MeshSpawningStaticMesh"), skeletal_actor: r.path("SkeletalMeshActor"),
        subdivisions: [r.unsigned("TextureUSubdivisions", 0), r.unsigned("TextureVSubdivisions", 0)],
        subdivision_range: [r.unsigned("SubdivisionStart", 0), r.unsigned("SubdivisionEnd", 0)],
        subdivision_scale: r.get("SubdivisionScale").and_then(Property::floats).unwrap_or_default(),
        collision_planes: r.get("CollisionPlanes").and_then(|p| p.structs(r.package)).unwrap_or_default().iter().map(|fields| {
            ["X", "Y", "Z", "W"].map(|name| find(fields, name).and_then(Property::float).unwrap_or(0.0))
        }).collect(),
        draw_style: [DrawStyle::Regular, DrawStyle::AlphaBlend, DrawStyle::Modulated, DrawStyle::Translucent, DrawStyle::AlphaModulate, DrawStyle::Darken, DrawStyle::Brighten].get(usize::from(r.byte("DrawStyle", 3))).copied().unwrap_or(DrawStyle::Translucent),
        ..Default::default()
    }
}

fn ribbon(r: &Read<'_, '_>) -> RibbonShape {
    RibbonShape {
        sample_rate: r.float("SampleRate", 0.05), decay_rate: r.float("DecayRate", 0.0), num_points: r.int("NumPoints", 20),
        acc_drop: r.byte("AccDrop", 0), drop_rate: r.int("PointsDropRate", 0), threshold: r.float("ThresholdTime", 0.0),
        min_points: r.int("MinPoints", 0), threshold2: r.float("ThresholdTime2", 0.0), width: r.float("RibbonWidth", 20.0),
        axis: r.byte("GetPointAxisFrom", 5), axis_normal: r.vector("AxisNormal", [0.0, 0.0, 1.0]),
        min_sample_dist: r.float("MinSampleDist", 1.0), min_sample_dot: r.float("MinSampleDot", 0.995), origin_offset: r.float("PointOriginOffset", 0.5),
        uv_scale: [r.float("RibbonTextureUScale", 1.0), r.float("RibbonTextureVScale", 1.0)], sheets: r.int("NumSheets", 0),
        sheet_scale: r.get("SheetScale").and_then(Property::floats).unwrap_or_default(),
        bone_offsets: [r.vector("StartBoneOffset", [0.0; 3]), r.vector("EndBoneOffset", [0.0; 3])],
        bone_names: [r.get("BoneNameStart").and_then(|p| p.name_value(r.package)).map(str::to_owned), r.get("BoneNameEnd").and_then(|p| p.name_value(r.package)).map(str::to_owned)],
        interpolate: r.flag("bUseInterpolation", false), control_points: r.int("CntrPoints", 5), scale_ratio: r.float("ScaleRatio", 3.0),
        flags: ["bSamplePoints", "bDecayPoints", "bDecayPointsWhenStopped", "bSyncDecayWhenKilled", "bLengthBasedTextureU", "bUseSheetScale", "bUseBones", "bUseBoneDistance"]
            .into_iter().enumerate().fold(0, |flags, (i, name)| flags | (u8::from(r.flag(name, i < 2)) << i)),
    }
}

fn beam(r: &Read<'_, '_>) -> BeamShape {
    BeamShape {
        distance: r.range("BeamDistanceRange", [0.0; 2]),
        endpoint_by: r.byte("DetermineEndPointBy", 0),
        u_scale: r.float("BeamTextureUScale", 1.0),
        v_scale: r.float("BeamTextureVScale", 1.0),
        rotating_sheets: r.unsigned("RotatingSheets", 0),
        trigger_endpoint: r.flag("TriggerEndpoint", false),
        low_noise: r.ranges("LowFrequencyNoiseRange", [[0.0; 2]; 3]),
        low_points: r.unsigned("LowFrequencyPoints", 3),
        high_noise: r.ranges("HighFrequencyNoiseRange", [[0.0; 2]; 3]),
        high_points: r.unsigned("HighFrequencyPoints", 10),
        low_scale_repeats: r.float("LFScaleRepeats", 0.0),
        high_scale_repeats: r.float("HFScaleRepeats", 0.0),
        use_low_scale: r.flag("UseLowFrequencyScale", false),
        use_high_scale: r.flag("UseHighFrequencyScale", false),
        noise_endpoint: r.flag("NoiseDeterminesEndPoint", false),
        dynamic_noise: r.ranges("DynamicHFNoiseRange", [[0.0; 2]; 3]),
        dynamic_points: r.range("DynamicHFNoisePointsRange", [0.0; 2]),
        dynamic_time: r.range("DynamicTimeBetweenNoiseRange", [0.0; 2]),
        use_branching: r.flag("UseBranching", false),
        branch_probability: r.range("BranchProbability", [0.0; 2]),
        branch_points: r.range("BranchHFPointsRange", [0.0; 2]),
        branch_emitter: r.int("BranchEmitter", -1),
        branch_amount: r.range("BranchSpawnAmountRange", [0.0; 2]),
        linkup_lifetime: r.flag("LinkupLifetime", false),
        endpoints: r.get("BeamEndPoints").and_then(|p| p.structs(r.package)).unwrap_or_default().iter().map(|properties| {
            let r = Read { properties, ..*r };
            BeamEndpoint { actor_tag: r.get("ActorTag").and_then(|p| p.name_value(r.package)).filter(|name| !name.eq_ignore_ascii_case("None")).map(str::to_owned), offset: r.ranges("Offset", [[0.0; 2]; 3]), weight: r.float("Weight", 0.0) }
        }).collect(),
        low_scale: r.vector_curve("LFScaleFactors", "RelativeLength", "FrequencyScale"),
        high_scale: r.vector_curve("HFScaleFactors", "RelativeLength", "FrequencyScale"),
    }
}

struct Read<'a, 'b> {
    package: &'a Package,
    properties: &'b [Property<'a>],
    package_name: Option<&'b str>,
}

impl<'a> Read<'a, '_> {
    fn get(&self, name: &str) -> Option<&Property<'a>> { find(self.properties, name) }
    fn float(&self, name: &str, default: f32) -> f32 { self.get(name).and_then(Property::float).unwrap_or(default) }
    fn int(&self, name: &str, default: i32) -> i32 { self.get(name).and_then(Property::int).unwrap_or(default) }
    fn unsigned(&self, name: &str, default: u32) -> u32 { self.get(name).and_then(Property::int).map(|v| v.max(0) as u32).unwrap_or(default) }
    fn byte(&self, name: &str, default: u8) -> u8 { self.get(name).and_then(Property::byte).unwrap_or(default) }
    fn flag(&self, name: &str, default: bool) -> bool { self.get(name).and_then(Property::bool).unwrap_or(default) }
    fn vector(&self, name: &str, default: [f32; 3]) -> [f32; 3] { self.get(name).and_then(Property::vector).unwrap_or(default) }
    fn rotator(&self, name: &str, default: [i32; 3]) -> [i32; 3] { self.get(name).and_then(Property::rotator).unwrap_or(default) }
    fn plane(&self, name: &str, default: [f32; 4]) -> [f32; 4] { self.get(name).and_then(|p| p.plane(self.package)).unwrap_or(default) }
    fn range(&self, name: &str, default: Range) -> Range { self.get(name).map_or(default, |p| range_of(self.package, p)) }
    fn ranges(&self, name: &str, default: [Range; 3]) -> [Range; 3] { self.get(name).map_or(default, |p| range_vector(self.package, p)) }
    fn object_path(&self, object: ObjectRef) -> Option<String> {
        match object {
            ObjectRef::Null => None,
            ObjectRef::Export(index) if self.package_name.is_some() => self.package_name.map(|name| format!("{name}.{}#{index}", self.package.object_path(object))),
            object => Some(self.package.object_path(object)),
        }
    }
    fn path(&self, name: &str) -> Option<String> { self.get(name).and_then(|p| p.object(self.package)).and_then(|o| self.object_path(o)) }
    fn curve(&self, name: &str, time: &str, value: &str) -> Vec<(f32, Option<Property<'a>>)> {
        self.get(name).and_then(|p| p.structs(self.package)).unwrap_or_default().iter().map(|point| {
            (find(point, time).and_then(Property::float).unwrap_or(0.0), find(point, value).copied())
        }).collect()
    }
    fn vector_curve(&self, name: &str, time: &str, value: &str) -> Vec<(f32, [f32; 3])> {
        self.curve(name, time, value).into_iter().map(|(t, p)| (t, p.and_then(|p| p.vector()).unwrap_or([0.0; 3]))).collect()
    }
}

pub(crate) fn range_of(package: &Package, range: &Property<'_>) -> Range {
    let fields = range.fields(package).unwrap_or_default();
    ["Min", "Max"].map(|name| find(&fields, name).and_then(Property::float).unwrap_or(0.0))
}

pub(crate) fn range_vector(package: &Package, ranges: &Property<'_>) -> [Range; 3] {
    let fields = ranges.fields(package).unwrap_or_default();
    ["X", "Y", "Z"].map(|axis| find(&fields, axis).map_or([0.0; 2], |range| range_of(package, range)))
}
