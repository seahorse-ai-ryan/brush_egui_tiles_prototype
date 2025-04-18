use eframe::egui;
use egui_tiles::{SimplificationOptions, Container, Tile, TileId, Tiles, Tree, UiResponse, Behavior};
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use std::cell::RefCell;
use std::rc::Rc;
// We need wasm-bindgen itself for JsCast to be found correctly sometimes
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

// Basic trait for all panels in our application
pub trait AppPanel {
    fn title(&self) -> String;
    fn panel_id(&self) -> PanelId;
    fn ui(&mut self, ui: &mut egui::Ui, context: &mut AppContext, tile_id: TileId, is_floating: bool);
    fn inner_margin(&self) -> f32 {
        12.0
    }
}

// Insert PanelId enum for strong typing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PanelId {
    Scene,
    Settings,
    Presets,
    Stats,
    Dataset,
}

// --- Event System ---
#[derive(Debug, Clone)]
enum UIEvent {
    UndockPanel { panel_id: PanelId, tile_id: TileId },
    DockPanel { panel_id: PanelId },
    ClosePanel { panel_id: PanelId, tile_id: Option<TileId> },
    ReopenPanel { panel_id: PanelId },
}

// --- Floating Panel State ---
struct FloatingPanelState {
    panel: Box<dyn AppPanel>,
    is_open: bool,
    rect: Option<egui::Rect>,  // For position/size
    last_parent_id: Option<TileId>, // Remember where it was docked
}

// App context to share state between panels
pub struct AppContext {
    pub egui_ctx: egui::Context,
    pub(crate) events: Rc<RefCell<Vec<UIEvent>>>, // Make pub(crate) to match UIEvent visibility
}

impl AppContext {
    fn new(ctx: egui::Context) -> Self {
        Self {
            egui_ctx: ctx,
            events: Rc::new(RefCell::new(Vec::new())), // Initialize event queue
        }
    }
}

// Behavior implementation for our tile tree
struct AppTree {
    context: Arc<RwLock<AppContext>>,
}

type PaneType = Box<dyn AppPanel>;

impl egui_tiles::Behavior<PaneType> for AppTree {
    fn tab_title_for_pane(&mut self, pane: &PaneType) -> egui::WidgetText {
        pane.title().into()
    }

    fn pane_ui(
        &mut self,
        ui: &mut egui::Ui,
        tile_id: TileId,
        pane: &mut PaneType,
    ) -> UiResponse {
        egui::Frame::new()
            .inner_margin(pane.inner_margin())
            .show(ui, |ui| {
                pane.ui(ui, &mut self.context.write().expect("Lock poisoned"), tile_id, false);
            });
        UiResponse::None
    }

    fn simplification_options(&self) -> SimplificationOptions {
        SimplificationOptions {
            all_panes_must_have_tabs: true,
            ..Default::default()
        }
    }

    fn gap_width(&self, _style: &egui::Style) -> f32 {
        0.5
    }
}

// Main app struct
pub struct App {
    tree: Tree<PaneType>,
    tree_ctx: AppTree,
    floating_panels: HashMap<PanelId, FloatingPanelState>, // Use PanelId for floating panels state
    context: Arc<RwLock<AppContext>>, // Keep a direct reference to context
}

// --- Panel Implementations ---

// Scene Panel
struct ScenePanel;

impl ScenePanel {
    fn new() -> Self {
        Self
    }
}

impl AppPanel for ScenePanel {
    fn title(&self) -> String {
        "Scene".to_string()
    }

    fn panel_id(&self) -> PanelId {
        PanelId::Scene
    }

