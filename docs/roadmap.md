# Delivery Roadmap

Each milestone ends in an executable artifact or recorded decision. A goal loop should take one milestone at a time and stop at its exit gate.

From the first runnable artifact onward, the latest successful default-branch demo is published to `thorui.yougotserved.dev`. A failed build or smoke check leaves the previous green deployment live. Preview work cannot replace production before its milestone checks pass.

## Milestone 0: Planning baseline

Deliver the domain language, architecture, quality rules, platform research, open assumptions, and phased roadmap.

Exit gate:

- Terms and scope are internally consistent.
- Browser facts are sourced and unknown Thor behavior is listed as probes.
- No execution topology is presented as proven.
- The next milestone can start without selecting a UI or game framework.

## Milestone 1: Thor capability lab

Build the smallest Rust/WASM PWA that observes behavior rather than presenting product UI. It should export one machine-readable capability report and a short human summary.

Probe:

- reported screens, display IDs, CSS viewport, physical pixels, density, color depth, orientation, and safe areas;
- actual frame pulse distributions on both surfaces at 60 Hz and 120 Hz settings;
- focus and Page Visibility state while both surfaces are visible;
- popup, fullscreen, Window Management, Presentation API, and installed PWA behavior;
- BroadcastChannel and MessagePort latency, throughput, ordering, and reconnect behavior;
- every controller button, axis, trigger, stick click, system button, hot-plug event, and haptic capability;
- simultaneous controller and multi-touch input, pointer capture, and gesture cancellation;
- WebGL2 and WebGPU capability, context loss, memory pressure, and canvas resize behavior;
- audio activation, wake lock, storage, offline launch, suspend, resume, and screen shutdown;
- single-surface behavior and moving an experience between surfaces.

If Chrome cannot place and sustain both contexts, build a disposable Android display-launch probe. Do not build the final host yet.

Exit gate:

- At least one real Thor capability report is committed.
- The latest green capability-lab build and its revision are available at `thorui.yougotserved.dev`.
- Unknowns that affect the topology have measured answers.
- An ADR selects Chrome-only or Android-assisted execution.
- Provisional latency and frame budgets are replaced with measured budgets.

## Milestone 2: Runtime contract

Validate the recommended experience interface with the acceptance spike, record the decision, then implement the smallest native-testable runtime kernel and versioned session protocol.

Deliver:

- initialization, fact batching, deterministic transition, projection, and effect contracts;
- authority and peer handshake, revisions, sequence checks, and resynchronization;
- fake time, seeded randomness, in-memory transport, and effect adapters;
- pure unit, property, and protocol compatibility tests;
- saved traces that can replay a short session deterministically.

Exit gate:

- A headless application counter and fixed-step game simulation use the same interface.
- Replaying the same trace produces byte-identical observable output.
- Duplicate, delayed, missing, and reordered peer messages have specified outcomes.
- Public interfaces contain no browser types.

## Milestone 3: UI renderer gate and dual-surface simulator

First validate the provisional Leptos CSR choice against Dioxus with one bounded control-and-canvas spike. Record the renderer decision, then build a browser lab that presents the two Thor profiles side by side and optionally in two desktop windows. Give each surface independent size, density, frame clock, visibility, and connection controls.

Deliver:

- optimized bundle, startup, focus-update, canvas, cleanup, accessibility, and build-time results for the two renderer spikes;
- an accepted renderer boundary that keeps reactive and browser types out of the runtime core;
- hot reload for experience code and theme tokens;
- 120/60, 60/60, slow peer, disconnected peer, resize, and suspend presets;
- controller, keyboard, mouse, and emulated touch input;
- frame timing, protocol, action, and projection diagnostics;
- trace record and replay through the visual host.

Exit gate:

- Leptos is accepted or rejected with measured reasons and a bounded migration seam.
- Every runtime failure behavior can be triggered without Thor hardware.
- Simulator and headless tests share the in-memory adapter rather than duplicate behavior.
- The simulator itself contains no experience-specific policy.

## Milestone 4: Input system

Implement input sampling, normalization, profiles, semantic action maps, focus navigation, touch gestures, chords, repeat, and remapping.

