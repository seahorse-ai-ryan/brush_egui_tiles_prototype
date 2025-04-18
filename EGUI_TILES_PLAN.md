# Egui Tiles + Floating Window Prototype Plan

## 1. Preamble & Context

**Goal:** Validate the core mechanics of a hybrid UI combining docked panels (managed by `egui_tiles`) and free-floating panels (`egui::Window`), potentially for integration into the Brush application. This includes panel transitions between docked and floating states via button interactions.

**Background:** This plan guided the development of an `egui_tiles`-based prototype, replacing a previous `egui_dock` attempt that faced instability. Lessons from the `egui_dock` prototype informed this approach.

**Outcome:** The prototype successfully implemented core docking/undocking/closing/reopening via button/menu interactions. Drag-and-drop tab reordering works when using default implementations. More complex interactions (drag-to-dock) remain non-functional or disabled due to complexity.

**Recent Progress:** We have successfully implemented a clean, stable version by starting fresh from Brush's app.rs structure, focusing only on the tab dragging functionality while deferring floating windows and docking/undocking features for future implementation.

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
*   Reorder tabs within a `Tabs` container by dragging them.
*   Split existing containers by dragging tabs to the edges.
*   (Future Goal) Persist layout state.
*   (Future Goal) More advanced layout manipulation (grid layouts).

**Initial Layout (Brush Parity):**
*   **Docked:** A main window split horizontally into three columns:
    *   Left Column (Vertical Split): "Settings"/"Presets"/"Properties" (tabbed) on top, "Stats" (tabbed) below.
    *   Center Column: "Scene" (tabbed).
    *   Right Column: "Dataset" (tabbed).
*   **Floating:** Starts with *no* floating panels initially.

**Interaction Model (Implemented in Previous Prototype / Planned for Re-implementation):**
*   **Docking:** Click "Dock Me" button OR click "📌" (Pin) icon inside floating panel content area. Docks to the *first available* `Tabs` container found in the tree. If the tree is empty, a new simple layout is created.
*   **Undocking:** Click "⏏" (Eject) icon on a docked tab's title bar. Panel becomes a floating window at a default position.
*   **Closing Docked:** Click "X" in the top-right of the tab area (closes active tab). Panel becomes "closed" and available in the "Reopen" menu.
*   **Closing Floating:** Click standard window "X". Panel becomes "closed" and available in the "Reopen" menu.
*   **Reopening:** Use "View" -> "Reopen Closed Panel" menu. Reopens the panel as a floating window.
*   **Tab Activation:** Clicking a tab makes its content visible.
*   **Tab Reordering:** Dragging tabs works with the default `Behavior` implementation.
*   **Drag-to-Split:** Dragging tabs to container edges splits the container (works).
*   **Drag-to-Dock/Undock:** Not implemented/disabled.

## 4. Lessons Learned from `egui_dock`, `egui_tiles` Development & Fresh Brush-Based Approach

*   **Initialization Sensitivity:** Complex layouts created during the first frame can still cause issues. Starting with a defined layout and managing state carefully is crucial.
*   **Deferred State Changes (Event Queue):** Using an event queue (`event_queue` and `process_ui_event`) processed *after* the main UI pass is essential for stability when modifying the tree or floating panel state in response to UI interactions.
*   **Floating `egui::Window` vs. Tiling Library:** Manually managed `egui::Window`s exist outside `egui_tiles` state management. Achieving seamless drag-and-drop between them is non-trivial. Button-based interaction is a reliable alternative.
*   **State Management:** Maintaining consistent state between the `tree` and `floating_panels` (ensuring a panel exists in only one place) is critical. Helper functions (`find_parent_of`) and careful event handling logic are needed. Recovering panels on failed operations (like docking) prevents state loss.
*   **API Discrepancies/Compiler Errors:** `egui_tiles` trait signatures (`Behavior::top_bar_right_ui`) must be matched precisely. Relying on the compiler and iteratively fixing errors is necessary.
*   **Isolation & Incremental Testing:** Commenting out features and using detailed logging (`println!`) were vital for debugging.
*   **Simplification (`egui_tiles`):** The `SimplificationOptions` (especially `prune_empty_tabs` and `prune_empty_containers`) are powerful but require understanding. They automate cleanup when containers become empty. Disabling `prune_single_child_*` might be necessary for certain initial layouts or behaviors.
*   **`egui_tiles` v0.12 Drag/Drop:** Standard tab reordering and drag-to-split work well with the default `Behavior`. Custom drag interactions (like drag-to-dock) require significant manual implementation.
*   **Fresh Approach Success:** Starting from scratch with a clean implementation based on Brush's app.rs structure proved highly successful. By focusing on core tab-dragging functionality first before adding floating windows and docking/undocking:
    * The UI was immediately stable and responsive
    * Tab dragging behavior matched Brush exactly
    * The clean separation made debugging easier
    * Simplified implementation reduced complexity
    * Better organized code that closely follows Brush's architecture

