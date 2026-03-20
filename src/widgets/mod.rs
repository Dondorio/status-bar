use mlua::FromLua;
use taffy::Layout;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Style {
    pub width: f32,
    pub height: f32,
}

impl FromLua for Style {
    fn from_lua(value: mlua::Value, _lua: &mlua::Lua) -> mlua::Result<Self> {
        if let mlua::Value::Table(t) = value {
            return Ok(Style {
                width: t.get("width")?,
                height: t.get("height")?,
            });
        }

        panic!("Widget's Style is not a table")
    }
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Widget {
    pub children: Option<Vec<Widget>>,
    // pub style: Style,
    pub layout: Option<Layout>,
    // pub draw: mlua::Function,
    // pub on_click: Option<mlua::Function>,
}

impl FromLua for Widget {
    fn from_lua(value: mlua::Value, _lua: &mlua::Lua) -> mlua::Result<Self> {
        if let mlua::Value::Table(t) = value {
            return Ok(Widget {
                children: t.get("children")?,
                // style: t.get("style")?,
                layout: None,
                // draw: t.get::<Function>("draw")?,
                // on_click: t.get::<Option<Function>>("on_click")?,
            });
        }

        panic!("Widget is not a table")
    }
}
