use killer_assets::client::Client;
use std::path::PathBuf;

fn client() -> Client {
    Client::open(&PathBuf::from(std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR obrigatório"))).unwrap()
}

#[test]
#[ignore]
fn wettexture_postload_preserves_palette_resizes_source_and_reads_all_drops() {
    let mut client = client();
    let catalog = client.catalog();
    let wet = "speaking2F_t.Effect.2SP_PRO_000";
    let source = "speaking2F_t.Effect.2f_water_proshader3";
    let actual = catalog.try_texture_raw(wet).unwrap();
    let expected = catalog.try_texture_raw(source).unwrap();
    assert_eq!(actual.format, expected.format);
    assert_eq!(actual.palette, expected.palette);
    assert_eq!(actual.mips.len(), 1);
    for (a, b) in actual.mips.iter().zip(&expected.mips) {
        assert_eq!((a.width, a.height, &a.data), (b.width, b.height, &b.data));
    }
    let material = catalog.material(wet).unwrap();
    assert_eq!(material.root_class, "wettexture");
    let l2_catalog::StageSource::Texture(texture) = &material.stages[0].source else { panic!("bitmap esperado") };
    assert_eq!(texture.clamp, [true; 2]);
    assert_eq!(texture.size, [128.0; 2]);
    let parent = catalog.material("speaking2F_t.Effect.2f_water_pro3").unwrap();
    for path in parent.texture_paths() { catalog.try_texture_raw(path).unwrap(); }
    for name in ["2f_water_proshader", "eeee"] {
        let wet = format!("speaking2F_t.Effect.{name}");
        let actual = catalog.try_texture_raw(&wet).unwrap();
        let expected = catalog.try_texture_raw("speaking2F_t.Effect.2f_water_pro2").unwrap();
        assert_eq!(actual.palette, expected.palette);
        assert_eq!(actual.mips.len(), 1);
        let wet_data=catalog.wet_texture(&wet).unwrap();
        assert_eq!(wet_data.simulation.drops.len(),if name=="eeee" { 4 } else { 2 });
        assert_eq!(wet_data.max_frame_rate,0.0);
        assert_eq!(wet_data.prime_count,0);
        let scale=if name=="eeee" { 2 } else { 1 };
        let a=&actual.mips[0];
        let b=&expected.mips[0];
        assert_eq!((a.width,a.height),(b.width*scale,b.height*scale));
        for y in 0..a.height {
            for x in 0..a.width {
                assert_eq!(a.data[(y*a.width+x) as usize],b.data[((y/scale)*b.width+x/scale) as usize]);
            }
        }
        assert!(catalog.material(&wet).is_some());
    }
}

#[test]
#[ignore]
fn all_original_wettextures_match_fire_dll_for_120_ticks() {
    use std::io::Write;
    use std::process::{Command, Stdio};
    use serde_json::{Value,json};
    let mut client=client();
    let python=std::env::var("KILLER_ORACLE_PYTHON").unwrap_or_else(|_|"python3".into());
    let script=std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../scripts/wettexture_reference.py");
    for name in ["2f_water_proshader","2SP_PRO_000","eeee"] {
        let wet=client.catalog().wet_texture(&format!("speaking2F_t.Effect.{name}")).unwrap();
        let mut state=wet.simulation.clone();
        let amplitude=if name=="2f_water_proshader" {112} else {128};
        let input=json!({"width":state.width,"height":state.height,"amplitude":amplitude,"drops":state.drops,"source":wet.source,"steps":[1,2,3,30,60,120]});
        let mut child=Command::new(&python).arg(&script).arg(std::env::var("KILLER_CLIENT_DIR").unwrap()).stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
        writeln!(child.stdin.take().unwrap(),"{input}").unwrap();
        let output=child.wait_with_output().unwrap();
        assert!(output.status.success());
        let original:Value=serde_json::from_slice(&output.stdout).unwrap();
        let mut random=ue2_assets::wet::FireRandom::from_seed(1);
        let mut prime=0;
        for step in 1..=120 {
            wet.evaluate(&mut state,&mut prime,f64::from(step)/60.0,&mut random);
            if let Some(frame)=original["frames"].as_array().unwrap().iter().find(|f|f["step"]==step) {
                assert_eq!(json!(state.fields),frame["fields"],"fields {name} step {step}");
                assert_eq!(json!(state.pixels),frame["pixels"],"pixels {name} step {step}");
            }
        }
        eprintln!("{name}: 120 ticks contra instruções nativas");
    }
}
