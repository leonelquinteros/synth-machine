use eframe::egui::{self, Color32, Key, RichText, Stroke, Ui, Vec2};

use crate::audio::{AudioSystem, MidiManager};
use crate::dsp::effects::ChorusMode;
use crate::dsp::lfo::{LfoTarget, LfoWaveform};
use crate::dsp::params::{FilterMode, SynthEvent, SynthParams, VoiceMode, Waveform};
use crate::dsp::presets::PresetManager;
use crate::ui::keyboard::KeyboardController;
use crate::ui::knob::RotaryKnob;
use crate::ui::visualizer::Oscilloscope;

pub struct SynthApp {
    audio: AudioSystem,
    midi: MidiManager,
    params: SynthParams,
    last_sent_params: SynthParams,
    preset_manager: PresetManager,
    keyboard: KeyboardController,
    oscilloscope: Oscilloscope,
}

impl SynthApp {
    pub fn new(audio: AudioSystem) -> Self {
        let midi = MidiManager::new(audio.event_tx.clone());
        let preset_manager = PresetManager::new();
        let params = preset_manager.current_preset().params;
        Self {
            audio,
            midi,
            params,
            last_sent_params: params,
            preset_manager,
            keyboard: KeyboardController::new(),
            oscilloscope: Oscilloscope::new(512),
        }
    }
}

