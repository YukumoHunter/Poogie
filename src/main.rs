use poogie::poogie_app::PoogieApp;

fn main() {
    env_logger::init();

    let poogie = PoogieApp::new(1920, 1080);

    poogie.render_loop();
}
