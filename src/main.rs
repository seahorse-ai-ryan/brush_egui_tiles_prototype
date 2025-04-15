use std::{collections::HashMap, sync::atomic::{AtomicU32, Ordering}};
use std::rc::Rc;
use std::cell::RefCell;

use egui::{Color32, Frame, Id, Response, Rect, RichText, WidgetText};
use egui_tiles::{Behavior, Container, SimplificationOptions, TabState, Tabs, Tile, TileId, Tiles, Tree, UiResponse};

// --- Constants ---

const TREE_BACKGROUND: Color32 = Color32::from_rgb(30, 30, 30); // Darker background
const FLOATING_BACKGROUND: Color32 = Color32::from_rgb(50, 50, 50); // Slightly lighter floaters

// Atomic counter for generating unique Tile IDs
static NEXT_TILE_ID: AtomicU32 = AtomicU32::new(1);

fn generate_tile_id_and_u64() -> (TileId, u64) {
    let id_u64 = NEXT_TILE_ID.fetch_add(1, Ordering::Relaxed) as u64;
    (TileId::from_u64(id_u64), id_u64)
}

// --- Structs for Panel Data ---

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
struct MockPanel {
    id: u32, // Panel ID, distinct from TileId
    title: String,
    is_permanent: bool, // Can this panel be closed/undocked?
}

impl MockPanel {
    fn new(title: &str, is_permanent: bool) -> Self {
        static NEXT_PANEL_ID: AtomicU32 = AtomicU32::new(1);
        Self {
            id: NEXT_PANEL_ID.fetch_add(1, Ordering::Relaxed),
            title: title.to_owned(),
            is_permanent,
        }
    }