## 5. Debugging Findings & Root Cause Analysis (Final)

*   **Compilation Errors (Resolved):** Multiple errors related to trait signatures (`Behavior`), pattern matching (`Container::Tabs`), missing types (`TreeNodeAction`), and incorrect field access (`Tabs::selected`) were fixed iteratively.
*   **Simplification Issue (Resolved):** The `prune_empty_tabs` and `prune_empty_containers` options work as expected.
*   **State Corruption (Resolved):** Initial crashes during docking/closing were likely due to attempting state modification directly in UI logic or incorrect handling of tile ownership/removal. The event queue pattern resolved this.
*   **Tab Reordering (Works):** Standard tab reordering functions correctly with the default `Behavior`. Previous issues were caused by custom overrides.
*   **Floating Window Resizing (Mostly Stable):** Seems okay, using `ScrollArea` helps.
*   **Button Glitches (Identified):** Placing Dock/Undock buttons using `egui::Area` inside panel content causes layering and resize issues (buttons appearing through windows, detaching on resize). **Resolution:** Move buttons to `Behavior::tab_ui` override.
*   **Web Build Setup (Completed):** Successfully configured the project for web deployment using `trunk`.
    *   Requires an `index.html` file at the project root.
    *   `Cargo.toml` needs web-specific dependencies: `wasm-bindgen`, `wasm-bindgen-futures`, `log`, `web-sys` (with features like `Document`, `Window`, `Element`, `HtmlCanvasElement`).
    *   Requires conditional compilation (`#[cfg(...)]`) in `src/app.rs` for separate `main` functions (native vs. web).
    *   The web `main` function needs to use `web-sys` to get the HTML canvas element by ID and pass it to `eframe::WebRunner::start`.
    *   `trunk build` is useful for checking compilation errors without starting the server.
    *   `trunk serve` runs the development server with hot-reloading.
*   Clean Implementation Approach: Starting fresh from Brush's app.rs structure yielded a much more stable foundation by focusing on the core egui_tiles functionality first, with future plans to carefully add floating window features.

## 6. Refined Implementation Strategy

*   **Stage 1 (Completed):** Implement clean Brush-like UI with tabs and stable dragging behavior.
*   **Stage 2 (Next):** Carefully add floating window and docking/undocking capabilities, following the detailed plan in Section 12, prioritizing the use of `PanelId` enum and `Behavior::tab_ui` for buttons.
*   **Stage 3 (Future):** Implement more advanced features like layout persistence and context-aware docking restoration.

**External Feedback:** Additional feedback and analysis were provided by external AI agents. See [04-mini_feedback.md](./04-mini_feedback.md) for details and evaluation. Key takeaways influencing this plan are incorporated below.

## 7. Alternative UX/Layout Considerations

(Keep for future reference if button-based UX proves insufficient.)
*   Explore newer versions of `egui_tiles` for potentially improved drag/drop handling for advanced interactions.
*   Consider custom drag-and-drop implementations outside of `egui_tiles` default behavior if needed for drag-to-dock.
*   Layout persistence (saving/loading - requires manual serialization).
*   Consider more advanced features based on limitations and challenges.
*   **Context-aware docking/restoration:** Enhance docking to remember the last parent container and reopening to restore panels to their last known state (docked container or floating rect/size).
*   **Web Compatibility:** Ensure changes work in web builds using `trunk serve`.

## 8. Current Status & Next Steps (Final)

