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
*   **Clean Implementation Approach:** Starting fresh from Brush's app.rs structure yielded a much more stable foundation by focusing on the core egui_tiles functionality first, with future plans to carefully add floating window features.

## 6. Refined Implementation Strategy

*   **Stage 1 (Completed):** Implement clean Brush-like UI with tabs and stable dragging behavior.
*   **Stage 2 (Next):** Carefully add floating window and docking/undocking capabilities, following the detailed plan in Section 12.
*   **Stage 3 (Future):** Implement more advanced features like layout persistence.

## 7. Alternative UX/Layout Considerations

(Keep for future reference if button-based UX proves insufficient.)
*   Explore newer versions of `egui_tiles` for potentially improved drag/drop handling for advanced interactions.
*   Consider custom drag-and-drop implementations outside of `egui_tiles` default behavior if needed for drag-to-dock.

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

## 9. Open Questions / Considerations (`egui_tiles`)

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
*   **Key State Handling Functions:** (All implemented and working in previous prototype)
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
   - Added `FloatingPanelState` struct.
   - Added `floating_panels: HashMap<String, FloatingPanelState>` to `App`.
   - Initialized `floating_panels` in `App::new()`.

3.  **Implement Events System (Completed)**
    *   Added `UIEvent` enum.
    *   Added `events: Rc<RefCell<Vec<UIEvent>>>` to `AppContext`.
    *   Added `process_events()` method stub to `App`.

### Phase 2: Undocking Implementation (Partially Completed)

1.  **Add Dock/Undock Button Inside Panel Content**
    *   Added `is_floating: bool` parameter to `AppPanel::ui` trait and updated implementations.
    *   Updated `AppTree::pane_ui` and floating window loop in `App::update` to pass correct `is_floating` value.
    *   **Chosen Strategy:** Use `egui::Area` positioned in the bottom-right corner of the panel's allocated space (outside any `ScrollArea`).
        *   Get the panel's outer rect (`ui.available_rect_before_wrap()`).
        *   Create the Area (`egui::Area::new(...).fixed_pos(...).order(egui::Order::Foreground)`).
        *   Inside the Area's `show` closure, check `is_floating`:
            *   If `true`, show Dock button ("⚓") queuing `UIEvent::DockPanel`.
            *   If `false`, show Undock button ("⏏") queuing `UIEvent::UndockPanel`.
        *   This is implemented for Settings, Presets, Stats, and Dataset panels.
    *   **Note:** `tile_id` is available when `is_floating` is false.

2.  **Implement Undock Handler (Completed)**
    *   Added `handle_undock_panel()` method to `App` within the `process_events` logic.
    *   Logic includes: finding parent, removing tile from parent, removing tile from tree, creating `FloatingPanelState`, adding to `floating_panels`, simplifying parent.

3.  **Add Floating Window Rendering (Completed)**
    *   Added loop in `App::update()` to render `egui::Window` for panels in `floating_panels` where `is_open` is true.
    *   Window uses panel title and stored `rect`.
    *   Calls `panel.ui` passing `is_floating: true`.
    *   Handles window close ('X') button by queuing `UIEvent::ClosePanel`.
    *   Stores updated window `rect` in `FloatingPanelState`.

### Phase 3: Manual Testing & Debugging (In Progress)

1.  **Test Undocking & Button Behavior**
    *   Verify Dock/Undock button appears correctly in both states for Settings, Presets, Stats, Dataset.
    *   Verify button queues the correct event.
    *   Verify panel disappears from tree and appears as window on Undock.
    *   Verify window appearance (title, resizable, close button).
    *   Check initial window size/position.
    *   Verify tree remains functional after undocking.
    *   **TODO:** Investigate/fix button layering issue where docked panel buttons can appear through floating windows.
    *   **TODO:** Investigate/fix resize glitch where Area button follows mouse beyond window bounds when shrinking window very small.
    *   **TODO:** Experiment with `Window::min_size` / `Window::max_size` to potentially mitigate resize glitches at small sizes.

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
    *   **TODO:** Decide if "Scene" panel should be undockable/closable or permanent. Implement undocking/docking for it if desired.
    *   **TODO:** Test docking when *no* tab containers exist in the tree (should `find_dock_target` handle this gracefully, e.g., by creating a new root container?).

### Phase 4: Docking From Floating Windows (Partially Completed)

1.  **Implement Dock Handler (Completed)**
    *   Added `handle_dock_panel()` method.
    *   Logic includes: removing from `floating_panels`, finding target, inserting pane into tree, adding to target container, activating tab, basic recovery.

2.  **Target Container Selection Strategy (TODO)**
    *   **Current:** Finds the *first* `Tabs` container.
    *   **TODO:** Handle case where no `Tabs` container exists.
    *   **Future:** Implement smarter docking (e.g., dock back to original container/split, dock to nearest, allow user selection, create new split).

### Phase 5: Close and Reopen Management (Next)

1.  **Window Close Handling (Completed in Phase 2)**

2.  **Add Close Button Inside Panel Content (TODO)**
    *   Add a Close button ('X') using the `egui::Area` strategy.
    *   On click, queue `UIEvent::ClosePanel { panel_title: self.title(), is_floating }`.
    *   Consider placement (e.g., top-right corner vs. bottom-right).

3.  **Implement Close Handler (Partially Implemented)**
    *   `handle_close_panel()` implemented for `is_floating: true`.
    *   **TODO:** Implement logic for `is_floating: false` (closing a docked panel).
    *   **TODO:** Handle closing the last panel in the tree (should tree become empty or show a placeholder?).

4.  **Add View Menu for Reopening (TODO)**
    *   Add top menu bar in `App::update()`.
    *   Implement submenu listing panels where `is_open` is false in `floating_panels`.
    *   **TODO:** Explore alternative menu presentations.

5.  **Implement Reopen Handler (TODO)**
    *   Add `handle_reopen_panel()` method.
    *   Implement logic to set `is_open = true` and assign default rect if needed.

### Phase 6: Advanced Features (Later)

1.  **Position Awareness for Docking (Deferred)**

2.  **Layout Persistence (Manual Serialization Required)**
    *   Define serializable representations for `Tree<PaneType>` and `floating_panels`.
    *   Implement save/load functionality. `egui` might persist some state but not the `egui_tiles` structure.
    *   **TODO:** Verify extent of default `egui` persistence.

3.  **Undock All / Dock All (TODO)**
    *   Consider adding buttons or menu options for mass dock/undock operations.

### Out of Scope for Initial Floating Window Implementation

*   **Drag-to-Dock:** Dragging a floating window onto the main layout to dock it.
*   **Multi-Tab Floating Windows:** Having multiple tabs within a single floating `egui::Window`.
*   **Cross-Window Dragging:** Dragging tabs between the main docked layout and floating windows.

### Testing Strategy

For each phase:
1. **Regression Testing**
   - Verify all existing functionality still works
   - Ensure tab dragging remains functional

2. **Manual Test Cases**
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

1.  Continue with **Phase 5: Close and Reopen Management**.
    *   Implement close button for docked panels (Phase 5, Step 2 & 3).
    *   Implement View menu and `handle_reopen_panel` (Phase 5, Step 4 & 5).
2.  Address remaining TODOs from Phase 3 testing (layering, resize glitch, Scene panel, empty tree docking).
3.  Refine Docking target strategy (Phase 4, Step 2).
4.  Consider advanced features from Phase 6. 