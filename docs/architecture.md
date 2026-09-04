# Architecture

## Status

This is the planning baseline. A two-Activity Android host is the leading execution topology, but the topology and graphics dependencies remain open until the hardware probe is complete.

## Architectural goal

ThorUI should make a dual-screen experience feel like one program even when Chrome or Android requires two execution contexts. Experience authors should describe state changes, semantic actions, effects, and surface projections without handling display IDs, raw controller mappings, refresh loops, or cross-context messages.

## Constraints

- The Thor main surface is 1920 by 1080 at up to 120 Hz.
- The Thor companion surface is 1240 by 1080 at up to 60 Hz.
- Both surfaces are touch displays and have different physical densities.
- Chrome and Android lifecycle rules may pause one execution context independently.
- Controller identity and mappings must be observed on hardware.
- The Snapdragon 865 Thor is the performance floor.
- Browser-only launch and placement on both displays is not assumed.
- The framework must degrade to one surface without breaking the experience.

## Functional core and host shell

The core owns deterministic policy. It accepts facts and returns data describing the next state, projections, and requested effects. It cannot read browser globals, clocks, storage, network state, or controller state directly.

Hosts own effects. They sample time and input, schedule frames, render projections, persist data, route messages, and report lifecycle changes. Every host fact enters the core as explicit data.

Local mutation is allowed inside a deep module for measured performance needs. It must not become shared mutable state or change the module's observable semantics.

## Runtime shape

```text
 input samples             frame and lifecycle facts
      |                              |
      v                              v
+-------------+   actions   +-------------------+   projections   +--------------+
| Input       |-----------> | Session Authority |---------------> | Surface Peer |
| Normalizer  |             | and Experience    |                 | main or      |
+-------------+             +-------------------+                 | companion    |
                                   |       ^                      +--------------+
                           effects |       | results                     |
                                   v       |                             v
                              +----------------+                    DOM or canvas
                              | Host Adapters  |
                              +----------------+
```

There is exactly one authority for a session. Surface peers never resolve conflicting state. They send timestamped input and host facts to the authority, then present the latest valid projection.

The seam between the authority and surface peers is real because it needs at least three adapters: in-process for tests and the simulator, browser messaging for Chrome contexts, and native routing if an Android host is required.

## Execution topologies

All topologies use the same session protocol.

### One browser context

The simulator presents both surfaces in one document. It uses an in-process adapter and independent virtual frame clocks. This is the fastest development path and the reference behavior for deterministic tests.

### Two Chrome contexts

Two same-origin documents coordinate through a browser transport. The probe must validate window placement, visibility, refresh scheduling, BroadcastChannel or MessagePort behavior, fullscreen, audio ownership, and reconnection on the Thor.

This is the preferred device topology only if both contexts remain active and placement is usable without fragile manual steps.

### Android-assisted contexts

A small Android host launches one web surface on each Android display and routes messages between them. The web content and framework remain Rust/WASM. Kotlin is limited to display discovery, activity lifecycle, WebView hosting, permissions, and message routing.

This becomes the supported device topology if Chrome cannot reliably launch, keep active, or recover both surfaces. It is not a second product implementation.

## Planned deep modules

Module names are provisional. Crates are created only when a module has earned an independent interface.

### Runtime kernel

Owns session authority, lifecycle, deterministic ordering, fixed simulation steps, effect tracking, recovery, and surface membership. Its interface should accept batches of explicit facts and return all observable work for hosts.

It hides message ordering, stale peer rejection, catch-up limits, pause and resume policy, and projection invalidation.

### Input

Converts browser or host input samples into actions. It owns controller profiles, dead zones, hysteresis, repeat timing, chords, focus movement, touch gestures, modality changes, and remapping.

Raw browser events do not cross this module's external interface. Experience code consumes actions and can define namespaced experience-specific actions.

### Surface runtime

Owns observed surface profiles, role assignment, per-surface frame pulses, quality policy, resize and density changes, and projection delivery. Simulation time is independent from presentation refresh.

Each surface renders on its own frame pulse. A 60 Hz simulation may be interpolated for a 120 Hz main surface while the companion surface renders only when its own 60 Hz pulse arrives.

### Design system

Owns token schemas, layout rules, interaction states, focus behavior, standard patterns, motion policy, and surface-aware defaults. It exposes intent and structure, not Thor pixel constants.

Framework-owned control primitives centralize accessibility, controller, focus, and touch semantics. Workspace-owned UI recipes combine those primitives with design tokens and Leptos presentation source. Editing a recipe changes appearance and composition without copying behavioral policy.

The first renderer is DOM and CSS through a replaceable Leptos CSR adapter because applications and companion controls need text, forms, accessibility, and touch behavior. Games receive a canvas region and the same tokens, actions, overlays, and lifecycle. A second production widget renderer is deferred until a real use case justifies it.

### Platform host

Owns browser bindings and, if needed, the Android bridge. It implements time, input sampling, rendering, storage, messaging, fullscreen, wake lock, audio activation, and lifecycle effects.

Browser types stay inside this module. Tests use in-memory adapters at its internal seams, while browser and Android contract tests verify real behavior.

### Demo delivery

The latest successful runnable demo is published at `thorui.yougotserved.dev`. This repository owns the built Worker Static Assets artifact and its Wrangler deployment. The sibling `kadajett-infrastructure` repository owns the Cloudflare custom-domain binding in an independent application edge stack.

Static hosting does not traverse the Kubernetes cluster or Cloudflare Tunnel. A tunnel is added only if a later server-side feature introduces a non-Cloudflare origin. See [Demo Delivery](./deployment.md) for the ownership and release contract.

### Session protocol

