use std::str::FromStr;

use derive_more::{Display, FromStr};
use mlua::{FromLua, Lua};
use smithay_client_toolkit::{
    seat::{keyboard::KeyEvent, pointer::PointerEvent},
    shell::wlr_layer::Anchor,
};

pub mod wayland;

#[allow(dead_code, unused_variables)]
pub trait Window {
    fn new(opts: Opts, lua: Lua) -> Self;
    fn run(&mut self);
    fn exit(&mut self);
    // TODO
    fn set_height(&mut self, height: u32) {}
    fn set_width(&mut self, width: u32) {}
    fn set_exclusive_zone(&mut self, exclusive_zone: u32) {}
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub enum Event {
    Resized {
        width: u32,
        height: u32,
    },

    PointerButtonPressed {
        button: PointerEvent,
        modifiers: Modifiers,
    },
    PointerButtonReleased {
        button: PointerEvent,
        modifiers: Modifiers,
    },
    PointerMoved {
        x: f64,
        y: f64,
    },
    PointerEntered {
        x: f64,
        y: f64,
    },
    PointerLeft,

    KeyboardKeyPressed {
        key: KeyEvent,
        modifiers: Modifiers,
    },
    KeyboardKeyReleased {
        key: KeyEvent,
        modifiers: Modifiers,
    },
    KeyboardEntered,
    KeyboardLeft,

    Exit,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Default)]
pub struct Modifiers {
    control: bool,
    shift: bool,
    alt: bool,
    meta: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Display, FromStr)]
#[display(rename_all = "lowercase")]
pub enum Layer {
    Background,
    Bottom,
    Overlay,
    Top,
}

#[derive(Debug, Clone)]
pub struct Opts {
    pub width: u32,
    pub height: u32,
    pub exclusive_zone: i32,
    pub layer: Layer,
    // TODO use custom enum for anchor
    pub anchor: Option<Anchor>,
    pub margin: Margin,
    pub namespace: Option<String>,
}

impl Default for Opts {
    fn default() -> Self {
        Self {
            width: 100,
            height: 100,
            exclusive_zone: -1,
            layer: Layer::Bottom,
            anchor: Some(Anchor::TOP),
            namespace: None,
            margin: Margin::default(),
        }
    }
}

impl FromLua for Opts {
    fn from_lua(value: mlua::Value, _: &Lua) -> mlua::Result<Self> {
        if let mlua::Value::Table(t) = value {
            let bits = t.get::<Option<u32>>("anchor").unwrap();
            let anchor = bits.and_then(Anchor::from_bits);

            return Ok(Opts {
                width: t.get("width").expect("a window must have a set width"),
                height: t.get("height").expect("a window must have a set height"),
                layer: match t.get::<String>("layer") {
                    Ok(s) => Layer::from_str(&s).expect("unexpected layer type"),
                    Err(_) => panic!("a window must have a set layer"),
                },
                anchor,
                namespace: t.get("namespace")?,
                margin: t.get::<Option<Margin>>("margin")?.unwrap_or_default(),
                exclusive_zone: t.get::<Option<i32>>("exclusive_zone")?.unwrap_or(-1),
            });
        }

        panic!("WindowOpts is not a table")
    }
}

// TODO lua
#[derive(Default, Debug, Clone, Copy)]
pub struct Margin {
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
    pub left: i32,
}

impl From<(i32, i32, i32, i32)> for Margin {
    fn from((top, right, bottom, left): (i32, i32, i32, i32)) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }
}

impl From<Margin> for (i32, i32, i32, i32) {
    fn from(m: Margin) -> Self {
        (m.top, m.right, m.bottom, m.left)
    }
}

// Maybe add the ability to use css like syntax
impl FromLua for Margin {
    fn from_lua(value: mlua::Value, _: &Lua) -> mlua::Result<Self> {
        if let mlua::Value::Table(t) = value {
            return Ok(Margin {
                top: t.get("top").unwrap_or_default(),
                right: t.get("right").unwrap_or_default(),
                bottom: t.get("bottom").unwrap_or_default(),
                left: t.get("left").unwrap_or_default(),
            });
        }

        // TODO
        Err(mlua::Error::FromLuaConversionError {
            from: "",
            to: "Margin".to_string(),
            message: Some(
                r#"
                    cannot convert type to Margin
                    expected { 
                      top? = number, 
                      right? = number, 
                      bottom? = number,
                      left? = number,
                    }
                "#
                .to_string(),
            ),
        })
    }
}
