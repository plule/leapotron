use faust_state::StateHandle;

pub struct Leapotron {
    dsp: StateHandle,
    volume: f32,
}

impl Leapotron {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>, dsp: StateHandle) -> Self {
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        Self { dsp, volume: -25. }
    }
}

impl eframe::App for Leapotron {
    /// Called each time the UI needs repainting, which may be many times per second.
    /// Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let Self { dsp, volume } = self;

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        frame.quit();
                    }
                });
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's

            ui.add(egui::Slider::new(volume, -96.0..=0.).text("Volume"));
            egui::warn_if_debug_build(ui);
        });

        dsp.set_by_path("volume", *volume)
            .expect("Failed to set volume.");
        dsp.send();
    }
}
