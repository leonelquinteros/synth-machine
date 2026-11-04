use crossbeam_channel::Sender;
use midir::{MidiInput, MidiInputConnection};

use crate::dsp::params::SynthEvent;

/// Decodes standard MIDI bytes into a synth event
pub fn parse_midi_message(message: &[u8]) -> Option<SynthEvent> {
    if message.is_empty() {
        return None;
    }

    let status = message[0];
    let msg_type = status & 0xF0;

    match msg_type {
        // Note On
        0x90 => {
            if message.len() >= 3 {
                let note = message[1];
                let vel_byte = message[2];
                if vel_byte > 0 {
                    let velocity = (vel_byte as f32 / 127.0).clamp(0.01, 1.0);
                    Some(SynthEvent::NoteOn { note, velocity })
                } else {
                    // Velocity 0 is standard MIDI Note Off
                    Some(SynthEvent::NoteOff { note })
                }
            } else {
                None
            }
        }
        // Note Off
        0x80 => {
            if message.len() >= 2 {
                let note = message[1];
                Some(SynthEvent::NoteOff { note })
            } else {
                None
            }
        }
        _ => None,
    }
}

/// Returns the names of all currently available MIDI input ports.
pub fn list_midi_input_ports() -> Vec<String> {
    let midi_in = match MidiInput::new("AGY-ANALOG 01 Query") {
        Ok(m) => m,
        Err(_) => return Vec::new(),
    };
    let ports = midi_in.ports();
    ports
        .iter()
        .filter_map(|p| midi_in.port_name(p).ok())
        .collect()
}

pub struct MidiManager {
    _connection: Option<MidiInputConnection<()>>,
    pub connected_device: Option<String>,
    pub is_available: bool,
    event_tx: Sender<SynthEvent>,
}

impl MidiManager {
    pub fn new(event_tx: Sender<SynthEvent>) -> Self {
        let mut mgr = Self {
            _connection: None,
            connected_device: None,
            is_available: false,
            event_tx: event_tx.clone(),
        };
        mgr.connect_to_port_index(0);
        mgr
    }

    /// Reconnect to a specific port by index (as returned by `list_midi_input_ports()`).
    pub fn connect_to_port_index(&mut self, port_index: usize) {
        let midi_in = match MidiInput::new("AGY-ANALOG 01 Input") {
            Ok(m) => m,
            Err(e) => {
                eprintln!("Failed to initialize MIDI input system: {e}");
                self.is_available = false;
                self._connection = None;
                self.connected_device = None;
                return;
            }
        };
        self.is_available = true;

        let ports = midi_in.ports();
        let port = match ports.get(port_index) {
            Some(p) => p,
            None => {
                // No ports available or index out of range — not an error, just stand by
                self._connection = None;
                self.connected_device = None;
                return;
            }
        };

        let port_name = midi_in
            .port_name(port)
            .unwrap_or_else(|_| "MIDI Keyboard".to_string());
        let tx = self.event_tx.clone();

        let conn_res = midi_in.connect(
            port,
            "agy-synth-port",
            move |_stamp, message, _| {
                if let Some(event) = parse_midi_message(message) {
                    let _ = tx.send(event);
                }
            },
            (),
        );

        match conn_res {
            Ok(conn) => {
                println!("Connected to MIDI device: [{}]", port_name);
                self._connection = Some(conn);
                self.connected_device = Some(port_name);
            }
            Err(err) => {
                eprintln!("Failed to connect to MIDI port {}: {err}", port_name);
                self._connection = None;
                self.connected_device = None;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_note_on() {
        let msg = [0x90, 60, 100]; // Note On, Middle C, vel 100
        let event = parse_midi_message(&msg);
        assert_eq!(
            event,
            Some(SynthEvent::NoteOn {
                note: 60,
                velocity: 100.0 / 127.0
            })
        );
    }

    #[test]
    fn test_parse_note_on_velocity_zero_is_note_off() {
        let msg = [0x90, 64, 0]; // Note On with velocity 0
        let event = parse_midi_message(&msg);
        assert_eq!(event, Some(SynthEvent::NoteOff { note: 64 }));
    }

    #[test]
    fn test_parse_note_off() {
        let msg = [0x80, 67, 64]; // Note Off
        let event = parse_midi_message(&msg);
        assert_eq!(event, Some(SynthEvent::NoteOff { note: 67 }));
    }

    #[test]
    fn test_parse_channel_offsets() {
        // Channel 2 Note On (0x91)
        let msg = [0x91, 72, 127];
        let event = parse_midi_message(&msg);
        assert_eq!(
            event,
            Some(SynthEvent::NoteOn {
                note: 72,
                velocity: 1.0
            })
        );
    }

    #[test]
    fn test_parse_empty_or_malformed() {
        assert_eq!(parse_midi_message(&[]), None);
        assert_eq!(parse_midi_message(&[0x90]), None); // truncated
    }
}
