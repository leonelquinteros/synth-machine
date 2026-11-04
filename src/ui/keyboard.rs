use eframe::egui::{self, Color32, Key, Pos2, Rect, Stroke, Ui};
use std::collections::HashSet;

use crate::dsp::params::SynthEvent;

pub struct PianoKeyDef {
    pub semitone_offset: u8,
    pub is_black: bool,
    pub label: &'static str,
    pub key_hint: &'static str,
}

pub const PIANO_KEYS: [PianoKeyDef; 17] = [
    // Octave 1
    PianoKeyDef { semitone_offset: 0, is_black: false, label: "C", key_hint: "A" },
    PianoKeyDef { semitone_offset: 1, is_black: true, label: "C#", key_hint: "W" },
    PianoKeyDef { semitone_offset: 2, is_black: false, label: "D", key_hint: "S" },
    PianoKeyDef { semitone_offset: 3, is_black: true, label: "D#", key_hint: "E" },
    PianoKeyDef { semitone_offset: 4, is_black: false, label: "E", key_hint: "D" },
    PianoKeyDef { semitone_offset: 5, is_black: false, label: "F", key_hint: "F" },
    PianoKeyDef { semitone_offset: 6, is_black: true, label: "F#", key_hint: "T" },
    PianoKeyDef { semitone_offset: 7, is_black: false, label: "G", key_hint: "G" },
    PianoKeyDef { semitone_offset: 8, is_black: true, label: "G#", key_hint: "Y" },
    PianoKeyDef { semitone_offset: 9, is_black: false, label: "A", key_hint: "H" },
    PianoKeyDef { semitone_offset: 10, is_black: true, label: "A#", key_hint: "U" },
    PianoKeyDef { semitone_offset: 11, is_black: false, label: "B", key_hint: "J" },
    // Octave 2
    PianoKeyDef { semitone_offset: 12, is_black: false, label: "C", key_hint: "K" },
    PianoKeyDef { semitone_offset: 13, is_black: true, label: "C#", key_hint: "O" },
    PianoKeyDef { semitone_offset: 14, is_black: false, label: "D", key_hint: "L" },
    PianoKeyDef { semitone_offset: 15, is_black: true, label: "D#", key_hint: "P" },
    PianoKeyDef { semitone_offset: 16, is_black: false, label: "E", key_hint: ";" },
];

pub struct KeyboardController {
    held_keyboard_notes: HashSet<u8>,
    mouse_held_note: Option<u8>,
}

impl KeyboardController {
    pub fn new() -> Self {
        Self {
            held_keyboard_notes: HashSet::new(),
            mouse_held_note: None,
        }
    }

    /// Process computer keyboard (QWERTY) events and return note triggers
    pub fn handle_qwerty_input(
        &mut self,
        ui: &mut Ui,
        base_octave: i32,
        events: &mut Vec<SynthEvent>,
    ) {
        let base_midi = ((4 + base_octave).clamp(1, 8) as u8) * 12;

        let key_map: &[(Key, u8)] = &[
            (Key::A, 0),
            (Key::W, 1),
            (Key::S, 2),
            (Key::E, 3),
            (Key::D, 4),
            (Key::F, 5),
            (Key::T, 6),
            (Key::G, 7),
            (Key::Y, 8),
            (Key::H, 9),
            (Key::U, 10),
            (Key::J, 11),
            (Key::K, 12),
            (Key::O, 13),
            (Key::L, 14),
            (Key::P, 15),
            (Key::Semicolon, 16),
        ];

        for &(key, semitone) in key_map {
            let note = base_midi + semitone;
            let is_down = ui.input(|i| i.key_down(key));

            if is_down && !self.held_keyboard_notes.contains(&note) {
                self.held_keyboard_notes.insert(note);
                events.push(SynthEvent::NoteOn {
                    note,
                    velocity: 0.85,
                });
            } else if !is_down && self.held_keyboard_notes.contains(&note) {
                self.held_keyboard_notes.remove(&note);
                events.push(SynthEvent::NoteOff { note });
            }
        }
    }

