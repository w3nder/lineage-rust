#![cfg(target_os = "android")]

include!("main.rs");

use winit::platform::android::EventLoopBuilderExtAndroid;
use winit::platform::android::activity::AndroidApp;

#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    android_logger::init_once(android_logger::Config::default().with_max_level(log::LevelFilter::Info));
    let client = app.external_data_path().map(|path| path.join("L2Killer"));
    let event_loop = EventLoop::builder().with_android_app(app).build();
    run(Args { client, ..Args::default() }, event_loop);
}
