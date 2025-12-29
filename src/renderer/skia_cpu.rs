use std::{fs, mem, sync::LazyLock};

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
                .expect("Failed to create Skia surface");

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

    pub fn clear(&mut self, color: impl Into<Color4f>) {
        self.surface.canvas().clear(color);
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

    pub fn draw_image(&mut self, path: &std::path::Path, position: (f32, f32), scale: (f32, f32)) {
        let i = fs::read(path).expect("Failed to read file");
        let data = Data::new_copy(&i);
        let image = Image::from_encoded(data).expect("Failed to decode file");
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

        self.paint.set_color(Color::YELLOW);
        self.draw_rect(
            (offset - padding, offset - padding),
            (width + padding * 2.0, height + padding * 2.0),
        );

        self.paint.set_color(Color::BLACK);
        self.paint.set_stroke_width(outline_width);
        self.paint.set_style(PaintStyle::Stroke);
        self.draw_rect(
            (offset - padding, offset - padding),
            (width + padding * 2.0, height + padding * 2.0),
        );

        self.paint.set_stroke_width(1.0);
        self.paint.set_style(PaintStyle::Fill);

        self.draw_text((offset, offset + height - bottom), str, &FONT_MONOSPACE);
    }

    pub fn draw_test_scene(&mut self, shift: u32) {
        self.clear(0xFF707070);

        // Smiley face
        self.paint.set_color(Color::YELLOW);
        self.draw_circle((500.0, 50.0), 20.0);
        self.paint.set_color(Color::BLACK);
        self.draw_line((495.0, 45.0), (495.0, 55.0));
        self.draw_line((505.0, 45.0), (505.0, 55.0));
        self.move_to((495.0, 60.0));
        self.bezier_curve_to((498.0, 61.0), (502.0, 61.0), (505.0, 60.0));
        self.draw_path_stroke();

        self.paint.set_style(PaintStyle::Fill);
        self.paint.set_color(Color::BLUE);
        self.draw_rect((shift as f32, 50.0), (150.0, 20.0));
    }
    // ---

    pub fn translate(&mut self, d: (f32, f32)) {
        self.canvas().translate(d);
    }

    pub fn scale(&mut self, scale: (f32, f32)) {
        self.canvas().scale(scale);
    }

    // Path
    pub fn move_to(&mut self, point: (f32, f32)) {
        self.begin_path();
        self.path.move_to(point);
    }

    pub fn line_to(&mut self, point: (f32, f32)) {
        self.path.line_to(point);
    }

    pub fn quad_to(&mut self, cp1: (f32, f32), to: (f32, f32)) {
        self.path.quad_to(cp1, to);
    }

    pub fn bezier_curve_to(&mut self, cp1: (f32, f32), cp2: (f32, f32), to: (f32, f32)) {
        self.path.cubic_to(cp1, cp2, to);
    }

    pub fn begin_path(&mut self) {
        let new_path = Path::new();
        self.surface.canvas().draw_path(&self.path, &self.paint);
        let _ = mem::replace(&mut self.path, new_path);
    }

    pub fn close_path(&mut self) {
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

    pub fn set_line_width(&mut self, width: f32) {
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