Deliver:

- a measured AYN Thor controller profile with calibration data;
- a standard web gamepad fallback and unknown-controller calibration flow;
- deterministic controller and gesture traces;
- interaction-mode feedback and accessible focus behavior;
- conflict rules for simultaneous controller, touch, and keyboard input.

Exit gate:

- All standard design-system controls work with controller and touch.
- Drift, held inputs, disconnects, reconnects, and lost pointer capture pass tests.
- No experience reads browser gamepad or pointer events directly.

## Milestone 5: Design system alpha

Implement foundations, layout primitives, focus and feedback states, and the initial dual-surface patterns in DOM and CSS.

Deliver:

- DTCG token resolver with CSS, typed Rust, diagnostics, and optional Tailwind output;
- one canonical workspace recipe crate, manifest, and validation task;
- framework-owned control primitives separated from workspace-owned presentation recipes;
- surface-aware layout, typography, color, spacing, radius, and motion;
- buttons, toggles, sliders, lists, tabs, dialogs, text input, notifications, and status;
- main/companion patterns and mandatory single-surface fallbacks;
- visual test catalog at both Thor profiles and common desktop/mobile sizes;
- accessibility checks for semantics, contrast, focus, motion, and touch size.

Exit gate:

- A small productivity experience uses only public design-system interfaces.
- A second visual identity can replace tokens and edit recipes without forking control behavior.
- DOM and canvas presentations consume the same resolved semantic tokens.
- Every control has controller, touch, keyboard, disabled, busy, error, and focus behavior.
- Screenshot and interaction tests cover all profiles and themes.

## Milestone 6: Game presentation alpha

Add canvas ownership, fixed-step scheduling, interpolation, quality profiles, asset loading, audio policy, overlays, and context recovery without becoming a game engine.

Deliver:

- WebGL2 baseline with an optional WebGPU experiment only if the probe supports it;
- independent render pulses and projection invalidation per surface;
- a canvas viewport that composes with DOM overlays and companion controls;
- resource loading, progress, failure, cache, and context-loss behavior;
- frame and input latency instrumentation.

Exit gate:

- A reference game maintains its measured budgets on the Snapdragon 865 target.
- The main surface can present at 120 Hz while the companion remains correct at 60 Hz.
- Simulation results do not change with render refresh rate.

## Milestone 7: Device host

Turn the selected Milestone 1 topology into a supported host.

For Chrome-only, deliver install, launch, pairing, reconnection, offline, fullscreen, and recovery flows. For Android-assisted, keep the host narrow and load the same built web artifacts used elsewhere.

Exit gate:

- One action launches or clearly guides both surfaces into the session.
- Suspend, resume, display off, peer loss, and experience upgrade recover predictably.
- Browser and Android paths pass the same protocol and experience contract tests.
- Host-specific code contains no experience or design-system policy.

## Milestone 8: Reference experiences and beta

Build one non-game application and one small game as framework consumers. Their purpose is to expose shallow interfaces, missing patterns, duplicated policy, and performance failures.

Exit gate:

- Both examples run dual-surface and single-surface.
- Neither imports private framework modules or raw browser bindings.
- A new experience can be scaffolded, tested, simulated, and installed from written instructions.
- A disposable second workspace decides whether source installation is real enough to justify the registry CLI.
- Public interfaces receive a strict maintainability review before beta tagging.

## Milestone 9: Release hardening

Add compatibility policy, migration notes, diagnostics export, package documentation, reproducible builds, supply-chain checks, and release automation.

Exit gate:

- Supported device, Chrome, and Android versions are explicit.
- Protocol and saved-state migrations are tested across supported versions.
- Performance, accessibility, offline, lifecycle, and hardware suites are green.
- A clean checkout produces the documented release artifacts.

## Recommended goal-loop order

1. Complete Milestone 1 before scaffolding the full workspace.
2. Complete Milestones 2 and 3 before implementing polished controls.
3. Complete Milestone 4 before reference experience behavior grows around raw input.
4. Develop Milestones 5 and 6 as separate vertical slices over the same runtime.
5. Start the final host only after the execution topology is measured and recorded.
