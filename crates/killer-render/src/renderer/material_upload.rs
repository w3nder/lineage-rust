use crate::pipeline::{MATERIAL_STRIDE, MaterialGpu};

const BATCH_BYTES: usize = 64 * 1024;

#[derive(Default)]
pub(super) struct MaterialUpload {
    bytes: Vec<u8>,
    offset: u64,
    next: u32,
}

impl MaterialUpload {
    pub fn push(&mut self, slot: u32, material: &MaterialGpu, emit: &mut impl FnMut(u64, &[u8])) {
        let bytes = bytemuck::bytes_of(material);
        if !self.bytes.is_empty() && (slot != self.next || self.bytes.len() + MATERIAL_STRIDE as usize > BATCH_BYTES) {
            self.flush(emit);
        }
        if self.bytes.is_empty() {
            self.offset = u64::from(slot) * MATERIAL_STRIDE;
        } else {
            self.bytes.resize(self.bytes.len().next_multiple_of(MATERIAL_STRIDE as usize), 0);
        }
        self.bytes.extend_from_slice(bytes);
        self.next = slot + 1;
    }

    pub fn flush(&mut self, emit: &mut impl FnMut(u64, &[u8])) {
        if !self.bytes.is_empty() {
            emit(self.offset, &self.bytes);
            self.bytes.clear();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytemuck::Zeroable;

    #[test]
    fn sparse_recycled_and_repeated_slots_match_individual_uploads() {
        let mut reference = vec![0xa5; 200 * MATERIAL_STRIDE as usize];
        let mut actual = reference.clone();
        let mut batch = MaterialUpload::default();
        let slots: Vec<u32> = (0..130).chain([150, 160, 159, 160, 198, 0, 1]).collect();
        let mut calls = 0;
        let mut emit = |offset: u64, bytes: &[u8]| {
            assert!(bytes.len() <= BATCH_BYTES);
            actual[offset as usize..offset as usize + bytes.len()].copy_from_slice(bytes);
            calls += 1;
        };
        for (index, slot) in slots.iter().copied().enumerate() {
            let material = MaterialGpu { zone: index as u32, alpha_ref: index as f32, ..MaterialGpu::zeroed() };
            let bytes = bytemuck::bytes_of(&material);
            let offset = slot as usize * MATERIAL_STRIDE as usize;
            reference[offset..offset + bytes.len()].copy_from_slice(bytes);
            batch.push(slot, &material, &mut emit);
        }
        batch.flush(&mut emit);
        for slot in 0..200 {
            let offset = slot * MATERIAL_STRIDE as usize;
            assert_eq!(&actual[offset..offset + size_of::<MaterialGpu>()], &reference[offset..offset + size_of::<MaterialGpu>()]);
        }
        assert!(calls < slots.len() / 10);
        let capacity = batch.bytes.capacity();
        batch.flush(&mut |_, _| panic!("lote vazio"));
        assert_eq!(batch.bytes.capacity(), capacity);
    }

    #[test]
    #[ignore]
    fn gpu_upload_cost_and_readback_match_individual_writes() {
        let adapter = pollster::block_on(wgpu::Instance::default().request_adapter(&Default::default())).unwrap();
        let (device, queue) = pollster::block_on(adapter.request_device(&Default::default())).unwrap();
        let size = 4096 * MATERIAL_STRIDE;
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: None, size, usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC, mapped_at_creation: false,
        });
        let readback = device.create_buffer(&wgpu::BufferDescriptor {
            label: None, size, usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ, mapped_at_creation: false,
        });
        for sparse in [false, true] {
            let updates: Vec<_> = (0..4096u32).filter(|slot| !sparse || slot % 8 == 0).map(|slot| {
                (slot, MaterialGpu { zone: slot, alpha_ref: slot as f32, ..MaterialGpu::zeroed() })
            }).collect();
            let mut batch = MaterialUpload::default();
            let mut samples = [Vec::new(), Vec::new()];
            let mut counts = [0; 2];
            let mut outputs = Vec::new();
            for sample in 0..8 {
                for mode in [sample % 2, 1 - sample % 2] {
                    let start = std::time::Instant::now();
                    let mut calls = 0;
                    for _ in 0..20 {
                        let mut emit = |offset, bytes: &[u8]| {
                            queue.write_buffer(&buffer, offset, bytes);
                            calls += 1;
                        };
                        for (slot, material) in &updates {
                            if mode == 0 {
                                emit(u64::from(*slot) * MATERIAL_STRIDE, bytemuck::bytes_of(material));
                            } else {
                                batch.push(*slot, material, &mut emit);
                            }
                        }
                        batch.flush(&mut emit);
                        queue.submit([]);
                        device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
                    }
                    if sample > 0 {
                        samples[mode].push(start.elapsed().as_secs_f64() * 1000.0 / 20.0);
                    }
                    counts[mode] = calls / 20;
                    if sample == 7 {
                        let mut encoder = device.create_command_encoder(&Default::default());
                        encoder.copy_buffer_to_buffer(&buffer, 0, &readback, 0, size);
                        queue.submit([encoder.finish()]);
                        let slice = readback.slice(..);
                        slice.map_async(wgpu::MapMode::Read, |result| result.unwrap());
                        device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
                        let mapped = slice.get_mapped_range().unwrap();
                        let output: Vec<u8> = updates.iter().flat_map(|(slot, _)| {
                            let offset = *slot as usize * MATERIAL_STRIDE as usize;
                            mapped[offset..offset + size_of::<MaterialGpu>()].iter().copied()
                        }).collect();
                        let expected: Vec<u8> = updates.iter().flat_map(|(_, material)| bytemuck::bytes_of(material).iter().copied()).collect();
                        assert_eq!(output, expected);
                        outputs.push(output);
                        drop(mapped);
                        readback.unmap();
                    }
                }
            }
            assert_eq!(outputs[0], outputs[1]);
            for sample in &mut samples { sample.sort_by(f64::total_cmp); }
            eprintln!("sparse={sparse}: {} updates, calls {}→{}, median {:.3}→{:.3} ms/upload frame; identical GPU uniforms", updates.len(), counts[0], counts[1], samples[0][3], samples[1][3]);
        }
    }

