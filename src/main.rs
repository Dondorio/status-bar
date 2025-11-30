mod window;
use crate::window::{Opts, Window};

fn main() {
    window::wayland::SimpleLayer::new(Opts::default()).run();
}
