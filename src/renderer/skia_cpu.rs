use std::{fs, mem, sync::LazyLock};

use mlua::UserData;
use skia_safe::{
    Borrows, Color, Color4f, Data, EncodedImageFormat, Font, FontMgr, Image, ImageInfo, Paint,
    PaintStyle, Path, Rect, Surface, surfaces,
};

static FONT_MONOSPACE: LazyLock<Font> = LazyLock::new(|| {
    Font::from_typeface(
        FontMgr::new()
            .match_family_style("monospace", skia_safe::FontStyle::normal())
            .unwrap(),
        18.0,
    )
});

pub struct Canvas<'a> {
    surface: Borrows<'a, Surface>,
    path: Path,
    paint: Paint,
}

#[allow(unused)]
impl Canvas<'_> {
    pub fn new(width: i32, height: i32, canvas_data: &mut [u8]) -> Canvas<'_> {
        let image_info = ImageInfo::new(
            (width, height),
            skia_safe::ColorType::BGRA8888,
            skia_safe::AlphaType::Premul,
            None,
        );

        let stride = (width * 4) as usize;

        let mut surface: Borrows<Surface> =
            surfaces::wrap_pixels(&image_info, canvas_data, stride, None)
                .expect("failed to create Skia surface");

        let canvas = surface.canvas();
        let path = skia_safe::Path::default();
        let mut paint = Paint::default();

        paint.set_color(Color::BLACK);
        paint.set_anti_alias(true);
        paint.set_stroke_width(1.0);

        canvas.clear(Color::WHITE);

        Canvas {
            surface,
            path,
            paint,
        }
    }

    pub fn clear(&mut self, colour: impl Into<Color4f>) {
        self.surface.canvas().clear(colour);
    }

    // Draw
    pub fn draw_line(&mut self, from: (f32, f32), to: (f32, f32)) {
        self.surface.canvas().draw_line(from, to, &self.paint);
    }

    pub fn draw_rect(&mut self, position: (f32, f32), scale: (f32, f32)) {
        self.surface
            .canvas()
            .draw_rect(Rect::from_point_and_size(position, scale), &self.paint);
    }

    pub fn draw_circle(&mut self, center: (f32, f32), radius: f32) {
        self.surface
            .canvas()
            .draw_circle(center, radius, &self.paint);
    }

    pub fn draw_text(&mut self, position: (f32, f32), str: &str, font: &Font) {
        self.surface
            .canvas()
            .draw_str(str, position, font, &self.paint);
    }

    pub fn draw_image(&mut self, position: (f32, f32), scale: (f32, f32), path: &str) {
        let i = fs::read(path).expect("failed to read file");
        let data = Data::new_copy(&i);
        let image = Image::from_encoded(data).expect("failed to decode file");
        let dst = Rect::from_point_and_size(position, scale);
        self.surface
            .canvas()
            .draw_image_rect(image, None, dst, &self.paint);
    }

    // TEMPORARY ---
    pub fn draw_fps(&mut self, fps: u32) {
        let str = &format!("{fps}fps");
        let padding = 4.0;
        let outline_width = 2.0;
        let offset = padding + outline_width / 2.0;

        let (
            width,
            Rect {
                left,
                top,
                right,
                bottom,
            },
        ) = FONT_MONOSPACE.measure_str(str, Some(&self.paint));
        let height = bottom - top;
        let width = right - left;

        self.set_paint_colour(Color::YELLOW);
        self.draw_rect(
            (offset - padding, offset - padding),
            (width + padding * 2.0, height + padding * 2.0),
        );

        self.set_paint_colour(Color::BLACK);
        self.set_stroke_width(outline_width);
        self.paint.set_style(PaintStyle::Stroke);
        self.draw_rect(
            (offset - padding, offset - padding),
            (width + padding * 2.0, height + padding * 2.0),
        );

        self.paint.set_stroke_width(1.0);
        self.paint.set_style(PaintStyle::Fill);

        self.draw_text((offset, offset + height - bottom), str, &FONT_MONOSPACE);
    }
    // ---

    pub fn translate(&mut self, d: (f32, f32)) {
        self.canvas().translate(d);
    }

    pub fn scale(&mut self, scale: (f32, f32)) {
        self.canvas().scale(scale);
    }

    // Path
    pub fn path_begin_from(&mut self, point: (f32, f32)) {
        self.path_begin();
        self.path.move_to(point);
    }

    pub fn path_line_to(&mut self, point: (f32, f32)) {
        self.path.line_to(point);
    }

    pub fn path_quad_to(&mut self, cp1: (f32, f32), to: (f32, f32)) {
        self.path.quad_to(cp1, to);
    }

    pub fn path_bezier_curve_to(&mut self, cp1: (f32, f32), cp2: (f32, f32), to: (f32, f32)) {
        self.path.cubic_to(cp1, cp2, to);
    }

    pub fn path_begin(&mut self) {
        let new_path = Path::new();
        self.surface.canvas().draw_path(&self.path, &self.paint);
        let _ = mem::replace(&mut self.path, new_path);
    }

    pub fn path_close(&mut self) {
        self.path.close();
    }

    pub fn draw_path_stroke(&mut self) {
        self.paint.set_style(PaintStyle::Stroke);
        self.surface.canvas().draw_path(&self.path, &self.paint);
    }

    pub fn draw_path_fill(&mut self) {
        self.paint.set_style(PaintStyle::Fill);
        self.surface.canvas().draw_path(&self.path, &self.paint);
    }

    pub fn set_paint_colour(&mut self, colour: impl Into<Color>) {
        self.paint.set_color(colour);
    }

    pub fn set_paint_style(&mut self, style: PaintStyle) {
        self.paint.set_style(style);
    }

    pub fn set_stroke_width(&mut self, width: f32) {
        self.paint.set_stroke_width(width);
    }

    // Other
    pub fn data(&mut self) -> Data {
        let image = self.surface.image_snapshot();
        let mut context = self.surface.direct_context();
        image
            .encode(context.as_mut(), EncodedImageFormat::PNG, None)
            .unwrap()
    }

    fn canvas(&mut self) -> &skia_safe::Canvas {
        self.surface.canvas()
    }
}

