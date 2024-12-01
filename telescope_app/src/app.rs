use egui::Color32;
use egui_commonmark::{CommonMarkCache, commonmark_str};
use crate::{config, oobe::OOBEStep};

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, Hash)]
pub enum UiState {
    OOBE(OOBEStep),
    Proxy,
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TelescopeApp {
    // Example stuff:
    label: String,

    #[serde(skip)] // This how you opt-out of serialization of a field
    value: f32,

    state: UiState,
    #[serde(skip)]
    md_cache: CommonMarkCache,
}

impl Default for TelescopeApp {
    fn default() -> Self {
        Self {
            // Example stuff:
            label: "Hello World!".to_owned(),
            value: 2.7,
            state: UiState::OOBE(OOBEStep::Resume),
            md_cache: CommonMarkCache::default(),
        }
    }
}

impl TelescopeApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Default::default()
    }

    pub fn render_oobe(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.heading(format!("{} Setup", config::BRAND));
        ui.label("Let's set up your environment!");
        match &self.state {
            UiState::OOBE(step) => {
                match step {
                    OOBEStep::Resume => {
                        ui.label("Loading OOBE state...");
                        ui.style_mut().url_in_tooltip = true;
                        self.state = UiState::OOBE(OOBEStep::Welcome);
                    },
                    OOBEStep::Welcome => {
                        ui.label(format!("Welcome to {}!", config::BRAND));
                        if ui.button("Next").clicked() {
                            self.state = UiState::OOBE(OOBEStep::LicenseAgreement);
                        }
                    },
                    OOBEStep::LicenseAgreement => {
                        commonmark_str!(ui, &mut self.md_cache, "telescope_app/assets/LICENSE.md"); 
                        if ui.button("Accept").clicked() {
                            self.state = UiState::OOBE(OOBEStep::SetupPath);
                        }
                    }
                    _ => {
                        ui.label(format!("Did not implement step {:?}", step));
                    }
                }
            },
            _ => {

            }
        }
    }

    pub fn catppucin_menu(&self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.menu_button("Catppucin Themes", |ui| {
            // TODO: reorder
            if ui.button("Mocha").clicked() {
                catppuccin_egui::set_theme(&ctx, catppuccin_egui::MOCHA);
            }
            if ui.button("Latte").clicked() {
                catppuccin_egui::set_theme(&ctx, catppuccin_egui::LATTE);
            }
            if ui.button("Frappe").clicked() {
                catppuccin_egui::set_theme(&ctx, catppuccin_egui::FRAPPE);
            }
            if ui.button("Macchiato").clicked() {
                catppuccin_egui::set_theme(&ctx, catppuccin_egui::MACCHIATO);
            }
        });
    }
}

impl eframe::App for TelescopeApp {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            if self.state != UiState::Proxy {
                egui::menu::bar(ui, |ui| {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        self.catppucin_menu(ui, ctx); // TODO: move to diff menu
                    });
                    ui.add_space(16.0);
                    egui::widgets::global_theme_preference_buttons(ui);
                    ui.add_space(16.0);
                    ui.colored_label(Color32::from_rgb(255, 0, 0), "Setup mode");
                });
            } else {
                egui::menu::bar(ui, |ui| {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        self.catppucin_menu(ui, ctx); // TODO: move to diff menu
                    });
                    ui.add_space(16.0);
                    egui::widgets::global_theme_preference_buttons(ui);
                });
            }
            
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            match &self.state {
                UiState::OOBE(step) => {
                    self.render_oobe(ui, ctx);
                },
                UiState::Proxy => {
                    // self.render_proxy(ui, ctx, frame);
                    ui.heading("TODO: Proxy");
                }
            }
        });
    }
}

fn powered_by_egui_and_eframe(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 0.0;
        ui.label("Powered by ");
        ui.hyperlink_to("egui", "https://github.com/emilk/egui");
        ui.label(" and ");
        ui.hyperlink_to(
            "eframe",
            "https://github.com/emilk/egui/tree/master/crates/eframe",
        );
        ui.label(".");
    });
}
