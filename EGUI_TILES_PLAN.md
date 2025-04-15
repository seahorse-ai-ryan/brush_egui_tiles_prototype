# Egui Tiles + Floating Window Prototype Plan

## 1. Preamble & Context

**Goal:** Validate the core mechanics of a hybrid UI combining docked panels (managed by `egui_tiles`) and free-floating panels (`egui::Window`), potentially for integration into the Brush application. This includes panel transitions between docked and floating states via button interactions.

**Background:** This plan guided the development of an `egui_tiles`-based prototype, replacing a previous `egui_dock` attempt that faced instability. Lessons from the `egui_dock` prototype informed this approach.

**Outcome:** The prototype successfully implemented core docking/undocking/closing/reopening via button/menu interactions. Drag-and-drop interactions (tab reordering, drag-to-dock) remain non-functional or disabled due to complexity/limitations in `egui_tiles` v0.12.

## 2. Terminology Clarification

*   **Panel:** Our `MockPanel` data struct (Settings, Scene, Stats...). Holds the application state for a specific view.
*   **Pane:** A `Tile::Pane` holding a `MockPanel`. This is the actual content unit within the layout tree. What you see inside a tab or simple docked area.
*   **Tile:** An ID (`TileId`) referring to an entry in the `tree.tiles` map, which can be either a `Pane` or a `Container`.
*   **Container:** A `Tile::Container` holding other `TileId`s (e.g., `Tabs`, `Horizontal`, `Vertical`). Defines how children are laid out.
*   **Tabs:** A specific type of `Container` that shows its children as clickable tabs.
*   **Tab:** The clickable UI element (rendered by `Behavior::tab_ui`) representing a `Pane` when it's inside a `Tabs` container.
*   **Window:** The floating `egui::Window` holding a `Panel` when it is *not* part of the main layout `Tree`.

## 3. Target Capabilities & User Experience (As Implemented)

The prototype allows users to:
*   View multiple panels simultaneously in a configurable docked layout based on the initial setup.
*   Have specific panels displayed in separate, floating windows.
*   Move panels between the docked layout and a floating state using explicit UI buttons.
*   Close panels (docked or floating), making them available to reopen later.
*   Reopen closed panels from a menu ("View" -> "Reopen Closed Panel").
*   (Future Goal) Persist layout state.
*   (Future Goal) More advanced layout manipulation (splitting, grid layouts).

**Initial Layout (Brush Parity):**
*   **Docked:** A main window split horizontally into three columns:
    *   Left Column (Vertical Split): "Settings"/"Presets"/"Properties" (tabbed) on top, "Stats" (tabbed) below.
    *   Center Column: "Scene" (tabbed).
    *   Right Column: "Dataset" (tabbed).
*   **Floating:** Starts with *no* floating panels initially.

**Interaction Model (Implemented):**
*   **Docking:** Click "Dock Me" button OR click "📌" (Pin) icon inside floating panel content area. Docks to the *first available* `Tabs` container found in the tree. If the tree is empty, a new simple layout is created.
*   **Undocking:** Click "⏏" (Eject) icon on a docked tab's title bar. Panel becomes a floating window at a default position.
*   **Closing Docked:** Click "X" in the top-right of the tab area (closes active tab). Panel becomes "closed" and available in the "Reopen" menu.
*   **Closing Floating:** Click standard window "X". Panel becomes "closed" and available in the "Reopen" menu.
*   **Reopening:** Use "View" -> "Reopen Closed Panel" menu. Reopens the panel as a floating window.
*   **Tab Activation:** Clicking a tab makes its content visible.
*   **Tab Reordering:** Dragging tabs is sensed but does **not** visually reorder them (v0.12 limitation/requires manual implementation). Feature is paused.
*   **Drag-to-Dock/Undock:** Not implemented/disabled.

## 4. Lessons Learned from `egui_dock` & `egui_tiles` Development

*   **Initialization Sensitivity:** Complex layouts created during the first frame can still cause issues. Starting with a defined layout and managing state carefully is crucial.
*   **Deferred State Changes (Event Queue):** Using an event queue (`event_queue` and `process_ui_event`) processed *after* the main UI pass is essential for stability when modifying the tree or floating panel state in response to UI interactions.
*   **Floating `egui::Window` vs. Tiling Library:** Manually managed `egui::Window`s exist outside `egui_tiles` state management. Achieving seamless drag-and-drop between them is non-trivial. Button-based interaction is a reliable alternative.
*   **State Management:** Maintaining consistent state between the `tree` and `floating_panels` (ensuring a panel exists in only one place) is critical. Helper functions (`find_parent_of`) and careful event handling logic are needed. Recovering panels on failed operations (like docking) prevents state loss.
*   **API Discrepancies/Compiler Errors:** `egui_tiles` trait signatures (`Behavior::top_bar_right_ui`) must be matched precisely. Relying on the compiler and iteratively fixing errors is necessary.
*   **Isolation & Incremental Testing:** Commenting out features and using detailed logging (`println!`) were vital for debugging.
*   **Simplification (`egui_tiles`):** The `SimplificationOptions` (especially `prune_empty_tabs` and `prune_empty_containers`) are powerful but require understanding. They automate cleanup when containers become empty. Disabling `prune_single_child_*` might be necessary for certain initial layouts or behaviors.
*   **`egui_tiles` v0.12 Drag/Drop:** Customizing drag interactions beyond basic tab-bar dragging appears limited. Visual tab reordering requires manual implementation in `Behavior::on_tab_drop`.

