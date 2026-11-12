#[test]
#[ignore]
fn both_original_landmark_classes_render_and_release() {
    use glam::Vec3;
    use killer_assets::{client::Client,options::ClientOptions,textures::{BankSource,TextureBank}};
    use killer_render::{camera::Camera,environment::Environment,headless::Headless,renderer::EffectPlacement};
    let root=std::env::var_os("KILLER_CLIENT_DIR").expect("KILLER_CLIENT_DIR");
    let mut client=Client::open(std::path::Path::new(&root)).unwrap();
    let options=ClientOptions::load(&mut client).unwrap();
    let camera=Camera {eye:Vec3::new(110.0,0.0,170.0),target:Vec3::ZERO,fov_x_deg:75.0,near:1.0,far:1000.0};
    for (index,class) in [killer_web::landmark::MARK_CLASS,killer_web::landmark::ARRIVAL_CLASS].into_iter().enumerate() {
        let mut headless=Headless::new(256,256).unwrap();
        let empty=headless.render(&camera,&Environment::default()).unwrap();
        let definition=client.emitter_class(class).expect(class);
        let life=client.class_float_defaults(class,&["LifeSpan"]).unwrap();
        assert!(life["LifeSpan"]>0.0);
        let asset=killer_assets::scene::emitters::effect_asset(&mut client,&definition,&options);
        let mut bank=TextureBank::new();
        let id=headless.renderer_mut().add_effect(&asset,&mut BankSource {bank:&mut bank,catalog:client.catalog()});
        headless.renderer_mut().place_effect(id,EffectPlacement {location:Vec3::ZERO,rotation:[0;3],zone:0,visible:true});
        let mut capture=Vec::new();
        for frame in 0..30 {
            headless.renderer_mut().set_time(frame as f32/60.0);
            let pixels=headless.render(&camera,&Environment::default()).unwrap();
            if pixels!=empty {capture=pixels;}
        }
        assert!(!capture.is_empty(),"efeito invisível: {class}");
        let mut ppm=b"P6\n256 256\n255\n".to_vec();
        for pixel in capture.as_chunks::<4>().0 {ppm.extend_from_slice(&pixel[..3]);}
        std::fs::write(format!("/tmp/killer-web-landmark-{index}.ppm"),ppm).unwrap();
        headless.renderer_mut().remove_effect(id);
        assert_eq!(headless.render(&camera,&Environment::default()).unwrap(),empty);
    }
}
