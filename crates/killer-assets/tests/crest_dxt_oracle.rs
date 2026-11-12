use killer_assets::crest_dxt::compress;

const GOLDEN: &[u8] = include_bytes!("fixtures/dxt1_oracle.bin");

fn word(data: &[u8], at: usize) -> usize {
    u32::from_le_bytes(data[at..at + 4].try_into().unwrap()) as usize
}

#[test]
fn the_lifted_compressor_matches_the_engine_on_every_oracle_vector() {
    let count = word(GOLDEN, 0);
    let mut at = 4;
    let mut transparent_cases = 0;
    for index in 0..count {
        let (w, h, transparent, size) = (word(GOLDEN, at), word(GOLDEN, at + 4), word(GOLDEN, at + 8), word(GOLDEN, at + 12));
        at += 16;
        let pixels = &GOLDEN[at..at + w * h * 4];
        let expected = &GOLDEN[at + w * h * 4..at + w * h * 4 + size];
        at += w * h * 4 + size;
        let actual = compress(pixels, w as u32, h as u32, transparent == 1).unwrap();
        assert_eq!(actual, expected, "vetor {index}: {w}x{h} transparente={transparent}");
        transparent_cases += transparent;
    }
    assert_eq!(count, 126);
    assert_eq!(transparent_cases, 63);
}
