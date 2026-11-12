use killer_ui::option_paint::{OptionPaintBackend, paint_option};
use killer_ui::ui_device::TextureInput;
use std::convert::Infallible;
use std::io::Write;
use std::process::{Command, Stdio};

struct Backend { values: [u32; 8], position: [i32; 2], size: [f32; 2], textures: [Option<u32>; 2], device: [u32; 5], events: Vec<u32> }
impl OptionPaintBackend for Backend {
    type Texture = u32;
    type Error = Infallible;
    fn position(&mut self) -> [i32; 2] { self.position }
    fn size(&mut self) -> [f32; 2] { self.size }
    fn texture(&mut self, slot: u32) -> Option<u32> { self.textures[usize::from(slot == 0xec)] }
    fn set_clip(&mut self, rect: [i32; 4]) -> Result<(), Infallible> {
        self.events.push(0); self.events.extend(rect.map(|v| v as u32));
        if self.values[5] & 1 != 0 { self.position = [17, 31]; }
        Ok(())
    }
    fn begin_origin(&mut self, position: [i32; 2]) { self.device = [position[0] as u32, position[1] as u32, position[1] as u32, 0, position[0] as u32]; }
    fn draw_texture(&mut self, input: TextureInput<u32>, clip: u32) -> Result<(), Infallible> {
        self.events.push(2); self.events.extend(input.rect.map(|v| v as u32)); self.events.extend(input.uv.map(|v| v as u32)); self.events.extend([input.texture, input.alpha, clip]);
        if input.texture == 7 {
            if self.values[5] & 2 != 0 { self.size = [f32::from_bits(self.values[6]), f32::from_bits(self.values[7])]; }
            if self.values[5] & 4 != 0 { self.textures[1] = Some(9); }
            if self.values[5] & 8 != 0 { self.textures[1] = None; }
        }
        Ok(())
    }
    fn clear_origin(&mut self) { self.device[0] = 0; self.device[1] = 0; }
    fn release_clip(&mut self) -> Result<(), Infallible> { self.events.extend([4, self.device[0], self.device[1]]); Ok(()) }
}

#[test]
#[ignore]
fn option_paint_matches_original_guards_reloads_geometry_and_device_lifecycle() {
    let edges = [0, 1, 0x80000000, 0x7f800000, 0xff800000, 0x7fc12345, 0x7f812345, 0x4f000000, 0xcf000000, 256.0f32.to_bits(), 480.0f32.to_bits(), 48.5f32.to_bits()];
    let mut cases = Vec::new();
    for mask in 0..4 {
        for mutation in 0..16 {
            for (index, width) in edges.iter().enumerate() {
                cases.push([index as u32, !(index as u32), *width, edges[(index + 3) % edges.len()], mask, mutation, edges[(index + 5) % edges.len()], edges[(index + 7) % edges.len()]]);
            }
        }
    }
    assert_eq!(cases.len(), 768);
    let mut child = Command::new(std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_| "python3".into()))
        .arg(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/option_paint_reference.py"))
        .arg(std::env::var("KILLER_CLIENT_DIR").unwrap())
        .stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
    let mut input = child.stdin.take().unwrap();
    for values in &cases { writeln!(input, "{}", values.iter().map(u32::to_string).collect::<Vec<_>>().join(" ")).unwrap(); }
    drop(input);
    let output = child.wait_with_output().unwrap(); assert!(output.status.success());
    let output = String::from_utf8(output.stdout).unwrap(); assert_eq!(output.lines().count(), cases.len());
    for (values, line) in cases.into_iter().zip(output.lines()) {
        let mut backend = Backend { values, position: [values[0] as i32, values[1] as i32], size: [f32::from_bits(values[2]), f32::from_bits(values[3])], textures: [(values[4] & 1 != 0).then_some(7), (values[4] & 2 != 0).then_some(8)], device: [99; 5], events: Vec::new() };
        let mut actual = vec![u32::from(paint_option(&mut backend).unwrap())]; actual.extend(backend.device); actual.extend(backend.events);
        let expected: Vec<u32> = line.split_whitespace().map(|word| word.parse().unwrap()).collect();
        assert_eq!(actual, expected, "{values:?}");
    }
}
