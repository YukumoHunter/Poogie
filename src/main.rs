use poogie::PoogieApp;

fn main() {
    env_logger::init();
    let poogie = PoogieApp::new(1280, 720).unwrap();

    poogie.render_loop();
}