## 5. Debugging Findings & Root Cause Analysis (Final)

*   **Compilation Errors (Resolved):** Multiple errors related to trait signatures (`Behavior`), pattern matching (`Container::Tabs`), missing types (`TreeNodeAction`), and incorrect field access (`Tabs::selected`) were fixed iteratively.
*   **Simplification Issue (Resolved):** The `prune_empty_tabs` and `prune_empty_containers` options work as expected.
*   **State Corruption (Resolved):** Initial crashes during docking/closing were likely due to attempting state modification directly in UI logic or incorrect handling of tile ownership/removal. The event queue pattern resolved this.
*   **Tab Reordering (Not Functional):** Dragging tabs is sensed by the UI, but the default `egui_tiles` drop handling doesn't reorder them visually. Requires manual implementation.
*   **Floating Window Resizing (Mostly Stable):** Seems okay, using `ScrollArea` helps.

## 6. Refined Implementation Strategy

(This section is now obsolete as the implementation is complete based on the iterative process.)

## 7. Alternative UX/Layout Considerations

(Keep for future reference if button-based UX proves insufficient.)
*   Implement manual tab reordering logic in `Behavior::on_tab_drop`.
*   Explore newer versions of `egui_tiles` for improved drag/drop handling.
*   Consider custom drag-and-drop implementations outside of `egui_tiles` default behavior if needed.

## 8. Current Status & Next Steps (Final)

**Current Functionality:**
*   Initial layout matches target Brush layout.
*   Docking via "Dock Me" / "📌" buttons works (docks to first available `Tabs` container or creates new layout if empty).
*   Undocking via '⏏' button on tabs works.
*   Closing docked panels ('X' on tab bar) works.
*   Closing floating panels (window 'X') works.
*   Reopening closed panels via View menu works (reopens as floating).
*   Tab activation on click works.
*   Resizing main splits works.
*   Floating window resizing seems stable.
*   Empty `Tabs` and other containers are correctly pruned when last child is removed.
*   Dock buttons are correctly located at the top of floating window content.
*   The concept of a persistent "primary container" was removed, simplifying logic.

**Known Issues / To Implement:**
*   **Tab Reordering (Non-Functional):** Dragging docked tabs does not visually reorder them. (Paused)
*   **Drag-to-Dock/Undock (Not Implemented):** Feature disabled.
*   **Position-Aware Docking (Not Implemented):** Docking always targets the first found `Tabs` container.
*   **Floating Window Minimize:** Minimize button is missing (using default window title bar).
*   **Visual Terminology Helper:** Could be improved.
*   Tab activation doesn't visually bring the window to the front (standard egui behavior).

**Future Goals / Next Steps:**
1.  **Polish UI:**
    *   Refine the visual Terminology Helper window.
    *   Decide if manual minimize button for floating windows is needed.
2.  **Revisit Tab Reordering:**
    *   Accept current limitation OR
    *   Attempt manual implementation in `on_tab_drop` OR
    *   Investigate newer `egui_tiles` versions.
3.  **Advanced Features:**
    *   Implement position-aware docking (e.g., docking to nearest container or splitting).
    *   Implement layout persistence (saving/loading).

## 9. Open Questions / Considerations (`egui_tiles`)

*   **Internal Drag/Reorder Logic (v0.12):** Why doesn't default drop handling reorder tabs visually? (Likely requires manual index manipulation in `on_tab_drop`).
*   **Simplification Interaction:** How best to define initial layouts with single-child `Tabs` containers if needed? (Current approach works by letting simplification run).
*   **Layout Persistence:** Best practices for serializing/deserializing `egui_tiles::Tree` state? (Requires defining custom serializable structures).
*   Explore `egui_tiles` examples/issues for advanced `Behavior` implementations if revisiting drag/drop.

## 10. Code Snippets / Implementation Details

*   **Event Queue System:** Implemented using `Rc<RefCell<Vec<UIEvent>>>` and processed after UI rendering in `App::update`. Event handlers return `Result<(), LayoutError>`.
*   **Error Handling:** Uses `Result<(), LayoutError>` for event handlers. Critical operations like docking include recovery logic to reinstate panels into `floating_panels` on failure.
*   **Current `Behavior` Implementation:**
    ```rust
    impl Behavior<MockPanel> for MyBehavior {
        // tab_ui: Renders selectable label + '⏏' undock button. Drag sense likely enabled but drop doesn't reorder.
        // tab_title_for_pane: Returns pane title.
        // pane_ui: Calls panel.ui() with unique content & scroll area.
        // top_bar_right_ui: Renders 'X' button, queues ClosePanelFromTree for active tab. Returns () as required by trait.
        // simplification_options: Returns custom options (prune_empty_tabs: true, prune_empty_containers: true).
        // // Other drag methods likely removed or minimal //
    }
    ```
*   **Key State Handling Functions:** (All implemented and working)
    *   `handle_dock_panel`
    *   `handle_undock_panel`
    *   `handle_close_from_tree`
    *   `handle_close_floating`
    *   `handle_reopen_panel`
    *   `handle_request_activate_tab`
    *   `find_parent_of` (helper)
    *   `find_dock_target` (simplified helper)
*   **Removed Complexity:** `TreeNodeAction` enum, `primary_container_id`, complex/unused docking functions (`handle_dock_panel_to_target`, etc.).

</rewritten_file> 