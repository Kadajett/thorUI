# Design System Plan

## Goal

ThorUI should feel built for a clamshell handheld, not like a desktop page divided between two small monitors. The design system gives experience authors surface-aware structure, controller and touch behavior, and consistent feedback while preserving normal web semantics.

The system is DOM-first for interface content and canvas-friendly for game worlds. Experience state remains in the session authority; the UI library presents projections and emits actions.

## Principles

- Compose one experience from two purposeful projections.
- Optimize the main surface for attention and the companion surface for hands-on interaction.
- Make every essential flow complete on one surface.
- Keep controller and touch equally capable, with feedback suited to each.
- Use the DOM for text, controls, forms, focus, and accessibility.
- Treat canvas as a focused rendering region, not the entire application shell.
- Adapt from observed surface profiles, not device names or raw pixel constants.
- Prefer calm hierarchy and short motion on the small OLED panels.

## Ownership and configuration

ThorUI uses a shadcn-style source ownership model with a stricter behavioral core. Authors own the presentation source they need, but the framework keeps controller, focus, touch, and accessibility behavior in one tested module.

There are three configuration layers:

1. **Control primitives** own interaction state machines and non-negotiable invariants. A copied button recipe cannot redefine disabled activation, focus visibility, pointer cancellation, or action dispatch.
2. **Design tokens and typed policies** configure color, type, space, density, motion, navigation, feedback, and surface adaptation. Invalid values fail validation before the application runs.
3. **UI recipes** are workspace-owned Leptos and style source. Authors can change markup, variants, slots, and composition without waiting for a framework release.

This is source ownership, not source duplication. A recipe is installed once into a shared workspace crate and reused by every experience in that workspace. Repeated behavior moves down into a control primitive; repeated visual composition becomes a shared recipe only after two real uses.

The alpha keeps one canonical recipe crate and validates it through `xtask`. A public registry is added only after a second workspace proves that copied-source distribution is a real interface.

The later Rust registry tool should provide:

- `thorui ui add` installs a recipe and its declared primitive and token dependencies;
- `thorui ui diff` compares owned source with a newer registry item;
- `thorui ui doctor` validates compatibility, tokens, states, and required interaction contracts;
- updates show a mergeable diff and never overwrite modified source silently.

Registry items carry a schema version, compatible ThorUI range, content hash, dependencies, token requirements, and source files. The repository keeps one canonical recipe source and builds registry artifacts from it; it never maintains a second internal copy merely to imitate a registry.

## Surface character

### Main surface

The main surface carries the primary task, game world, detailed content, or selected item. It favors wider compositions, lower control density, and readable information at a slightly greater viewing distance.

Common regions are a title/status rail, primary viewport, transient overlay layer, and optional command strip. Touch remains supported, but controller focus should not cover important content.

### Companion surface

The companion surface carries direct controls, inventory, map, navigation, detail, text entry, or contextual tools. It favors large targets, short labels, compact grids, and vertical scrolling within its near-square shape.

It must never become a miscellaneous panel. Every companion projection has one named purpose and a clear relationship to the main projection.

### Single surface

Single-surface mode composes the same information through tabs, drawers, overlays, or mode switches. It is a first-class projection, not an error page. Essential actions cannot depend on the missing surface.

## Adaptation matrix

| Concern | Main | Companion | Single-surface fallback |
|---|---|---|---|
| Primary content | Persistent and spacious | Summary or direct manipulation | Persistent |
| Navigation | Compact rail or command strip | Tabs, grid, or short list | Tabs or drawer |
| Context detail | Overlay or side pane when space allows | Preferred location | Sheet or full view |
| Text entry | Shows resulting content | Preferred keyboard/form surface | Modal or full view form |
| Game world | Preferred canvas | Map, inventory, controls, or second view | Canvas plus switchable overlay |
| Alerts | Brief shared status | Actionable detail | Combined notification |
| Focus | Sparse, content-aware | Strong and obvious | Conventional roving focus |

## Token model

Design tokens use the W3C Design Tokens Community Group format and Resolver Module as canonical source data. A small ThorUI manifest maps runtime facts, such as surface role and interaction mode, to resolver contexts without repeating token values or merge rules. A resolver validates these inputs, then generates:

- CSS custom properties for DOM recipes;
- typed Rust values for canvas and renderer policy;
- a resolved manifest for visual tests and diagnostics;
- an optional Tailwind theme adapter for recipe authoring.