    fn ui(&mut self, ui: &mut egui::Ui, context: &mut AppContext, _tile_id: TileId, is_floating: bool) {
        ui.heading("Scene View");
        
        // Wrap content in a ScrollArea to handle resizing
        egui::ScrollArea::both().auto_shrink([false, false]).show(ui, |ui| {
            // Allocate desired minimum size or use available space
            let desired_size = ui.available_size_before_wrap(); // Use available space within scroll area
            let (rect, _response) = ui.allocate_exact_size(desired_size, egui::Sense::hover());

            // Draw a simple grid as placeholder within the allocated rect
            let painter = ui.painter_at(rect);
            
            let grid_size = 30.0;
            let grid_color = egui::Color32::from_rgb(60, 60, 60);
            
            for x in (0..(rect.width() as i32)).step_by(grid_size as usize) {
                let x = rect.left() + x as f32;
                painter.line_segment(
                    [egui::pos2(x, rect.top()), egui::pos2(x, rect.bottom())],
                    (1.0, grid_color),
                );
            }
            
            for y in (0..(rect.height() as i32)).step_by(grid_size as usize) {
                let y = rect.top() + y as f32;
                painter.line_segment(
                    [egui::pos2(rect.left(), y), egui::pos2(rect.right(), y)],
                    (1.0, grid_color),
                );
            }
            
            // Draw a simple circle in the center of the allocated rect
            let center = rect.center();
            painter.circle_filled(
                center, 
                50.0, // Keep fixed radius for now
                egui::Color32::from_rgb(100, 150, 250)
            );
        }); // End ScrollArea
        
        // --- Button Area in Bottom Right --- 
        let outer_rect = ui.available_rect_before_wrap(); // Use outer_rect for positioning
        let button_size = egui::vec2(20.0, 20.0);
        egui::Area::new(ui.id().with("_dock_undock_area")) // Unique ID
            .fixed_pos(egui::pos2(outer_rect.right() - button_size.x - 5.0, outer_rect.bottom() - button_size.y - 5.0)) // Ensure using outer_rect
            .order(egui::Order::Foreground) // Revert to Foreground to prevent disappearing
            .show(ui.ctx(), |ui| {
                // --- Corrected Logic --- 
                if is_floating { // Use the correct variable name
                    // Show Dock button if floating
                    if ui.button("⚓").on_hover_text("Dock Panel").clicked() {
                        println!("[DEBUG] Dock button clicked for {:?} panel (Floating)", self.panel_id());
                        context.events.borrow_mut().push(UIEvent::DockPanel { // Use context without underscore
                            panel_id: self.panel_id(),
                        });
                    }
                } else {
                    // Show Undock button if docked 
                    if ui.button("⏏").on_hover_text("Undock Panel").clicked() {
                        println!("[DEBUG] Undock button clicked for {:?} panel (Tile ID: {:?})", self.panel_id(), _tile_id);
                        context.events.borrow_mut().push(UIEvent::UndockPanel { // Use context without underscore
                            panel_id: self.panel_id(), 
                            tile_id: _tile_id
                        });
                    }
                }
                // --- End Corrected Logic --- 
            });
    }
}

// Settings Panel
struct SettingsPanel;

impl SettingsPanel {
    fn new() -> Self {
        Self
    }
}

impl AppPanel for SettingsPanel {
    fn title(&self) -> String {
        "Settings".to_string()
    }

    fn panel_id(&self) -> PanelId {
        PanelId::Settings
    }

    fn ui(&mut self, ui: &mut egui::Ui, context: &mut AppContext, _tile_id: TileId, is_floating: bool) {
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| { 
            ui.heading("Model Settings");
            ui.label("Spherical Harmonics Degree:");
            ui.add(egui::Slider::new(&mut 3, 0..=10).text("SH Degree"));
            
            ui.add_space(10.0);
            ui.label("Max Image Resolution:");
            ui.add(egui::Slider::new(&mut 1920, 512..=4096).text("Resolution"));
            
            ui.add_space(10.0);
            ui.label("Max Splats:");
            ui.add(egui::Slider::new(&mut 100000, 1000..=1000000).text("Splats"));
            
            ui.add_space(10.0);
            ui.checkbox(&mut true, "Limit max frames");
            ui.checkbox(&mut false, "Split dataset for evaluation");
            
            ui.add_space(20.0);
            ui.heading("Training Settings");
            ui.label("Train:");
            ui.add(egui::Slider::new(&mut 30000, 1000..=100000).text("Steps"));
        }); // End of ScrollArea
        
        // --- Button Area in Bottom Right --- 
        let outer_rect = ui.available_rect_before_wrap();
        let button_size = egui::vec2(20.0, 20.0);
        egui::Area::new(ui.id().with("_dock_undock_area")) 
            .fixed_pos(egui::pos2(outer_rect.right() - button_size.x - 5.0, outer_rect.bottom() - button_size.y - 5.0)) 
            .order(egui::Order::Foreground) // Use Foreground to prevent disappearing
            .show(ui.ctx(), |ui| {
                if is_floating {
                    // Show Dock button if floating
                    if ui.button("⚓").on_hover_text("Dock Panel").clicked() {
                        println!("[DEBUG] Dock button clicked for {:?} panel (Floating)", self.panel_id());
                        context.events.borrow_mut().push(UIEvent::DockPanel {
                            panel_id: self.panel_id(),
                        });
                    }
                } else {
                    // Show Undock button if docked
                    if ui.button("⏏").on_hover_text("Undock Panel").clicked() {
                        println!("[DEBUG] Undock button clicked for {:?} panel (Tile ID: {:?})", self.panel_id(), _tile_id);
                        context.events.borrow_mut().push(UIEvent::UndockPanel {
                            panel_id: self.panel_id(), 
                            tile_id: _tile_id
                        });
                    }
                }
            }); // Ensure this }); closes the .show() closure
    } // Ensure this } closes the ui method
} // Ensure this } closes the impl AppPanel block

// Presets Panel
struct PresetsPanel;

impl PresetsPanel {
    fn new() -> Self {
        Self
    }
}

impl AppPanel for PresetsPanel {
    fn title(&self) -> String {
        "Presets".to_string()
    }

    fn panel_id(&self) -> PanelId {
        PanelId::Presets
    }

