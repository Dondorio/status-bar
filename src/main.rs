mod renderer;
mod window;
use std::fs::read_to_string;

use mlua::Lua;

use crate::window::{Opts, Window};

fn main() {
    let conf = read_to_string("./config.lua").unwrap();
    let lua = Lua::new();

    lua.load(conf).exec().unwrap();

    window::wayland::SimpleLayer::new(Opts::default(), lua).run();
}