    #[test]
    #[ignore]
    fn every_heine_material_preserves_uniform_bytes_and_slot_gaps() {
        use crate::pipeline::DrawContext;
        use killer_assets::{client::Client, options::ClientOptions, scene};
        let root = std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR");
        let mut client = Client::open(std::path::Path::new(&root)).unwrap();
        let level = client.load_level("23_24").unwrap();
        let options = ClientOptions::load(&mut client).unwrap();
        let scene = scene::build_scene_with(&mut client, &level.level, &options, scene::Neighbours::default());
        let mut expected = std::collections::BTreeMap::new();
        let mut actual = std::collections::BTreeMap::new();
        let mut batch = MaterialUpload::default();
        let mut calls = 0;
        let mut emit = |offset: u64, bytes: &[u8]| {
            calls += 1;
            for (index, uniform) in bytes.chunks(MATERIAL_STRIDE as usize).enumerate() {
                actual.insert(offset / MATERIAL_STRIDE + index as u64, uniform[..size_of::<MaterialGpu>()].to_vec());
            }
        };
        let mut slot = 0;
        for draw in &scene.draws {
            let context = DrawContext {
                ambient: draw.actor.map(|actor| actor.ambient).unwrap_or_default(), kind: draw.kind,
                sun_affect: draw.sun_affect, zone: u32::from(draw.zone), has_mask: draw.mask.is_some(),
                lightmap_slices: 0, modulate_2x: draw.modulate_2x, fade: draw.fade, tint: draw.tint,
            };
            for pass in &draw.material.passes {
                if pass.animated() || context.tint.is_some() || context.ambient.glow == 255 || draw.material.pass_stages(pass).iter().any(|stage| stage.animated()) {
                    let uniform = MaterialGpu::new(&draw.material, pass, &context, 12.5);
                    expected.insert(u64::from(slot), bytemuck::bytes_of(&uniform).to_vec());
                    batch.push(slot, &uniform, &mut emit);
                }
                slot += 1;
            }
        }
        batch.flush(&mut emit);
        assert!(!expected.is_empty());
        assert_eq!(actual, expected);
        eprintln!("Heine: {} draws, {slot} passes, {} animated uniforms, {}→{calls} upload calls; all bytes and gaps preserved", scene.draws.len(), expected.len(), expected.len());
    }
}
