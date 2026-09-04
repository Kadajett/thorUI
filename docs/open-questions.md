# Open Product Decisions

These questions affect product scope but do not block the capability lab. Each has a provisional default so work can continue without silently inventing intent.

## Distribution

Question: Must the final experience run in stock Chrome with no installed APK, or is a small Android host acceptable?

Provisional default: ship the same installable web build inside a narrow two-Activity Android host for reliable dual-display launch, while retaining direct Chrome use for development and single-surface fallback. Current Chromium documentation does not support web-controlled window placement on Android.

Decision point: after the Milestone 1 Chrome and Android launch probes.

## Framework audience

Question: Is ThorUI primarily for this repository's experiences or a public framework for unrelated authors?

Provisional default: design public-quality interfaces, but optimize the first release for experiences maintained in the same workspace. Extract and publish packages only after both reference experiences validate the seam.

Decision point: before beta packaging in Milestone 8.

## Device scope

Question: Is support limited to the AYN Thor or expected for other dual-screen Android devices?

Provisional default: guarantee the Thor and keep hardware facts in device profiles. Other hardware is best-effort until it contributes a measured capability report.

Decision point: when a second physical device is available.

## Graphics integration

Question: Should the first game path target raw WebGL2, `wgpu`, or an existing Rust game engine?

Provisional default: keep the runtime graphics-neutral, prove WebGL2 on the lowest-end Thor, and evaluate `wgpu` only after the capability report. Do not make a full game engine part of ThorUI.

Decision point: at the start of Milestone 6.

## UI renderer

Question: Should the DOM adapter use Leptos or a more actively evolving Rust UI framework?

Provisional default: use pinned Leptos 0.8 client-side rendering behind a narrow adapter. Its fine-grained DOM model fits ThorUI, but its May 2026 light-maintenance announcement makes Dioxus 0.7 a required acceptance-spike challenger.

Decision point: at the start of Milestone 3, after measuring both spikes on the Milestone 1 device topology.

## Styling identity

Question: Should the visual language closely echo AYN's Android launcher or establish an independent ThorUI identity?

Provisional default: create a quiet, OLED-aware identity based on handheld ergonomics, with replaceable brand tokens. Copy interaction expectations, not proprietary artwork.

Decision point: before design-system visual work in Milestone 5.

## Hardware access

Question: Which Thor variant, firmware, Chrome version, and Android System WebView version are available for acceptance testing?

Provisional default: treat the Snapdragon 865 model as the performance floor and record every tested software version in capability reports.

Decision point: before Milestone 1 is considered complete.

## Licensing and release

Question: Will the framework be open source, and under which license?

Provisional default: keep dependencies compatible with a permissive future release, but add no license banner until the owner chooses one.

Decision point: before accepting dependencies that constrain distribution.