impl<'a> UserData for Canvas<'a> {
    fn add_methods<M: mlua::UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method_mut("clear", |_, this, col: u32| {
            this.clear(col);
            Ok(())
        });

        methods.add_method_mut("draw_rect", |_, this, (px, py, sx, sy)| {
            this.draw_rect((px, py), (sx, sy));
            Ok(())
        });

        methods.add_method_mut("draw_circle", |_, this, (px, py, radius)| {
            this.draw_circle((px, py), radius);
            Ok(())
        });

        methods.add_method_mut("draw_line", |_, this, (fx, fy, tx, ty)| {
            this.draw_line((fx, fy), (tx, ty));
            Ok(())
        });

        methods.add_method_mut("draw_text", |_, this, (px, py, str): (f32, f32, String)| {
            // TODO implement UserData for Font
            this.draw_text((px, py), &str, &FONT_MONOSPACE);
            Ok(())
        });

        methods.add_method_mut(
            "draw_image",
            |_, this, (px, py, sx, sy, path): (f32, f32, f32, f32, String)| {
                this.draw_image((px, py), (sx, sy), &path);
                Ok(())
            },
        );

        methods.add_method_mut("draw_path_stroke", |_, this, ()| {
            this.draw_path_stroke();
            Ok(())
        });

        methods.add_method_mut("draw_path_fill", |_, this, ()| {
            this.draw_path_fill();
            Ok(())
        });

        methods.add_method_mut("set_paint_colour", |_, this, colour: u32| {
            this.set_paint_colour(colour);
            Ok(())
        });

        methods.add_method_mut("set_paint_style", |_, this, style: String| {
            let s = match style.as_str() {
                "fill" => PaintStyle::Fill,
                "stroke" => PaintStyle::Stroke,
                _ => panic!("Unknown stroke style"),
            };
            this.set_paint_style(s);
            Ok(())
        });

        methods.add_method_mut("set_stroke_width", |_, this, width| {
            this.set_stroke_width(width);
            Ok(())
        });

        methods.add_method_mut(
            "path_bezier_curve_to",
            |_, this, (cp1x, cp1y, cp2x, cp2y, px, py)| {
                this.path_bezier_curve_to((cp1x, cp1y), (cp2x, cp2y), (px, py));
                Ok(())
            },
        );

        methods.add_method_mut("path_begin_from", |_, this, (px, py)| {
            this.path_begin_from((px, py));
            Ok(())
        });
    }
}