    fn ui(&mut self, ui: &mut egui::Ui, context: &mut AppContext, _tile_id: TileId, is_floating: bool) {
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            ui.heading("Presets");
            
            let presets = ["Default", "High Quality", "Fast Training", "Mobile-friendly"];
            
            for preset in presets {
                if ui.selectable_label(false, preset).clicked() {
                    // Would apply preset in real app
                }
            }
            
            ui.separator();
            ui.add_space(10.0);
            
            ui.horizontal(|ui| {
                ui.label("New preset name:");
                ui.text_edit_singleline(&mut String::new());
            });
            
            if ui.button("Save Current Settings as Preset").clicked() {
                // Would save preset in real app
            }
        });
        
        // --- Button Area in Bottom Right --- 
        let outer_rect = ui.available_rect_before_wrap();
        let button_size = egui::vec2(20.0, 20.0);
        egui::Area::new(ui.id().with("_dock_undock_area"))
            .fixed_pos(egui::pos2(outer_rect.right() - button_size.x - 5.0, outer_rect.bottom() - button_size.y - 5.0)) // Ensure using outer_rect
            .order(egui::Order::Foreground) // Use Foreground to prevent disappearing
            .show(ui.ctx(), |ui| {
                if is_floating {
                    if ui.button("⚓").on_hover_text("Dock Panel").clicked() {
                        println!("[DEBUG] Dock button clicked for {:?} panel (Floating)", self.panel_id());
                        context.events.borrow_mut().push(UIEvent::DockPanel {
                            panel_id: self.panel_id(),
                        });
                    }
                } else {
                    if ui.button("⏏").on_hover_text("Undock Panel").clicked() {
                        println!("[DEBUG] Undock button clicked for {:?} panel (Tile ID: {:?})", self.panel_id(), _tile_id);
                        context.events.borrow_mut().push(UIEvent::UndockPanel {
                            panel_id: self.panel_id(), 
                            tile_id: _tile_id
                        });
                    }
                }
            });
    }
}

// Stats Panel
struct StatsPanel;

impl StatsPanel {
    fn new() -> Self {
        Self
    }
}

impl AppPanel for StatsPanel {
    fn title(&self) -> String {
        "Stats".to_string()
    }

    fn panel_id(&self) -> PanelId {
        PanelId::Stats
    }

    fn ui(&mut self, ui: &mut egui::Ui, context: &mut AppContext, _tile_id: TileId, is_floating: bool) {
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            ui.heading("Performance Stats");
            
            ui.horizontal(|ui| {
                ui.label("Splats:");
                ui.label("112627");
            });
            
            ui.horizontal(|ui| {
                ui.label("SH Degree:");
                ui.label("3");
            });
            
            ui.horizontal(|ui| {
                ui.label("Train step:");
                ui.label("150");
            });
            
            ui.horizontal(|ui| {
                ui.label("Steps/s:");
                ui.label("56.8");
            });
            
            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);
            
            ui.heading("GPU Memory");
            
            ui.horizontal(|ui| {
                ui.label("Bytes in use:");
                ui.label("135.90 MB");
            });
            
            ui.horizontal(|ui| {
                ui.label("Bytes reserved:");
                ui.label("1.26 GB");
            });
        });
        
        // --- Button Area in Bottom Right --- 
        let outer_rect = ui.available_rect_before_wrap();
        let button_size = egui::vec2(20.0, 20.0);
        egui::Area::new(ui.id().with("_dock_undock_area"))
            .fixed_pos(egui::pos2(outer_rect.right() - button_size.x - 5.0, outer_rect.bottom() - button_size.y - 5.0)) // Ensure using outer_rect
            .order(egui::Order::Foreground) // Use Foreground to prevent disappearing
            .show(ui.ctx(), |ui| {
                if is_floating {
                    if ui.button("⚓").on_hover_text("Dock Panel").clicked() {
                        println!("[DEBUG] Dock button clicked for {:?} panel (Floating)", self.panel_id());
                        context.events.borrow_mut().push(UIEvent::DockPanel {
                            panel_id: self.panel_id(),
                        });
                    }
                } else {
                    if ui.button("⏏").on_hover_text("Undock Panel").clicked() {
                        println!("[DEBUG] Undock button clicked for {:?} panel (Tile ID: {:?})", self.panel_id(), _tile_id);
                        context.events.borrow_mut().push(UIEvent::UndockPanel {
                            panel_id: self.panel_id(), 
                            tile_id: _tile_id
                        });
                    }
                }
            });
    }
}

// Dataset Panel
struct DatasetPanel;

impl DatasetPanel {
    fn new() -> Self {
        Self
    }
}

impl AppPanel for DatasetPanel {
    fn title(&self) -> String {
        "Dataset".to_string()
    }

    fn panel_id(&self) -> PanelId {
        PanelId::Dataset
    }