    // Basic UI for the panel content area
    fn ui(&mut self, ui: &mut egui::Ui, is_floating: bool, event_queue: Option<&Rc<RefCell<Vec<UIEvent>>>>) {
        // Panel Title Label (Non-draggable)
        let _response = ui.add(egui::Label::new(
            RichText::new(format!("Panel: '{}' (ID: {})", self.title, self.id))
                .color(Color32::WHITE)
                .strong()
        ));
        
        ui.separator();
        
        // ADD DOCK BUTTON HERE (Top of content area, only if floating)
        if is_floating {
            if let Some(queue) = event_queue {
                // Ensure buttons are laid out horizontally
                ui.horizontal(|ui| { 
                    // Replace "Dock Me" and Pin with a single Dock Icon button
                    let dock_button = egui::Button::new("📥").small(); // Inbox Tray icon
                    if ui.add(dock_button).on_hover_text("Dock Panel").clicked() {
                         println!("[DEBUG] Dock button (Tray Icon) clicked for panel ID {}", self.id);
                         queue.borrow_mut().push(UIEvent::DockPanel { panel_id: self.id });
                    }
                });
                ui.add_space(4.0); // Add a little space below buttons
            } else {
                ui.label(RichText::new("Error: Event queue missing for floating panel.").color(Color32::RED));
            }
        }
        
        // --- Content Area with Scroll --- 
        egui::ScrollArea::vertical() // Make content vertically scrollable
            .auto_shrink([false, false]) // Don't shrink vertically or horizontally
            .show(ui, |ui| {
                ui.label(RichText::new(format!("Content for {}", self.title)).color(Color32::WHITE));
                match self.title.as_str() {
                    "Scene" => {
                        ui.label("Shows the main 3D view.");
                        ui.label(LOREM_IPSUM_SHORT);
                    },
                    "Settings" => {
                        ui.checkbox(&mut false, "Enable Feature X");
                        ui.checkbox(&mut false, "Another Setting");
                        ui.label(LOREM_IPSUM_LONG);
                    },
                    "Properties" => {
                        ui.label("Object Name: Cube");
                        ui.horizontal(|ui| {
                            ui.label("Position X:");
                            ui.add(egui::DragValue::new(&mut 0.0));
                        });
                        ui.label(LOREM_IPSUM_SHORT);
                    },
                    "Stats" => {
                        ui.label("Frames: 1234");
                        ui.label("Vertices: 56789");
                        ui.label(LOREM_IPSUM_SHORT);
                    },
                    "Presets" => {
                        ui.button("Load Preset A");
                        ui.button("Load Preset B");
                        ui.label(LOREM_IPSUM_LONG);
                    },
                     "Dataset" => {
                        ui.label("Loaded dataset: example.zip");
                        ui.label("Item count: 100");
                        ui.label(LOREM_IPSUM_SHORT);
                    },
                    "Placeholder" => {
                        ui.label("This panel is just a placeholder.");
                        ui.label(LOREM_IPSUM_SHORT);
                    },
                    _ => {
                        ui.label("Default panel content area.");
                        ui.label(LOREM_IPSUM_LONG);
                    }
                }
        });
        // --- End Content Area ---
    }
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
struct FloatingPanelState {
    panel: MockPanel,
    is_open: bool,
    current_rect: Option<Rect>,
}

impl FloatingPanelState {
    fn new(panel: MockPanel, is_open: bool, initial_rect: Option<Rect>) -> Self {
        Self {
            panel,
            is_open,
            current_rect: initial_rect,
        }
    }
}

// --- UI Event Enum ---

#[derive(Debug, Clone)]
enum UIEvent {
    ClosePanelFromTree { tile_id: TileId },
    ClosePanelFloating { panel_id: u32 },
    ReopenPanel { panel_id: u32 },
    DockPanel { panel_id: u32 },
    DockPanelToTarget { 
        panel_id: u32, 
        target_id: Option<TileId>,
        position: Option<egui::Pos2>
    },
    UndockPanel { tile_id: TileId },
    // Request to activate a specific tab within its parent container
    RequestActivateTab { tile_id: TileId },
}

// --- Layout Error Type ---
type LayoutError = String;

// --- App State and Behavior ---

struct App {
    tree: Tree<MockPanel>,
    floating_panels: HashMap<u32, FloatingPanelState>,
    event_queue: Rc<RefCell<Vec<UIEvent>>>,
    behavior: MyBehavior,
    show_terminology_window: bool,
}

impl Default for App {
    fn default() -> Self {
        let event_queue = Rc::new(RefCell::new(Vec::new()));
        println!("[SETUP] Creating Brush-like initial layout with persistent primary container");
        
        let mut tiles = Tiles::default();

        // --- Create Panes & Tiles ---
        // All panels are now closable (not permanent)
        let settings_panel = MockPanel::new("Settings", false);
        let presets_panel = MockPanel::new("Presets", false);
        let properties_panel = MockPanel::new("Properties", false);
        let stats_panel = MockPanel::new("Stats", false);
        let scene_panel = MockPanel::new("Scene", false); // Make Scene closable again
        let dataset_panel = MockPanel::new("Dataset", false);

        // --- Create Tile IDs and insert Panes (as before) ---
        let settings_pane_id = generate_tile_id_and_u64().0;
        tiles.insert(settings_pane_id, Tile::Pane(settings_panel));
        let presets_pane_id = generate_tile_id_and_u64().0;
        tiles.insert(presets_pane_id, Tile::Pane(presets_panel));
        let properties_pane_id = generate_tile_id_and_u64().0;
        tiles.insert(properties_pane_id, Tile::Pane(properties_panel));
        let stats_pane_id = generate_tile_id_and_u64().0;
        tiles.insert(stats_pane_id, Tile::Pane(stats_panel));
        let scene_pane_id = generate_tile_id_and_u64().0;
        tiles.insert(scene_pane_id, Tile::Pane(scene_panel));
        let dataset_pane_id = generate_tile_id_and_u64().0;
        tiles.insert(dataset_pane_id, Tile::Pane(dataset_panel));
        println!("[SETUP] Created Panes: Settings({:?}), Presets({:?}), Properties({:?}), Stats({:?}), Scene({:?}), Dataset({:?})",
                 settings_pane_id, presets_pane_id, properties_pane_id, stats_pane_id, scene_pane_id, dataset_pane_id);

        // --- Create Containers ---
        let mut left_top_tabs = Tabs::default(); // Settings/Presets/Properties
        left_top_tabs.add_child(settings_pane_id);
        left_top_tabs.add_child(presets_pane_id);
        left_top_tabs.add_child(properties_pane_id);
        left_top_tabs.active = Some(settings_pane_id); // Default to Settings
        let left_tabs_id = generate_tile_id_and_u64().0;
        tiles.insert(left_tabs_id, Tile::Container(Container::Tabs(left_top_tabs)));
        
        let mut stats_tabs = Tabs::default(); // Stats
        stats_tabs.add_child(stats_pane_id);
        stats_tabs.active = Some(stats_pane_id);
        let stats_tabs_id = generate_tile_id_and_u64().0;
        tiles.insert(stats_tabs_id, Tile::Container(Container::Tabs(stats_tabs)));

        let left_column_id = generate_tile_id_and_u64().0; // Vertical: left_tabs | stats_tabs
        let left_column_container = Container::new_vertical(vec![left_tabs_id, stats_tabs_id]);
        tiles.insert(left_column_id, Tile::Container(left_column_container));
        
        // This is our primary container - stores Scene tab
        let mut scene_tabs = Tabs::default(); // Scene
        scene_tabs.add_child(scene_pane_id);
        scene_tabs.active = Some(scene_pane_id);
        let scene_tabs_id = generate_tile_id_and_u64().0;
        tiles.insert(scene_tabs_id, Tile::Container(Container::Tabs(scene_tabs)));
        
        let mut dataset_tabs = Tabs::default(); // Dataset
        dataset_tabs.add_child(dataset_pane_id);
        dataset_tabs.active = Some(dataset_pane_id);
        let dataset_tabs_id = generate_tile_id_and_u64().0;
        tiles.insert(dataset_tabs_id, Tile::Container(Container::Tabs(dataset_tabs)));
        
        // Root is the Horizontal layout container again
        let root_id = generate_tile_id_and_u64().0;
        let root_container = Container::new_horizontal(vec![left_column_id, scene_tabs_id, dataset_tabs_id]);
        tiles.insert(root_id, Tile::Container(root_container));
        println!("[SETUP] Created Root container: {:?}", root_id);
        
        // --- Create Tree ---
        let tree = Tree::new(egui::Id::new("brush_tree"), root_id, tiles);
        
        // --- Create App State ---
        let floating_panels = HashMap::new();
        
        let app = Self {
            tree,
            floating_panels,
            event_queue: event_queue.clone(),
            behavior: MyBehavior { 
                event_queue,
            },
            show_terminology_window: false,
        };
        
        println!("[SETUP] App created with initial docked layout.");
        app
    }
}

// Helper for default panel positioning (implementation in Step 2)
impl App {
    fn get_default_rect(&self, panel_title: &str) -> Rect {
        // Simple positioning based on title - refine later
        match panel_title {
            "Settings" => Rect::from_min_size(egui::pos2(750.0, 50.0), egui::vec2(250.0, 300.0)),
            "Properties" => Rect::from_min_size(egui::pos2(750.0, 400.0), egui::vec2(250.0, 300.0)),
            _ => {
                // Default fallback position for unknown panels
                println!("[WARN] No default rect defined for '{}', using fallback.", panel_title);
                Rect::from_min_size(egui::pos2(100.0, 100.0), egui::vec2(200.0, 200.0))
            }
        }
    }

    // Improved method to find tab containers in the tree
    fn find_main_tabs_container_id(&self) -> Option<TileId> {
        // First, try to find the tabs container created during initialization
        if let Some(root_id) = self.tree.root() {
            // Breadth-first search for the first tabs container
            let mut queue = vec![root_id];
            while let Some(current_id) = queue.pop() {
                if let Some(tile) = self.tree.tiles.get(current_id) {
                    match tile {
                        Tile::Container(Container::Tabs(_)) => {
                            // Found a tabs container
                            return Some(current_id);
                        },
                        Tile::Container(container) => {
                            // Add all children to the queue
                            for child_id in container.children() {
                                queue.push(*child_id);
                            }
                        },
                        _ => {} // Skip panes
                    }
                }
            }
        }
        
        None
    }
    
