//! Music instrument based on the Leap Motion and Faust

#![warn(clippy::all, rust_2018_idioms)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![cfg_attr(not(feature = "leap"), allow(dead_code))] // When building without leap support for tests, allow dead code
#![cfg_attr(not(feature = "leap"), allow(unused_variables))] // When building without leap support for tests, allow dead code

/// DSP controllable parameters
mod controls;

/// Thread computing the DSP and sending parameter updates
mod dsp_thread;

/// Thread reading the hand positions
#[cfg(feature = "leap")]
mod leap_thread;

/// Application settings
mod settings;

/// Music related types and algorithms
mod solfege;

/// Poor man's Step implementation
mod step_iter;

/// User interface
pub mod ui;

/// Generated Faust DSP
#[allow(clippy::all)]
#[rustfmt::skip]
mod dsp;

use cpal::traits::StreamTrait;
use default_boxed::DefaultBoxed;
use faust_state::DspHandle;
use settings::Settings;
pub use step_iter::StepIter;

/// Theremotion version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Theremotion icon data
const ICON: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/icon"));

fn main() {
    // Log to stdout (if you run with `RUST_LOG=debug`).
    tracing_subscriber::fmt::init();

    // Read application settings
    let settings = Settings::read();

    if settings.system.high_priority_process {
        set_high_priority();
    }

    // Init communication channels
    let (settings_tx, settings_rx) = crossbeam_channel::unbounded(); // Settings update to leap thread
    let (ui_tx, ui_rx) = crossbeam_channel::unbounded(); // UI update messages
    let (dsp_tx, dsp_rx) = crossbeam_channel::unbounded(); // DSP parameter update messages

    // Init DSP
    let dsp = dsp::Instrument::default_boxed();
    let (dsp, state) = DspHandle::<dsp::Instrument>::from_dsp(dsp);

    // Init the controls struct
    let controls = controls::Controls::from(&state);

    // Queue the initialization messages
    settings_tx.send(settings.clone()).unwrap();
    settings
        .current_preset
        .send_to_dsp(&controls, &dsp_tx)
        .unwrap();

    // Init sound output
    let stream = dsp_thread::run(dsp, state, dsp_rx);
    stream.play().expect("Failed to play stream");

    // Init leap thread
    #[cfg(feature = "leap")]
    let leap_worker = leap_thread::run(controls.clone(), settings_rx, ui_tx, dsp_tx.clone());

    // Start UI
    let fullscreen = settings.system.fullscreen;
    let initial_window_size = if fullscreen {
        None
    } else {
        Some(egui::vec2(800.0, 480.0))
    };
    let native_options = eframe::NativeOptions {
        initial_window_pos: Some(egui::Pos2 { x: 0.0, y: 0.0 }),
        initial_window_size,
        fullscreen,
        icon_data: Some(eframe::IconData {
            rgba: ICON.to_vec(),
            width: 128,
            height: 128,
        }),
        ..Default::default()
    };

    eframe::run_native(
        format!("Theremotion v{VERSION}").as_str(),
        native_options,
        Box::new(move |cc| {
            Box::new(ui::App::new(
                cc,
                ui_rx,
                dsp_tx.clone(),
                settings_tx,
                controls.clone(),
                settings,
            ))
        }),
    )
    .expect("Failed to run the UI");

    #[cfg(feature = "leap")]
    leap_worker
        .join()
        .expect("Error when stopping the leap worker");
}

#[cfg(target_os = "windows")]
fn set_high_priority() {
    unsafe {
        let process = windows::Win32::System::Threading::GetCurrentProcess();
        windows::Win32::System::Threading::SetPriorityClass(
            process,
            windows::Win32::System::Threading::REALTIME_PRIORITY_CLASS,
        )
        .expect("Failed to set high priority");
    }
}

#[cfg(target_os = "linux")]
fn set_high_priority() {
    log::warn!("High priority process is not supported on Linux yet");
}
