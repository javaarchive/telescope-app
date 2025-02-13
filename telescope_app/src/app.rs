use std::{net::SocketAddr, path::PathBuf, sync::{Arc, RwLock}};

use egui::{Color32, Id, Modal, Rect, ScrollArea};
use egui_commonmark::{commonmark, commonmark_str, CommonMarkCache};
use egui_file_dialog::FileDialog;
use egui_taffy::{taffy::Style, tui, virtual_tui::{VirtualGridRowHelper, VirtualGridRowHelperParams}, Tui, TuiBuilderLogic};
use egui_taffy::taffy::prelude::*;
use serde::{Deserialize, Serialize};
use telescope_core::{certs::CertDerivable, config::Config, resource::{Flow, FlowContent, RequestMeta}};
use tokio::{runtime::Runtime, sync::watch};
use crate::{config, oobe::OOBEStep, settings::{self, resolve_user_data_directory}, states::DialogUiState};

pub struct ProxyUiState {
}

impl Default for ProxyUiState {
    fn default() -> Self {
        Self {

        }
    }
}

pub enum UiState {
    OOBE(OOBEStep),
    Proxy(ProxyUiState),
}

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct AppFlags {
    pub is_first_run: bool,
    pub show_logs: bool
}

pub enum PaneState {
    OOBE,
    Blank,
    FlowList
}

impl Default for PaneState {
    fn default() -> Self {
        Self::Blank
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)] 
pub struct AppState {
    #[serde(skip)]
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
    #[serde(skip)]
    pub flags: AppFlags,
    #[serde(skip)]
    pub proxy: Option<telescope_core::proxy::TelescopeProxyRef>,
    #[serde(skip)]
    pub flow_storage: Option<Arc<RwLock<telescope_core::proxy::FlowStorage>>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub enum FlowDetail {
    URL,
    Method,
    Path,
    Host
}

impl FlowDetail {
    pub fn as_str(&self) -> &'static str {
        match self {
            FlowDetail::URL => "URL",
            FlowDetail::Method => "Method",
            FlowDetail::Path => "Path",
            FlowDetail::Host => "Host"
        }
    }
}

pub const FLOW_DETAILS_ORDER_DEFAULT: [FlowDetail; 3] = [FlowDetail::Path, FlowDetail::Method, FlowDetail::Host];

