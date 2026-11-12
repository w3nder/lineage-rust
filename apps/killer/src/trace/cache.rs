use super::{Node, TraceMesh};
use glam::Vec3;
use std::io::{self, Read, Write};

const MAGIC: &[u8; 8] = b"KLRTRC01";

fn invalid() -> io::Error { io::Error::new(io::ErrorKind::InvalidData, "cache de colisão inválido") }
fn u32_read(input: &mut impl Read) -> io::Result<u32> {
    let mut bytes = [0; 4];
    input.read_exact(&mut bytes)?;
    Ok(u32::from_le_bytes(bytes))
}
fn vector(input: &mut impl Read) -> io::Result<[f32; 3]> {
    let value = [f32::from_bits(u32_read(input)?), f32::from_bits(u32_read(input)?), f32::from_bits(u32_read(input)?)];
    if value.iter().any(|v| !v.is_finite()) { return Err(invalid()); }
    Ok(value)
}

impl TraceMesh {
    pub fn write_cache(&self, mut out: impl Write) -> io::Result<()> {
        out.write_all(MAGIC)?;
        for count in [self.positions.len(), self.indices.len(), self.nodes.len()] {
            out.write_all(&u32::try_from(count).map_err(|_| invalid())?.to_le_bytes())?;
        }
        for position in &self.positions { for value in position { out.write_all(&value.to_le_bytes())?; } }
        for index in &self.indices { out.write_all(&index.to_le_bytes())?; }
        for terrain in &self.terrain { out.write_all(&[u8::from(*terrain)])?; }
        for node in &self.nodes {
            for value in node.min.to_array().into_iter().chain(node.max.to_array()) { out.write_all(&value.to_le_bytes())?; }
            for value in [node.first, node.count, node.right] { out.write_all(&value.to_le_bytes())?; }
        }
        Ok(())
    }

