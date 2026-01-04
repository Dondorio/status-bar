mod renderer;
mod window;
use std::fs::read_to_string;

use mlua::{Function, Lua};

use crate::window::Window;

fn main() {
    let conf = read_to_string("./config.lua").unwrap();
    let lua = Lua::new();

    lua.load(conf).exec().unwrap();

    // TODO call SimpleLayer::new(...).run() from inside of lua
    let g = lua.globals();
    let opts = g.get::<Function>("opts").unwrap().call(()).unwrap();

    window::wayland::SimpleLayer::new(opts, lua).run();
}
