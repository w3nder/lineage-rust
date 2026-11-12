use crate::texture_modifier::TextureModifyInfo;

pub trait AbnormalEffectHost {
    type Actor: Copy;
    type Mesh: Copy;
    type Emitter: Copy;
    type Name: Copy;
    type Material: Copy;

    fn destroy_emitter(&mut self, owner: Self::Actor, emitter: Self::Emitter);
    fn report_missing_emitter_owner(&mut self);
    fn is_pawn(&self, actor: Self::Actor) -> bool;
    fn active_associated_pawn(&self, actor: Self::Actor) -> Option<Self::Actor>;
    fn skeletal_instance(&self, actor: Self::Actor) -> Option<Self::Mesh>;
    fn texture_info(&mut self, actor: Self::Actor) -> &mut TextureModifyInfo;
    fn clear_texture_modifier(&mut self, actor: Self::Actor);
    fn head_bone(&mut self, actor: Self::Actor) -> Self::Name;
    fn clear_bone_scaler(&mut self, mesh: Self::Mesh, name: Self::Name);
    fn clear_scale_fields(&mut self, mesh: Self::Mesh);
    fn root_scaler_index(&self, mesh: Self::Mesh) -> Option<usize>;
    fn remove_scaler(&mut self, mesh: Self::Mesh, index: usize);
    fn restore_field_4bc(&mut self, actor: Self::Actor, value: u32);
    fn has_game_data(&self) -> bool;
    fn has_mesh_instance(&self, actor: Self::Actor) -> bool;
    fn has_npc_texture_table(&self, actor: Self::Actor) -> bool;
    fn normal_npc_textures(&mut self, actor: Self::Actor) -> Vec<Self::Name>;
    fn find_material_sequence(&mut self, name: Self::Name) -> Option<Self::Material>;
    fn load_material_sequence(&mut self, name: Self::Name) -> Option<Self::Material>;
    fn trigger_material_sequence(&mut self, material: Self::Material);
    fn set_npc_texture(&mut self, actor: Self::Actor, name: Self::Name, index: usize);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AbnormalDestroyError {
    OverlayWithoutOwner,
}

pub struct AbnormalEffect<Actor, Emitter> {
    pub owner: Option<Actor>,
    pub emitter: Option<Emitter>,
    pub mask: u32,
    pub resource: String,
    pub saved_field_4bc: u32,
}

impl<Actor: Copy, Emitter: Copy> AbnormalEffect<Actor, Emitter> {
    pub fn destroy<H: AbnormalEffectHost<Actor = Actor, Emitter = Emitter>>(
        self,
        host: &mut H,
    ) -> Result<(), AbnormalDestroyError> {
        if let Some(emitter) = self.emitter {
            if let Some(owner) = self.owner {
                host.destroy_emitter(owner, emitter);
            } else {
                host.report_missing_emitter_owner();
            }
        }
        match self.mask {
            0x800 => {
                let owner = self.owner.ok_or(AbnormalDestroyError::OverlayWithoutOwner)?;
                Self::restore_overlay(host, owner);
                if host.is_pawn(owner)
                    && let Some(associated) = host.active_associated_pawn(owner)
                {
                    Self::restore_overlay(host, associated);
                }
            }
            0x2000 | 0x10000 => {
                if let Some(owner) = self.owner.filter(|owner| host.is_pawn(*owner)) {
                    let actor = host.active_associated_pawn(owner).unwrap_or(owner);
                    if let Some(mesh) = host.skeletal_instance(actor) {
                        if self.mask == 0x2000 {
                            let name = host.head_bone(actor);
                            host.clear_bone_scaler(mesh, name);
                        } else if let Some(index) = host.root_scaler_index(mesh) {
                            host.remove_scaler(mesh, index);
                        }
                        host.clear_scale_fields(mesh);
                        if self.mask == 0x10000 {
                            host.restore_field_4bc(actor, self.saved_field_4bc);
                        }
                    }
                }
            }
            0x8000 => {
                if let Some(owner) = self.owner.filter(|owner| host.is_pawn(*owner))
                    && host.has_game_data()
                    && host.has_mesh_instance(owner)
                    && host.has_npc_texture_table(owner)
                {
                    for (index, name) in host.normal_npc_textures(owner).into_iter().enumerate() {
                        if let Some(material) =
                            host.find_material_sequence(name).or_else(|| host.load_material_sequence(name))
                        {
                            host.trigger_material_sequence(material);
                        }
                        host.set_npc_texture(owner, name, index);
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn restore_overlay<H: AbnormalEffectHost<Actor = Actor, Emitter = Emitter>>(host: &mut H, actor: Actor) {
        let info = host.texture_info(actor);
        if info.flags & 5 == 5 {
            info.color |= 0x00ffffff;
        } else {
            host.clear_texture_modifier(actor);
        }
    }
}
