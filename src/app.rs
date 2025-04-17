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
    fn ui(&mut self, ui: &mut egui::Ui, context: &mut AppContext, tile_id: TileId, is_floating: bool);
    fn inner_margin(&self) -> f32 {
        12.0
    }
}

// --- Event System ---
#[derive(Debug, Clone)] // Added Debug and Clone
enum UIEvent {
    UndockPanel { panel_title: String, tile_id: TileId },
    DockPanel { panel_title: String },
    ClosePanel { panel_title: String, is_floating: bool },
    ReopenPanel { panel_title: String },
}

// --- Floating Panel State ---
struct FloatingPanelState {
    panel: Box<dyn AppPanel>,
    is_open: bool,
    rect: Option<egui::Rect>,  // For position/size
}

// App context to share state between panels
pub struct AppContext {
    pub egui_ctx: egui::Context,
    pub events: Rc<RefCell<Vec<UIEvent>>>, // Added event queue
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
    floating_panels: HashMap<String, FloatingPanelState>, // Added floating panels state
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

    fn ui(&mut self, ui: &mut egui::Ui, _context: &mut AppContext, _tile_id: TileId, _is_floating: bool) {
        ui.heading("Scene View");
        
        // Draw a simple grid as placeholder
        let rect = ui.available_rect_before_wrap();
        let painter = ui.painter();
        
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
        
        // Draw a simple circle in the center
        let center = rect.center();
        painter.circle_filled(
            center, 
            50.0, 
            egui::Color32::from_rgb(100, 150, 250)
        );
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

    fn ui(&mut self, ui: &mut egui::Ui, context: &mut AppContext, tile_id: TileId, is_floating: bool) {
        let outer_rect = ui.available_rect_before_wrap(); // Get rect for Area

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

        // --- Button Area outside ScrollArea --- 
        let button_size = egui::vec2(20.0, 20.0);
        egui::Area::new(ui.id().with("_dock_undock_button_area"))
            .fixed_pos(egui::pos2(outer_rect.right() - button_size.x - 5.0, outer_rect.bottom() - button_size.y - 5.0))
            .order(egui::Order::Foreground)
            .show(ui.ctx(), |ui| {
                if is_floating {
                    // Show Dock button if floating
                    if ui.button("⚓").clicked() { // Dock icon
                        println!("[DEBUG] Dock button clicked for Settings panel (Floating)");
                        context.events.borrow_mut().push(UIEvent::DockPanel {
                            panel_title: self.title(),
                        });
                        // TODO: Find a way to signal window close on dock?
                    }
                } else {
                    // Show Undock button if docked
                    if ui.button("⏏").clicked() { // Undock icon
                        println!("[DEBUG] Undock button clicked for Settings panel (Tile ID: {:?})", tile_id);
                        context.events.borrow_mut().push(UIEvent::UndockPanel {
                            panel_title: self.title(), 
                            tile_id
                        });
                    }
                }
            });
        // --- End Button Area ---
    }
}

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