impl Default for AppState {
    fn default() -> Self {
        Self {
            state: UiState::OOBE(OOBEStep::Resume),
            md_cache: CommonMarkCache::default(),
            staged_workspace_path: resolve_user_data_directory(),
            config_watch: None,
            file_dialog: FileDialog::new(),
            dialog_ui_state: DialogUiState::None,
            runtime: None,
            flags: AppFlags::default(),
            proxy: None,
            flow_storage: None
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

    pub fn ui_for_grid(&self, tui: &mut Tui, flow_detail: &FlowDetail, flow: &Flow) {
        if let FlowContent::RequestResponse(httppair) = &flow.content {
            let meta = &httppair.request.meta;
            let request: &RequestMeta = meta.unwrap_request_ref();
            match flow_detail {
                FlowDetail::URL => {
                    tui.label(request.url.as_str());
                },
                FlowDetail::Method => {
                    tui.label(request.method.as_str());
                },
                FlowDetail::Path => {
                    tui.label(request.url.path());
                },
                FlowDetail::Host => {
                    tui.label(request.url.host_str().unwrap_or("undefined"));
                },
                _ => {
                    tui.label("prop not implemented");
                }
            }
        }else{
            // futureproofing
            tui.label("NA");
        }
    }

    pub fn ui(&mut self, ui: &mut egui::Ui, pane: &mut PaneState) {
        // don't show anything for OOBE this is handled by a modal
        match pane {
            PaneState::FlowList => {
                // ui.label(format!("avali width: {}", ui.available_width()));
                ui.set_width(ui.available_width());
                if let UiState::Proxy(proxy_ui_state) = &mut self.state {
                    if let Some(flow_storage) = &self.flow_storage {
                        let flow_storage = flow_storage.read().unwrap();
                        /*if flow_storage.len() == 0 {
                            ui.label("No flows recorded yet. Connect the proxy to see flows..");
                        }*/
                        /*proxy_ui_state.flow_vlist.ui_custom_layout(ui, flow_storage.len(), |ui, start_index| {
                            let flow = flow_storage.flow_by_index(start_index).unwrap();
                            match &flow.content {
                                telescope_core::resource::FlowContent::RequestResponse(httppair) => {
                                    match &httppair.request.meta {
                                        telescope_core::resource::RequestOrResponseMeta::Request(request_meta) => {
                                            ui.label(format!("{} {} {}", request_meta.method, request_meta.url, request_meta.version));
                                        },
                                        telescope_core::resource::RequestOrResponseMeta::Response(response_meta) => panic!("resp meta in req prop"),
                                        _ => {
                                            panic!("not request or response???");
                                        }
                                    }
                                },
                                _ => {
                                    ui.label("Flow content type not implemented.");
                                }
                            }
                            
                            1
                        });*/
                        // https://github.com/PPakalns/egui_taffy/blob/main/examples/demo.rs#L193
                        // virtual_grid used from there
                        tui(ui, ui.id().with("flow_storage"))
                            .reserve_available_space()
                            .style(Style {
                                display: egui_taffy::taffy::Display::Flex,
                                flex_direction: egui_taffy::taffy::FlexDirection::Column,
                                size: percent(1.),
                                max_size: percent(1.),
                                ..Default::default()
                            })
                            .show(|tui| {
                                // println!("rect: {}", tui.root_rect());
                                tui.style(Style {
                                    display: egui_taffy::taffy::Display::Grid,
                                    grid_template_columns: vec![fr(4.), fr(1.), fr(2.)],
                                    // gap: length(8.),
                                    overflow: egui_taffy::taffy::Point {
                                        x: egui_taffy::taffy::Overflow::Clip,
                                        y: egui_taffy::taffy::Overflow::Scroll,
                                    },
                                    size: egui_taffy::taffy::Size {
                                        width:  percent(1.),
                                        height: auto(),
                                    },
                                    max_size: percent(1.),
                                    grid_auto_rows: vec![min_content()],
                                    ..Default::default()
                                }).add_with_border(|tui| {
                                    VirtualGridRowHelper::show(VirtualGridRowHelperParams {
                                        header_row_count: 1,
                                        row_count: flow_storage.len(),
                                    }, tui, |tui, info| {
                                        let mut idgen = info.id_gen();
                                        let mut_grid_row_param = info.grid_row_setter();
                                        let flow = flow_storage.flow_by_index(info.idx).unwrap();
                                        for flow_detail in FLOW_DETAILS_ORDER_DEFAULT.iter() {
                                            let _ = tui
                                                .id(idgen())
                                                .wrap_mode(egui::TextWrapMode::Truncate)
                                                .mut_style(&mut_grid_row_param)
                                                .mut_style(|style| {
                                                    // style.padding = length(2.);
                                                    // style.max_size = percent(1.);
                                                    // style.size = percent(1.);
                                                    // style.min_size = percent(1.);
                                                    style.display = egui_taffy::taffy::Display::Block;
                                                    /*style.overflow = egui_taffy::taffy::Point {
                                                        x: egui_taffy::taffy::Overflow::Clip,
                                                        y: egui_taffy::taffy::Overflow::Scroll,
                                                    };*/
                                                })
                                                .button(|tui| {
                                                    self.ui_for_grid(tui, flow_detail, flow);
                                                });
                                        }
                                        
                                    });
                                        // draw header
                                    for idx in 0..FLOW_DETAILS_ORDER_DEFAULT.len() {
                                        let flow_detail = &FLOW_DETAILS_ORDER_DEFAULT[idx];
                                        tui.sticky([false, true].into())
                                            .style(egui_taffy::taffy::Style {
                                                grid_row: egui_taffy::taffy::style_helpers::line(1 as i16),
                                                padding: length(4.),
                                                display: egui_taffy::taffy::Display::Block,
                                                ..Default::default()
                                            })
                                            .id(egui_taffy::tid(("header", 1, idx)))
                                            .add_with_background_color(|tui| {
                                                tui.label(flow_detail.as_str());
                                            });
                                    }
                                })
                            })
                    } else {
                        ui.label("Flow storage not loaded");
                    }
                }
            },
            _ => {

            }
        }
    }

    pub fn is_server_running(&self) -> bool {
        self.config_watch.is_some() && matches!(self.state, UiState::Proxy(_))
    }

    pub fn accept_path(&mut self){
        // load config
        let config = Config::try_load_or_default(&self.staged_workspace_path);
        self.config_watch = Some(tokio::sync::watch::channel(config));
        if self.should_run_full_oobe() {
            self.state = UiState::OOBE(OOBEStep::Welcome);
        } else {
            self.state = UiState::Proxy(ProxyUiState::default());
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
            PaneState::Blank => "Blank Test Pane".into(),
            PaneState::FlowList => "Flows".into()
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
            let cells = vec![tiles.insert_pane(PaneState::FlowList), tiles.insert_pane(PaneState::OOBE)];
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
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                            if ui.button("Next").clicked() {
                                self.app_state.state = UiState::OOBE(OOBEStep::LicenseAgreement);
                            }
                        });
                    },
                    OOBEStep::LicenseAgreement => {
                        commonmark_str!(ui, &mut self.app_state.md_cache, "telescope_app/assets/LICENSE.md"); 
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                            if ui.button("Accept").clicked() {
                                self.app_state.state = UiState::OOBE(OOBEStep::SetupCerts);
                            }
                        });
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
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                            if ui.button("Continue").clicked() {
                                self.app_state.accept_path();
                            }
                        });
                    },
                    OOBEStep::SetupCerts => {
                        commonmark!(ui, &mut self.app_state.md_cache, "## Setup Certificates\nWe'll need to generate a certificate for your browser to trust the certificate. This is a one-time step but you can repeat it anytime.");
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                            if ui.button("Skip").clicked() {
                                self.app_state.state = UiState::OOBE(OOBEStep::StartProxy);
                            }
                            if ui.button("Generate automatically").clicked() {
                                let runtime = self.app_state.runtime.as_ref().unwrap();
                                // TODO: make this not block ui
                                runtime.block_on(async {
                                    self.app_state.get_config_send().send_modify(|config| { 
                                        config.derive_cert().unwrap();
                                    });
                                });
                                self.app_state.state = UiState::OOBE(OOBEStep::StartProxy);
                            }
                            
                        });
                    },
                    OOBEStep::StartProxy => {
                        
                        commonmark!(ui, &mut self.app_state.md_cache, "## Start Proxy\nYou may wish to listen on a different host and port combination than the default. You may configure this here.");
                        let sender = self.app_state.get_config_send();
                        let recv = self.app_state.get_config_recv();
                        match &mut self.app_state.dialog_ui_state {
                            DialogUiState::ChooseBindAddress(addr) => {
                                ui.horizontal(|ui| {
                                    sender.send_modify(|config| { 
                                        if addr.parse::<SocketAddr>().is_ok() {
                                            ui.label("Listen on: ");
                                        } else {
                                            ui.colored_label(Color32::from_rgb(255, 0, 0), "Listen on (invalid address): ");
                                        }
                                        ui.text_edit_singleline(addr);
                                        if ui.button("Apply").clicked() {
                                            match addr.parse::<SocketAddr>() {
                                                Ok(addr) => {
                                                    config.addr = addr;
                                                },
                                                Err(err) => {
                                                    // display more detailed error
                                                }
                                            }
                                        }
                                    });
                                });
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::TOP), |ui| {
                                    if ui.button("Go").clicked() {
                                        // start the proxy for real
                                        let runtime = self.app_state.runtime.as_ref().unwrap();
                                        let config_recv_copy = recv.clone();
                                        let proxy = telescope_core::proxy::TelescopeProxy::new(config_recv_copy);
                                        let proxy_wrapper = telescope_core::proxy::TelescopeProxyRef::wrap(proxy);
                                        let flow_storage = proxy_wrapper.proxy.read().unwrap().storage.clone();
                                        let proxy_wrapper_clone = proxy_wrapper.clone();
                                        let handle = runtime.spawn(async move {
                                            proxy_wrapper_clone.start().await.unwrap();
                                            // enter proxy state
                                        });

                                        self.app_state.flow_storage = Some(flow_storage);
                                        self.app_state.proxy = Some(proxy_wrapper);
                                        self.app_state.state = UiState::Proxy(ProxyUiState::default());
                                        ctx.request_repaint();
                                    }
                                });
                            },
                            _ => {
                                ui.label("Getting current address...");
                                self.app_state.dialog_ui_state = DialogUiState::ChooseBindAddress(format!("{}", recv.borrow().addr));
                            }
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

    pub fn debug_menu(&self, ui: &mut egui::Ui, ctx: &egui::Context) {
        ui.menu_button("Debug Tools", |ui| {
            ui.label("Use these to debug the app.");
            if ui.button("Toggle hover debug").clicked() {
                ctx.style_mut(|style| {
                    style.debug.debug_on_hover = !style.debug.debug_on_hover;
                });
            }
            if ui.button("Show expansion causes").clicked() {
                ctx.style_mut(|style| {
                    style.debug.show_expand_width = true;
                    style.debug.show_expand_height = true;
                });
            }
            if ui.button("Hide expansion causes").clicked() {
                ctx.style_mut(|style| {
                    style.debug.show_expand_width = false;
                    style.debug.show_expand_height = false;
                });
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
        ctx.options_mut(|options| {
            options.max_passes = std::num::NonZeroUsize::new(2).unwrap();
        });

        // ensure OOBE is not displayed in proxy mode
        if matches!(self.app_state.state, UiState::Proxy(_)) {
            for tile_id in self.tree.active_tiles() {
                let mut remove = false;
                if let Some(pane) = self.tree.tiles.get_pane(&tile_id) {
                    if matches!(pane, PaneState::OOBE) {
                        remove = true;
                    }
                }
                if remove {
                    self.tree.remove_recursively(tile_id);
                }
            }
        }
        
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            // if self.app_state.state != UiState::Proxy {
            if !matches!(self.app_state.state, UiState::Proxy(_)) {
                egui::menu::bar(ui, |ui| {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        self.debug_menu(ui, ctx);
                        self.catppucin_menu(ui, ctx); // TODO: move to diff menu
                    });
                    ui.add_space(16.0);
                    egui::widgets::global_theme_preference_buttons(ui);
                    ui.add_space(16.0);
                    ui.colored_label(Color32::from_rgb(255, 0, 0), "Setup mode");
                    if self.app_state.runtime.is_none() {
                        ui.colored_label(Color32::from_rgb(255, 0, 0), "MISSING ASYNC RUNTIME!!!");
                    }
                });
            } else {
                egui::menu::bar(ui, |ui| {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                        self.debug_menu(ui, ctx);
                        self.catppucin_menu(ui, ctx); // TODO: move to diff menu
                    });
                    ui.add_space(8.0);
                    ui.menu_button("Help", |ui| {
                        if ui.button("Logs").clicked() {
                            self.app_state.flags.show_logs = !self.app_state.flags.show_logs;
                        }
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


            if self.app_state.flags.show_logs {
                egui::Window::new("Log").show(ctx, |ui| {
                    // draws the logger ui.
                    egui_logger::logger_ui().show(ui);
                });
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