    // Find all available tabs containers in the tree (for more advanced docking)
    fn find_all_tabs_containers(&self) -> Vec<TileId> {
        let mut tab_containers = Vec::new();
        
        if let Some(root_id) = self.tree.root() {
            // Breadth-first search for all tabs containers
            let mut queue = vec![root_id];
            while let Some(current_id) = queue.pop() {
                if let Some(tile) = self.tree.tiles.get(current_id) {
                    match tile {
                        Tile::Container(Container::Tabs(_)) => {
                            // Found a tabs container
                            tab_containers.push(current_id);
                        },
                        Tile::Container(container) => {
                            // Add all children to the queue
                            for child_id in container.children() {
                                queue.push(*child_id);
                            }
                        },
                        _ => {} // Skip panes
                    }
                }
            }
        }
        
        tab_containers
    }
}

// Our custom behavior struct
struct MyBehavior {
    event_queue: Rc<RefCell<Vec<UIEvent>>>,
}

impl Behavior<MockPanel> for MyBehavior {
    // Defines the look and behavior of tabs
    fn tab_ui(
        &mut self,
        tiles: &mut Tiles<MockPanel>,
        ui: &mut egui::Ui,
        _tab_id: Id,
        tile_id: TileId,
        tab_state: &TabState,
    ) -> Response {
        let Some(Tile::Pane(pane)) = tiles.get(tile_id) else {
            let response = ui.label("ERR");
            return response.interact(egui::Sense::click());
        };
        let title = self.tab_title_for_pane(pane);
        let is_permanent = pane.is_permanent;

        // Use a single horizontal layout for the whole tab content
        // The response of this layout determines drag sense etc.
        let response = ui.horizontal(|ui| {
            // Selectable label for the title
            let label_response = ui.selectable_label(tab_state.active, title);
            
            // Only show undock button if NOT permanent
            if !is_permanent { 
                let undock_button = egui::Button::new("⏏") 
                    .small()
                    .frame(false); // Make it less intrusive
                if ui.add(undock_button).on_hover_text("Undock Panel").clicked() {
                    // Use the event queue, no need to return anything special here
                    self.event_queue
                        .borrow_mut()
                        .push(UIEvent::UndockPanel { tile_id });
                }
            } 
            
            // Return the label response for click detection below
            label_response 
        }).inner; // Get the response of the inner label

        // Check for activation click based on the label's response
        // Important: Do this *after* the horizontal layout is built
        if response.clicked() && !tab_state.active {
            println!("[DEBUG] Tab clicked: {:?}, Queueing RequestActivateTab.", tile_id);
            self.event_queue
                .borrow_mut()
                .push(UIEvent::RequestActivateTab { tile_id });
        }

        // Return the response of the *entire horizontal layout* 
        // This response should correctly sense clicks and drags for egui_tiles default handling
        response.union(ui.rect_contains_pointer(ui.max_rect())) // Ensure full area senses hover/drag
    }

    // REQUIRED: Defines the title displayed in a tab.
    fn tab_title_for_pane(&mut self, pane: &MockPanel) -> WidgetText {
        pane.title.clone().into()
    }

    // REQUIRED: Defines the content of a pane within a tile.
    fn pane_ui(
        &mut self,
        ui: &mut egui::Ui,
        _tile_id: TileId,
        pane: &mut MockPanel,
    ) -> UiResponse {
        pane.ui(ui, false, None);
        UiResponse::None
    }

    // Correct name for adding widgets to the right of the tabs
    fn top_bar_right_ui(
        &mut self,
        _tiles: &Tiles<MockPanel>,
        ui: &mut egui::Ui,
        _tab_id: TileId,
        tabs: &Tabs,
        _scroll_offset: &mut f32,
    ) {
        // Get the pane IDs from tabs container
        let pane_ids = &tabs.children;
        if pane_ids.is_empty() || tabs.active.is_none() {
            return; // Return nothing (void)
        }
        
        let selected_pane_id = tabs.active.unwrap();
        
        // Show buttons for the selected tab
        ui.horizontal(|ui| {
            // Show ONLY the close button for all tabs
            if ui.button("✖").on_hover_text("Close tab").clicked() {
                println!("[DEBUG] Close button clicked for tile: {:?}", selected_pane_id);
                self.event_queue
                    .borrow_mut()
                    .push(UIEvent::ClosePanelFromTree { tile_id: selected_pane_id });
            }
            
            // REMOVE the Undock button from here
            /*
            if ui.button("⏏").on_hover_text("Undock tab").clicked() {
                println!("[DEBUG] Undock button clicked for tile: {:?}", selected_pane_id);
                self.event_queue
                    .borrow_mut()
                    .push(UIEvent::UndockPanel { tile_id: selected_pane_id });
            }
            */
        });
    }

    // Simplification options
    fn simplification_options(&self) -> SimplificationOptions {
        // Allow pruning of both empty tabs AND empty containers
        println!("[DEBUG] Applying simplification options (prune empty containers and tabs)");
        SimplificationOptions {
            prune_empty_tabs: true,        // Prune empty tab groups
            prune_empty_containers: true,  // CHANGED: Remove empty containers too
            prune_single_child_tabs: false, // Keep disabled for stability
            prune_single_child_containers: false, // Keep disabled
            all_panes_must_have_tabs: false, // Allow panes without tabs
            ..Default::default()
        }
    }
}

// --- eframe App Implementation ---

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        println!("[DEBUG] App::update frame start");

        // Top Menu Bar
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                // View Menu (as before)
                ui.menu_button("View", |ui| {
                    ui.label("Reopen Closed Panel:");
                    ui.separator();

                    // Collect closed panel IDs and titles
                    let closed_panels: Vec<(u32, String)> = self.floating_panels
                        .iter()
                        .filter(|(_, state)| !state.is_open)
                        .map(|(id, state)| (*id, state.panel.title.clone()))
                        .collect();

                    if closed_panels.is_empty() {
                        ui.weak("(None)");
                    } else {
                        for (panel_id, title) in closed_panels {
                            if ui.button(format!("{} (ID: {})", title, panel_id)).clicked() {
                                println!("[DEBUG] Reopen button clicked for panel ID {}", panel_id);
                                // Queue the reopen event
                                self.event_queue
                                    .borrow_mut()
                                    .push(UIEvent::ReopenPanel { panel_id });
                                ui.close_menu(); // Close menu after clicking
                            }
                        }
                    }
                });
                