Defines versioned messages between authority and peers. Initial messages cover hello and capabilities, role assignment, input batches, lifecycle, projection updates, acknowledgements, health, and resynchronization.

Messages are deterministic data with explicit schema versions. The first codec should favor debuggability; binary encoding is added only after measurement shows it is needed.

## Provisional workspace layout

Milestone 1 starts with only the capability lab. The wider workspace is created after the topology decision so the first structure reflects real seams.

```text
crates/
  thorui-core/       runtime, protocol, profiles, and pure input policy
  thorui-web/        WASM entry points and browser host adapters
  thorui-ui/         pure control behavior and token contracts
  thorui-leptos/     DOM adapter and Leptos control bindings
tools/
  thorui-lab/        device probe and dual-surface simulator
hosts/
  android/           optional display launcher and WebView message router
ui/
  thor-default/      canonical tokens and workspace-owned recipes
examples/
  application/       non-game reference experience
  game/              fixed-step reference experience
xtask/               repeatable project and release tasks
```

This is a map, not a crate quota. Protocol or input becomes a separate crate only when independent consumers, compile targets, or release needs justify the seam. Recipe validation begins in `xtask`; a distributable `thorui-cli` is deferred until a second workspace validates the registry boundary. A root convenience crate is deferred until the public interfaces settle.

The expected dependency direction is `examples -> recipes -> leptos adapter -> ui/core`; the web host composes the adapter with browser I/O. The Android host loads web artifacts and speaks the session protocol but cannot own experience or design-system policy.

## Experience interface direction

Three public interface shapes were compared in [Experience Interface Options](./interface-options.md). The provisional recommendation is a minimal typed reducer with default application and game constructors. It is not accepted until the Milestone 2 spike passes.

The final interface must support:

- one initialization path;
- deterministic state transitions;
- explicit requested effects;
- independent main and companion projections;
- fixed-step games and event-driven applications;
- single-surface fallback;
- state save and restore;
- test use without a browser;
- local optimization without observable shared mutation.

A rough form is:

```rust
// Provisional shape, not an accepted interface.
fn start(config: StartConfig) -> Transition<State>;
fn react(state: State, facts: FactBatch) -> Transition<State>;
fn project(state: &State, surface: SurfaceProfile) -> Projection;
```

`Transition` contains the next state and requested effects. Time, input, random values, and effect results arrive through `FactBatch`; the experience does not fetch them.

## Time and rendering

- Monotonic host timestamps are converted to session time at one seam.
- Simulation uses a fixed step selected by the experience, with a bounded catch-up count.
- Rendering never advances canonical session state.
- Each surface receives its own frame pulse and interpolation value.
- Background suspension produces an explicit lifecycle fact, not a large hidden delta.
- Resume restores or advances by declared policy; it never simulates an unbounded backlog.
- Quality may differ by surface, but state and interaction semantics may not.

The default game policy is a 60 Hz simulation, 120 Hz-capable main presentation, and 60 Hz companion presentation. Applications may be event-driven and render only invalidated projections.

## Input behavior

Controller input is polled once per active host frame, diffed, timestamped, and normalized. A profile maps observed axes and buttons to physical controls; an action map assigns meaning for the current experience and context.

Touch uses Pointer Events where available. The host captures pointers, preserves simultaneous touches, and records pressure and geometry only when supported. Gesture recognition and controller repeat use session time so they remain deterministic.

Interaction mode changes only after meaningful input. Minor stick drift cannot steal focus from touch. Every standard control must be reachable by controller and touch unless the experience explicitly declares otherwise.

## Design language

### Foundations

- Color roles include canvas, surface, raised surface, text, muted text, accent, warning, danger, focus, and disabled.
- Type uses a compact scale tuned separately for main and companion viewing distance.
- Space and radius use small token scales instead of free values.
- Motion has standard durations and a reduced-motion path.
- Touch size is expressed in logical units and validated in physical size on hardware.
- OLED-aware themes avoid large bright fields while keeping contrast accessible.

### Layout rules

- Main and companion projections are composed separately; the companion is not a squeezed main view.
- Layout keys off surface profile and role, never a user-agent string.
- Primary actions remain stable when a projection moves between surfaces.
- Critical state cannot exist only on a surface that may be absent or suspended.
- Text wrapping, focus order, and touch size are tested at the smallest supported profile.

### Initial patterns

- Main view plus companion controls
- Main canvas plus companion inventory or map
- List on one surface plus detail on the other
- Shared status and transient notifications
- Controller-first menu and command bar
- Touch keyboard or numeric entry on the companion surface
- Single-surface tabs as the fallback for every dual-surface pattern

## State and messaging rules

- The authority assigns a monotonically increasing session revision.
- Peers apply only complete projections matching their surface and a newer revision.
- Input batches carry peer identity, sequence, source timestamp, and current mapping revision.
- Duplicate input is safe to reject by sequence.
- On a gap or reconnect, a peer requests a complete projection instead of guessing.
- Slow peers may skip obsolete projections; input and effect results may not be skipped.
- Protocol logging redacts user content by default.

## Failure behavior

- If the companion disappears, its critical controls become available on the main surface.
- If a peer reconnects, it negotiates capabilities and receives a complete current projection.
- If refresh rate changes, simulation continues and presentation policy is recalculated.
- If controller access fails, touch remains complete.
- If an optional capability fails, the experience receives a typed result and selects a fallback.
- If the authority is lost, the session stops cleanly in the first release; authority migration is deferred.

## Scope limits for the first release

- No general ECS, physics engine, editor, or asset pipeline.
- No remote multiplayer or internet transport.
- No required WebGPU path.
- No render-neutral widget tree with multiple full renderers.
- No authority migration between peers.
- No hard-coded assumption that display zero is the main surface.
- No framework-wide abstraction for a dependency with only one implementation.
