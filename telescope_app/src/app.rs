use std::path::PathBuf;

use egui::Color32;
use egui_commonmark::{commonmark, commonmark_str, CommonMarkCache};
use crate::{config, oobe::OOBEStep, settings};

#[derive(serde::Deserialize, serde::Serialize, PartialEq, Eq, Hash)]
pub enum UiState {
    OOBE(OOBEStep),
    Proxy,
}

pub enum PaneState {
    OOBE,
    Blank
}

impl Default for PaneState {
    fn default() -> Self {
        Self::Blank
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] 
pub struct AppState {
    pub state: UiState,
    #[serde(skip)]
    pub md_cache: CommonMarkCache,
    #[serde(skip)]
    pub cur_path: PathBuf,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            state: UiState::OOBE(OOBEStep::Resume),
            cur_path: settings::resolve_user_data_directory(),
            md_cache: CommonMarkCache::default(),
        }
    }
}

/// We derive Deserialize/Serialize so we can persist app state on shutdown
#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] // if we add new fields, give them default values when deserializing old state
pub struct TelescopeApp {
    #[serde(skip)]
    pub app_state: AppState,
    #[serde(skip)]
    pub tree: egui_tiles::Tree<PaneState>,
}

impl egui_tiles::Behavior<PaneState> for AppState {
    fn tab_title_for_pane(&mut self, pane: &PaneState) -> egui::WidgetText {
        format!("Test").into()
    }

    fn pane_ui(
        &mut self,
        ui: &mut egui::Ui,
        _tile_id: egui_tiles::TileId,
        pane: &mut PaneState,
    ) -> egui_tiles::UiResponse {
        ui.label("OWO");
        egui_tiles::UiResponse::None
    }
}

impl Default for TelescopeApp {
    fn default() -> Self {
        Self {
            app_state: AppState::default(), 
            tree: Self::create_tree()
        }
    }
}

impl TelescopeApp {

    pub fn create_tree() -> egui_tiles::Tree<PaneState> {
        let mut tiles = egui_tiles::Tiles::default();

        let mut tabs = vec![];
        tabs.push({
            let cells = vec![tiles.insert_pane(PaneState::OOBE)];
            tiles.insert_grid_tile(cells)
        });
        let root = tiles.insert_tab_tile(tabs);

        egui_tiles::Tree::new("app_tree", root, tiles)
    }

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

    /*pub fn render_oobe(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.heading(format!("{} Setup", config::BRAND));
        ui.label("Let's set up your environment!");
        match &self.state {
            UiState::OOBE(step) => {
                match step {
                    OOBEStep::Resume => {
                        ui.label("Loading OOBE state...");
                        // some setup stuff
                        ui.style_mut().url_in_tooltip = true;
                        if let Some(viewport_cmd) = egui::ViewportCommand::center_on_screen(ctx) {
                            ctx.send_viewport_cmd(viewport_cmd);
                        }
                        ctx.send_viewport_cmd(egui::ViewportCommand::Focus);
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
                    },
                    OOBEStep::SetupPath => {
                        commonmark!(ui, &mut self.md_cache, "## Setup Data Path");
                        ui.label(format!("Your path is currently set to: {}", self.cur_path.display()));
                        ui.label("You may wish to change this outside of the program. For example, you can create a portable.ini file to force portable mode to store data in the current directory.");
                        if ui.button("Exit Now").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        if ui.button("Continue").clicked() {
                            self.state = UiState::OOBE(OOBEStep::SetupCerts);
                        }
                    },
                    OOBEStep::SetupCerts => {
                        commonmark!(ui, &mut self.md_cache, "## Setup Certificates\nWe'll need to generate a certificate for your browser to trust the certificate. This is a one-time step but you can repeat it anytime.");
                        if ui.button("Continue").clicked() {
                            
                        }
                    },
                    _ => {
                        ui.label(format!("Did not implement step {:?}", step));
                    }
                }
            },
            _ => {

            }
        }
    }*/

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
            if ui.button("Test non-catppuccin").clicked() {
                /*ctx.style_mut(|style| {
                    
                });*/
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
            if self.app_state.state != UiState::Proxy {
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
            self.tree.ui(&mut self.app_state, ui);
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