impl eframe::App for SynthApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Request repaint for smooth 60fps real-time oscilloscope
        ui.ctx().request_repaint();

        // Drain audio output from ringbuffer into visualizer
        self.oscilloscope.update(&mut self.audio.viz_consumer);

        // Global octave shift hotkeys [Z] and [X]
        if ui.ctx().input(|i| i.key_pressed(Key::Z)) {
            self.params.octave_shift = (self.params.octave_shift - 1).max(-2);
        }
        if ui.ctx().input(|i| i.key_pressed(Key::X)) {
            self.params.octave_shift = (self.params.octave_shift + 1).min(2);
        }

        let mut events: Vec<SynthEvent> = Vec::new();

        // Dark analog hardware theme styling
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = Color32::from_rgb(20, 22, 26);
        visuals.window_fill = Color32::from_rgb(24, 27, 32);
        visuals.override_text_color = Some(Color32::from_rgb(230, 235, 240));
        ui.ctx().set_visuals(visuals);

        egui::CentralPanel::default().show(ui, |ui| {
            // Process QWERTY keyboard inputs
            self.keyboard
                .handle_qwerty_input(ui, self.params.octave_shift, &mut events);

            // ── 1. Top Header Bar ──────────────────────────────────────────
            render_header(
                ui,
                &mut self.params,
                &mut self.preset_manager,
                &self.audio,
                &self.midi,
            );

            ui.add_space(6.0);

            // ── 2. Main Synth Control Racks ────────────────────────────────
            // Use columns() which auto-divides available width evenly and
            // resizes gracefully as the window shrinks or grows.
            let available_w = ui.available_width();
            let col_w = (available_w - 24.0) / 4.0;

            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing.x = 8.0;

                // Column 1: DUAL OSCILLATORS & GLIDE
                ui.allocate_ui_with_layout(
                    Vec2::new(col_w, ui.available_height()),
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        render_rack_panel(ui, "DUAL OSC", Color32::from_rgb(240, 160, 40), |ui| {
                            render_osc_panel(ui, &mut self.params);
                        });
                    },
                );

                // Column 2: VCF FILTER & ENVELOPE
                ui.allocate_ui_with_layout(
                    Vec2::new(col_w, ui.available_height()),
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        render_rack_panel(ui, "VCF / FILTER", Color32::from_rgb(50, 200, 180), |ui| {
                            render_filter_panel(ui, &mut self.params);
                        });
                    },
                );

                // Column 3: LFO MODULATION
                ui.allocate_ui_with_layout(
                    Vec2::new(col_w, ui.available_height()),
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        render_rack_panel(ui, "LFO MOD", Color32::from_rgb(70, 170, 255), |ui| {
                            render_lfo_panel(ui, &mut self.params);
                        });
                    },
                );

                // Column 4: VCA ENVELOPE
                ui.allocate_ui_with_layout(
                    Vec2::new(col_w, ui.available_height()),
                    egui::Layout::top_down(egui::Align::LEFT),
                    |ui| {
                        render_rack_panel(ui, "VCA ENVELOPE", Color32::from_rgb(180, 120, 240), |ui| {
                            render_envelope_panel(ui, &mut self.params);
                        });
                    },
                );
            });

            ui.add_space(6.0);

            // ── 3. Master Effects Rack (Drive // Chorus // Delay) ───────────
            render_rack_panel(
                ui,
                "EFFECTS RACK  //  ANALOG DRIVE • JUNO CHORUS • TAPE DELAY",
                Color32::from_rgb(255, 90, 140),
                |ui| {
                    render_effects_panel(ui, &mut self.params);
                },
            );

            ui.add_space(6.0);

            // ── 4. Oscilloscope (fills remaining vertical space above keyboard) ──
            let kb_height = 112.0; // keyboard + hint label
            let footer_height = 24.0;
            let scope_height = (ui.available_height() - kb_height - footer_height - 16.0)
                .max(55.0);

            ui.label(
                RichText::new("OSCILLOSCOPE  //  REAL-TIME OUTPUT")
                    .small()
                    .color(Color32::from_rgb(140, 150, 160)),
            );
            self.oscilloscope.show(ui, scope_height);

            ui.add_space(6.0);

            // ── 4. Interactive Keyboard Panel ──────────────────────────────
            ui.horizontal(|ui| {
                ui.label(
                    RichText::new("KEYBOARD  //  LIVE INPUT")
                        .small()
                        .color(Color32::from_rgb(140, 150, 160)),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if self.keyboard.is_any_note_held() {
                        ui.label(
                            RichText::new("● GATE")
                                .strong()
                                .color(Color32::from_rgb(255, 170, 40)),
                        );
                    } else {
                        ui.label(
                            RichText::new("○ STANDBY")
                                .small()
                                .color(Color32::from_rgb(100, 105, 115)),
                        );
                    }
                });
            });

            self.keyboard.show(ui, self.params.octave_shift, &mut events, 100.0);

            ui.add_space(2.0);
            ui.label(
                RichText::new(
                    "QWERTY: A–L / W–P  •  Z/X: Octave  •  Drag Knobs (Shift=fine)  •  Dbl-click: Reset",
                )
                .small()
                .color(Color32::from_rgb(110, 115, 125)),
            );
        });

        // Forward musical events to audio thread
        for event in events {
            let _ = self.audio.event_tx.send(event);
        }

        // If synth parameters changed, push update to audio thread
        if self.params != self.last_sent_params {
            let _ = self
                .audio
                .event_tx
                .send(SynthEvent::UpdateParams(self.params));
            self.last_sent_params = self.params;
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
//  Section renderers
// ─────────────────────────────────────────────────────────────────────────────

fn render_header(
    ui: &mut Ui,
    params: &mut SynthParams,
    preset_manager: &mut PresetManager,
    audio: &AudioSystem,
    midi: &MidiManager,
) {
    // Row 1: brand + preset + master volume
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;

        ui.label(
            RichText::new("⚡ AGY-ANALOG 01")
                .strong()
                .size(16.0)
                .color(Color32::from_rgb(255, 185, 40)),
        );

        ui.separator();

        ui.label(RichText::new("Preset:").strong());
        let current_name = preset_manager.current_preset().name.clone();
        let preset_items: Vec<(String, String)> = preset_manager
            .presets()
            .iter()
            .map(|p| (p.name.clone(), p.category.clone()))
            .collect();
        let current_index = preset_manager.current_index();
        let mut selected_preset: Option<usize> = None;
        egui::ComboBox::from_id_salt("preset_select")
            .selected_text(&current_name)
            .width(210.0)
            .show_ui(ui, |ui| {
                for (i, (name, category)) in preset_items.iter().enumerate() {
                    let label = format!("[{}] {}", category, name);
                    if ui.selectable_label(i == current_index, label).clicked() {
                        selected_preset = Some(i);
                    }
                }
            });
        if let Some(loaded) = selected_preset.and_then(|i| preset_manager.select_preset(i)) {
            *params = loaded;
        }

        ui.separator();

        for &mode in &VoiceMode::ALL {
            let lbl = match mode {
                VoiceMode::Mono => "MONO",
                VoiceMode::Poly8 => "POLY 8V",
            };
            if ui.selectable_label(params.voice_mode == mode, lbl).clicked() {
                params.voice_mode = mode;
            }
        }

        ui.separator();

        ui.label("Vol:");
        ui.add(
            egui::Slider::new(&mut params.master_volume, 0.0..=1.0)
                .show_value(false)
                .custom_formatter(|n, _| format!("{:.0}%", n * 100.0)),
        );
        ui.label(
            RichText::new(format!("{:.0}%", params.master_volume * 100.0))
                .color(Color32::from_rgb(220, 225, 235)),
        );
    });

    // Row 2: octave + device status — compact, right-aligned on wider screens
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;

        ui.label(
            RichText::new("Octave:")
                .small()
                .color(Color32::from_rgb(170, 175, 185)),
        );
        if ui.small_button("◀ Z").clicked() {
            params.octave_shift = (params.octave_shift - 1).max(-2);
        }
        ui.label(
            RichText::new(format!("{:+}", params.octave_shift))
                .strong()
                .color(Color32::from_rgb(255, 180, 50)),
        );
        if ui.small_button("X ▶").clicked() {
            params.octave_shift = (params.octave_shift + 1).min(2);
        }

        ui.separator();

        // MIDI status
        if let Some(ref dev) = midi.connected_device {
            ui.label(
                RichText::new(format!("🎹 {}", dev))
                    .small()
                    .color(Color32::from_rgb(255, 190, 50)),
            );
        } else {
            ui.label(
                RichText::new("🎹 MIDI Ready")
                    .small()
                    .color(Color32::from_rgb(140, 145, 155)),
            );
        }

        ui.separator();

        // Audio device
        ui.label(
            RichText::new(format!(
                "🔊 {}  @  {:.1} kHz",
                audio.device_name,
                audio.sample_rate / 1000.0
            ))
            .small()
            .color(Color32::from_rgb(100, 200, 160)),
        );
    });
}