    fn ui(&mut self, ui: &mut egui::Ui, context: &mut AppContext, _tile_id: TileId, is_floating: bool) {
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            ui.heading("Dataset");
            
            // Restore placeholder image drawing logic
            let rect = ui.available_rect_before_wrap();
            let painter = ui.painter();
            let img_rect = rect.shrink(20.0);
            painter.rect_filled(
                img_rect,
                0.0,
                egui::Color32::from_rgb(40, 40, 40)
            );
            
            // Keep image details controls
            ui.horizontal(|ui| {
                if ui.button("◀").clicked() {}
                ui.add(egui::Slider::new(&mut 1, 1..=311).text(""));
                if ui.button("▶").clicked() {}
                ui.label("images/DSCF4667.JPG (779×519 rgb)");
            });
        });
        
        // --- Button Area in Bottom Right --- 
        let outer_rect = ui.available_rect_before_wrap();
        let button_size = egui::vec2(20.0, 20.0);
        egui::Area::new(ui.id().with("_dock_undock_area"))
            .fixed_pos(egui::pos2(outer_rect.right() - button_size.x - 5.0, outer_rect.bottom() - button_size.y - 5.0)) // Ensure using outer_rect
            .order(egui::Order::Foreground) // Use Foreground to prevent disappearing
            .show(ui.ctx(), |ui| {
                if is_floating {
                    if ui.button("⚓").on_hover_text("Dock Panel").clicked() {
                        println!("[DEBUG] Dock button clicked for {:?} panel (Floating)", self.panel_id());
                        context.events.borrow_mut().push(UIEvent::DockPanel {
                            panel_id: self.panel_id(),
                        });
                    }
                } else {
                    if ui.button("⏏").on_hover_text("Undock Panel").clicked() {
                        println!("[DEBUG] Undock button clicked for {:?} panel (Tile ID: {:?})", self.panel_id(), _tile_id);
                        context.events.borrow_mut().push(UIEvent::UndockPanel {
                            panel_id: self.panel_id(), 
                            tile_id: _tile_id
                        });
                    }
                }
            });
    }
}

impl App {
    pub fn new(cc: &eframe::CreationContext) -> Self {
        // Set dark theme
        cc.egui_ctx.set_visuals(egui::Visuals::dark());
        
        let context = AppContext::new(cc.egui_ctx.clone());
        let context = Arc::new(RwLock::new(context));
        
        let mut tiles: Tiles<PaneType> = Tiles::default();
        
        // Create all the panels
        let scene_pane_id = tiles.insert_pane(Box::new(ScenePanel::new()));
        let settings_pane_id = tiles.insert_pane(Box::new(SettingsPanel::new()));
        let presets_pane_id = tiles.insert_pane(Box::new(PresetsPanel::new()));
        let stats_pane_id = tiles.insert_pane(Box::new(StatsPanel::new()));
        let dataset_pane_id = tiles.insert_pane(Box::new(DatasetPanel::new()));
        
        // Create left side tabs (Settings/Presets)
        let settings_tabs_id = tiles.insert_tab_tile(vec![settings_pane_id, presets_pane_id]);
        
        // Create a vertical arrangement with settings tabs and stats
        let left_panel_id = tiles.insert_vertical_tile(vec![settings_tabs_id, stats_pane_id]);
        
        // Create scene and dataset tabs
        let scene_tabs_id = tiles.insert_tab_tile(vec![scene_pane_id]);
        let dataset_tabs_id = tiles.insert_tab_tile(vec![dataset_pane_id]);
        
        // Create the main horizontal layout
        let root_id = tiles.insert_horizontal_tile(vec![left_panel_id, scene_tabs_id, dataset_tabs_id]);
        
        // Adjust sizes for the panels
        if let Some(Tile::Container(Container::Linear(lin))) = tiles.get_mut(root_id) {
            lin.shares.set_share(left_panel_id, 0.25);
            lin.shares.set_share(scene_tabs_id, 0.45);
            lin.shares.set_share(dataset_tabs_id, 0.3);
        }
        
        // Create the final tree
        let tree = Tree::new("main_tree", root_id, tiles);
        
        let tree_ctx = AppTree { context: context.clone() }; // Clone Arc for tree behavior
        
        Self {
            tree,
            tree_ctx,
            floating_panels: HashMap::new(), // Initialize empty floating panels map
            context, // Store the context directly in App
        }
    }

    // Helper function to find the parent TileId of a given child TileId
    fn find_parent_of(&self, child_id: TileId) -> Option<TileId> {
        for (parent_candidate_id, tile) in self.tree.tiles.iter() {
            if let Tile::Container(container) = tile {
                if container.children().any(|id| *id == child_id) {
                    return Some(*parent_candidate_id);
                }
            }
        }
        None // No parent found
    }

    // Stub for event processing logic
    fn process_events(&mut self) {
        let events_queue_clone = self.context.read().expect("Lock poisoned").events.clone();
        let events_to_process = events_queue_clone.borrow_mut().drain(..).collect::<Vec<_>>();

        if !events_to_process.is_empty() {
            println!("[DEBUG] Processing {} events...", events_to_process.len());
            for event in events_to_process {
                println!("[DEBUG] Event: {:?}", event);
                let result = match event {
                    UIEvent::UndockPanel { panel_id, tile_id } => self.handle_undock_panel(panel_id, tile_id),
                    UIEvent::DockPanel { panel_id } => self.handle_dock_panel(panel_id),
                    UIEvent::ClosePanel { panel_id, tile_id } => self.handle_close_panel(panel_id, tile_id),
                    UIEvent::ReopenPanel { panel_id } => {
                        // Call the actual handler
                        self.handle_reopen_panel(panel_id)
                    }
                };

                if let Err(e) = result {
                    eprintln!("[ERROR] Failed to process event: {}", e);
                    // TODO: Consider how to handle errors more robustly (e.g., logging, UI feedback)
                }
            }
        }
    }