    /// Render visual interactive piano keyboard
    pub fn show(
        &mut self,
        ui: &mut Ui,
        base_octave: i32,
        events: &mut Vec<SynthEvent>,
        height: f32,
    ) {
        let base_midi = ((4 + base_octave).clamp(1, 8) as u8) * 12;

        // Count white keys
        let white_key_count = PIANO_KEYS.iter().filter(|k| !k.is_black).count();
        let total_width = ui.available_width();
        let white_key_width = (total_width / white_key_count as f32).max(24.0);
        let black_key_width = white_key_width * 0.62;
        let black_key_height = height * 0.62;

        let (rect, _response) = ui.allocate_exact_size(
            egui::vec2(white_key_width * white_key_count as f32, height),
            egui::Sense::click_and_drag(),
        );

        let painter = ui.painter_at(rect);
        let pointer_pos = ui.input(|i| i.pointer.hover_pos());
        let pointer_down = ui.input(|i| i.pointer.primary_down());

        let mut white_key_index = 0;
        let mut hovered_note = None;

        // First pass: Calculate and draw white keys
        for key in &PIANO_KEYS {
            if key.is_black {
                continue;
            }

            let note = base_midi + key.semitone_offset;
            let x = rect.left() + (white_key_index as f32) * white_key_width;
            let key_rect = Rect::from_min_size(
                Pos2::new(x, rect.top()),
                egui::vec2(white_key_width - 1.0, height),
            );

            let is_active = self.held_keyboard_notes.contains(&note)
                || self.mouse_held_note == Some(note);

            let is_hovered = pointer_pos.is_some_and(|pos| key_rect.contains(pos));
            if is_hovered && hovered_note.is_none() {
                hovered_note = Some(note);
            }

            let bg_color = if is_active {
                Color32::from_rgb(255, 170, 50) // Analog amber glow
            } else if is_hovered && pointer_down {
                Color32::from_rgb(230, 200, 160)
            } else {
                Color32::from_rgb(240, 240, 240)
            };

            painter.rect_filled(key_rect, 3.0, bg_color);
            painter.rect_stroke(
                key_rect,
                3.0,
                Stroke::new(1.0, Color32::from_rgb(40, 40, 40)),
                egui::StrokeKind::Inside,
            );

            // Label key hints
            let text_pos = Pos2::new(key_rect.center().x, key_rect.bottom() - 18.0);
            painter.text(
                text_pos,
                egui::Align2::CENTER_CENTER,
                format!("{}\n[{}]", key.label, key.key_hint),
                egui::FontId::proportional(10.0),
                Color32::from_rgb(60, 60, 60),
            );

            white_key_index += 1;
        }

        // Second pass: Draw black keys on top (so they take priority for click/hover)
        white_key_index = 0;
        for key in &PIANO_KEYS {
            if !key.is_black {
                white_key_index += 1;
                continue;
            }

            let note = base_midi + key.semitone_offset;
            // Black key is positioned between white_key_index - 1 and white_key_index
            let x = rect.left() + (white_key_index as f32) * white_key_width - (black_key_width * 0.5);
            let key_rect = Rect::from_min_size(
                Pos2::new(x, rect.top()),
                egui::vec2(black_key_width, black_key_height),
            );

            let is_hovered = pointer_pos.is_some_and(|pos| key_rect.contains(pos));
            if is_hovered {
                hovered_note = Some(note);
            }

            let is_active = self.held_keyboard_notes.contains(&note)
                || self.mouse_held_note == Some(note);

            let bg_color = if is_active {
                Color32::from_rgb(255, 140, 20) // Vibrant active amber
            } else if is_hovered && pointer_down {
                Color32::from_rgb(60, 60, 65)
            } else {
                Color32::from_rgb(25, 25, 28)
            };

            painter.rect_filled(key_rect, 2.0, bg_color);
            painter.rect_stroke(
                key_rect,
                2.0,
                Stroke::new(1.0, Color32::from_rgb(10, 10, 10)),
                egui::StrokeKind::Inside,
            );

            // Text on black keys
            let text_pos = Pos2::new(key_rect.center().x, key_rect.bottom() - 14.0);
            painter.text(
                text_pos,
                egui::Align2::CENTER_CENTER,
                format!("[{}]", key.key_hint),
                egui::FontId::proportional(9.0),
                Color32::from_rgb(180, 180, 180),
            );
        }

        // Mouse click/drag note trigger handling
        if pointer_down {
            if let Some(target_note) = hovered_note
                && self.mouse_held_note != Some(target_note) {
                    if let Some(prev) = self.mouse_held_note {
                        events.push(SynthEvent::NoteOff { note: prev });
                    }
                    self.mouse_held_note = Some(target_note);
                    events.push(SynthEvent::NoteOn {
                        note: target_note,
                        velocity: 0.9,
                    });
                }
        } else if let Some(prev) = self.mouse_held_note.take() {
            events.push(SynthEvent::NoteOff { note: prev });
        }
    }

    pub fn is_any_note_held(&self) -> bool {
        !self.held_keyboard_notes.is_empty() || self.mouse_held_note.is_some()
    }
}

impl Default for KeyboardController {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_piano_keys_data_integrity() {
        assert_eq!(PIANO_KEYS.len(), 17);

        let white_count = PIANO_KEYS.iter().filter(|k| !k.is_black).count();
        let black_count = PIANO_KEYS.iter().filter(|k| k.is_black).count();
        assert_eq!(white_count, 10);
        assert_eq!(black_count, 7);

        // Verify semitones are 0..=16
        for (i, key) in PIANO_KEYS.iter().enumerate() {
            assert_eq!(key.semitone_offset as usize, i);
            assert!(!key.label.is_empty());
            assert!(!key.key_hint.is_empty());
        }
    }

    #[test]
    fn test_keyboard_controller_initial_state() {
        let mut controller = KeyboardController::default();
        assert!(!controller.is_any_note_held());

        controller.held_keyboard_notes.insert(60);
        assert!(controller.is_any_note_held());

        controller.held_keyboard_notes.clear();
        assert!(!controller.is_any_note_held());

        controller.mouse_held_note = Some(64);
        assert!(controller.is_any_note_held());

        controller.mouse_held_note = None;
        assert!(!controller.is_any_note_held());
    }
}