fn render_osc_panel(ui: &mut Ui, params: &mut SynthParams) {
    let osc1_color = Color32::from_rgb(255, 190, 60);
    let osc2_color = Color32::from_rgb(255, 130, 40);
    let mix_color = Color32::from_rgb(240, 200, 80);

    // ── OSC 1 ──
    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new("OSC 1").strong().color(osc1_color));
        for &wf in &Waveform::ALL {
            let lbl = waveform_label(wf);
            if ui
                .selectable_label(params.osc1_waveform == wf, lbl)
                .clicked()
            {
                params.osc1_waveform = wf;
            }
        }
    });

    ui.horizontal_wrapped(|ui| {
        ui.label("Oct:");
        if ui.small_button("◀").clicked() {
            params.osc1_octave = (params.osc1_octave - 1).max(-2);
        }
        ui.label(RichText::new(format!("{:+}", params.osc1_octave)).strong());
        if ui.small_button("▶").clicked() {
            params.osc1_octave = (params.osc1_octave + 1).min(2);
        }
        if params.osc1_waveform == Waveform::Square {
            ui.add_space(4.0);
            RotaryKnob::new("PW1", &mut params.osc1_pulse_width, 0.05, 0.95, 0.5, osc1_color)
                .format(|n| format!("{:.0}%", n * 100.0))
                .show(ui, 14.0);
        }
    });

    ui.separator();

    // ── OSC 2 ──
    ui.horizontal_wrapped(|ui| {
        ui.label(RichText::new("OSC 2").strong().color(osc2_color));
        for &wf in &Waveform::ALL {
            let lbl = waveform_label(wf);
            if ui
                .selectable_label(params.osc2_waveform == wf, lbl)
                .clicked()
            {
                params.osc2_waveform = wf;
            }
        }
    });

    ui.horizontal_wrapped(|ui| {
        ui.label("Oct:");
        if ui.small_button("◀").clicked() {
            params.osc2_octave = (params.osc2_octave - 1).max(-2);
        }
        ui.label(RichText::new(format!("{:+}", params.osc2_octave)).strong());
        if ui.small_button("▶").clicked() {
            params.osc2_octave = (params.osc2_octave + 1).min(2);
        }
        ui.label("Semi:");
        ui.add(egui::DragValue::new(&mut params.osc2_semitone).range(-12..=12));
    });

    ui.add_space(4.0);
    ui.horizontal_wrapped(|ui| {
        RotaryKnob::new("Detune", &mut params.osc2_detune_cents, -50.0, 50.0, 0.0, osc2_color)
            .format(|n| format!("{:.1}¢", n))
            .show(ui, 14.0);

        if params.osc2_waveform == Waveform::Square {
            RotaryKnob::new("PW2", &mut params.osc2_pulse_width, 0.05, 0.95, 0.5, osc2_color)
                .format(|n| format!("{:.0}%", n * 100.0))
                .show(ui, 14.0);
        }

        RotaryKnob::new("Mix", &mut params.osc_mix, 0.0, 1.0, 0.5, mix_color)
            .format(|n| format!("{:.0}%", n * 100.0))
            .show(ui, 14.0);

        RotaryKnob::new("Glide", &mut params.glide_time, 0.0, 1.0, 0.0, Color32::from_rgb(255, 215, 90))
            .format(|n| {
                if n < 0.005 {
                    "Off".to_string()
                } else {
                    format!("{:.0}ms", n * 1000.0)
                }
            })
            .show(ui, 14.0);
    });
}

