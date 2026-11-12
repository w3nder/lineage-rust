#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TextureModifyInfo {
    pub flags: u32,
    pub color: u32,
    pub value_08: u32,
    pub value_0c: u32,
}

impl TextureModifyInfo {
    pub fn clear(&mut self) {
        self.flags &= !7;
        self.color = u32::MAX;
        self.value_08 = 1;
        self.value_0c = 1;
    }

    pub fn is_modified_texture(self) -> bool {
        self.flags & 5 == 5
    }

    pub fn uses_modifier(self) -> bool {
        self.flags & 1 != 0
    }

    pub fn clear_alpha(&mut self) {
        if self.value_0c == 1 {
            self.clear();
        } else {
            self.flags &= !4;
            self.value_08 = 1;
            self.color |= 0xff000000;
        }
    }

    pub fn set_alpha(&mut self, alpha: u8) {
        self.flags |= 5;
        self.value_08 = 2;
        self.color = (self.color & 0x00ffffff) | (u32::from(alpha) << 24);
    }

    pub fn set_overlay(&mut self, rgb: [u8; 3], mode: u8) {
        self.set_color(rgb, match mode { 2 => 4, 4 => 5, _ => 3 });
    }

    pub fn set_subtract(&mut self, rgb: [u8; 3]) {
        self.set_color(rgb, 9);
    }

    pub fn set_add(&mut self, rgb: [u8; 3]) {
        self.set_color(rgb, 10);
    }

    pub fn set_change_color(&mut self, rgb: [u8; 3]) {
        self.set_color(rgb, 2);
    }

    pub fn set_overlay_depend_alpha(&mut self, rgb: [u8; 3]) {
        self.set_color(rgb, 13);
    }

    pub fn overlay_factor(self) -> [f32; 4] {
        let multiplier = match self.value_0c {
            3 => 1.0,
            4 => 2.0,
            5 => 4.0,
            _ => 0.0,
        };
        if self.flags & 1 == 0 || multiplier == 0.0 {
            return [0.0; 4];
        }
        let channel = |shift: u32| ((self.color >> shift) & 0xff) as f32 / 255.0;
        [channel(16), channel(8), channel(0), multiplier]
    }

    fn set_color(&mut self, [red, green, blue]: [u8; 3], operation: u32) {
        self.flags |= 1;
        self.color = (self.color & 0xff000000)
            | (u32::from(red) << 16)
            | (u32::from(green) << 8)
            | u32::from(blue);
        self.value_0c = operation;
    }
}

#[cfg(test)]
mod overlay_tests {
    use super::*;

    #[test]
    fn overlay_factor_is_the_tfactor_color_with_the_modulate_scale() {
        let mut info = TextureModifyInfo { flags: 0, color: u32::MAX, value_08: 1, value_0c: 1 };
        assert_eq!(info.overlay_factor(), [0.0; 4]);
        info.set_overlay([0x48, 0x69, 0x8c], 1);
        assert_eq!(info.overlay_factor(), [72.0 / 255.0, 105.0 / 255.0, 140.0 / 255.0, 1.0]);
        info.set_overlay([255, 0, 0], 2);
        assert_eq!(info.overlay_factor()[3], 2.0);
        info.set_add([1, 2, 3]);
        assert_eq!(info.overlay_factor(), [0.0; 4]);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct L2ColorModifier {
    pub material: Option<usize>,
    pub color: u32,
    pub flags: u32,
    pub byte_50: u8,
    pub byte_51: u8,
}

impl L2ColorModifier {
    fn update(&mut self, info: TextureModifyInfo) {
        self.flags = (self.flags & !3) | ((info.flags >> 1) & 3);
        self.color = info.color;
        self.byte_50 = info.value_08 as u8;
        self.byte_51 = info.value_0c as u8;
    }
}

#[derive(Debug, Default)]
pub struct TextureModifierCache {
    material: Option<L2ColorModifier>,
}

impl TextureModifierCache {
    pub fn with_material(material: L2ColorModifier) -> Self {
        Self { material: Some(material) }
    }

    pub fn get_modified_texture_material<E>(
        &mut self,
        info: TextureModifyInfo,
        create: impl FnOnce() -> Result<L2ColorModifier, E>,
    ) -> Result<&mut L2ColorModifier, E> {
        let material = match &mut self.material {
            Some(material) => material,
            slot @ None => slot.insert(create()?),
        };
        material.update(info);
        Ok(material)
    }
}
