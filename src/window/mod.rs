use smithay_client_toolkit::{
    seat::{keyboard::KeyEvent, pointer::PointerEvent},
    shell::wlr_layer::{Anchor, Layer},
};

pub mod wayland;

#[allow(dead_code, unused_variables)]
pub trait Window {
    fn new(opts: Opts) -> Self;
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

#[derive(Debug, Clone)]
pub struct Opts {
    pub width: u32,
    pub height: u32,
    pub exclusive_zone: i32,
    // TODO use custom enum for layer, anchor
    pub layer: Layer,
    pub anchor: Option<Anchor>,
    pub margin: Margin,
    pub namespace: Option<String>,
}

impl Default for Opts {
    fn default() -> Self {
        Self {
            width: 100,
            height: 256,
            exclusive_zone: -1,
            layer: Layer::Bottom,
            anchor: Some(Anchor::TOP),
            namespace: None,
            margin: Margin::default(),
        }
    }
}

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