fn render_filter_panel(ui: &mut Ui, params: &mut SynthParams) {
    let color = Color32::from_rgb(50, 200, 180);

    ui.label(RichText::new("Mode").strong());
    ui.horizontal_wrapped(|ui| {
        for &mode in &FilterMode::ALL {
            let lbl = match mode {
                FilterMode::LowPass => "LPF",
                FilterMode::BandPass => "BPF",
                FilterMode::HighPass => "HPF",
            };
            if ui.selectable_label(params.filter_mode == mode, lbl).clicked() {
                params.filter_mode = mode;
            }
        }
    });

    ui.add_space(4.0);

    ui.horizontal_wrapped(|ui| {
        RotaryKnob::new("CUTOFF", &mut params.filter_cutoff, 20.0, 20000.0, 2500.0, color)
            .logarithmic(true)
            .format(|n| {
                if n >= 1000.0 {
                    format!("{:.1}k", n / 1000.0)
                } else {
                    format!("{:.0}Hz", n)
                }
            })
            .show(ui, 18.0);

        RotaryKnob::new("RESO", &mut params.filter_resonance, 0.5, 15.0, 2.0, color)
            .format(|n| format!("Q{:.1}", n))
            .show(ui, 18.0);

        RotaryKnob::new("ENV", &mut params.filter_env_depth, -4.0, 4.0, 0.0, color)
            .format(|n| format!("{:+0.1}o", n))
            .show(ui, 18.0);
    });

    ui.separator();

    ui.label(RichText::new("Filter ADSR").strong());
    ui.horizontal_wrapped(|ui| {
        RotaryKnob::new("ATK", &mut params.filter_attack, 0.001, 2.0, 0.01, color)
            .format(|n| format!("{:.0}ms", n * 1000.0))
            .show(ui, 13.0);

        RotaryKnob::new("DEC", &mut params.filter_decay, 0.001, 2.0, 0.3, color)
            .format(|n| format!("{:.0}ms", n * 1000.0))
            .show(ui, 13.0);

        RotaryKnob::new("SUS", &mut params.filter_sustain, 0.0, 1.0, 0.0, color)
            .format(|n| format!("{:.0}%", n * 100.0))
            .show(ui, 13.0);

        RotaryKnob::new("REL", &mut params.filter_release, 0.001, 4.0, 0.3, color)
            .format(|n| format!("{:.0}ms", n * 1000.0))
            .show(ui, 13.0);
    });
}