                // Help Menu (New)
                ui.menu_button("Help", |ui| {
                    if ui.button("Terminology").clicked() {
                        self.show_terminology_window = true;
                        ui.close_menu();
                    }
                });
            });
        });

        // Main tiled area
        egui::CentralPanel::default()
            .frame(egui::Frame::NONE)
            .show(ctx, |ui| {
            let style = ui.style_mut();
            style.spacing.window_margin = egui::Margin::same(0);

            if self.tree.is_empty() {
                // Show a message when the dock area is empty
                ui.centered_and_justified(|ui| {
                    ui.label("Dock Area is Empty");
                });
            } else {
                // Render the tile tree using behavior WITHOUT primary container ID
                let mut behavior = MyBehavior {
                    event_queue: self.event_queue.clone(),
                };
                self.tree.ui(&mut behavior, ui);
            }
        });

        // Render floating windows
        let open_floating_panel_ids: Vec<u32> = self.floating_panels
            .iter()
            .filter(|(_, state)| state.is_open)
            .map(|(id, _)| *id)
            .collect();

        for panel_id in open_floating_panel_ids {
            if let Some(state) = self.floating_panels.get_mut(&panel_id) {
                 let window_id = Id::new("floating_window").with(panel_id);
                 let mut is_window_open = state.is_open;

                 // Use standard title bar again
                 let mut window = egui::Window::new(&state.panel.title) // Use original title directly
                    .id(window_id)
                    .open(&mut is_window_open)
                    .resizable(true)
                    .min_width(150.0)
                    .min_height(100.0) // Keep min height
                    .default_height(300.0) // Keep default height
                    .max_height(1000.0) // ADD MAX HEIGHT
                    .frame(Frame::window(&ctx.style()).fill(FLOATING_BACKGROUND));
                    // .title_bar(true); // Default is true, no need to set explicitly

                 if let Some(rect) = state.current_rect {
                     println!("[RESIZE DEBUG] Window {} using stored rect: {:?}", panel_id, rect);
                     window = window.current_pos(rect.min);
                 }
                 
                 let queue_clone = self.event_queue.clone();

                 // Show window with default title bar, render content inside
                let response = window.show(ctx, |ui| {
                    // Render actual panel content 
                    state.panel.ui(ui, true, Some(&queue_clone));
                });

                 // Update state after window interaction
                 state.is_open = is_window_open; // Reflect if user closed window via 'X'
                 if !state.is_open {
                     println!("[DEBUG] Floating window closed by user: ID {}", panel_id);
                     // Queue event using shared queue
                     self.event_queue
                         .borrow_mut()
                         .push(UIEvent::ClosePanelFloating { panel_id });
                 }

                 // Store position if window was shown and moved/resized
                 if let Some(inner_response) = response {
                    if inner_response.response.rect.min.is_finite() && inner_response.response.rect.max.is_finite() {
                        // Log if the rect has changed
                        if state.current_rect != Some(inner_response.response.rect) {
                            println!("[RESIZE DEBUG] Storing new rect for {}: {:?}", panel_id, inner_response.response.rect);
                        }
                        state.current_rect = Some(inner_response.response.rect);
                    } else {
                         println!("[WARN] Invalid rect obtained for floating panel {}: {:?}", panel_id, inner_response.response.rect);
                    }
                 }
            }
        }

        // --- Terminology Window --- 
        let mut show_terminology_window = self.show_terminology_window;
        egui::Window::new("UI Component Terminology")
            .open(&mut show_terminology_window)
            .resizable(true)
            .default_width(450.0)
            .show(ctx, |ui| {
                ui.label("Understanding the parts of the UI:");

                ui.add_space(5.0);
                ui.strong("Standard egui Components:");
                egui::Frame::new().inner_margin(5.0).fill(Color32::from_rgb(55, 55, 55)).show(ui, |ui| {
                    ui.label("Window (egui::Window): Floats freely, holds content.");
                    egui::Frame::new().inner_margin(5.0).fill(Color32::from_rgb(65, 65, 65)).show(ui, |ui|{
                         ui.label("Ui (egui::Ui): The context for adding widgets inside a Window/Panel/Area.");
                         ui.label("Widget (egui::Button, Label, etc.): Basic interactive elements.");
                         ui.label("Panel (MockPanel): Our custom struct holding the data/state FOR a specific view (Scene, Settings...). Not a direct UI element itself.");
                    });
                });

                ui.add_space(10.0);
                ui.strong("egui_tiles Specific Components:");
                egui::Frame::new().inner_margin(5.0).fill(Color32::from_rgb(50, 50, 70)).show(ui, |ui| {
                     ui.label("Tree (egui_tiles::Tree): Manages the entire docked layout.");
                     egui::Frame::new().inner_margin(5.0).fill(Color32::from_rgb(60, 60, 80)).show(ui, |ui|{
                        ui.label("Tile (egui_tiles::TileId): An ID for an item within the Tree.");
                        ui.label("Container (Tile::Container): Holds other Tiles (Horizontal, Vertical, Tabs).");
                        ui.label("Pane (Tile::Pane): Holds one of our 'Panels' (e.g., Settings data).");
                    });
                    ui.add_space(5.0);
                    egui::Frame::new().inner_margin(5.0).fill(Color32::from_rgb(70, 70, 90)).show(ui, |ui|{
                         ui.label("Tabs (Container::Tabs): A Container showing child Panes as tabs.");
                         ui.horizontal(|ui|{
                             ui.label("Tab:");
                             egui::Frame::new().inner_margin(2.0).fill(Color32::from_gray(80)).show(ui, |ui|{ui.label("Settings");});
                             egui::Frame::new().inner_margin(2.0).fill(Color32::from_gray(60)).show(ui, |ui|{ui.label("Presets");});
                         });
                     });
                });

            });
        self.show_terminology_window = show_terminology_window;
        // --- End Terminology Window ---

        // --- Process Event Queue (Placeholder for later steps) ---
        let events_to_process = self.event_queue.borrow_mut().drain(..).collect::<Vec<_>>();
        if !events_to_process.is_empty() {
            println!("[DEBUG] Processing {} events...", events_to_process.len());
        }
        for event in events_to_process {
            // Use catch_unwind if necessary, but primarily rely on Result
            match self.process_ui_event(event) {
                Ok(_) => println!("[INFO] Event processed successfully."),
                Err(e) => eprintln!("[ERROR] Failed to process event: {}", e),
            }
        }

        println!("[DEBUG] App::update frame end");

        // --- Validate Invariants (Placeholder) ---
        self.validate_invariants();
    }
}

