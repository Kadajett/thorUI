use crate::showcase::{Point, ShowcaseModel};
use js_sys::Reflect;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

pub struct Stage {
    canvas: HtmlCanvasElement,
    context: CanvasRenderingContext2d,
}

impl Stage {
    pub fn new(canvas: HtmlCanvasElement) -> Result<Self, JsValue> {
        let context = canvas
            .get_context("2d")?
            .ok_or_else(|| JsValue::from_str("2D canvas is unavailable"))?
            .dyn_into()?;
        Ok(Self { canvas, context })
    }

    pub fn draw(&mut self, model: &ShowcaseModel, role: &str, time: f64) {
        let (width, height, density) = self.resize();
        let _ = self
            .context
            .set_transform(density, 0.0, 0.0, density, 0.0, 0.0);
        self.draw_backdrop(width, height, time);
        self.draw_grid(width, height, time);
        self.draw_marks(model, width, height, time);
        self.draw_cursor(model.cursor(), model.color().css(), width, height, time);
        if role == "companion" {
            self.draw_companion_orbit(width, height, time);
        }
    }

    fn resize(&mut self) -> (f64, f64, f64) {
        let width = f64::from(self.canvas.client_width().max(1));
        let height = f64::from(self.canvas.client_height().max(1));
        let density = super::super::helpers::window()
            .ok()
            .map_or(1.0, |window| window.device_pixel_ratio().clamp(1.0, 2.5));
        let pixel_width = (width * density).round();
        let pixel_height = (height * density).round();
        if (f64::from(self.canvas.width()) - pixel_width).abs() > 0.5
            || (f64::from(self.canvas.height()) - pixel_height).abs() > 0.5
        {
            let _ = Reflect::set(self.canvas.as_ref(), &"width".into(), &pixel_width.into());
            let _ = Reflect::set(self.canvas.as_ref(), &"height".into(), &pixel_height.into());
        }
        (width, height, density)
    }

    fn draw_backdrop(&self, width: f64, height: f64, time: f64) {
        self.context.set_fill_style_str("#050814");
        self.context.fill_rect(0.0, 0.0, width, height);
        let Ok(gradient) = self.context.create_radial_gradient(
            width * 0.54,
            height * 0.48,
            4.0,
            width * 0.54,
            height * 0.48,
            width.max(height) * 0.72,
        ) else {
            return;
        };
        let pulse = 0.14 + (time * 0.000_35).sin() * 0.025;
        let _ = gradient.add_color_stop(0.0, &format!("rgba(54,92,178,{pulse:.3})"));
        let _ = gradient.add_color_stop(0.42, "rgba(42,31,101,0.12)");
        let _ = gradient.add_color_stop(1.0, "rgba(5,8,20,0)");
        self.context.set_fill_style_canvas_gradient(&gradient);
        self.context.fill_rect(0.0, 0.0, width, height);
        self.draw_stars(width, height, time);
    }

    fn draw_stars(&self, width: f64, height: f64, time: f64) {
        for index in 0..54 {
            let seed = f64::from(index);
            let x = ((seed * 83.17).sin() * 0.5 + 0.5) * width;
            let base_y = ((seed * 41.73).cos() * 0.5 + 0.5) * height;
            let y = (base_y + time * (0.002 + seed % 3.0) * 0.03) % height;
            let alpha = 0.2 + ((time * 0.001 + seed).sin() * 0.5 + 0.5) * 0.55;
            self.context
                .set_fill_style_str(&format!("rgba(190,222,255,{alpha:.3})"));
            self.context.begin_path();
            let _ = self
                .context
                .arc(x, y, 0.7 + seed % 2.0, 0.0, std::f64::consts::TAU);
            self.context.fill();
        }
    }

    fn draw_grid(&self, width: f64, height: f64, time: f64) {
        self.context.set_stroke_style_str("rgba(105,147,255,0.075)");
        self.context.set_line_width(1.0);
        for line in 0..12 {
            let phase = (f64::from(line) / 12.0 + time * 0.000_025) % 1.0;
            let y = height * phase.powf(1.7);
            self.context.begin_path();
            self.context.move_to(0.0, y);
            self.context.line_to(width, y);
            self.context.stroke();
        }
        for line in 0..16 {
            let x = width * f64::from(line) / 15.0;
            self.context.begin_path();
            self.context.move_to(width * 0.5, height * 0.45);
            self.context.line_to(x, height);
            self.context.stroke();
        }
    }

    fn draw_marks(&self, model: &ShowcaseModel, width: f64, height: f64, time: f64) {
        let marks = model.marks();
        for pair in marks.windows(2) {
            let first = pair[0];
            let second = pair[1];
            if first.color != second.color || distance(first.point, second.point) > 0.18 {
                continue;
            }
            self.context.set_stroke_style_str(second.color.css());
            self.context.set_global_alpha(0.18 + second.strength * 0.28);
            self.context.set_line_width(1.2 + second.strength * 2.8);
            self.context.begin_path();
            self.context
                .move_to(first.point.x * width, first.point.y * height);
            self.context
                .line_to(second.point.x * width, second.point.y * height);
            self.context.stroke();
        }
        for mark in marks {
            let shimmer = ((time * 0.004 + f64::from(mark.sequence)).sin() + 1.0) * 0.5;
            self.context.set_global_alpha(0.55 + shimmer * 0.4);
            self.context.set_fill_style_str(mark.color.css());
            self.context.set_shadow_color(mark.color.css());
            self.context.set_shadow_blur(10.0 + mark.strength * 18.0);
            self.context.begin_path();
            let radius = 2.6 + mark.strength * 5.2 + shimmer * 1.8;
            let _ = self.context.arc(
                mark.point.x * width,
                mark.point.y * height,
                radius,
                0.0,
                std::f64::consts::TAU,
            );
            self.context.fill();
        }
        self.context.set_global_alpha(1.0);
        self.context.set_shadow_blur(0.0);
    }

    fn draw_cursor(&self, point: Point, color: &str, width: f64, height: f64, time: f64) {
        let radius = 14.0 + (time * 0.004).sin() * 3.0;
        self.context.set_stroke_style_str(color);
        self.context.set_shadow_color(color);
        self.context.set_shadow_blur(18.0);
        self.context.set_line_width(2.0);
        self.context.begin_path();
        let _ = self.context.arc(
            point.x * width,
            point.y * height,
            radius,
            0.0,
            std::f64::consts::TAU,
        );
        self.context.stroke();
        self.context.set_shadow_blur(0.0);
    }

    fn draw_companion_orbit(&self, width: f64, height: f64, time: f64) {
        let radius = width.min(height) * 0.23;
        let rotation = time * 0.000_18;
        self.context.set_stroke_style_str("rgba(114,244,196,0.18)");
        self.context.set_line_width(1.0);
        for ring in 0..3 {
            self.context.begin_path();
            let spread = radius + f64::from(ring) * 19.0 + rotation.sin() * 3.0;
            let _ = self.context.arc(
                width * 0.5,
                height * 0.5,
                spread,
                rotation,
                rotation + std::f64::consts::PI * 1.55,
            );
            self.context.stroke();
        }
    }
}

fn distance(first: Point, second: Point) -> f64 {
    ((first.x - second.x).powi(2) + (first.y - second.y).powi(2)).sqrt()
}
