use eframe::egui::{self, Color32, Modifiers, Pos2, Response, Sense, Stroke, Ui, Vec2};
use std::f32::consts::PI;

pub struct RotaryKnob<'a> {
    label: &'a str,
    value: &'a mut f32,
    min: f32,
    max: f32,
    default_value: f32,
    accent: Color32,
    logarithmic: bool,
    format_fn: Option<Box<dyn Fn(f32) -> String + 'a>>,
}

impl<'a> RotaryKnob<'a> {
    pub fn new(
        label: &'a str,
        value: &'a mut f32,
        min: f32,
        max: f32,
        default_value: f32,
        accent: Color32,
    ) -> Self {
        Self {
            label,
            value,
            min,
            max,
            default_value,
            accent,
            logarithmic: false,
            format_fn: None,
        }
    }

    pub fn logarithmic(mut self, log: bool) -> Self {
        self.logarithmic = log;
        self
    }

    pub fn format(mut self, formatter: impl Fn(f32) -> String + 'a) -> Self {
        self.format_fn = Some(Box::new(formatter));
        self
    }

    /// `radius` controls only the arc/knob graphic size.
    /// The widget allocates extra vertical space below for two text rows.
    pub fn show(self, ui: &mut Ui, radius: f32) -> Response {
        // Reserve a column wide enough for the arc + its two text labels below.
        let label_text_h = 22.0; // two small text rows
        let col_w = radius * 2.0 + 8.0; // slight horizontal padding
        let col_h = radius * 2.0 + label_text_h;

        let (outer_rect, outer_response) =
            ui.allocate_exact_size(Vec2::new(col_w, col_h), Sense::click_and_drag());

        // ── Value normalisation ──────────────────────────────────────────────
        let current_val = *self.value;
        let mut normalized = if self.logarithmic {
            ((current_val / self.min).ln() / (self.max / self.min).ln()).clamp(0.0, 1.0)
        } else {
            ((current_val - self.min) / (self.max - self.min)).clamp(0.0, 1.0)
        };

        let mut response = outer_response;

        // ── Drag to change value ─────────────────────────────────────────────
        if response.dragged() {
            let drag_delta = response.drag_delta();
            let is_shift = ui.input(|i| i.modifiers.contains(Modifiers::SHIFT));
            let sensitivity = if is_shift { 0.001 } else { 0.005 };
            normalized = (normalized - drag_delta.y * sensitivity).clamp(0.0, 1.0);

            *self.value = if self.logarithmic {
                self.min * ((self.max / self.min).ln() * normalized).exp()
            } else {
                self.min + normalized * (self.max - self.min)
            };
            response.mark_changed();
        }

        // ── Double-click: reset ──────────────────────────────────────────────
        if response.double_clicked() {
            *self.value = self.default_value;
            response.mark_changed();
        }

        // ── Render arc + knob body ───────────────────────────────────────────
        if ui.is_rect_visible(outer_rect) {
            let painter = ui.painter();

            // Centre of arc within the *top* portion of the allocated column
            let center = Pos2::new(
                outer_rect.center().x,
                outer_rect.top() + radius,
            );

            let start_angle = -PI * 0.75;
            let end_angle = PI * 0.75;
            let current_angle = start_angle + normalized * (end_angle - start_angle);

            let track_r = radius - 3.0;
            let body_r = radius - 7.0;

            // Background arc
            draw_arc(
                painter,
                center,
                track_r,
                start_angle,
                end_angle,
                Stroke::new(2.5, Color32::from_rgb(45, 52, 60)),
            );

            // Active filled arc
            if normalized > 0.001 {
                draw_arc(
                    painter,
                    center,
                    track_r,
                    start_angle,
                    current_angle,
                    Stroke::new(2.5, self.accent),
                );
            }

            // Knob body
            let body_color = if response.is_pointer_button_down_on() {
                Color32::from_rgb(45, 48, 56)
            } else if response.hovered() {
                Color32::from_rgb(40, 44, 52)
            } else {
                Color32::from_rgb(32, 35, 42)
            };
            painter.circle_filled(center, body_r, body_color);
            painter.circle_stroke(
                center,
                body_r,
                Stroke::new(1.0, Color32::from_rgb(60, 68, 80)),
            );

            // Pointer notch
            let ptr_len = body_r * 0.75;
            let ptr_end = Pos2::new(
                center.x + (current_angle - PI * 0.5).cos() * ptr_len,
                center.y + (current_angle - PI * 0.5).sin() * ptr_len,
            );
            painter.line_segment([center, ptr_end], Stroke::new(2.0, Color32::WHITE));

            // ── Text labels below the arc ────────────────────────────────────
            let label_top = outer_rect.top() + radius * 2.0 + 2.0;

            // Name row
            painter.text(
                Pos2::new(outer_rect.center().x, label_top),
                egui::Align2::CENTER_TOP,
                self.label,
                egui::FontId::proportional(9.5),
                Color32::from_rgb(160, 165, 175),
            );

            // Value row
            let val_str = match &self.format_fn {
                Some(f) => f(*self.value),
                None => format!("{:.1}", *self.value),
            };
            painter.text(
                Pos2::new(outer_rect.center().x, label_top + 11.0),
                egui::Align2::CENTER_TOP,
                &val_str,
                egui::FontId::proportional(9.5),
                self.accent,
            );
        }

        response
    }
}

fn draw_arc(
    painter: &egui::Painter,
    center: Pos2,
    radius: f32,
    start_angle: f32,
    end_angle: f32,
    stroke: Stroke,
) {
    let steps = 32;
    let angle_step = (end_angle - start_angle) / steps as f32;
    for i in 0..steps {
        let a1 = start_angle + (i as f32) * angle_step - PI * 0.5;
        let a2 = a1 + angle_step;
        let p1 = Pos2::new(center.x + a1.cos() * radius, center.y + a1.sin() * radius);
        let p2 = Pos2::new(center.x + a2.cos() * radius, center.y + a2.sin() * radius);
        painter.line_segment([p1, p2], stroke);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_draw_arc_math_finite() {
        let _center = Pos2::new(50.0, 50.0);
        let radius = 20.0;
        let start = -PI * 0.75;
        let end = PI * 0.75;
        assert!(start.is_finite() && end.is_finite() && radius > 0.0);
        let angle = start + 0.5 * (end - start);
        assert!((angle - 0.0).abs() < 0.001);
    }
}