    // Helper to find a suitable target TileId for docking
    fn find_dock_target(&self) -> Result<TileId, String> {
        // Simple strategy: Find the first Tabs container
        for (id, tile) in self.tree.tiles.iter() {
            if let Tile::Container(Container::Tabs(_)) = tile {
                println!("[DEBUG] Found Tabs container {:?} as dock target.", id);
                return Ok(*id);
            }
        }

        // If no Tabs container is found, return an error.
        // The user must manually create a suitable spot via splitting first.
        println!("[WARN] No Tabs container found for docking.");
        Err("No suitable Tabs container found for docking.".to_string())
    }

    // Handler for docking a floating panel
    fn handle_dock_panel(&mut self, panel_id: PanelId) -> Result<(), String> {
        println!("[INFO] Attempting to dock panel '{:?}'", panel_id);

        // 1. Remove panel from floating_panels, get the Panel data and state
        let floating_state = self.floating_panels.remove(&panel_id)
            .ok_or_else(|| format!("Panel '{:?}' not found in floating_panels for docking.", panel_id))?;
        let panel_to_dock = floating_state.panel;
        let last_parent_id = floating_state.last_parent_id; // Get the last parent ID
        println!("[DEBUG] Removed '{:?}' from floating panels.", panel_id);

        // 2. Determine target container: Try last parent first, fallback to find_dock_target
        let maybe_target_id = match last_parent_id {
            Some(parent_id) => {
                // Check if the last parent still exists and is a valid Tabs container
                let is_valid_target = self.tree.tiles.get(parent_id)
                    .map_or(false, |tile| matches!(tile, Tile::Container(Container::Tabs(_))));
                if is_valid_target {
                    println!("[DEBUG] Using last known parent {:?} as dock target for {:?}", parent_id, panel_id);
                    Ok(parent_id) // Use the last parent ID
                } else {
                    println!("[WARN] Last parent {:?} for {:?} is invalid/gone. Falling back to find_dock_target.", parent_id, panel_id);
                    self.find_dock_target() // Fallback - call find_dock_target and pass its Result
                }
            }
            None => {
                println!("[DEBUG] No last parent known for {:?}. Using find_dock_target.", panel_id);
                self.find_dock_target() // No last parent known, call find_dock_target
            }
        };

        // 3. Attempt to dock based on the target finding result
        match maybe_target_id {
            Ok(target_container_id) => {
                // --- Dock into existing Tabs container --- 
                println!("[DEBUG] Docking {:?} into existing container {:?}", panel_id, target_container_id);
                let new_pane_id = self.tree.tiles.insert_pane(panel_to_dock);
                println!("[DEBUG] Inserted new pane tile {:?} for '{:?}'.", new_pane_id, panel_id);

                if let Some(Tile::Container(Container::Tabs(tabs))) = self.tree.tiles.get_mut(target_container_id) {
                    tabs.add_child(new_pane_id);
                    tabs.set_active(new_pane_id);
                    println!("[DEBUG] Added pane {:?} to tabs container {:?} and activated it.", new_pane_id, target_container_id);
                    // Ensure the tree is simplified
                    self.tree.simplify_children_of_tile(target_container_id, &self.tree_ctx.simplification_options());
                    println!("[INFO] Successfully docked panel '{:?}' into container {:?}'", panel_id, target_container_id);
                    Ok(())
                } else {
                    // Error handling: Target wasn't actually Tabs, or became invalid between check and get_mut.
                    eprintln!("[ERROR] Target container {:?} is not Tabs or could not be modified.", target_container_id);
                    // Attempt to recover the panel
                    if let Some(Tile::Pane(recovered_panel)) = self.tree.tiles.remove(new_pane_id) {
                         println!("[DEBUG] Recovering panel '{:?}' after failed dock attempt.", panel_id);
                         let recovered_state = FloatingPanelState {
                            panel: recovered_panel,
                            is_open: true, 
                            rect: floating_state.rect, 
                            last_parent_id, 
                         };
                         self.floating_panels.insert(panel_id, recovered_state);
                         Err(format!("Failed to add pane to target container {:?}. Panel recovered.", target_container_id))
                    } else {
                         Err(format!("CRITICAL ERROR: Failed to recover panel '{:?}' after failed dock to {:?}. Panel lost!", panel_id, target_container_id))
                    }
                }
            }
            Err(_) => {
                // --- No suitable target found - Create new root --- 
                println!("[WARN] No suitable docking target found for {:?}. Creating new root.", panel_id);
                let mut current_tiles = std::mem::take(&mut self.tree.tiles);
                let new_pane_id = current_tiles.insert_pane(panel_to_dock);
                // Create the Tabs struct first
                let mut new_tabs_struct = egui_tiles::Tabs::new(vec![new_pane_id]);
                new_tabs_struct.active = Some(new_pane_id); // Make the new pane active
                // Insert the container tile with the Tabs struct
                let new_tabs_id = current_tiles.insert_new(Tile::Container(Container::Tabs(new_tabs_struct)));
                self.tree = Tree::new("main_tree", new_tabs_id, current_tiles); // Recreate tree
                println!("[INFO] Successfully docked panel '{:?}' by creating new root {:?}", panel_id, new_tabs_id);
                Ok(())
                // NOTE: Recovery path is complex if Tree::new fails. Assume it won't for now.
            }
        }
    }