Tailwind is not the token source of truth. A recipe that uses plain CSS and one that uses Tailwind must consume the same semantic variables. Arbitrary values inside standard recipes fail validation unless the recipe declares a narrow exception.

Tokens use semantic roles rather than component names. Scoped overrides may vary by theme, contrast preference, surface role, density, interaction mode, and motion preference. They do not create a copied token set for every combination.

Every surface root exposes stable attributes for styling and tests:

```text
data-surface     main | companion
data-density     compact | standard | generous
data-input       controller | touch | keyboard-pointer
data-theme       theme identifier
```

Controls expose `data-slot`, `data-state`, `data-variant`, and `data-size`. CSS is ordered through `reset`, `tokens`, `base`, `recipes`, `utilities`, and `overrides` cascade layers so author overrides are explicit rather than specificity contests.

### Color

- canvas, surface, raised surface, scrim;
- text, muted text, inverse text;
- accent and accent contrast;
- focus, selected, pressed, disabled;
- success, warning, danger, and information;
- game-overlay roles with explicit contrast requirements.

Themes may change values but not the meaning of roles. OLED themes use true black deliberately, avoid large maximum-brightness fields, and keep near-black layers distinguishable.

### Type

- display, title, heading, body, label, caption, and numeric roles;
- main and companion scales derived from the surface profile;
- line height and maximum line length included in each role;
- tabular numerals available for timers, counters, and telemetry.

No control shrinks text to fit. It wraps, truncates with an accessible full label, changes composition, or moves to a larger projection.

### Space and shape

- a short spacing scale used by layout primitives;
- compact, standard, and generous density modes selected by profile and user setting;
- small, medium, large, and pill radii;
- touch-target minimum validated on physical Thor panels;
- focus outlines that remain visible over DOM and canvas content.

### Motion

- instant feedback, short transition, view transition, and ambient durations;
- standard easing by purpose rather than by control;
- no layout motion that blocks controller input;
- reduced-motion mode removes travel and preserves state feedback;
- main and companion animations use elapsed time, not assumed frame counts.

## Layout primitives

The first set should stay small and deep:

- `SurfaceRoot` applies role, profile, safe areas, theme, density, and interaction mode.
- `Stack` handles one-axis layout, spacing, alignment, and wrapping policy.
- `Cluster` handles compact groups that wrap as a unit.
- `Grid` handles minimum cell size, navigation order, and responsive columns.
- `Pane` gives a named content region with title, actions, scroll, and empty state.
- `CanvasViewport` owns drawing size, render scale, overlays, and context status.
- `Switcher` chooses dual-role, tabbed, drawer, or overlay composition from capability.

These names are provisional until usage prototypes show the smallest useful interfaces. They must hide CSS mechanics and focus bookkeeping instead of forwarding every CSS property.

## Control behavior

Standard controls share a headless behavior module and one styled DOM presentation. This prevents buttons, toggles, sliders, lists, and canvas overlays from each inventing action dispatch, disabled rules, or sound and haptic feedback.

Every interactive control specifies:

- idle, focused, pressed, selected, disabled, busy, and error states where meaningful;
- controller action mapping and focus neighbors;
- pointer capture and cancellation behavior;
- touch target and gesture conflict policy;
- accessible role, name, value, description, and live feedback;
- optional sound and haptic requests as effects;
- reduced-motion and high-contrast behavior.

Touch does not create a sticky hover state. Controller focus is visible after meaningful controller input and quiet after direct touch. Input modality changes do not discard the logical focused item.

Recipes receive typed variants and state from primitives. They may expose stable slots, semantic token overrides, and a final presentation-class hook. They cannot accept raw controller bindings, invent private focus state, or bypass the standard action path.

## Controller navigation

Focus navigation is spatial within a declared focus scope. Layout primitives supply geometry, while controls may override an ambiguous neighbor explicitly. DOM tab order remains logical for keyboard and assistive tools.

The standard action vocabulary begins with Navigate, Confirm, Cancel, Menu, PagePrevious, PageNext, Context, and Pause. Experience-specific actions use their own namespace and never depend on raw button numbers.

Repeated navigation has an initial delay and bounded repeat rate. Analog sticks use dead zones and hysteresis before becoming navigation. Small drift cannot switch interaction mode or move focus.

Prompts show the active controller profile's glyph where it is known and a semantic label otherwise. No prompt assumes Nintendo, Xbox, or PlayStation face-button lettering.

## Initial controls

The alpha catalog includes:

