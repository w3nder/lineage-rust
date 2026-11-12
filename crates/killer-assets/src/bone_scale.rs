pub const MAX_SLOT: usize = 0x100;
pub const ALL_BONES_NAME: &str = "BoneName_AllBone";

#[derive(Clone, Debug, PartialEq)]
pub struct BoneScaler {
    pub bone: i32,
    pub name: Option<String>,
    pub scale: f32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct BoneScalers(pub Vec<BoneScaler>);

impl BoneScalers {
    pub fn set(&mut self, slot: usize, scale: f32, name: Option<&str>, all_bones: bool, match_ref_bone: impl Fn(&str) -> Option<usize>) -> bool {
        if slot > MAX_SLOT {
            return false;
        }
        while self.0.len() <= slot {
            self.0.push(BoneScaler { bone: -1, name: None, scale: 0.0 });
        }
        let entry = &mut self.0[slot];
        if scale == 1.0 || (name.is_none() && !all_bones) {
            entry.bone = -1;
            entry.name = None;
            return true;
        }
        if all_bones {
            *entry = BoneScaler { bone: 0, name: Some(ALL_BONES_NAME.to_owned()), scale };
            return true;
        }
        let Some(name) = name else { return true };
        if let Some(bone) = match_ref_bone(name) {
            *entry = BoneScaler { bone: bone as i32, name: Some(name.to_owned()), scale };
        }
        true
    }

    pub fn is_bone_scaler(&self, bone: usize, bone_count: usize) -> Option<f32> {
        self.0.iter().find(|s| s.bone >= 0 && (s.bone as usize) < bone_count && s.bone as usize == bone).map(|s| s.scale)
    }

    pub fn clear(&mut self, name: &str, bone_count: usize) -> bool {
        let Some(index) = self.0.iter().position(|s| s.bone >= 0 && (s.bone as usize) < bone_count && s.name.as_deref().is_some_and(|n| n.eq_ignore_ascii_case(name))) else { return false };
        self.0.remove(index);
        true
    }

    pub fn root_index(&self) -> Option<usize> {
        self.0.iter().position(|s| s.bone == 0)
    }

    pub fn remove(&mut self, index: usize) {
        if index < self.0.len() {
            self.0.remove(index);
        }
    }

    pub fn factors(&self, bone_count: usize) -> Vec<f32> {
        let mut factors = vec![1.0; bone_count];
        for scaler in self.0.iter().filter(|s| s.bone >= 0 && (s.bone as usize) < bone_count) {
            factors[scaler.bone as usize] *= scaler.scale;
        }
        factors
    }

    pub fn is_empty(&self) -> bool {
        self.0.iter().all(|s| s.bone < 0)
    }
}

pub fn is_big_head(skeletal: bool, head_bone: Option<usize>, scalers: Option<&BoneScalers>, bone_count: usize, out: &mut f32) -> bool {
    if !skeletal {
        *out = 0.0;
        return false;
    }
    let Some(head) = head_bone else { return false };
    if let Some(scale) = scalers.and_then(|scalers| scalers.is_bone_scaler(head, bone_count)) {
        *out = scale;
    }
    true
}

pub fn big_head_section(mesh_name: &str) -> &str {
    let name = mesh_name.rsplit('.').next().unwrap_or(mesh_name);
    name.find('_').map_or("", |index| &name[..index])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lookup(name: &str) -> Option<usize> {
        match name {
            "Bip01 Head" => Some(7),
            "Bip01" => Some(0),
            _ => None,
        }
    }

    #[test]
    fn set_bone_scale_10679210_follows_every_branch() {
        let mut scalers = BoneScalers::default();
        assert!(!scalers.set(0x101, 2.0, Some("Bip01 Head"), false, lookup));
        assert!(scalers.set(2, 1.5, Some("Bip01 Head"), false, lookup));
        assert_eq!(scalers.0.len(), 3);
        assert_eq!(scalers.0[0].bone, -1);
        assert_eq!(scalers.0[2], BoneScaler { bone: 7, name: Some("Bip01 Head".into()), scale: 1.5 });
        assert!(scalers.set(2, 1.0, Some("Bip01 Head"), false, lookup));
        assert_eq!((scalers.0[2].bone, scalers.0[2].name.as_deref(), scalers.0[2].scale), (-1, None, 1.5));
        assert!(scalers.set(0, 1.2, None, true, lookup));
        assert_eq!(scalers.0[0], BoneScaler { bone: 0, name: Some(ALL_BONES_NAME.into()), scale: 1.2 });
        assert!(scalers.set(0, 1.2, Some("ausente"), false, lookup));
        assert_eq!(scalers.0[0].bone, 0);
        assert!(scalers.set(0, 2.0, None, false, lookup));
        assert_eq!(scalers.0[0].bone, -1);
    }

    #[test]
    fn factors_multiply_scalers_on_the_same_bone_and_clear_removes_the_first_match() {
        let mut scalers = BoneScalers::default();
        scalers.set(0, 2.0, Some("Bip01 Head"), false, lookup);
        scalers.set(1, 1.5, Some("Bip01 Head"), false, lookup);
        assert_eq!(scalers.factors(8)[7], 3.0);
        assert_eq!(scalers.is_bone_scaler(7, 8), Some(2.0));
        assert_eq!(scalers.is_bone_scaler(7, 7), None);
        assert!(scalers.clear("bip01 head", 8));
        assert_eq!(scalers.factors(8)[7], 1.5);
        assert_eq!(scalers.root_index(), None);
    }

    #[test]
    fn is_big_head_10312f94_writes_only_when_the_engine_does() {
        let mut scalers = BoneScalers::default();
        scalers.set(0, 2.5, Some("Bip01 Head"), false, lookup);
        let mut out = 7.0;
        assert!(!is_big_head(false, Some(7), Some(&scalers), 8, &mut out));
        assert_eq!(out, 0.0);
        out = 7.0;
        assert!(!is_big_head(true, None, Some(&scalers), 8, &mut out));
        assert_eq!(out, 7.0);
        assert!(is_big_head(true, Some(7), Some(&scalers), 8, &mut out));
        assert_eq!(out, 2.5);
        out = 0.0;
        assert!(is_big_head(true, Some(7), Some(&scalers), 7, &mut out));
        assert_eq!(out, 0.0);
        assert!(is_big_head(true, Some(3), None, 8, &mut out));
        assert_eq!(out, 0.0);
    }

    #[test]
    fn big_head_section_is_the_mesh_name_before_the_underscore() {
        assert_eq!(big_head_section("LineageMonsters.MFighter_m000_u"), "MFighter");
        assert_eq!(big_head_section("FElf"), "");
    }
}