    // Handler for undocking a panel
    fn handle_undock_panel(&mut self, panel_id: PanelId, tile_id: TileId) -> Result<(), String> {
        println!("[INFO] Attempting to undock panel '{:?}' (Tile ID: {:?})", panel_id, tile_id);

        // 1. Find the parent ID
        let parent_id = self.find_parent_of(tile_id).ok_or_else(|| 
            format!("Could not find parent for tile {:?}.", tile_id)
        )?;

        // 2. Remove the tile ID from the parent container's children
        if let Some(Tile::Container(parent_container)) = self.tree.tiles.get_mut(parent_id) {
            parent_container.remove_child(tile_id);
            println!("[DEBUG] Removed child {:?} from parent container {:?}", tile_id, parent_id);
        } else {
             return Err(format!("Parent tile {:?} is not a container or not found.", parent_id));
        }

        // 3. Remove the tile itself from the main tiles map and get the panel
        let panel_to_move = match self.tree.tiles.remove(tile_id) {
            Some(Tile::Pane(panel)) => {
                println!("[DEBUG] Removed pane tile {:?} from tree.tiles map.", tile_id);
                panel // The actual Box<dyn AppPanel>
            },
            Some(_) => return Err(format!("Tile {:?} is not a Pane, cannot undock.", tile_id)),
            None => return Err(format!("Tile {:?} not found in tree.tiles when undocking.", tile_id)),
        };

        // 4. Create floating state - MARK AS OPEN
        let default_rect = Some(egui::Rect::from_min_size(egui::pos2(100.0, 100.0), egui::vec2(250.0, 300.0))); // Simple default
        let new_floating_state = FloatingPanelState {
            panel: panel_to_move,
            is_open: true,
            rect: default_rect, // TODO: Improve default position/size later
            last_parent_id: Some(parent_id), // Remember where it was docked
        };

        // 5. Add to floating_panels map
        if self.floating_panels.insert(panel_id, new_floating_state).is_some() {
            eprintln!("[WARN] Panel '{:?}' already existed in floating_panels. Overwriting.'", panel_id);
        }
        println!("[INFO] Added panel '{:?}' to floating_panels (open).'", panel_id);

        // 6. Optional: Simplify the parent container now that a child is removed.
        //    We might defer this or rely on implicit simplification during the next tree.ui call.
        println!("[INFO] Simplifying parent container {:?} after child removal.", parent_id);
        self.tree.simplify_children_of_tile(parent_id, &self.tree_ctx.simplification_options());

        Ok(())
    }