    pub fn read_cache(mut input: impl Read, size: u64) -> io::Result<Self> {
        let mut magic = [0; 8];
        input.read_exact(&mut magic)?;
        if &magic != MAGIC { return Err(invalid()); }
        let positions = u32_read(&mut input)? as usize;
        let indices = u32_read(&mut input)? as usize;
        let nodes = u32_read(&mut input)? as usize;
        let expected = 20 + positions as u64 * 12 + indices as u64 * 4 + indices as u64 / 3 + nodes as u64 * 36;
        if expected != size || !indices.is_multiple_of(3) || nodes != super::node_count(indices / 3) { return Err(invalid()); }
        let mut mesh = Self { positions: Vec::new(), indices: Vec::new(), terrain: Vec::new(), nodes: Vec::new() };
        mesh.positions.try_reserve_exact(positions).map_err(io::Error::other)?;
        mesh.indices.try_reserve_exact(indices).map_err(io::Error::other)?;
        mesh.terrain.try_reserve_exact(indices / 3).map_err(io::Error::other)?;
        mesh.nodes.try_reserve_exact(nodes).map_err(io::Error::other)?;
        for _ in 0..positions { mesh.positions.push(vector(&mut input)?); }
        for _ in 0..indices {
            let index = u32_read(&mut input)?;
            if index as usize >= positions { return Err(invalid()); }
            mesh.indices.push(index);
        }
        for _ in 0..indices / 3 {
            let mut flag = [0];
            input.read_exact(&mut flag)?;
            if flag[0] > 1 { return Err(invalid()); }
            mesh.terrain.push(flag[0] != 0);
        }
        for index in 0..nodes {
            let min = Vec3::from(vector(&mut input)?);
            let max = Vec3::from(vector(&mut input)?);
            let first = u32_read(&mut input)?;
            let count = u32_read(&mut input)?;
            let right = u32_read(&mut input)?;
            if min.cmpgt(max).any() || (count == 0 && (right as usize <= index + 1 || right as usize >= nodes))
                || (count > 0 && (count as usize > super::LEAF_TRIANGLES || first as u64 + count as u64 > indices as u64 / 3)) { return Err(invalid()); }
            mesh.nodes.push(Node { min, max, first, count, right });
        }
        Ok(mesh)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn heine_trace_cache_restores_entire_original_tree() {
        let root = std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR");
        let mut client = killer_assets::client::Client::open(std::path::Path::new(&root)).unwrap();
        let level = client.load_level("23_24").unwrap();
        let options = killer_assets::options::ClientOptions { render_deco: true, ..Default::default() };
        let scene = killer_assets::scene::build_scene_with(&mut client,&level.level,&options,killer_assets::scene::Neighbours::default());
        drop(level);
        drop(client);
        let original = TraceMesh::from_owned_scene(scene);
        let path = std::env::temp_dir().join(format!("killer-trace-cache-{}.bin",std::process::id()));
        {
            let mut output = std::io::BufWriter::with_capacity(1 << 20,std::fs::File::create(&path).unwrap());
            original.write_cache(&mut output).unwrap();
            output.flush().unwrap();
        }
        drop(original);
        let mut client = killer_assets::client::Client::open(std::path::Path::new(&root)).unwrap();
        let level = client.load_level("23_24").unwrap();
        let options = killer_assets::options::ClientOptions { render_deco: true, ..Default::default() };
        let scene = killer_assets::scene::build_scene_with(&mut client, &level.level, &options, killer_assets::scene::Neighbours::default());
        drop(level);
        drop(client);
        let mut builder = super::super::TraceBuilder::new(&scene);
        killer_assets::cache::encode_consuming(std::io::sink(), &killer_assets::cache::SourceStamp { size: 0, hash: 0 }, scene, |draw| builder.push(draw)).unwrap();
        let original = builder.build();
        let size = std::fs::metadata(&path).unwrap().len();
        let loaded = TraceMesh::read_cache(std::io::BufReader::with_capacity(1 << 20,std::fs::File::open(&path).unwrap()),size).unwrap();
        std::fs::remove_file(path).unwrap();
        assert!(original.positions == loaded.positions,"posições diferentes");
        assert!(original.indices == loaded.indices,"índices diferentes");
        assert!(original.terrain == loaded.terrain,"flags diferentes");
        assert!(original.nodes == loaded.nodes,"árvore diferente");
        assert_eq!(original.triangles(),8_487_662);
        assert_eq!(original.nodes.len(),2_295_259);
        eprintln!("Heine inteiro: {} triângulos, {} nós, {size} bytes; cache e serialização consumidora idênticos",original.triangles(),original.nodes.len());
    }

    #[test]
    fn cache_retains_tree_and_rejects_truncation_and_invalid_indices() {
        let mut original = TraceMesh { positions: vec![[0.,0.,0.], [1.,0.,0.], [0.,1.,0.]], indices: vec![0,1,2], terrain: vec![true], nodes: vec![] };
        original.build();
        let mut bytes = Vec::new();
        original.write_cache(&mut bytes).unwrap();
        let loaded = TraceMesh::read_cache(bytes.as_slice(), bytes.len() as u64).unwrap();
        assert_eq!(original.positions, loaded.positions);
        assert_eq!(original.indices, loaded.indices);
        assert_eq!(original.terrain, loaded.terrain);
        assert_eq!(original.nodes, loaded.nodes);
        assert_eq!(original.nearest(Vec3::new(0.2,0.2,1.), Vec3::new(0.2,0.2,-1.)), loaded.nearest(Vec3::new(0.2,0.2,1.), Vec3::new(0.2,0.2,-1.)));
        assert!(TraceMesh::read_cache(&bytes[..bytes.len()-1], bytes.len() as u64 - 1).is_err());
        bytes[56..60].copy_from_slice(&u32::MAX.to_le_bytes());
        assert!(TraceMesh::read_cache(bytes.as_slice(), bytes.len() as u64).is_err());
    }
}