- action button and icon button;
- toggle, checkbox, and segmented choice;
- slider and stepper;
- text field, search field, numeric field, and validation message;
- tabs, list, grid, and selectable card;
- dialog, sheet, menu, and command bar;
- progress, status, toast, and persistent alert;
- controller prompt and action legend;
- scroll region and focus-visible scrollbar;
- canvas overlay, reticle, HUD group, and pause layer.

Controls enter the catalog only with their complete state and input contract. Composite domain widgets belong in experiences until two real consumers demonstrate shared behavior.

## Initial patterns

### Application patterns

- list on companion, detail on main;
- document or media on main, tools on companion;
- dashboard on main, filters and commands on companion;
- result on main, form or keypad on companion;
- overview on main, direct manipulation on companion.

### Game patterns

- world on main, map on companion;
- world on main, inventory and loadout on companion;
- world on main, tactical commands on companion;
- synchronized viewpoints with different quality and refresh policy;
- paused main view with complete companion menu.

### Failure patterns

- companion disconnected with controls promoted to a main overlay;
- main unavailable with a safe session control view on companion;
- reconnecting projection with stale controls disabled;
- renderer lost while DOM recovery controls remain available;
- controller disconnected with touch guidance visible.

## UI authoring technology

Leptos 0.8 client-side rendering is the provisional alpha choice. It matches the web-first product, updates the real DOM through fine-grained reactivity, mounts into a known surface root, and leaves CSS architecture to ThorUI. Server rendering, hydration, routing, and server functions stay out of the device bundle unless a real use case appears.

This is a qualified choice. Leptos' creator described the framework as feature-complete and moving to light maintenance in May 2026. ThorUI therefore pins the accepted 0.8 release, tracks security and browser compatibility, and keeps Leptos behind a presentation adapter. Dioxus 0.7 is the active challenger because its web tooling and maintenance trajectory are stronger. Yew remains a conservative fallback, not a third implementation target.

Before the simulator grows around Leptos, a bounded acceptance spike renders one representative control catalog and one canvas overlay in Leptos and Dioxus. It measures:

- optimized WASM size and cold start on the Snapdragon 865 Thor;
- large-list updates and rapid controller focus changes;
- typed custom actions, pointer capture, and imperative focus;
- mount, unmount, cleanup, and repeated surface-peer reconnection;
- canvas ownership, resize, and graphics-context recovery;
- generated DOM, browser accessibility tree, and debugging quality;
- compile time, dependency weight, maintenance risk, and migration surface.

Leptos wins unless it misses a measured budget, cannot meet an interaction contract cleanly, or its maintenance state worsens before adoption. The spike is not permission to build two production renderers.

Canonical experience state remains outside Leptos. Each surface peer mounts one root, receives its latest projection through a read-only reactive boundary, and emits typed actions. A recipe may own short-lived presentation state such as an open animation or an uncommitted text draft; session meaning and cross-surface state remain with the authority.

Leptos signals, owners, DOM nodes, and browser types cannot cross the design-system adapter into the runtime core. Canvas code receives an owned element through the adapter and exclusively owns drawing in that region.

The default recipe toolchain uses Tailwind CSS v4 as a build-time authoring adapter because it supports semantic theme variables and a familiar shadcn workflow. Generated CSS is bundled offline and has no runtime dependency. Rust source must use complete static utility names and explicitly register source paths; dynamic class-name construction is forbidden. Plain layered CSS remains supported for recipes that do not benefit from utilities.

## Visual and interaction validation

- Render every catalog state at main, companion, and single-surface profiles.
- Capture light, dark, OLED, high-contrast, and reduced-motion variants.
- Test 100%, enlarged, and smallest supported text scales.
- Verify focus order and action reachability with no pointer.
- Verify touch with no controller and controller plus touch together.
- Measure physical target size, text legibility, glare, and thumb reach on hardware.
- Test long translated labels and bidirectional layout before beta.
- Keep screenshot changes reviewable by control and profile, not one giant image set.

## Definition of native-feeling

For ThorUI, native-feeling means:

- one launch owns both panels or gives a clear recovery action;
- system bars and browser chrome do not interrupt the intended session;
- controller focus, Back, suspend, resume, and audio behave consistently;
- touch feedback begins immediately and never waits for network work;
- the two projections agree without visible state races;
- motion matches the actual surface cadence;
- a missing surface produces a deliberate layout;
- loading, updates, permission failures, and renderer loss have designed states.

It does not mean copying Android widgets or a Nintendo interface. The web layer earns its place through flexible layout, typography, accessibility, and iteration while the Android host supplies display ownership.