    // Handler for reopening a closed panel
    fn handle_reopen_panel(&mut self, panel_id: PanelId) -> Result<(), String> {
        println!("[INFO] Attempting to reopen panel '{:?}'", panel_id);

        let mut target_parent_id_opt: Option<TileId> = None; // Store target parent if docking

        // 1. Check current state in floating_panels
        if let Some(state) = self.floating_panels.get_mut(&panel_id) {
            if state.is_open {
                println!("[INFO] Panel '{:?}' is already open.", panel_id);
                return Ok(()); // Already open, nothing to do
            }
            println!("[DEBUG] Reopen: Panel {:?} found, currently closed.", panel_id);

            // 2. Check if it was previously docked
            if let Some(parent_id) = state.last_parent_id {
                println!("[DEBUG] Reopen: Checking parent validity for panel {:?}", panel_id);
                // 3. Check if the parent container still exists and is valid (Tabs)
                let parent_is_valid_tabs = self.tree.tiles.get(parent_id)
                    .map_or(false, |tile| matches!(tile, Tile::Container(Container::Tabs(_))));

                if parent_is_valid_tabs {
                    println!("[DEBUG] Reopen: Parent container {:?} is valid. Preparing for re-dock.", parent_id);
                    target_parent_id_opt = Some(parent_id);
                } else {
                    println!("[WARN] Reopen: Parent container {:?} for panel {:?} no longer valid. Reopening as floating.", parent_id, panel_id);
                    state.is_open = true; // Set open here for floating case
                    state.last_parent_id = None; // Clear invalid parent
                }
            } else {
                println!("[DEBUG] Reopen: Panel {:?} was last floating. Reopening as floating.", panel_id);
                state.is_open = true; // Set open here for floating case
            }
        } else {
            return Err(format!("Cannot reopen panel '{:?}': state not found.", panel_id));
        }

        // --- Perform Docking (if target parent was valid) --- 
        if let Some(target_parent_id) = target_parent_id_opt {
            println!("[DEBUG] Reopen: Proceeding with re-docking logic for {:?}.", panel_id);
            // Remove state from floating_panels now, taking ownership of the panel
            if let Some(state_to_dock) = self.floating_panels.remove(&panel_id) {
                let panel_to_dock = state_to_dock.panel;
                println!("[DEBUG] Attempting to dock {:?} into {:?}", panel_id, target_parent_id);
                
                // Insert pane into tree
                let new_pane_id = self.tree.tiles.insert_pane(panel_to_dock);
                
                // Add pane to target container
                if let Some(Tile::Container(Container::Tabs(tabs))) = self.tree.tiles.get_mut(target_parent_id) {
                    tabs.add_child(new_pane_id);
                    tabs.set_active(new_pane_id);
                    println!("[INFO] Successfully re-docked panel '{:?}' into container {:?}", panel_id, target_parent_id);
                    self.tree.simplify_children_of_tile(target_parent_id, &self.tree_ctx.simplification_options());
                    // Docking successful
                } else {
                    eprintln!("[ERROR] Reopen: Failed to get target Tabs container {:?} during re-dock. Reverting to floating.", target_parent_id);
                    // Docking failed: Put the panel back into floating_panels state
                    // Retrieve the panel we just inserted (and remove it from tree)
                    let recovered_panel = match self.tree.tiles.remove(new_pane_id) {
                        Some(Tile::Pane(p)) => p,
                        _ => return Err(format!("CRITICAL: Failed to recover panel {:?} after failed re-dock target lookup.", panel_id))
                    };
                    let recovered_state = FloatingPanelState {
                         panel: recovered_panel, // Give panel back
                         is_open: true, // Keep it open
                         rect: None, // TODO: Restore previous rect if available?
                         last_parent_id: None, // Clear parent as docking failed
                    };
                    self.floating_panels.insert(panel_id, recovered_state);
                    return Err(format!("Failed to find/modify target container {:?} for re-dock.", target_parent_id));
                }
            } else {
                return Err(format!("Logic error: State for {:?} disappeared during reopen->dock.", panel_id));
            }
        } else {
             println!("[INFO] Panel '{:?}' reopened as floating window (is_open should be true).", panel_id);
        }

        Ok(())
    }