// --- Event Processing Logic ---
impl App {
    // Helper function to find the parent TileId of a given child TileId
    fn find_parent_of(&self, child_id: TileId) -> Option<TileId> {
        for (parent_candidate_id, tile) in self.tree.tiles.iter() {
            if let Tile::Container(container) = tile {
                // `container.children()` provides an iterator over child TileIds
                if container.children().any(|id| *id == child_id) {
                    return Some(*parent_candidate_id);
                }
            }
        }
        None // No parent found
    }

    fn process_ui_event(&mut self, event: UIEvent) -> Result<(), LayoutError> {
        println!("[DEBUG] Processing event: {:?}", event);
        
        // Add detailed event-specific diagnostics 
        match &event {
            UIEvent::DockPanel { panel_id } => {
                println!("[DOCK DEBUG] Dock attempt for panel ID: {}", panel_id);
                println!("[DOCK DEBUG] Current tree structure BEFORE docking:");
                self.print_tree_diagnostic();
                println!("[DOCK DEBUG] Available floating panels: {:?}", 
                    self.floating_panels.keys().collect::<Vec<_>>());
            },
            _ => {}
        }
        
        let result = match event {
            UIEvent::ClosePanelFromTree { tile_id } => self.handle_close_from_tree(tile_id),
            UIEvent::ClosePanelFloating { panel_id } => self.handle_close_floating(panel_id),
            UIEvent::ReopenPanel { panel_id } => self.handle_reopen_panel(panel_id),
            UIEvent::DockPanel { panel_id } => self.handle_dock_panel(panel_id),
            UIEvent::DockPanelToTarget { panel_id, target_id, position } => 
                self.handle_dock_panel_to_target(panel_id, target_id, position),
            UIEvent::UndockPanel { tile_id } => self.handle_undock_panel(tile_id),
            UIEvent::RequestActivateTab { tile_id } => self.handle_request_activate_tab(tile_id),
        };
        
        // After processing, add status diagnostics
        if let Ok(()) = result {
            if matches!(event, UIEvent::DockPanel { .. } | UIEvent::DockPanelToTarget { .. }) {
                println!("[DOCK DEBUG] Dock successful! Tree structure AFTER docking:");
                self.print_tree_diagnostic();
            }
        }
        
        result
    }

    fn handle_close_from_tree(&mut self, tile_id_to_close: TileId) -> Result<(), LayoutError> {
        println!("[INFO] Attempting to close tile {:?} from tree.", tile_id_to_close);

        // 1. Find the tile, verify it's a Pane, AND check permanence
        let panel_to_move = match self.tree.tiles.get(tile_id_to_close) {
            Some(Tile::Pane(panel)) => {
                if panel.is_permanent {
                    return Err(format!("Cannot close permanent panel '{}' (Tile ID {:?}).", panel.title, tile_id_to_close));
                }
                panel.clone() // Clone only if not permanent
            },
            Some(_) => return Err(format!("Tile {:?} is not a Pane.", tile_id_to_close)),
            None => return Err(format!("Tile {:?} not found.", tile_id_to_close)),
        };

        // 2. Find the parent ID
        let parent_id = self.find_parent_of(tile_id_to_close).ok_or_else(|| 
            format!("Could not find parent for tile {:?}.", tile_id_to_close)
        )?;

        // 3. Remove the tile ID from the parent container's children
        if let Some(Tile::Container(parent_container)) = self.tree.tiles.get_mut(parent_id) {
            parent_container.remove_child(tile_id_to_close);
            println!("[INFO] Removed child {:?} from parent container {:?}", tile_id_to_close, parent_id);
        } else {
             return Err(format!("Parent tile {:?} is not a container or not found.", parent_id));
        }

        // 4. Remove the tile itself from the main tiles map
        if self.tree.tiles.remove(tile_id_to_close).is_none() {
            eprintln!("[WARN] Tile {:?} was already removed from map when trying to close.", tile_id_to_close);
        }

        // 5. Create floating state (mark as closed)
        let panel_id = panel_to_move.id;
        let panel_title = panel_to_move.title.clone();
        println!(
            "[INFO] Closed panel '{}' (ID {}) from tree (Tile ID {:?})",
            panel_title, panel_id, tile_id_to_close
        );
        let new_floating_state = FloatingPanelState::new(panel_to_move, false, None); // is_open: false
        self.floating_panels.insert(panel_id, new_floating_state);
        println!("[INFO] Added panel ID {} to floating_panels (closed).", panel_id);

        // 6. Simplification should happen implicitly when needed by the tree.ui call.
        // We might still want to simplify the parent explicitly after removal
        println!("[INFO] Simplifying parent container {:?} after child removal.", parent_id);
        self.tree.simplify_children_of_tile(parent_id, &self.behavior.simplification_options());

        Ok(())
    }

    fn handle_close_floating(&mut self, panel_id: u32) -> Result<(), LayoutError> {
        // Logic: Find panel in floating_panels, set is_open=false
        // Remember to handle potential errors (e.g., panel not found)
        if let Some(state) = self.floating_panels.get_mut(&panel_id) {
             if state.is_open { // Only act if it was open
                 state.is_open = false;
                 println!("[INFO] Marked floating panel {} as closed.", panel_id);
                 Ok(())
             } else {
                 // Already closed, not an error, but maybe log as debug/warn
                 println!("[DEBUG] Panel {} was already closed.", panel_id);
                 Ok(())
             }
         } else {
             Err(format!("Panel with ID {} not found in floating_panels.", panel_id))
         }
    }