**Current Functionality:**
*   Clean implementation with stable tab dragging based on Brush's app.rs structure.
*   Initial layout matches target Brush layout with Settings/Presets, Scene, and Dataset panels.
*   Tab activation on click works correctly.
*   Tab reordering by dragging works correctly.
*   Splitting containers by dragging tabs to edges works.
*   Resizing main splits works seamlessly.

**Floating Windows & Docking Challenges (For Future Implementation):**
*   **UI Button Placement:** Finding optimal placement for undock buttons without interfering with standard tab and window title bars.
*   **Panel Naming Consistency:** Passing the tab name to buttons inside panel contents when undocking.
*   **Docking Target Selection:** Deciding where to dock a tab from a floating window (first available container? nearest? create new?).
*   **Empty Tree Handling:** Managing the closing of the last tab in the last panel since Brush only drags to empty and lacks a close button.
*   **Required vs. Optional Panels:** Determining if Scene panel should be closable or fixed/required.
*   **Panel Recovery:** Adding menu options to bring back closed tabs or windows.
*   **Window Resizing:** Ensuring floating windows can be resized along both axes.

**Known Limitations (For Future Consideration):**
*   **Drag-to-Dock Without Buttons:** Implementing drag-and-drop of floating windows to dock without using buttons appears difficult with current egui_tiles.
*   **Multi-Tab Floating Windows:** Adding multiple tabs inside a single floating window requires custom implementation.
*   **Cross-Window Tab Dragging:** Dragging tabs between the main background window and floating windows is challenging and may require custom implementation.

**Future Goals / Next Steps:**
1.  **Add Floating Windows:**
    *   Carefully implement undocking to floating windows.
    *   Implement docking from floating windows back to main layout.
    *   Add window management (closing, reopening).
2.  **Polish UI:**
    *   Add appropriate buttons for undocking/docking.
    *   Implement menu for reopening closed panels.
3.  **Advanced Features:**
    *   Layout persistence (saving/loading - requires manual serialization).
    *   Consider more advanced features based on limitations and challenges.
    *   **Context-aware docking/restoration:** Enhance docking to remember the last parent container and reopening to restore panels to their last known state (docked container or floating rect/size).
    *   **Web Compatibility:** Ensure changes work in web builds using `trunk serve`.

## 9. Open Questions / Considerations (`egui_tiles`)

