use eframe::egui::{self, Color32, Pos2, Stroke, Ui};
use ringbuf::traits::Consumer;
use ringbuf::HeapCons;

pub struct Oscilloscope {
    history: Vec<f32>,
    history_len: usize,
}

impl Oscilloscope {
    pub fn new(capacity: usize) -> Self {
        Self {
            history: vec![0.0; capacity],
            history_len: capacity,
        }
    }

    /// Read any new samples from the ringbuffer into the rolling display history
    pub fn update(&mut self, consumer: &mut HeapCons<f32>) {
        while let Some(sample) = consumer.try_pop() {
            if self.history.len() >= self.history_len {
                self.history.remove(0);
            }
            self.history.push(sample);
        }
    }

    /// Render the oscilloscope waveform screen in egui
    pub fn show(&self, ui: &mut Ui, height: f32) {
        let desired_size = egui::vec2(ui.available_width(), height);
        let (rect, _response) = ui.allocate_exact_size(desired_size, egui::Sense::hover());

        let painter = ui.painter_at(rect);

        // Dark oscilloscope CRT screen background
        painter.rect_filled(rect, 4.0, Color32::from_rgb(12, 18, 22));
        painter.rect_stroke(
            rect,
            4.0,
            Stroke::new(1.0, Color32::from_rgb(30, 50, 60)),
            egui::StrokeKind::Inside,
        );

        // Center line (zero crossing)
        let mid_y = rect.center().y;
        painter.line_segment(
            [Pos2::new(rect.left(), mid_y), Pos2::new(rect.right(), mid_y)],
            Stroke::new(1.0, Color32::from_rgba_premultiplied(40, 70, 80, 80)),
        );

        if self.history.is_empty() {
            return;
        }

        // Draw waveform with glowing phosphor line
        let num_points = self.history.len();
        let step_x = rect.width() / (num_points.max(1) as f32);
        let amp_scale = (rect.height() * 0.42).min(100.0);

        let points: Vec<Pos2> = self
            .history
            .iter()
            .enumerate()
            .map(|(i, &s)| {
                let x = rect.left() + (i as f32) * step_x;
                let y = mid_y - s.clamp(-1.2, 1.2) * amp_scale;
                Pos2::new(x, y)
            })
            .collect();

        // Glow pass (softer wide stroke)
        for window in points.windows(2) {
            painter.line_segment(
                [window[0], window[1]],
                Stroke::new(3.0, Color32::from_rgba_premultiplied(0, 180, 160, 60)),
            );
        }

        // Core bright line
        for window in points.windows(2) {
            painter.line_segment(
                [window[0], window[1]],
                Stroke::new(1.5, Color32::from_rgb(80, 240, 210)),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ringbuf::traits::{Producer, Split};
    use ringbuf::HeapRb;

    #[test]
    fn test_oscilloscope_capacity_and_update() {
        let mut scope = Oscilloscope::new(10);
        assert_eq!(scope.history.len(), 10);
        assert_eq!(scope.history_len, 10);

        let rb = HeapRb::<f32>::new(64);
        let (mut prod, mut cons) = rb.split();

        // Push 15 samples into ring buffer
        for i in 0..15 {
            let _ = prod.try_push(i as f32);
        }

        scope.update(&mut cons);

        // Scope should retain exactly capacity (10 samples)
        assert_eq!(scope.history.len(), 10);
        // Should contain the latest samples: 5..15
        assert_eq!(scope.history.last(), Some(&14.0));
        assert_eq!(scope.history.first(), Some(&5.0));
    }
}
