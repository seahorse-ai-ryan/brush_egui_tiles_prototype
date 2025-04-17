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
    enum PanelId { Settings, Presets, Stats, Dataset, Scene /* ... */ } // Added PanelId
    type PaneType = Box<dyn AppPanel>; // Or a struct holding the panel and metadata

    impl Behavior<PaneType> for AppTree {
        fn tab_title_for_pane(&mut self, pane: &PaneType) -> egui::WidgetText { /* ... */ }

        fn pane_ui(&mut self, ui: &mut egui::Ui, tile_id: TileId, pane: &mut PaneType) -> UiResponse { /* ... */ }

        // PRIORITIZED: Override tab_ui for inline buttons, replacing Area approach
        fn tab_ui(&mut self, tiles: &Tiles<PaneType>, ui: &mut egui::Ui, tile_id: TileId, pane: &mut PaneType) -> egui::Response {
            // Example structure:
            ui.horizontal(|ui| {
                // Get panel id (assuming PaneType holds it or can derive it)
                // let panel_id = pane.id(); // Example

                // Undock button (only shown when docked)
                if ui.small_button("⏏").clicked() {
                     // Queue UndockPanel event using panel_id and tile_id
                     // self.context.events.borrow_mut().push(UIEvent::UndockPanel { panel_id, tile_id });
                }

                // Label with drag sense
                let title = self.tab_title_for_pane(pane);
                let response = ui.label(title).sense(egui::Sense::click_and_drag());

                // TODO: Add inline Close button maybe?

                response // Return the label's response for drag detection
            }).response
        }

        fn simplification_options(&self) -> SimplificationOptions { /* ... */ }

        fn gap_width(&self, _style: &egui::Style) -> f32 { /* ... */ }

        // Removed: top_bar_right_ui (prefer inline controls in tab_ui)
        // Removed: Other drag methods likely removed or minimal (using default drag)
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

4.  **Define `PanelId` Enum (TODO)**
    *   Create `enum PanelId { Settings, Scene, ... }`.
    *   Update `floating_panels` HashMap key type.
    *   Update `UIEvent` variants (`UndockPanel`, `DockPanel`, `ClosePanel`, `ReopenPanel`) to use `PanelId` instead of `String`.
    *   Update relevant handler function signatures (`handle_dock_panel`, etc.).
    *   Ensure `AppPanel` trait or implementations provide a way to get the `PanelId`.

### Phase 2: Undocking Implementation (Partially Completed - Needs Rework)

1.  **Add Undock Button via `Behavior::tab_ui` Override (TODO - Replaces Area approach)**
    *   **Chosen Strategy:** Implement `Behavior::tab_ui` override.
    *   Inside the override, use `ui.horizontal` to place an Undock button ("⏏") next to the standard tab label.
    *   The label itself should retain `Sense::click_and_drag()` for drag functionality.
    *   Button click queues `UIEvent::UndockPanel { panel_id: ..., tile_id: ... }`.
    *   This replaces the `egui::Area` buttons previously added in `AppPanel::ui` methods and resolves associated layering/resize issues.
    *   **Remove** the `egui::Area` code from individual panel `ui` methods (Settings, Presets, Stats, Dataset).
    *   **Note:** Dock button ("⚓") will be handled within the floating window UI (Phase 3).

2.  **Implement Undock Handler (Completed - Adapt for `PanelId`)**
    *   Adapt `handle_undock_panel()` to accept and use `PanelId`.
    *   **Enhancement:** Store the `parent_id` (the container the panel was docked in) within the `FloatingPanelState` when undocking. This is needed for context-aware docking later.

3.  **Add Floating Window Rendering (Completed - Adapt for `PanelId`)**
    *   Adapt loop in `App::update()` to iterate over `floating_panels` using `PanelId`.
    *   Window title can be derived from the `PanelId` or `AppPanel::title()`.
    *   Calls `panel.ui`. **Remove** `is_floating` parameter from `AppPanel::ui` if no longer needed after removing Area buttons. (Alternatively, keep it for the floating window's Dock button).
    *   Handles window close ('X') button by queuing `UIEvent::ClosePanel { panel_id: ..., is_floating: true }`.
    *   Stores updated window `rect` in `FloatingPanelState`.
    *   **Add Dock Button:** Inside the floating window's UI (likely within the `panel.ui` call, potentially still using `egui::Area` carefully or another method), add a Dock button ("⚓") that queues `UIEvent::DockPanel { panel_id: ... }`.

### Phase 3: Manual Testing & Debugging (In Progress)

1.  **Test Undocking & Button Behavior**
    *   Verify Undock button appears correctly in tab bar via `Behavior::tab_ui`.
    *   Verify button queues the correct event with `PanelId` and `tile_id`.
    *   Verify panel disappears from tree and appears as window on Undock.
    *   Verify window appearance (title, resizable, close button, **Dock button**).
    *   Check initial window size/position.
    *   Verify tree remains functional after undocking.
    *   ~~**TODO:** Investigate/fix button layering issue where docked panel buttons can appear through floating windows.~~ (Resolved by moving to `tab_ui`)
    *   ~~**TODO:** Investigate/fix resize glitch where Area button follows mouse beyond window bounds when shrinking window very small.~~ (Resolved by moving to `tab_ui`)
    *   **TODO:** Experiment with `Window::min_size` / `Window::max_size` to potentially mitigate resize glitches at small sizes (if any remain with floating window content).

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

### Phase 4: Docking From Floating Windows (Partially Completed - Needs Rework)

1.  **Implement Dock Handler (Completed - Adapt for `PanelId` and Context)**
    *   Adapt `handle_dock_panel()` to accept `PanelId`.
    *   Update logic to use context-aware docking (see Step 2 below).
    *   Logic includes: removing from `floating_panels`, finding target, inserting pane into tree, adding to target container, activating tab, basic recovery.

2.  **Target Container Selection Strategy (TODO - Implement Context-Aware)**
    *   **Primary Strategy:** Retrieve the stored `last_parent_id` from `FloatingPanelState`.
        *   Attempt to dock back into the container with `last_parent_id` if it still exists and is a `Tabs` container (or adaptable).
    *   **Fallback 1:** If `last_parent_id` container is gone, attempt to find its last known sibling/neighbor and perform a split (e.g., right split) to create a new `Tabs` container for the docking panel.
    *   **Fallback 2:** If no suitable original or neighbor container found, find the *first available* `Tabs` container in the tree.
    *   **Fallback 3:** If *no* `Tabs` container exists, create a new root (e.g., a horizontal split with the existing root, or replace root if empty) containing a new `Tabs` container for the panel.
    *   **Refinement:** This logic requires helper functions to check tile existence and find neighbors/suitable insertion points.

### Phase 5: Close and Reopen Management (Next)

1.  **Window Close Handling (Completed in Phase 2 - Adapt for `PanelId`)**
    *   Ensure close event uses `PanelId`.

2.  **Add Close Button (TODO - Decide Location)**
    *   **Option 1 (Tab Bar):** Add an inline Close button ('X') within the `Behavior::tab_ui` override, next to the Undock button. Queues `UIEvent::ClosePanel { panel_id: ..., is_floating: false }`. This keeps controls consolidated.
    *   **Option 2 (Panel Content):** Add a Close button using `egui::Area` within the panel content (similar to the original Dock/Undock, but only for Close). Might be needed if `tab_ui` becomes too crowded.
    *   Choose one strategy for consistency.

3.  **Implement Close Handler (Partially Implemented - Adapt for `PanelId`, Implement Docked)**
    *   Adapt `handle_close_panel()` for `PanelId`.
    *   Implement logic for `is_floating: false` (closing a docked panel):
        *   Find the panel's `tile_id`.
        *   Find its `parent_id`.
        *   Remove the tile from the parent container.
        *   Remove the tile from `tree.tiles`, retrieving the `Panel`.
        *   Create a `FloatingPanelState` with `is_open: false`, store the `Panel`, and potentially the `last_parent_id` and last known floating `rect` (if available).
        *   Add/update the entry in `floating_panels`.
        *   Simplify the parent container.
    *   **TODO:** Handle closing the last panel in the tree (should tree become empty or show a placeholder?).

4.  **Add View Menu for Reopening (TODO)**
    *   Add top menu bar in `App::update()`.
    *   Implement submenu listing panels based on `PanelId` where `is_open` is false in `floating_panels`.
    *   Use icons (Ref: [04-mini_feedback.md](./04-mini_feedback.md) - UI #2) and tooltips (UI #6) here.
    *   Menu item click queues `UIEvent::ReopenPanel { panel_id: ... }`.

5.  **Implement Reopen Handler (TODO)**
    *   Add `handle_reopen_panel()` method accepting `PanelId`.
    *   **Stateful Restore Logic:**
        *   Check the state stored in `floating_panels` for the given `PanelId`.
        *   **If last state was floating:** Set `is_open = true`. The existing `rect` will be used by the rendering loop.
        *   **If last state was docked:** Attempt to dock the panel back using the context-aware docking logic (similar to `handle_dock_panel`, using the stored `last_parent_id` etc.). This might involve removing the panel from `floating_panels` and re-inserting into the tree. Assign a default floating rect only if docking fails completely and it needs to be opened as a floating window.
    *   Add UI hints on already open (Ref: [04-mini_feedback.md](./04-mini_feedback.md) - UI #6).

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

## 14. Next Steps (Resuming)

1.  **Refactor for `PanelId` (Phase 1, Step 4).**
2.  **Implement `Behavior::tab_ui` override for Undock button** (Phase 2, Step 1), removing old `Area` buttons.
3.  **Adapt Handlers:** Update `handle_undock_panel`, `handle_dock_panel`, `handle_close_panel` for `PanelId`.
4.  **Implement Context-Aware Docking** (Phase 4, Step 2) and store `last_parent_id` during undock (Phase 2, Step 2 enhancement).
5.  **Continue with Phase 5: Close and Reopen Management**, implementing the Close button, `handle_close_panel` for docked tabs, the View Menu, and `handle_reopen_panel` with stateful restore logic.
6.  Address remaining TODOs from Phase 3 testing (Scene panel, empty tree docking).
7.  **Integrate Web Testing:** Start running `trunk serve` periodically. 
8.  **Address Build Warnings:** Clean up unused imports and visibility issues identified during `trunk build`. 