*   **Simplification Interaction:** How best to define initial layouts with single-child `Tabs` containers if needed? (Current approach works by letting simplification run).
*   **Layout Persistence:** Best practices for serializing/deserializing `egui_tiles::Tree` state? (Requires defining custom serializable structures).
*   **State Management:** Evaluate transitioning to a single authoritative data model for panel state in the future (Ref: [04-mini_feedback.md](./04-mini_feedback.md) - Arch #8).
*   Explore `egui_tiles` examples/issues for advanced `Behavior` implementations if revisiting drag/drop.

## 10. Code Snippets / Implementation Details

*   **Event Queue System:** Implemented using `Rc<RefCell<Vec<UIEvent>>>` within `Arc<RwLock<AppContext>>` and processed after UI rendering in `App::update`. Event handlers return `Result<(), LayoutError>`. (Note: `Arc<RwLock>` seems necessary for sharing context between `AppTree` and `App` update loop, ref: [04-mini_feedback.md](./04-mini_feedback.md) - Arch #5 Analysis).
*   **Error Handling:** Uses `Result<(), LayoutError>` for event handlers. Critical operations like docking include recovery logic to reinstate panels into `floating_panels` on failure.
*   **Panel Identification:** Refactor to use a `PanelId` enum instead of string titles for keys in `floating_panels` and relevant `UIEvent` variants (Ref: [04-mini_feedback.md](./04-mini_feedback.md) - Arch #1).
*   **Planned `Behavior` Implementation:**
    ```rust
    struct AppTree { /* ... */ }
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub enum PanelId { Settings, Presets, Stats, Dataset, Scene /* ... */ }
    pub trait AppPanel {
        fn title(&self) -> String;
        fn ui(&mut self, ui: &mut egui::Ui, context: &mut AppContext, tile_id: TileId, is_floating: bool);
        fn inner_margin(&self) -> f32 { 12.0 }
    }
    type PaneType = Box<dyn AppPanel>;

    impl Behavior<PaneType> for AppTree {
        fn tab_title_for_pane(&mut self, pane: &PaneType) -> egui::WidgetText { /* ... */ }

        fn pane_ui(&mut self, ui: &mut egui::Ui, tile_id: TileId, pane: &mut PaneType) -> UiResponse { /* ... */ }

        fn tab_ui(
            &mut self,
            tiles: &mut Tiles<PaneType>,
            ui: &mut egui::Ui,
            _egui_id: egui::Id,
            tile_id: TileId,
            _tab_state: &egui_tiles::TabState,
        ) -> egui::Response {
            let pane = tiles.get_mut(tile_id).expect("Tile not found");
            let maybe_panel_id = {
                if let Tile::Pane(p) = pane {
                    Some(p.id())
                } else {
                    None
                }
            };

            ui.horizontal(|ui| {
                if let Some(panel_id) = maybe_panel_id {
                    if ui.small_button("⏏").clicked() {
                        self.context.write().expect("Lock poisoned").events.borrow_mut().push(
                            UIEvent::UndockPanel { panel_id, tile_id }
                        );
                    }
                }
                if let Some(panel_id) = maybe_panel_id {
                    if ui.small_button("x").clicked() {
                        self.context.write().expect("Lock poisoned").events.borrow_mut().push(
                            UIEvent::ClosePanel { panel_id, tile_id: Some(tile_id) }
                        );
                    }
                }

                let title = self.tab_title_for_pane(pane);
                ui.label(title).interact(egui::Sense::click_and_drag())

            }).response
        }

        fn simplification_options(&self) -> SimplificationOptions { /* ... */ }

        fn gap_width(&self, _style: &egui::Style) -> f32 { /* ... */ }
    }
    ```
*   **Key State Handling Functions:** (Existing functions to be potentially adapted for `PanelId` and context-aware docking)
    *   `handle_dock_panel`
    *   `handle_undock_panel`
    *   `handle_close_from_tree`
    *   `handle_close_floating`
    *   `handle_reopen_panel`
    *   `handle_request_activate_tab`
    *   `find_parent_of` (helper)
    *   `find_dock_target` (simplified helper)
*   **Removed Complexity:** `TreeNodeAction` enum, `primary_container_id`, complex/unused docking functions (`handle_dock_panel_to_target`, etc.).

## 11. Brush-Based Implementation Notes

*   **Clean Architecture:** Following Brush's architecture with clean trait-based panel implementation resulted in stable code.
*   **Minimal Dependencies:** Using only the essential components and gradually building up functionality.
*   **Tab Dragging:** The egui_tiles tab dragging works perfectly when implemented in this clean approach.
*   **Future Integration:** The floating window and docking functionality will be carefully added on top of this stable foundation.

## 12. Floating Windows Implementation Strategy

Based on our experience developing the tab dragging functionality, here's a detailed roadmap for carefully implementing floating windows:

### Phase 1: Preparation and State Management (Completed)

1.  **Eliminate Redundant Files (Completed)**
    *   Removed `main.rs`.
    *   Added `[[bin]]` section to `Cargo.toml` pointing to `src/app.rs`.

2.  **Add State Storage for Floating Windows (Completed)**
    *   Added `FloatingPanelState` struct.
    *   Added `floating_panels: HashMap<PanelId, FloatingPanelState>` to `App` (Changed key to `PanelId`).
    *   Initialized `floating_panels` in `App::new()`.

3.  **Implement Events System (Completed)**
    *   Added `UIEvent` enum (variants to use `PanelId`).
    *   Added `events: Rc<RefCell<Vec<UIEvent>>>` to `AppContext`.
    *   Added `process_events()` method stub to `App`.

4.  **Define `PanelId` Enum (Completed)**
    *   Created `enum PanelId`.
    *   Updated `floating_panels` HashMap key type.
    *   Updated `UIEvent` variants.
    *   Updated relevant handler function signatures.
    *   **TODO:** Add `fn panel_id(&self) -> PanelId` to `AppPanel` trait and implement it for each panel struct for cleaner lookups (instead of matching on `title()`).

### Phase 2: Undocking Implementation (Partially Completed - Needs Rework)

1.  **Add Undock Button via `Behavior::tab_ui` Override (Completed)**
    *   Implemented `Behavior::tab_ui` override with correct signature.
    *   Includes inline Undock button ("⏏") queuing `UIEvent::UndockPanel`.
    *   Label retains `Sense::click_and_drag()`.
    *   Replaced `egui::Area` approach in panel `ui` methods.

2.  **Implement Undock Handler (Partially Completed - Needs `last_parent_id`)**
    *   Adapted `handle_undock_panel()` for `PanelId`.
    *   **NEXT (Technical Detail):** Modify `handle_undock_panel` in `src/app.rs`:
        *   Add `last_parent_id: Option<TileId>` field to `FloatingPanelState` struct.
        *   Inside `handle_undock_panel`, call `let parent_id = self.find_parent_of(tile_id);` *before* modifying the parent container or removing the tile.
        *   When creating `new_floating_state`, set `last_parent_id: parent_id`.

3.  **Add Floating Window Rendering (Completed - Adapt for `PanelId`)**
    *   Loop in `App::update()` iterates over `floating_panels`.
    *   Window title uses `panel.title()`.
    *   Calls `panel.ui`. Removed `is_floating` parameter from `panel.ui`.
    *   Window close ('X') queues `UIEvent::ClosePanel { panel_id: ..., tile_id: None }`. (Updated event).
    *   Stores updated `rect` in `FloatingPanelState`.
    *   Dock button ("⚓") remains inside `panel.ui` using `egui::Area`, guarded by `if is_floating { ... }`.

### Phase 3: Manual Testing & Debugging (Ongoing)

1.  **Test Undocking & Button Behavior**
    *   Verify Undock button appears correctly in tab bar via `Behavior::tab_ui`.
    *   Verify button queues the correct event with `PanelId` and `tile_id`.
    *   Verify panel disappears from tree and appears as window on Undock.
    *   Verify window appearance (title, resizable, close button, **Dock button**).
    *   Check initial window size/position.
    *   Verify tree remains functional after undocking.
    *   Verify Close button ('x') appears correctly in tab bar via `Behavior::tab_ui`.
    *   Verify Close button queues correct event `ClosePanel { ..., tile_id: Some(...) }`.
    *   Verify floating window 'X' button queues correct event `ClosePanel { ..., tile_id: None }`.

2.  **Fix Common Issues (Logger Setup)**
    *   Add detailed logging for all operations.
    *   Track panel state transitions (docked -> floating -> docked).
    *   Verify no ownership issues or double removals.

3.  **Edge Cases to Test**
    *   Undocking the last tab in a `Tabs` container.
    *   Undocking when only one panel remains docked.
    *   Undocking and closing the window immediately.
    *   Undocking and docking immediately.
    *   Rapidly undocking/docking multiple panels.
    *   **TODO:** Decide if "Scene" panel should be undockable/closable or permanent. Implement `tab_ui` button logic conditionally if needed.
    *   **TODO:** Test docking when *no* tab containers exist in the tree (should `find_dock_target` handle this gracefully by creating a new root container?).

### Phase 4: Docking From Floating Windows (Partially Completed - Needs Context-Aware Logic)

1.  **Implement Dock Handler (Completed - Adapt for `PanelId`)**
    *   Adapted `handle_dock_panel()` for `PanelId`. Basic logic exists.

2.  **Target Container Selection Strategy (TODO - Implement Context-Aware)**
    *   **NEXT (Technical Detail):** Requires `last_parent_id` to be stored (Phase 2 Step 2). Enhance `handle_dock_panel` in `src/app.rs`:
        *   Check `state.last_parent_id` retrieved from `floating_panels`.
        *   If `Some(parent_id)`:
            *   Check `self.tree.tiles.get(parent_id)`. Is it `Some(Tile::Container(Container::Tabs(_)))`?
            *   If yes: Proceed with inserting the pane into *this* specific `parent_id` container.
            *   If no (parent gone/invalid): Implement fallback. **Initial Fallback:** Call the existing `self.find_dock_target()` and proceed as before.
        *   If `None`: Proceed with existing `self.find_dock_target()` logic.
    *   **Future Refinement:** Implement more sophisticated fallbacks (find neighbor, split).

### Phase 5: Close and Reopen Management (In Progress)

1.  **Window Close Handling (Completed)**
    *   Floating window 'X' queues `ClosePanel { ..., tile_id: None }`.

2.  **Add Close Button (Completed)**
    *   Inline Close button ('x') added within `Behavior::tab_ui` override. Queues `ClosePanel { ..., tile_id: Some(tile_id) }`.

3.  **Implement Close Handler (Partially Implemented - Needs Docked Logic)**
    *   Updated `handle_close_panel()` signature: `(panel_id: PanelId, tile_id: Option<TileId>)`.
    *   `None` arm (floating close) implemented: sets `state.is_open = false` in `floating_panels`.
    *   **NEXT (Technical Detail):** Implement `Some(tile_id_to_close)` arm in `handle_close_panel` in `src/app.rs`:
        1.  `let parent_id = self.find_parent_of(tile_id_to_close)?;` (Handle error).
        2.  Get `parent_container = self.tree.tiles.get_mut(parent_id)?;` (Match `Tile::Container`, handle error/type).
        3.  `parent_container.remove_child(tile_id_to_close);`
        4.  `let panel = match self.tree.tiles.remove(tile_id_to_close)? { Tile::Pane(p) => p, _ => return Err(...) };`
        5.  `let state = self.floating_panels.entry(panel_id).or_insert_with(|| FloatingPanelState { panel: panel.clone(), /* default state */ });` (Use `entry` API to handle existing/new state).
        6.  `state.panel = panel;` // Overwrite panel in case it was stale
        7.  `state.is_open = false;`
        8.  `state.last_parent_id = Some(parent_id);` // Store where it came from
        9.  `self.tree.simplify_children_of_tile(parent_id, ...);`
        10. Return `Ok(())`.

4.  **Add View Menu for Reopening (TODO)**
    *   **NEXT (Technical Detail):** Implement in `App::update` in `src/app.rs`:
        *   Add `egui::TopBottomPanel::top("top_panel").show(ctx, |ui| { ... });`.
        *   Use `egui::menu::bar` and `ui.menu_button("View", |ui| { ... });`.
        *   Iterate `self.floating_panels.iter().filter(|(_, state)| !state.is_open)`.
        *   For each, add `if ui.button(state.panel.title()).clicked() { queue_event!(UIEvent::ReopenPanel { panel_id: *panel_id }); ui.close_menu(); }`.

5.  **Implement Reopen Handler (TODO)**
    *   **NEXT (Technical Detail):** Implement `handle_reopen_panel(&mut self, panel_id: PanelId)` in `src/app.rs`:
        1.  `let state = self.floating_panels.get_mut(&panel_id)?;` (Handle error).
        2.  If `state.is_open`, log/toast and return `Ok(())`.
        3.  Match `state.last_parent_id`.
            *   `Some(parent_id)`: Try context-aware restore.
                *   Check if `parent_id` tile exists and is `Tabs`.
                *   If yes: Set `state.is_open = true; let panel = state.panel.clone();` (or take ownership carefully). **Remove `panel_id` entry from `floating_panels`.** Call logic to insert pane into `parent_id` container (similar to `handle_dock_panel` but targeted). Handle potential failure (put panel back in `floating_panels`?).
                *   If no: **Fallback:** Set `state.is_open = true;` (Reopens as floating).
            *   `None`: Set `state.is_open = true;` (Reopens as floating).
        4.  Return `Ok(())`.

### Phase 6: Advanced Features (Later)

1.  **Position Awareness for Docking (Deferred)**

2.  **Layout Persistence (Manual Serialization Required)**
    *   Define serializable representations for `Tree<PaneType>` and `floating_panels`.
    *   Implement save/load functionality. `egui` might persist some state but not the `egui_tiles` structure.
    *   **TODO:** Verify extent of default `egui` persistence.

3.  **Undock All / Dock All (TODO)**
    *   Consider adding buttons or menu options for mass dock/undock operations.

4.  **Single Data Model:** Consider refactoring to a single authoritative state map `HashMap<PanelId, PanelState>` where `PanelState` tracks location (`Docked(TileId, last_parent_id)`, `Floating(Rect)`) and open status (Ref: [04-mini_feedback.md](./04-mini_feedback.md) - Arch #8). This is a significant refactor, best done after core features are stable.

### Out of Scope for Initial Floating Window Implementation

*   **Drag-to-Dock:** Dragging a floating window onto the main layout to dock it.
*   **Multi-Tab Floating Windows:** Having multiple tabs within a single floating `egui::Window`.
*   **Cross-Window Dragging:** Dragging tabs between the main docked layout and floating windows.

### Testing Strategy

For each phase:
1. **Regression Testing**
   - Verify all existing functionality still works (tab dragging, splitting).
   - Ensure undocking/docking/closing/reopening works as expected.

2. **Web Compatibility Testing (NEW)**
   - Regularly build and test using `trunk serve`.
   - Ensure UI elements render correctly and events function as expected in a web browser.
   - Identify and fix any desktop-specific assumptions or APIs.

3. **Manual Test Cases**
   - Create a testing matrix covering all actions
   - Test error cases (e.g., dock when target container is missing)
   - Test with different panel combinations

3. **Debugging Aids**
   - Add clear visual indicators of success/failure
   - Add toggle for verbose debug logging

### Documentation

Throughout implementation:
1. Update comments to explain complex logic
2. Document known limitations
3. Track issues encountered for future reference 

## 13. Ruled-Out Approaches & Lessons

During development, several approaches were explored and discarded:

*   **`egui_dock` Crate:** Initial attempts used `egui_dock`, but it proved unstable for the desired combination of docking and floating windows, leading to the switch to `egui_tiles`.
*   **Initial `egui_tiles` Implementation:** A previous `egui_tiles` implementation (separate `main.rs` and `prototype.rs`) became overly complex, managing state inconsistently and leading to crashes, particularly during docking/undocking operations. This highlighted the need for a clean start and careful state management (like the event queue).
*   **Custom `Behavior::tab_ui` Overrides:** Attempts to significantly customize tab appearance by fully overriding `tab_ui` interfered with `egui_tiles`' default drag-and-drop sensing and state management, causing instability and preventing tab reordering. Minimal overrides might be possible but were deemed too risky.
*   **Buttons Inside `ScrollArea`:** Placing Dock/Undock buttons within the panel's main `ScrollArea` caused the buttons to scroll off-screen with the content, which was not the desired UX.
*   **`egui::Area` Inside Panel Content (Original Attempt):** Using `egui::Area` *inside* the panel UI (before ScrollArea) to position buttons led to rendering glitches (button appearing detached during resize) and layering issues (buttons from docked panels showing through floating windows).
*   **Dedicated `TopBottomPanel` in Floating Window:** Creating a separate bottom panel within the floating `egui::Window` specifically for buttons (tested on StatsPanel) worked functionally for resizing but created an inconsistent UI compared to placing the button directly within the panel's area and wasn't the preferred visual.
*   **Separate `main.rs` and `app.rs`:** The initial split was deemed redundant as the main application logic and `eframe::App` implementation could reside entirely within `app.rs` (or be structured like Brush with `app.rs` as the lib entry).
*   **Buttons in Tab Bar (`Behavior::top_bar_right_ui`, `Behavior::tab_ui`):** Attempts to place Undock/Close buttons in the tab bar itself (either aligned right or inline) interfered with default tab dragging behavior or had layout issues. **Decision:** Keep controls within the panel content area.
*   **Buttons in Floating Window Title Bar (`Window::title_bar`, `Window::frame`):** Adding buttons directly to the standard floating window title bar proved difficult/unsupported without significant custom frame drawing. **Decision:** Keep controls within the panel content area.

## 14. Next Steps (Resuming)

1.  **Implement `last_parent_id` storage** in `FloatingPanelState` and `handle_undock_panel` (Phase 2, Step 2 enhancement).
2.  **Implement Docked Panel Closing logic** in `handle_close_panel` (Phase 5, Step 3).
3.  **Add View Menu** in `App::update` (Phase 5, Step 4).
4.  **Implement Reopen Handler** `handle_reopen_panel` (Phase 5, Step 5), including basic context-aware restore/fallback.
5.  **Implement Context-Aware Docking** in `handle_dock_panel` using `last_parent_id` (Phase 4, Step 2).
6.  Address remaining TODOs (Scene panel closable?, `AppPanel::panel_id` trait method).
7.  Integrate Web Testing.
8.  Address Build Warnings. 