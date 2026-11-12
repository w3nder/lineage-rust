use std::collections::HashSet;

pub fn sub_skin(skins: Option<&[Option<usize>; 7]>, index: i32) -> Option<usize> {
    skins?.get(index as u32 as usize).copied().flatten()
}

pub trait MaterialContext {
    fn has_actor(&self) -> bool;
    fn skin(&mut self, index: usize) -> Option<usize>;
    fn first_skin(&self) -> Option<Option<usize>>;
    fn sub_skin(&mut self, index: i32) -> Option<usize>;
    fn mesh_texture(&self, index: usize) -> Option<Option<usize>>;
    fn modification_enabled(&self) -> bool;
    fn modifier(&mut self) -> Option<usize>;
    fn shader_flags(&self, material: usize) -> Option<u32>;
    fn color_modifier_flags(&self, material: usize) -> Option<u32>;
    fn set_color_modifier_flags(&mut self, material: usize, flags: u32);
    fn set_modifier_material(&mut self, modifier: usize, material: Option<usize>);
    fn base_texture(&self, material: usize) -> Option<usize>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidTextureIndex(pub i32);

pub trait AttachedMaterialContext: MaterialContext {
    fn is_pawn(&self) -> bool;
    fn attached_skin(&mut self, slot: i32, index: usize) -> Option<usize>;
}

pub trait MaterialSelectionContext {
    fn has_actor(&self) -> bool;
    fn skin(&mut self, index: usize) -> Option<usize>;
    fn first_skin(&self) -> Option<Option<usize>>;
    fn sub_skin(&mut self, index: i32) -> Option<usize>;
    fn mesh_texture(&self, index: usize) -> Option<Option<usize>>;
}

impl<T: MaterialContext> MaterialSelectionContext for T {
    fn has_actor(&self) -> bool {
        MaterialContext::has_actor(self)
    }
    fn skin(&mut self, index: usize) -> Option<usize> {
        MaterialContext::skin(self, index)
    }
    fn first_skin(&self) -> Option<Option<usize>> {
        MaterialContext::first_skin(self)
    }
    fn sub_skin(&mut self, index: i32) -> Option<usize> {
        MaterialContext::sub_skin(self, index)
    }
    fn mesh_texture(&self, index: usize) -> Option<Option<usize>> {
        MaterialContext::mesh_texture(self, index)
    }
}

pub trait AttachedSelectionContext: MaterialSelectionContext {
    fn is_pawn(&self) -> bool;
    fn attached_skin(&mut self, slot: i32, index: usize) -> Option<usize>;
}

impl<T: AttachedMaterialContext> AttachedSelectionContext for T {
    fn is_pawn(&self) -> bool {
        AttachedMaterialContext::is_pawn(self)
    }
    fn attached_skin(&mut self, slot: i32, index: usize) -> Option<usize> {
        AttachedMaterialContext::attached_skin(self, slot, index)
    }
}

pub fn get_attached_weapon_texture(
    context: &mut impl AttachedMaterialContext,
    index: i32,
    slot: i32,
) -> Result<Option<usize>, InvalidTextureIndex> {
    let Some(material) = select_attached_weapon_texture(context, index, slot)? else { return Ok(None) };
    if MaterialContext::has_actor(context)
        && context.modification_enabled()
        && let Some(modifier) = context.modifier()
    {
        context.set_modifier_material(modifier, Some(material));
        return Ok(Some(modifier));
    }
    Ok(Some(material))
}

pub fn select_attached_weapon_texture(
    context: &mut impl AttachedSelectionContext,
    index: i32,
    slot: i32,
) -> Result<Option<usize>, InvalidTextureIndex> {
    if !(8..=12).contains(&slot) {
        return Ok(None);
    }
    let index = usize::try_from(index).map_err(|_| InvalidTextureIndex(index))?;
    let material = if context.has_actor() && context.attached_skin(slot, index).is_some() {
        context.attached_skin(slot, index)
    } else if index != 0 && context.mesh_texture(index).is_some() {
        context.mesh_texture(index).flatten()
    } else {
        let fallback_index = if slot == 11 { index } else { 0 };
        if context.has_actor()
            && (slot != 11 || (context.is_pawn() && index < 3))
            && context.attached_skin(slot, fallback_index).is_some()
        {
            context.attached_skin(slot, fallback_index)
        } else {
            context.mesh_texture(index).or_else(|| context.mesh_texture(0)).flatten()
        }
    };
    Ok(material)
}

pub fn get_material(context: &mut impl MaterialContext, index: i32) -> Result<Option<usize>, InvalidTextureIndex> {
    let material = select_material(context, index)?;
    Ok(modify_material(context, material, false))
}

pub fn select_material(
    context: &mut impl MaterialSelectionContext,
    index: i32,
) -> Result<Option<usize>, InvalidTextureIndex> {
    let index = usize::try_from(index).map_err(|_| InvalidTextureIndex(index))?;
    let material = if context.has_actor() && context.skin(index).is_some() {
        context.skin(index)
    } else if index != 0 && context.mesh_texture(index).is_some() {
        context.mesh_texture(index).flatten()
    } else if context.has_actor() && context.first_skin().is_some() {
        context.first_skin().flatten()
    } else {
        context.mesh_texture(index).or_else(|| context.mesh_texture(0)).flatten()
    };
    Ok(material)
}

pub fn get_sub_material(
    context: &mut impl MaterialContext,
    index: i32,
    sub_index: i32,
) -> Result<Option<usize>, InvalidTextureIndex> {
    let material = select_sub_material(context, index, sub_index)?;
    Ok(modify_material(context, material, true))
}

pub fn select_sub_material(
    context: &mut impl MaterialSelectionContext,
    index: i32,
    sub_index: i32,
) -> Result<Option<usize>, InvalidTextureIndex> {
    let index = usize::try_from(index).map_err(|_| InvalidTextureIndex(index))?;
    let material = if index == 0 && context.has_actor() && context.sub_skin(sub_index).is_some() {
        context.sub_skin(sub_index)
    } else {
        context.mesh_texture(index).flatten()
    };
    Ok(material)
}

pub fn get_extra_sub_material(
    context: &mut impl MaterialContext,
    index: i32,
    extra_material: Option<usize>,
) -> Result<Option<usize>, InvalidTextureIndex> {
    let material = select_extra_sub_material(context, index, extra_material)?;
    Ok(modify_material(context, material, true))
}

pub fn select_extra_sub_material(
    context: &mut impl MaterialSelectionContext,
    index: i32,
    extra_material: Option<usize>,
) -> Result<Option<usize>, InvalidTextureIndex> {
    let index = usize::try_from(index).map_err(|_| InvalidTextureIndex(index))?;
    let material =
        if index == 0 && extra_material.is_some() { extra_material } else { context.mesh_texture(index).flatten() };
    Ok(material)
}

fn modify_material(context: &mut impl MaterialContext, material: Option<usize>, base_only: bool) -> Option<usize> {
    let material = material?;
    if !MaterialContext::has_actor(context) || !context.modification_enabled() {
        return Some(material);
    }
    let modifier = context.modifier();
    let shader_flags = context.shader_flags(material);
    let Some(modifier) = modifier else {
        return Some(material);
    };
    if let (Some(shader), Some(color)) = (shader_flags, context.color_modifier_flags(modifier)) {
        context.set_color_modifier_flags(modifier, (color & !1) | (shader & 1));
    }
    let modified = if base_only { context.base_texture(material) } else { Some(material) };
    context.set_modifier_material(modifier, modified);
    Some(if modified.is_some() { modifier } else { material })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextureNode {
    Texture,
    FinalBlend { material: Option<usize> },
    Shader { diffuse: Option<usize> },
    Combiner { material_4c: Option<usize> },
    Other,
}

pub fn base_texture(nodes: &[TextureNode], root: Option<usize>) -> Option<usize> {
    let mut current = root?;
    let mut seen = HashSet::new();
    loop {
        if !seen.insert(current) {
            return None;
        }
        current = match nodes.get(current)? {
            TextureNode::Texture => return Some(current),
            TextureNode::FinalBlend { material } => (*material)?,
            TextureNode::Shader { diffuse } => (*diffuse)?,
            TextureNode::Combiner { material_4c } => (*material_4c)?,
            TextureNode::Other => return None,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn material_chain_matches_texture_modifier_shader_and_combiner_routes() {
        let nodes = [
            TextureNode::Texture,
            TextureNode::FinalBlend { material: Some(0) },
            TextureNode::Shader { diffuse: Some(1) },
            TextureNode::Combiner { material_4c: Some(2) },
            TextureNode::Other,
            TextureNode::Shader { diffuse: None },
        ];
        for root in 0..4 {
            assert_eq!(base_texture(&nodes, Some(root)), Some(0));
        }
        for root in [None, Some(4), Some(5), Some(6)] {
            assert_eq!(base_texture(&nodes, root), None);
        }
    }

    #[test]
    fn invalid_cycles_and_deep_chains_do_not_overflow_stack() {
        let nodes = [TextureNode::FinalBlend { material: Some(1) }, TextureNode::Shader { diffuse: Some(0) }];
        assert_eq!(base_texture(&nodes, Some(0)), None);
        let mut nodes: Vec<_> = (1..100000).map(|index| TextureNode::Shader { diffuse: Some(index) }).collect();
        nodes.push(TextureNode::Texture);
        assert_eq!(base_texture(&nodes, Some(0)), Some(99999));
    }
}
