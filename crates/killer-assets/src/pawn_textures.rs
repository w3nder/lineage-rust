use std::collections::BTreeMap;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SkinArray {
    len: usize,
    entries: BTreeMap<usize, usize>,
}

impl SkinArray {
    pub fn len(&self) -> usize {
        self.len
    }
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    pub fn get(&self, index: usize) -> Option<Option<usize>> {
        (index < self.len).then(|| self.entries.get(&index).copied())
    }
    pub fn clear(&mut self) {
        self.len = 0;
        self.entries.clear();
    }
    fn insert_material(&mut self, index: usize, material: usize) {
        self.len = self.len.max(index + 1);
        self.entries.insert(index, material);
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PawnTextures {
    pub sub_skins: [Option<usize>; 7],
    pub skins: SkinArray,
    pub attached: [Option<usize>; 8],
}

pub trait TextureResources {
    fn load(&mut self, slot: i32, target: &mut Option<usize>) -> bool;
    fn enqueue(&mut self, slot: i32, texture_index: i32);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidSkinIndex {
    pub slot: i32,
    pub index: i32,
}

impl PawnTextures {
    pub fn attached_skin(&self, slot: i32, index: usize) -> Option<usize> {
        let offset = match slot {
            8..=10 if index == 0 => (slot - 8) as usize,
            11 if index < 3 => 3 + index,
            12 if index < 2 => 6 + index,
            _ => return None,
        };
        self.attached[offset]
    }

    pub fn set_texes(
        &mut self,
        slot: i32,
        has_name: bool,
        index: i32,
        resources: &mut impl TextureResources,
    ) -> Result<(), InvalidSkinIndex> {
        if !(0..=12).contains(&slot) {
            return Ok(());
        }
        if slot == 7 {
            if !has_name {
                return Ok(());
            }
            if !(0..i32::MAX).contains(&index) {
                return Err(InvalidSkinIndex { slot, index });
            }
            let mut material = None;
            if !resources.load(slot, &mut material) {
                resources.enqueue(slot, index);
            }
            if index == 0 {
                self.skins.clear();
            }
            if let Some(material) = material {
                self.skins.insert_material(index as usize, material);
            }
            return Ok(());
        }
        let target = match slot {
            0..=6 => &mut self.sub_skins[slot as usize],
            8..=10 => &mut self.attached[(slot - 8) as usize],
            11 if (0..3).contains(&index) => &mut self.attached[3 + index as usize],
            11 => return Err(InvalidSkinIndex { slot, index }),
            12 => &mut self.attached[6],
            _ => return Ok(()),
        };
        if !has_name {
            *target = None;
        } else if !resources.load(slot, target) {
            resources.enqueue(slot, index);
        }
        Ok(())
    }
}