    fn handle_reopen_panel(&mut self, panel_id: u32) -> Result<(), LayoutError> {
        // First, check if panel exists and is closed, and get the info we need
        let needs_rect_and_reopen = if let Some(state) = self.floating_panels.get(&panel_id) {
            if !state.is_open {
                // Panel exists, is closed, and needs rect if it doesn't have one
                let needs_rect = state.current_rect.is_none();
                let title = state.panel.title.clone();
                Some((needs_rect, title))
            } else {
                // Panel is already open - just note and return
                println!("[WARN] ReopenPanel event for already open panel ID {}. Ignoring.", panel_id);
                return Ok(());
            }
        } else {
            // Panel not found - return error
            eprintln!("[ERROR] ReopenPanel event for unknown panel ID {}. Not found in floating_panels.", panel_id);
            return Err(format!("Panel ID {} not found for reopening.", panel_id));
        };

        // Now handle the case where we need to reopen the panel
        if let Some((needs_rect, title)) = needs_rect_and_reopen {
            // Calculate the rect if needed
            let rect = if needs_rect {
                Some(self.get_default_rect(&title))
            } else {
                None
            };

            // Now update the panel state
            if let Some(state) = self.floating_panels.get_mut(&panel_id) {
                state.is_open = true;
                
                // Apply the rect if we calculated one
                if let Some(default_rect) = rect {
                    state.current_rect = Some(default_rect);
                }
                
                println!("[INFO] Reopened panel '{}' (ID {}) as floating window.", title, panel_id);
            }
            
            Ok(())
        } else {
            // This shouldn't happen if the code above is correct
            Err("Unexpected error reopening panel".to_string())
        }
    }

    fn handle_dock_panel(&mut self, panel_id: u32) -> Result<(), LayoutError> {
        println!("[DEBUG] Handling dock panel for panel_id: {}", panel_id);
        
        // Check if the panel exists in floating_panels
        if !self.floating_panels.contains_key(&panel_id) {
            return Err(format!("Panel ID {} not found in floating_panels.", panel_id));
        }
        
        // Remove panel from floating panels and capture it
        let floating_state = self.floating_panels.remove(&panel_id).unwrap();
        let panel_to_dock = floating_state.panel;
        let panel_title = panel_to_dock.title.clone();
        println!("[DEBUG] Removed panel '{}' (ID {}) from floating panels for docking.", 
                 panel_title, panel_id);
        
        // Find a target container for docking
        let target_container_id = if self.tree.is_empty() {
            println!("[DEBUG] Tree is empty, creating a simple layout with just the panel being docked");
            
            // Create a simple layout with just this panel
            let (root_id, _) = generate_tile_id_and_u64();
            let root_container = Container::new_vertical(Vec::new());
            self.tree.tiles.insert(root_id, Tile::Container(root_container));
            
            // Create a tabs container for the panel
            let (tabs_id, _) = generate_tile_id_and_u64();
            let tabs_container = Container::Tabs(Tabs::default());
            self.tree.tiles.insert(tabs_id, Tile::Container(tabs_container));
            
            // Add the tabs container to the root
            if let Some(Tile::Container(container)) = self.tree.tiles.get_mut(root_id) {
                container.add_child(tabs_id);
            }
            
            // Set the root
            self.tree.root = Some(root_id);
            
            tabs_id
        } else {
            // Find an existing container to dock into
            self.find_dock_target()?
        };
        
        // Create a new pane tile for the panel
        let (new_pane_id, _) = generate_tile_id_and_u64();
        let new_pane_tile = Tile::Pane(panel_to_dock);
        
        // Insert the new pane into the tree BEFORE we borrow the container
        self.tree.tiles.insert(new_pane_id, new_pane_tile);
        
        // Now get the container and add the pane to it
        let mut success = false;
        if let Some(Tile::Container(Container::Tabs(tabs))) = self.tree.tiles.get_mut(target_container_id) {
            tabs.add_child(new_pane_id);
            tabs.active = Some(new_pane_id); // Activate the newly added tab
            success = true;
            println!("[DEBUG] Added panel '{}' to tabs container {:?} and activated it", 
                    panel_title, target_container_id);
        }
        
        // Handle failure case - recover the panel if needed
        if !success {
            println!("[ERROR] Failed to add panel to container {:?}", target_container_id);
            
            // Try to recover the panel
            if let Some(Tile::Pane(panel)) = self.tree.tiles.remove(new_pane_id) {
                let recovered_state = FloatingPanelState::new(
                    panel, 
                    true, 
                    floating_state.current_rect.or_else(|| Some(self.get_default_rect(&panel_title)))
                );
                self.floating_panels.insert(panel_id, recovered_state);
                return Err(format!("Failed to add panel to container {:?}", target_container_id));
            } else {
                return Err("Critical error: Failed to recover panel during failed docking".to_string());
            }
        }
        
        Ok(())
    }

    // Simplified helper method to find a target container for docking
    fn find_dock_target(&self) -> Result<TileId, String> {
        // Try to find the first tabs container available
        for (id, tile) in self.tree.tiles.iter() {
            if let Tile::Container(Container::Tabs(_)) = tile {
                println!("[DEBUG] Found Tabs container {:?} as dock target.", id);
                return Ok(*id); // Return the ID of the first Tabs container found
            }
        }
        
        // If no Tabs container is found, return an error
        println!("[WARN] No Tabs container found for docking.");
        Err("No Tabs container found for docking".to_string())
    }