    fn ui(&mut self, ui: &mut egui::Ui, context: &mut AppContext, tile_id: TileId, is_floating: bool) {
        let outer_rect = ui.available_rect_before_wrap(); // Get rect for Area

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

        // --- Button Area outside ScrollArea --- 
        let button_size = egui::vec2(20.0, 20.0);
        egui::Area::new(ui.id().with("_dock_undock_button_area"))
            .fixed_pos(egui::pos2(outer_rect.right() - button_size.x - 5.0, outer_rect.bottom() - button_size.y - 5.0))
            .order(egui::Order::Foreground)
            .show(ui.ctx(), |ui| {
                 if is_floating {
                    if ui.button("⚓").clicked() {
                        println!("[DEBUG] Dock button clicked for Presets panel (Floating)");
                        context.events.borrow_mut().push(UIEvent::DockPanel {
                            panel_title: self.title(),
                        });
                    }
                } else {
                    if ui.button("⏏").clicked() {
                        println!("[DEBUG] Undock button clicked for Presets panel (Tile ID: {:?})", tile_id);
                        context.events.borrow_mut().push(UIEvent::UndockPanel {
                            panel_title: self.title(), 
                            tile_id
                        });
                    }
                }
            });
        // --- End Button Area ---
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

    fn ui(&mut self, ui: &mut egui::Ui, context: &mut AppContext, tile_id: TileId, is_floating: bool) {
        let outer_rect = ui.available_rect_before_wrap(); // Get rect for Area

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

        // --- Button Area outside ScrollArea --- 
        let button_size = egui::vec2(20.0, 20.0); // Icon only size
        egui::Area::new(ui.id().with("_dock_undock_button_area")) 
            .fixed_pos(egui::pos2(outer_rect.right() - button_size.x - 5.0, outer_rect.bottom() - button_size.y - 5.0))
            .order(egui::Order::Foreground)
            .show(ui.ctx(), |ui| {
                if is_floating {
                    // Show Dock button if floating
                    if ui.button("⚓").clicked() { // Dock icon
                        println!("[DEBUG] Dock button clicked for Stats panel (Floating)");
                        context.events.borrow_mut().push(UIEvent::DockPanel {
                            panel_title: self.title(),
                        });
                    }
                } else {
                    // Show Undock button if docked
                    if ui.button("⏏").clicked() { // Undock icon
                        println!("[DEBUG] Undock button clicked for Stats panel (Tile ID: {:?})", tile_id);
                        context.events.borrow_mut().push(UIEvent::UndockPanel {
                            panel_title: self.title(), 
                            tile_id
                        });
                    }
                }
            });
        // --- End Button Area ---
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

    fn ui(&mut self, ui: &mut egui::Ui, context: &mut AppContext, tile_id: TileId, is_floating: bool) {
        let outer_rect = ui.available_rect_before_wrap(); // Get rect for Area

        // Reverting to Area for button
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

        // --- Button Area outside ScrollArea --- 
        let button_size = egui::vec2(20.0, 20.0);
        egui::Area::new(ui.id().with("_dock_undock_button_area"))
            .fixed_pos(egui::pos2(outer_rect.right() - button_size.x - 5.0, outer_rect.bottom() - button_size.y - 5.0))
            .order(egui::Order::Foreground)
            .show(ui.ctx(), |ui| {
                 if is_floating {
                    if ui.button("⚓").clicked() {
                        println!("[DEBUG] Dock button clicked for Dataset panel (Floating)");
                        context.events.borrow_mut().push(UIEvent::DockPanel {
                            panel_title: self.title(),
                        });
                    }
                } else {
                    if ui.button("⏏").clicked() {
                        println!("[DEBUG] Undock button clicked for Dataset panel (Tile ID: {:?})", tile_id);
                        context.events.borrow_mut().push(UIEvent::UndockPanel {
                            panel_title: self.title(), 
                            tile_id
                        });
                    }
                }
            });
        // --- End Button Area ---
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
                    UIEvent::UndockPanel { panel_title, tile_id } => self.handle_undock_panel(panel_title, tile_id),
                    // Add DockPanel handler call
                    UIEvent::DockPanel { panel_title } => self.handle_dock_panel(panel_title),
                    UIEvent::ClosePanel { panel_title, is_floating } => self.handle_close_panel(panel_title, is_floating),
                    // Placeholder for ReopenPanel
                    UIEvent::ReopenPanel { .. } => {
                        println!("[WARN] ReopenPanel not yet implemented.");
                        Ok(())
                    }
                    // Removed catch-all '_' as we should handle all defined events
                    // _ => {
                    //     println!("[WARN] Unhandled event type: {:?}", event);
                    //     Ok(())
                    // }
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
        // TODO: Handle case where no Tabs container exists (e.g., create one?)
        println!("[WARN] No Tabs container found for docking.");
        Err("No suitable Tabs container found for docking.".to_string())
    }

    // Handler for docking a floating panel
    fn handle_dock_panel(&mut self, panel_title: String) -> Result<(), String> {
        println!("[INFO] Attempting to dock panel '{}'", panel_title);

        // 1. Remove panel from floating_panels, get the Panel data
        let floating_state = self.floating_panels.remove(&panel_title)
            .ok_or_else(|| format!("Panel '{}' not found in floating_panels for docking.", panel_title))?;
        let panel_to_dock = floating_state.panel;
        println!("[DEBUG] Removed '{}' from floating panels.", panel_title);

        // 2. Find a target container
        let target_container_id = self.find_dock_target()?;

        // 3. Insert the Panel as a new Pane tile
        // Ensure we use the AppPanel trait object correctly
        let new_pane_id = self.tree.tiles.insert_pane(panel_to_dock);
        println!("[DEBUG] Inserted new pane tile {:?} for '{}'.", new_pane_id, panel_title);

        // 4. Add the new Pane to the target container
        if let Some(Tile::Container(Container::Tabs(tabs))) = self.tree.tiles.get_mut(target_container_id) {
            tabs.add_child(new_pane_id);
            tabs.set_active(new_pane_id); // Activate the newly docked tab (Removed Some())
            println!("[DEBUG] Added pane {:?} to tabs container {:?} and activated it.", new_pane_id, target_container_id);
        } else {
            // Error handling: If the target isn't a Tabs container (shouldn't happen with current find_dock_target)
            // or if adding fails somehow, we need to recover.
            eprintln!("[ERROR] Target container {:?} is not a Tabs container or could not be modified.", target_container_id);
            
            // Attempt to recover the panel
            if let Some(Tile::Pane(recovered_panel)) = self.tree.tiles.remove(new_pane_id) {
                 println!("[DEBUG] Recovering panel '{}' after failed dock attempt.", panel_title);
                 let recovered_state = FloatingPanelState {
                    panel: recovered_panel,
                    is_open: true, // Keep it open as it failed to dock
                    rect: floating_state.rect, // Preserve old rect
                 };
                 self.floating_panels.insert(panel_title.clone(), recovered_state);
                 return Err(format!("Failed to add pane to target container {:?}. Panel recovered.", target_container_id));
            } else {
                 // Critical error - panel lost
                 return Err(format!("CRITICAL ERROR: Failed to recover panel '{}' after failed dock to {:?}. Panel lost!", panel_title, target_container_id));
            }
        }

        // 5. Ensure the tree is simplified if needed (optional, might happen on next ui call)
        self.tree.simplify_children_of_tile(target_container_id, &self.tree_ctx.simplification_options());

        println!("[INFO] Successfully docked panel '{}' into container {:?}", panel_title, target_container_id);
        Ok(())
    }

    // Handler for undocking a panel
    fn handle_undock_panel(&mut self, panel_title: String, tile_id: TileId) -> Result<(), String> {
        println!("[INFO] Attempting to undock panel '{}' (Tile ID: {:?})", panel_title, tile_id);

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
        };

        // 5. Add to floating_panels map
        if self.floating_panels.insert(panel_title.clone(), new_floating_state).is_some() {
            eprintln!("[WARN] Panel title '{}' already existed in floating_panels. Overwriting.", panel_title);
        }
        println!("[INFO] Added panel '{}' to floating_panels (open).", panel_title);

        // 6. Optional: Simplify the parent container now that a child is removed.
        //    We might defer this or rely on implicit simplification during the next tree.ui call.
        println!("[INFO] Simplifying parent container {:?} after child removal.", parent_id);
        self.tree.simplify_children_of_tile(parent_id, &self.tree_ctx.simplification_options());

        Ok(())
    }

    // Handler for closing a panel (either docked or floating)
    fn handle_close_panel(&mut self, panel_title: String, is_floating: bool) -> Result<(), String> {
        if is_floating {
            // Mark the floating panel as closed, but keep its state
            if let Some(state) = self.floating_panels.get_mut(&panel_title) {
                if state.is_open { // Only act if it was open
                    state.is_open = false;
                    println!("[INFO] Marked floating panel '{}' as closed.", panel_title);
                    Ok(())
                } else {
                    println!("[DEBUG] Floating panel '{}' was already closed.", panel_title);
                    Ok(())
                }
            } else {
                Err(format!("Floating panel '{}' not found to close.", panel_title))
            }
        } else {
            // TODO: Implement closing a DOCKED panel (Phase 5)
            println!("[WARN] Closing docked panels not yet implemented (Panel: '{}').", panel_title);
            Ok(())
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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

        for (title, state) in &mut self.floating_panels {
            if state.is_open {
                let mut still_open = true;
                let window_id = egui::Id::new(title as &str);

                let mut window = egui::Window::new(title)
                    .id(window_id)
                    .open(&mut still_open)
                    .resizable(true)
                    .default_size([250.0, 300.0]);
                
                if let Some(rect) = state.rect {
                    window = window.default_rect(rect); 
                }

                let response = window.show(ctx, |ui| {
                    let dummy_tile_id = TileId::from_u64(u64::MAX);
                    state.panel.ui(ui, &mut context_clone.write().expect("Lock poisoned"), dummy_tile_id, true);
                });

                if !still_open {
                    println!("[DEBUG] Floating window '{}' closed by user.", title);
                    events_to_queue.push(UIEvent::ClosePanel {
                        panel_title: title.clone(),
                        is_floating: true,
                    });
                }

                if let Some(inner_response) = response {
                    if inner_response.response.rect.is_finite() {
                        state.rect = Some(inner_response.response.rect);
                    } else {
                        eprintln!("[WARN] Invalid rect obtained for floating panel '{}': {:?}", title, inner_response.response.rect);
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
