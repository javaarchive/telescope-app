use std::path::PathBuf;

use egui::{Color32, Id, Modal};
use egui_commonmark::{commonmark, commonmark_str, CommonMarkCache};
use egui_file_dialog::FileDialog;
use telescope_core::config::Config;
use tokio::{runtime::Runtime, sync::watch};
use crate::{config, oobe::OOBEStep, settings::{self, resolve_user_data_directory}, states::DialogUiState};

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
    pub staged_workspace_path: PathBuf,
    #[serde(skip)]
    pub config_watch: Option<(watch::Sender<Config>, watch::Receiver<Config>)>,
    #[serde(skip)]
    pub file_dialog: FileDialog,
    #[serde(skip)]
    pub dialog_ui_state: DialogUiState,
    #[serde(skip)]
    pub runtime: Option<Runtime>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            state: UiState::OOBE(OOBEStep::Resume),
            md_cache: CommonMarkCache::default(),
            staged_workspace_path: resolve_user_data_directory(),
            config_watch: None,
            file_dialog: FileDialog::new(),
            dialog_ui_state: DialogUiState::None,
            runtime: None
        }
    }
}

impl AppState {

    pub fn get_default_runtime() -> Runtime {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .thread_name("telescope-app-internal-runtime")
            .build()
            .expect("Internal tokio runtime could not be constructed")
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, pane: &mut PaneState) {
        // don't show anything for OOBE this is handled by a modal
        
    }

    pub fn is_server_running(&self) -> bool {
        self.config_watch.is_some() && self.state == UiState::Proxy
    }

    pub fn accept_path(&mut self){
        // load config
        let config = Config::try_load_or_default(&self.staged_workspace_path);
        self.config_watch = Some(tokio::sync::watch::channel(config));
        if self.should_run_full_oobe() {
            self.state = UiState::OOBE(OOBEStep::Welcome);
        } else {
            self.state = UiState::Proxy;
        }
    }

    pub fn should_run_full_oobe(&self) -> bool {
        let telescope_config_path = self.staged_workspace_path.join("telescope.toml");
        !telescope_config_path.exists()
    }
    
    pub fn get_config_send(&self) -> watch::Sender<Config> {
        self.config_watch.as_ref().unwrap().0.clone()
    }

    pub fn get_config_recv(&self) -> watch::Receiver<Config> {
        self.config_watch.as_ref().unwrap().1.clone()
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
        match pane {
            PaneState::OOBE => "Out of box experience".into(),
            PaneState::Blank => "Blank Test Pane".into()
        }
    }

    fn pane_ui(
        &mut self,
        ui: &mut egui::Ui,
        _tile_id: egui_tiles::TileId,
        pane: &mut PaneState,
    ) -> egui_tiles::UiResponse {
        let title = self.tab_title_for_pane(pane);

        if ui
            .add(egui::Button::new(title.text()).sense(egui::Sense::drag()))
            .drag_started()
        {
            return egui_tiles::UiResponse::DragStarted;
        }
        self.ui(ui, pane);
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
        tabs.push(tiles.insert_pane(PaneState::Blank));
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

    pub fn render_oobe(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.heading(format!("{} Setup", config::BRAND));
        ui.label("Let's set up your environment!");
        match &self.app_state.state {
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
                        self.app_state.state = UiState::OOBE(OOBEStep::SetupPath);
                    },
                    OOBEStep::Welcome => {
                        ui.label(format!("Welcome to {}!", config::BRAND));
                        if ui.button("Next").clicked() {
                            self.app_state.state = UiState::OOBE(OOBEStep::LicenseAgreement);
                        }
                    },
                    OOBEStep::LicenseAgreement => {
                        commonmark_str!(ui, &mut self.app_state.md_cache, "telescope_app/assets/LICENSE.md"); 
                        if ui.button("Accept").clicked() {
                            self.app_state.state = UiState::OOBE(OOBEStep::SetupCerts);
                        }
                    },
                    OOBEStep::SetupPath => {
                        commonmark!(ui, &mut self.app_state.md_cache, "## Setup Data Path");
                        ui.label("Select the location to store your data");
                        ui.horizontal(|ui| {
                            ui.label("Data Directory: ");
                            // read only text field
                            let path = self.app_state.staged_workspace_path.display().to_string();
                            ui.label(path);
                            if ui.button("Select").clicked() {
                                self.app_state.dialog_ui_state = DialogUiState::ChooseWorkspacePath;
                                self.app_state.file_dialog.pick_directory();
                            }
                            if ui.button("Auto").clicked() {
                                self.app_state.staged_workspace_path = resolve_user_data_directory();
                            }
                        });

                        ui.set_width(ui.available_width());
                        if ui.button("Continue").clicked() {
                            self.app_state.accept_path();
                        }
                    },
                    OOBEStep::SetupCerts => {
                        commonmark!(ui, &mut self.app_state.md_cache, "## Setup Certificates\nWe'll need to generate a certificate for your browser to trust the certificate. This is a one-time step but you can repeat it anytime.");
                        

                        
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
    }

    pub fn catppucin_menu(&self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.menu_button("Catppucin Themes", |ui| {
            // TODO: reorder
            /*if ui.button("Mocha").clicked() {
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
            }*/
            ui.label("Unavailable while dependency needs to be updated");
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
            self.app_state.file_dialog.update(ctx);
            self.tree.ui(&mut self.app_state, ui);
            // modals
            if matches!(self.app_state.state, UiState::OOBE(_)) && !matches!(self.app_state.dialog_ui_state, DialogUiState::ChooseWorkspacePath) {
                let modal = Modal::new(Id::new("oobe_setup_modal")).show(ctx, |ui| {
                    ui.set_width(200.0);
                    self.render_oobe(ui, ctx);
                });
            }

            if let DialogUiState::ChooseWorkspacePath = self.app_state.dialog_ui_state {
                if let Some(path) = self.app_state.file_dialog.take_picked() {
                    self.app_state.staged_workspace_path = path;
                    self.app_state.dialog_ui_state = DialogUiState::None;
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