    fn handle_dock_panel_to_target(
        &mut self, 
        panel_id: u32, 
        _target_id_opt: Option<TileId>, // Prefix unused parameter
        _position: Option<egui::Pos2>   // Prefix unused parameter
    ) -> Result<(), LayoutError> {
        println!("[TARGET DOCK DEBUG] Handling DockPanelToTarget for panel ID: {}", panel_id);
        
        // --- Pre-modification Check ---
        println!("[TARGET DOCK DEBUG] Tree structure BEFORE any modification:");
        self.print_tree_diagnostic();
        // --- End Check ---

        // 1. Verify panel exists
        if !self.floating_panels.contains_key(&panel_id) {
            return Err(format!("Panel ID {} not found for docking.", panel_id));
        }
        
        // 2. Determine target container (Simplified: Ignore target_id_opt and position for now, use default)
        println!("[TARGET DOCK DEBUG] Ignoring position/target_id, finding default tabs container.");
        let target_id = self.find_main_tabs_container_id().ok_or_else(|| 
            "No default tabs container found.".to_string()
        )?;
        println!("[TARGET DOCK DEBUG] Determined target container: {:?}", target_id);
        
        // Ensure target is actually a Tabs container before proceeding
        if !matches!(self.tree.tiles.get(target_id), Some(Tile::Container(Container::Tabs(_)))) {
             return Err(format!("Target {:?} is not a Tabs container.", target_id));
        }

        // 3. Now safe to remove the panel
        let floating_state = self.floating_panels.remove(&panel_id).unwrap(); 
        let panel_to_dock = floating_state.panel;
        let panel_title = panel_to_dock.title.clone();
        println!("[TARGET DOCK DEBUG] Removed '{}' (ID {}) from floating for docking to {:?}.", 
                 panel_title, panel_id, target_id);

        // 4. Create new Tile::Pane and insert it
        let new_pane_tile_id = generate_tile_id_and_u64().0;
        let new_pane_tile = Tile::Pane(panel_to_dock);
        println!("[TARGET DOCK DEBUG] Inserting new pane tile {:?} into tree.tiles map.", new_pane_tile_id);
        self.tree.tiles.insert(new_pane_tile_id, new_pane_tile);
        
        // --- Post-Insert Check ---
        println!("[TARGET DOCK DEBUG] Tree structure AFTER inserting new pane tile:");
        self.print_tree_diagnostic();
        // --- End Check ---

        // 5. Add to tabs container
        let mut success = false;
        println!("[TARGET DOCK DEBUG] Attempting to add child {:?} to tabs container {:?}", new_pane_tile_id, target_id);
        if let Some(Tile::Container(Container::Tabs(tabs))) = self.tree.tiles.get_mut(target_id) {
            tabs.add_child(new_pane_tile_id);
            tabs.active = Some(new_pane_tile_id);
            println!("[TARGET DOCK DEBUG] Successfully added child and set active.");
            success = true;
        } else {
             println!("[TARGET DOCK DEBUG] Failed to get mutable access to tabs container {:?}", target_id);
        }
        
        // --- Post-AddChild Check ---
        println!("[TARGET DOCK DEBUG] Tree structure AFTER attempting add_child:");
        self.print_tree_diagnostic();
        // --- End Check ---

        // 6. Handle failure and recovery if needed
        if !success {
            println!("[TARGET DOCK DEBUG] Docking failed. Attempting recovery...");
            // Try to recover the panel
            if let Some(removed_tile) = self.tree.tiles.remove(new_pane_tile_id) {
                if let Tile::Pane(recovered_panel) = removed_tile {
                    let rect = floating_state.current_rect.or_else(|| 
                        Some(self.get_default_rect(&panel_title)));
                    let recovered_state = FloatingPanelState::new(recovered_panel, true, rect);
                    self.floating_panels.insert(panel_id, recovered_state);
                    println!("[WARN] Recovered panel during failed targeted docking: {}", panel_id);
                    return Err("Failed to add panel to specified tabs container.".to_string());
                }
            }
            eprintln!("[ERROR] Critical failure - Panel lost during targeted docking: {}", panel_id);
            return Err("Critical failure - Panel lost during docking operation.".to_string());
        }

        println!("[TARGET DOCK DEBUG] Docking to target seems successful.");
        Ok(())
    }
    
    // Helper to find the nearest tabs container to a given position
    fn find_nearest_tabs_container(&self, position: egui::Pos2) -> Option<TileId> {
        let tab_containers = self.find_all_tabs_containers();
        if tab_containers.is_empty() {
            return None;
        }
        
        // Get all containers with their screen rects
        let mut container_distances = Vec::new();
        
        for container_id in &tab_containers {
            // Simple distance calculation (can be improved with actual UI rect)
            // For now, just use the first child's position as a proxy for container position
            if let Some(Tile::Container(Container::Tabs(tabs))) = self.tree.tiles.get(*container_id) {
                if let Some(_first_child_id) = tabs.children.first() {
                    // Use an arbitrary point (center) for distance calculation
                    // In a real implementation, you'd use the container's actual UI rect
                    let arbitrary_center = egui::Pos2::new(400.0, 300.0);
                    let distance = arbitrary_center.distance(position);
                    container_distances.push((*container_id, distance));
                }
            }
        }
        
        // Find the container with minimum distance
        container_distances.into_iter()
            .min_by(|(_, dist1), (_, dist2)| 
                dist1.partial_cmp(dist2).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(id, _)| id)
            .or_else(|| tab_containers.first().copied()) // Fallback to first if distance calc fails
    }

    // --- Invariant Check (Placeholder) ---
    fn validate_invariants(&self) {
        // Run these checks in both debug and release builds for now
        println!("[DEBUG] --- Validating Invariants ---");

        // Check 1: Panel ID uniqueness (exists in tree OR floating, not both)
        let mut tree_panel_ids = std::collections::HashSet::new();
        for tile in self.tree.tiles.iter() {
            if let Tile::Pane(panel) = tile.1 {
                tree_panel_ids.insert(panel.id);
            }
        }

        for (floating_id, _) in self.floating_panels.iter() {
            if tree_panel_ids.contains(floating_id) {
                eprintln!("[INVARIANT FAIL] Panel ID {} exists in BOTH tree and floating_panels!", floating_id);
            }
        }

        // Check 2: Tree structure print and tile existence check
        println!("[DEBUG] Tree Structure:");
        fn print_tree(tiles: &Tiles<MockPanel>, id: TileId, depth: usize) {
            let indent = "  ".repeat(depth);
            if let Some(tile) = tiles.get(id) {
                match tile {
                    Tile::Pane(pane) => {
                        println!("{}Pane #{:?} '{}' (ID: {})", indent, id, pane.title, pane.id);
                    },
                    Tile::Container(container) => {
                        println!(
                            "{}Container #{:?} {:?} with {} children: {:?}",
                            indent,
                            id,
                            container.kind(),
                            container.children().count(),
                            container.children().collect::<Vec<_>>()
                        );
                        for child_id in container.children() {
                            print_tree(tiles, *child_id, depth + 1);
                        }
                    }
                }
            } else {
                 eprintln!("{}[INVARIANT FAIL] Tile ID #{:?} referenced in tree structure but not found in tiles map!", indent, id);
            }
        }

        if let Some(root) = self.tree.root() {
            print_tree(&self.tree.tiles, root, 0);
        } else {
            println!("[DEBUG] Tree is empty.");
        }
         println!("[DEBUG] --- Invariants Validated ---");
    }