fn render_lfo_panel(ui: &mut Ui, params: &mut SynthParams) {
    let color = Color32::from_rgb(70, 170, 255);

    ui.label(RichText::new("Waveform").strong());
    ui.horizontal_wrapped(|ui| {
        for &wf in &LfoWaveform::ALL {
            let lbl = match wf {
                LfoWaveform::Sine => "Sin",
                LfoWaveform::Triangle => "Tri",
                LfoWaveform::Square => "Sqr",
            };
            if ui.selectable_label(params.lfo_waveform == wf, lbl).clicked() {
                params.lfo_waveform = wf;
            }
        }
    });

    ui.add_space(4.0);
    ui.label(RichText::new("Target").strong());
    ui.horizontal_wrapped(|ui| {
        for &target in &LfoTarget::ALL {
            let lbl = match target {
                LfoTarget::Pitch => "Pitch",
                LfoTarget::Filter => "Cutoff",
            };
            if ui.selectable_label(params.lfo_target == target, lbl).clicked() {
                params.lfo_target = target;
            }
        }
    });

    ui.add_space(6.0);
    ui.horizontal_wrapped(|ui| {
        RotaryKnob::new("RATE", &mut params.lfo_rate, 0.05, 20.0, 4.0, color)
            .logarithmic(true)
            .format(|n| format!("{:.1}Hz", n))
            .show(ui, 18.0);

        RotaryKnob::new("DEPTH", &mut params.lfo_depth, 0.0, 1.0, 0.0, color)
            .format(|n| format!("{:.0}%", n * 100.0))
            .show(ui, 18.0);
    });

    ui.add_space(2.0);
    let hint = match params.lfo_target {
        LfoTarget::Pitch => "Vibrato (±1 semitone)",
        LfoTarget::Filter => "Auto-wah (±2 octaves)",
    };
    ui.label(
        RichText::new(hint)
            .small()
            .color(Color32::from_rgb(130, 160, 190)),
    );
}

fn render_envelope_panel(ui: &mut Ui, params: &mut SynthParams) {
    let color = Color32::from_rgb(180, 120, 240);

    ui.label(RichText::new("Amp ADSR").strong());
    ui.add_space(4.0);

    ui.horizontal_wrapped(|ui| {
        RotaryKnob::new(
            "ATK",
            &mut params.amp_attack,
            0.001,
            2.0,
            0.01,
            color,
        )
        .format(|n| format!("{:.0}ms", n * 1000.0))
        .show(ui, 16.0);

        RotaryKnob::new(
            "DEC",
            &mut params.amp_decay,
            0.001,
            2.0,
            0.2,
            color,
        )
        .format(|n| format!("{:.0}ms", n * 1000.0))
        .show(ui, 16.0);

        RotaryKnob::new(
            "SUS",
            &mut params.amp_sustain,
            0.0,
            1.0,
            0.6,
            color,
        )
        .format(|n| format!("{:.0}%", n * 100.0))
        .show(ui, 16.0);

        RotaryKnob::new(
            "REL",
            &mut params.amp_release,
            0.001,
            4.0,
            0.25,
            color,
        )
        .format(|n| format!("{:.0}ms", n * 1000.0))
        .show(ui, 16.0);
    });
}

fn waveform_label(wf: Waveform) -> &'static str {
    match wf {
        Waveform::Saw => "Saw",
        Waveform::Square => "Sqr",
        Waveform::Triangle => "Tri",
        Waveform::Sine => "Sin",
    }
}

