use poogie::PoogieApp;

fn main() {
    env_logger::init();
    let poogie = PoogieApp::builder()
        .debug_graphics(true)
        .resolution([1920, 1080])
        .title("Awesome Poogie App Winning".to_string())
        .build()
        .unwrap();

    poogie.render_loop();
}
