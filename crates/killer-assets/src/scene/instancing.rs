use super::{MeshDraw, Vertex};
use glam::{Mat4, Vec3};

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct DecorationInstance {
    pub world: [[f32; 4]; 4],
    pub mask_uv: [f32; 2],
    pub color: [f32; 4],
}

impl DecorationInstance {
    pub fn vertex(&self, vertex: Vertex) -> Vertex {
        Vertex {
            position: Mat4::from_cols_array_2d(&self.world).transform_point3(Vec3::from(vertex.position)).to_array(),
            mask_uv: self.mask_uv,
            color: self.color,
            ..vertex
        }
    }
}

impl MeshDraw {
    pub fn world_vertex_count(&self) -> usize {
        self.vertices.len() * self.instances.len().max(1)
    }

    pub fn world_index_count(&self) -> usize {
        self.indices.len() * self.instances.len().max(1)
    }

    pub fn world_vertices(&self) -> impl Iterator<Item = Vertex> + '_ {
        (0..self.instances.len().max(1)).flat_map(move |index| {
            let instance = self.instances.get(index);
            self.vertices.iter().map(move |&vertex| instance.map_or(vertex, |instance| instance.vertex(vertex)))
        })
    }

    pub fn world_indices(&self) -> impl Iterator<Item = u32> + '_ {
        (0..self.instances.len().max(1)).flat_map(move |index| {
            let base = (index * self.vertices.len()) as u32;
            self.indices.iter().map(move |index| base + index)
        })
    }

    pub fn world_triangles(&self) -> impl Iterator<Item = [Vertex; 3]> + '_ {
        (0..self.instances.len().max(1)).flat_map(move |index| {
            let instance = self.instances.get(index);
            self.indices.as_chunks::<3>().0.iter().map(move |triangle| {
                triangle.map(|index| {
                    let vertex = self.vertices[index as usize];
                    instance.map_or(vertex, |instance| instance.vertex(vertex))
                })
            })
        })
    }
}