    // Handler for closing a panel (either docked or floating)
    fn handle_close_panel(&mut self, panel_id: PanelId, tile_id: Option<TileId>) -> Result<(), String> {
        match tile_id {
            None => {
                // --- Handle closing a FLOATING panel --- 
                // Mark the floating panel as closed, but keep its state
                if let Some(state) = self.floating_panels.get_mut(&panel_id) {
                    if state.is_open { // Only act if it was open
                        state.is_open = false;
                        println!("[INFO] Marked floating panel '{:?}' as closed.", panel_id);
                        Ok(())
                    } else {
                        println!("[DEBUG] Floating panel '{:?}' was already closed.", panel_id);
                        Ok(())
                    }
                } else {
                    Err(format!("Floating panel '{:?}' not found to close.", panel_id))
                }
            }
            Some(tile_id_to_close) => {
                // --- Handle closing a DOCKED panel --- 
                println!("[INFO] Closing docked panel '{:?}' (Tile ID: {:?})", panel_id, tile_id_to_close);
                
                // 1. Find the parent ID
                let parent_id = self.find_parent_of(tile_id_to_close).ok_or_else(|| 
                    format!("Could not find parent for tile {:?} to close.", tile_id_to_close)
                )?;

                // 2. Remove the child from the parent container
                if let Some(Tile::Container(parent_container)) = self.tree.tiles.get_mut(parent_id) {
                    parent_container.remove_child(tile_id_to_close);
                    println!("[DEBUG] Removed child {:?} from parent container {:?}", tile_id_to_close, parent_id);
                } else {
                     return Err(format!("Parent tile {:?} is not a container or not found.", parent_id));
                }

                // 3. Remove the tile itself and get the panel
                let panel = match self.tree.tiles.remove(tile_id_to_close) {
                    Some(Tile::Pane(panel)) => {
                        println!("[DEBUG] Removed pane tile {:?} from tree.tiles map.", tile_id_to_close);
                        panel
                    },
                    Some(_) => return Err(format!("Tile {:?} is not a Pane, cannot close.", tile_id_to_close)),
                    None => return Err(format!("Tile {:?} not found in tree.tiles when closing.", tile_id_to_close)),
                };

                // 4. Update or insert into floating_panels using entry API to avoid clone
                use std::collections::hash_map::Entry; 
                match self.floating_panels.entry(panel_id) {
                    Entry::Occupied(mut occupied) => {
                        // Panel state already exists, update it
                        println!("[DEBUG] Updating existing floating state for closed panel {:?}", panel_id);
                        let state = occupied.get_mut();
                        state.panel = panel; // Transfer ownership of the removed panel
                        state.is_open = false;
                        state.last_parent_id = Some(parent_id);
                    }
                    Entry::Vacant(vacant) => {
                        // Panel state doesn't exist, insert a new one
                        println!("[DEBUG] Creating new floating state for closed panel {:?}", panel_id);
                        let new_state = FloatingPanelState {
                            panel, // Transfer ownership of the removed panel
                            is_open: false,
                            rect: None,
                            last_parent_id: Some(parent_id),
                        };
                        vacant.insert(new_state);
                    }
                }
                println!("[INFO] Marked docked panel '{:?}' as closed, stored state.", panel_id);

                // 5. Simplify the parent container
                println!("[INFO] Simplifying parent container {:?} after child removal.", parent_id);
                self.tree.simplify_children_of_tile(parent_id, &self.tree_ctx.simplification_options());

                Ok(())
            }
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // --- Top Menu Bar --- 
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("View", |ui| {
                    let mut close_requested = false;
                    // Iterate over floating_panels to find closed panels
                    for (panel_id, state) in self.floating_panels.iter() {
                        if !state.is_open {
                            if ui.button(state.panel.title()).clicked() {
                                println!("[DEBUG] Reopen requested via menu for panel: {:?}", panel_id);
                                let context_lock = self.context.clone();
                                context_lock.write().expect("Lock poisoned").events.borrow_mut().push(
                                    UIEvent::ReopenPanel { panel_id: *panel_id }
                                );
                                close_requested = true;
                            }
                        }
                    }
                    if close_requested {
                        ui.close_menu();
                    }
                });
                // Add other menus here if needed (e.g., File, Edit)
            });
        });

        // Dark background
        let frame = egui::Frame::central_panel(ctx.style().as_ref())
            .inner_margin(0.0)
            .fill(egui::Color32::from_rgb(30, 30, 30));
        
        egui::CentralPanel::default()
            .frame(frame)
            .show(ctx, |ui| {
                // Restore the tree UI
                self.tree.ui(&mut self.tree_ctx, ui);
            });

        // --- Render Floating Windows --- 
        let mut events_to_queue = vec![];
        let context_clone = self.context.clone();

        for (panel_id, state) in &mut self.floating_panels {
            if state.is_open {
                let mut still_open = true;
                let window_id = egui::Id::new(*panel_id);

                let mut window = egui::Window::new(state.panel.title())
                    .id(window_id)
                    .open(&mut still_open)
                    .resizable(true)
                    .default_height(300.0)
                    .default_size([250.0, 300.0]);
                
                if let Some(rect) = state.rect {
                    window = window.default_rect(rect); 
                }

                let response = window.show(ctx, |ui| {
                    let dummy_tile_id = TileId::from_u64(u64::MAX);
                    state.panel.ui(ui, &mut context_clone.write().expect("Lock poisoned"), dummy_tile_id, true);
                });

                if !still_open {
                    println!("[DEBUG] Floating window '{:?}' closed by user.", panel_id);
                    events_to_queue.push(UIEvent::ClosePanel {
                        panel_id: *panel_id,
                        tile_id: None, // Indicate it was a floating panel
                    });
                }

                if let Some(inner_response) = response {
                    if inner_response.response.rect.is_finite() {
                        state.rect = Some(inner_response.response.rect);
                    } else {
                        eprintln!("[WARN] Invalid rect obtained for floating panel '{:?}: {:?}", panel_id, inner_response.response.rect);
                    }
                }
            }
        }

        if !events_to_queue.is_empty() {
            self.context.write().expect("Lock poisoned").events.borrow_mut().extend(events_to_queue);
        }
        
        self.process_events();
    }
}

// Native entry point
#[cfg(not(target_arch = "wasm32"))]
pub fn main() -> Result<(), eframe::Error> {
    // Use NativeOptions for desktop
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("UI Prototype Tiles"),
        ..Default::default()
    };
    
    // Run the native application
    eframe::run_native(
        "UI Prototype Tiles",
        options,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
} 

// Web entry point
#[cfg(target_arch = "wasm32")]
pub fn main() {
    // Redirect `log` message to `console.log` and friends:
    eframe::WebLogger::init(log::LevelFilter::Debug).ok();

    let web_options = eframe::WebOptions::default();

    // Define the async main function for web
    wasm_bindgen_futures::spawn_local(async {
        // Get the canvas element
        let runner = eframe::WebRunner::new();
        let canvas = web_sys::window()
            .and_then(|win| win.document())
            .and_then(|doc| doc.get_element_by_id("the_canvas_id"))
            .and_then(|elem| elem.dyn_into::<web_sys::HtmlCanvasElement>().ok())
            .expect("Could not find canvas element with id='the_canvas_id'");

        runner
            .start(
                canvas, // Pass the actual canvas element
                web_options,
                Box::new(|cc| Ok(Box::new(App::new(cc)))),
            )
            .await
            .expect("failed to start eframe");
    });
} 
