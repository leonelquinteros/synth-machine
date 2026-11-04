use eframe::egui;
use synth::audio::AudioSystem;
use synth::ui::SynthApp;

fn main() -> eframe::Result {
    println!("Initializing AGY-ANALOG 01 audio engine...");
    let audio = match AudioSystem::new() {
        Ok(a) => {
            println!(
                "Audio stream started successfully on [{}] @ {:.0} Hz",
                a.device_name, a.sample_rate
            );
            a
        }
        Err(err) => {
            eprintln!("Failed to initialize audio output stream: {err}");
            std::process::exit(1);
        }
    };

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("AGY-ANALOG 01 // Synthesizer")
            .with_inner_size([1040.0, 780.0])
            .with_min_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    eframe::run_native(
        "AGY-ANALOG 01",
        native_options,
        Box::new(|_cc| Ok(Box::new(SynthApp::new(audio)))),
    )
}