fn render_effects_panel(ui: &mut Ui, params: &mut SynthParams) {
    let drive_color = Color32::from_rgb(255, 110, 110);
    let chorus_color = Color32::from_rgb(80, 215, 255);
    let delay_color = Color32::from_rgb(255, 195, 80);

    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = 12.0;

        // ── 1. DRIVE / TUBE SATURATION ──
        ui.vertical(|ui| {
            ui.label(RichText::new("DRIVE / SAT").strong().color(drive_color));
            ui.add_space(2.0);
            RotaryKnob::new("DRIVE", &mut params.drive, 0.0, 1.0, 0.0, drive_color)
                .format(|n| {
                    if n < 0.01 {
                        "Clean".to_string()
                    } else {
                        format!("{:.0}%", n * 100.0)
                    }
                })
                .show(ui, 17.0);
        });

        ui.separator();

        // ── 2. STEREO CHORUS (ROLAND JUNO BBD) ──
        ui.vertical(|ui| {
            ui.label(RichText::new("JUNO CHORUS").strong().color(chorus_color));
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                for &mode in &ChorusMode::ALL {
                    let lbl = match mode {
                        ChorusMode::Off => "Off",
                        ChorusMode::Mode1 => "I",
                        ChorusMode::Mode2 => "II",
                        ChorusMode::Mode1And2 => "I+II",
                    };
                    if ui.selectable_label(params.chorus_mode == mode, lbl).clicked() {
                        params.chorus_mode = mode;
                    }
                }

                ui.add_space(4.0);
                RotaryKnob::new("MIX", &mut params.chorus_mix, 0.0, 1.0, 0.5, chorus_color)
                    .format(|n| format!("{:.0}%", n * 100.0))
                    .show(ui, 17.0);
            });
        });

        ui.separator();

        // ── 3. STEREO TAPE DELAY ──
        ui.vertical(|ui| {
            ui.label(RichText::new("TAPE DELAY").strong().color(delay_color));
            ui.add_space(2.0);
            ui.horizontal(|ui| {
                RotaryKnob::new("TIME", &mut params.delay_time, 0.02, 1.0, 0.35, delay_color)
                    .format(|n| {
                        if n >= 1.0 {
                            format!("{:.2}s", n)
                        } else {
                            format!("{:.0}ms", n * 1000.0)
                        }
                    })
                    .show(ui, 17.0);

                RotaryKnob::new("FEEDBACK", &mut params.delay_feedback, 0.0, 0.85, 0.35, delay_color)
                    .format(|n| format!("{:.0}%", n * 100.0))
                    .show(ui, 17.0);

                RotaryKnob::new("TONE", &mut params.delay_tone, 0.1, 1.0, 0.6, delay_color)
                    .format(|n| {
                        if n < 0.35 {
                            "Dark".to_string()
                        } else if n > 0.75 {
                            "Bright".to_string()
                        } else {
                            "Warm".to_string()
                        }
                    })
                    .show(ui, 17.0);

                RotaryKnob::new("MIX", &mut params.delay_mix, 0.0, 1.0, 0.0, delay_color)
                    .format(|n| format!("{:.0}%", n * 100.0))
                    .show(ui, 17.0);
            });
        });
    });
}

// ─────────────────────────────────────────────────────────────────────────────
//  Shared panel frame helper
// ─────────────────────────────────────────────────────────────────────────────

fn render_rack_panel<R>(
    ui: &mut Ui,
    title: &str,
    accent: Color32,
    content: impl FnOnce(&mut Ui) -> R,
) -> R {
    egui::Frame::group(ui.style())
        .fill(Color32::from_rgb(28, 32, 38))
        .stroke(Stroke::new(1.0, Color32::from_rgb(45, 52, 62)))
        .inner_margin(egui::Margin::same(10))
        .show(ui, |ui| {
            // Coloured accent bar + title
            ui.horizontal(|ui| {
                let (rect, _) = ui.allocate_exact_size(Vec2::new(4.0, 13.0), egui::Sense::hover());
                ui.painter().rect_filled(rect, 1.0, accent);
                ui.label(
                    RichText::new(title)
                        .strong()
                        .color(Color32::from_rgb(220, 225, 235)),
                );
            });
            ui.separator();
            content(ui)
        })
        .inner
}

// ─────────────────────────────────────────────────────────────────────────────
//  Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_params_equal_identical() {
        let a = SynthParams::default();
        let b = SynthParams::default();
        assert_eq!(a, b);
    }

    #[test]
    fn test_params_equal_difference_detected() {
        let a = SynthParams::default();

        let mut b = a;
        b.filter_cutoff += 50.0;
        assert_ne!(a, b);

        let mut c = a;
        c.osc1_waveform = Waveform::Sine;
        assert_ne!(a, c);

        let mut d = a;
        d.osc2_detune_cents = 12.0;
        assert_ne!(a, d);

        let mut e = a;
        e.osc_mix = 0.8;
        assert_ne!(a, e);

        let mut f = a;
        f.glide_time = 0.05;
        assert_ne!(a, f);

        let mut g = a;
        g.filter_env_depth = 2.0;
        assert_ne!(a, g);

        let mut h = a;
        h.lfo_rate = 8.0;
        assert_ne!(a, h);

        let mut i = a;
        i.voice_mode = VoiceMode::Poly8;
        assert_ne!(a, i);

        let mut j = a;
        j.drive = 0.5;
        assert_ne!(a, j);

        let mut k = a;
        k.chorus_mode = ChorusMode::Mode1;
        assert_ne!(a, k);

        let mut l = a;
        l.delay_mix = 0.3;
        assert_ne!(a, l);
    }
}