    // Helper method to print tree diagnostics
    fn print_tree_diagnostic(&self) {
        if let Some(root) = self.tree.root() {
            self.print_tree_node(root, 0);
        } else {
            println!("[DIAGNOSTIC] Tree is empty!");
        }
    }
    
    fn print_tree_node(&self, node_id: TileId, depth: usize) {
        let indent = "  ".repeat(depth);
        if let Some(tile) = self.tree.tiles.get(node_id) {
            match tile {
                Tile::Pane(pane) => {
                    println!("{}Pane {:?}: '{}' (ID: {})", indent, node_id, pane.title, pane.id);
                },
                Tile::Container(container) => {
                    let kind = match container {
                        Container::Tabs(_) => "TABS",
                        container => match container.kind() {
                            egui_tiles::ContainerKind::Horizontal => "HORIZONTAL",
                            egui_tiles::ContainerKind::Vertical => "VERTICAL",
                            _ => "OTHER",
                        }
                    };
                    println!("{}Container {:?}: {} with {} children:", 
                        indent, node_id, kind, container.children().count());
                    
                    // Print each child
                    for child_id in container.children() {
                        self.print_tree_node(*child_id, depth + 1);
                    }
                }
            }
        } else {
            println!("{}ERROR: Node {:?} not found in tree!", indent, node_id);
        }
    }

    fn handle_undock_panel(&mut self, tile_id_to_undock: TileId) -> Result<(), LayoutError> {
        println!("[INFO] Attempting to undock tile {:?} from tree.", tile_id_to_undock);

        // 1. Find the tile, verify it's a Pane, AND check permanence
        let panel_to_move = match self.tree.tiles.get(tile_id_to_undock) {
            Some(Tile::Pane(panel)) => {
                if panel.is_permanent {
                    return Err(format!("Cannot undock permanent panel '{}' (Tile ID {:?}).", panel.title, tile_id_to_undock));
                }
                panel.clone() // Clone only if not permanent
            },
            Some(_) => return Err(format!("Tile {:?} is not a Pane.", tile_id_to_undock)),
            None => return Err(format!("Tile {:?} not found for undocking.", tile_id_to_undock)),
        };

        // 2. Find the parent ID using our helper method
        let parent_id = self.find_parent_of(tile_id_to_undock).ok_or_else(|| {
            if self.tree.root() == Some(tile_id_to_undock) {
                 format!("Cannot undock the root tile {:?}.", tile_id_to_undock)
            } else {
                 format!("Could not find parent for tile {:?}. Tree state might be inconsistent.", tile_id_to_undock)
            }
        })?;

        // 3. Remove the tile ID from the parent container's children
        if let Some(Tile::Container(parent_container)) = self.tree.tiles.get_mut(parent_id) {
            parent_container.remove_child(tile_id_to_undock);
            println!("[INFO] Removed child {:?} from parent container {:?}", tile_id_to_undock, parent_id);
        } else {
             return Err(format!("Parent tile {:?} is not a container or not found.", parent_id));
        }

        // 4. Remove the tile itself from the main tiles map
        if self.tree.tiles.remove(tile_id_to_undock).is_none() {
            eprintln!("[WARN] Tile {:?} was already removed from map when trying to undock.", tile_id_to_undock);
        }

        // 5. Create floating state - MARK AS OPEN
        let panel_id = panel_to_move.id;
        let panel_title = panel_to_move.title.clone();
        println!(
            "[INFO] Undocked panel '{}' (ID {}) from tree (Tile ID {:?})",
            panel_title, panel_id, tile_id_to_undock
        );
        let rect = Some(self.get_default_rect(&panel_title));
        let new_floating_state = FloatingPanelState::new(panel_to_move, true, rect); // is_open: true
        self.floating_panels.insert(panel_id, new_floating_state);
        println!("[INFO] Added panel ID {} to floating_panels (open).", panel_id);

        // 6. Explicitly simplify the parent now that a child is removed.
        println!("[INFO] Simplifying parent container {:?} after child removal.", parent_id);
        self.tree.simplify_children_of_tile(parent_id, &self.behavior.simplification_options());

        Ok(())
    }

    fn handle_request_activate_tab(&mut self, tile_id_to_activate: TileId) -> Result<(), LayoutError> {
        println!("[INFO] Requesting to activate tab for tile: {:?}", tile_id_to_activate);

        // 1. Find the parent Tabs container ID using our helper method
        let parent_tabs_id = self.find_parent_of(tile_id_to_activate)
            .ok_or_else(|| format!("Could not find parent for tile {:?}.", tile_id_to_activate))?;

        // 2. Get mutable access to the parent Tabs container and set its active child
        if let Some(Tile::Container(Container::Tabs(tabs))) = self.tree.tiles.get_mut(parent_tabs_id) {
            if !tabs.children.contains(&tile_id_to_activate) {
                return Err(format!(
                    "Tile {:?} is not a child of its supposed parent Tabs container {:?}.",
                    tile_id_to_activate,
                    parent_tabs_id
                ));
            }

            println!("[INFO] Activating tile {:?} in Tabs container {:?}", tile_id_to_activate, parent_tabs_id);
            tabs.active = Some(tile_id_to_activate);
            Ok(())
        } else {
            Err(format!("Parent tile {:?} is not a Tabs container or not found.", parent_tabs_id))
        }
    }
}

// Add some constants for lorem ipsum text
const LOREM_IPSUM_SHORT: &str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.";
const LOREM_IPSUM_LONG: &str = "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.";

// --- Main Function ---

fn main() -> Result<(), eframe::Error> {
    println!("[INFO] App starting...");
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1024.0, 768.0]) // Default size
            .with_min_inner_size([600.0, 400.0]), // Minimum size
        ..Default::default()
    };
    eframe::run_native(
        "Egui Tiles Prototype", // Window title
        options,
        Box::new(|_cc| {
            // Creation context could be used for setup if needed (e.g., loading assets)
            println!("[INFO] Creating App instance...");
            Ok(Box::<App>::default()) // Wrap in Ok
        }),
    )
} 