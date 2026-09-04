# Rust/WASM UI stack research

Status: decision input  
Evidence checked: 2026-09-04  
Scope: offline CSR UI in two Android Chromium WebViews, with DOM controls around an independently rendered canvas

## Recommendation

Use stable Leptos 0.8 CSR as the provisional UI renderer, behind a narrow `thorui_ui_web` boundary. Give Dioxus 0.7.10 one bounded acceptance test before that choice becomes permanent. Do not let either framework own session state, timing, input policy, or the surface protocol.

Build the design system as two layers:

1. A centralized ThorUI behavior crate owns state machines, accessible interaction, focus, overlays, controller activation, and semantic actions.
2. Workspace-owned Leptos components and CSS own markup and appearance. They are editable like shadcn/ui recipes, but they delegate behavior instead of copying it.

This preserves shadcn-style control without duplicating bug-prone behavior. A future registry should distribute thin presentation recipes, tokens, and patterns. It should not copy independent dialog, menu, or focus implementations into every application.

The choice remains provisional because Leptos has a real maintenance risk. Its maintainer said in May 2026 that it is “lightly maintained,” largely feature-complete, and not expected to receive major new development without more maintainers ([upstream status](https://github.com/leptos-rs/leptos/issues/4707)). Dioxus is the serious fallback, not an abstract option.

## Facts: current framework choices

| Option | Current stable on 2026-09-04 | Relevant facts | ThorUI reading |
|---|---:|---|---|
| Leptos | 0.8.20; 0.9.0-beta is a prerelease | Fine-grained reactivity updates DOM nodes without a virtual DOM. CSR, arbitrary-element mounting, cleanup handles, typed `web-sys` nodes, and portals with an explicit mount are supported ([crate](https://docs.rs/crate/leptos/latest), [repository](https://github.com/leptos-rs/leptos), [mounting](https://docs.rs/leptos/latest/leptos/mount/index.html), [portal](https://docs.rs/leptos/latest/leptos/portal/fn.Portal.html), [NodeRef](https://docs.rs/leptos/latest/leptos/prelude/struct.NodeRef.html)). MIT. | Best provisional fit for a web-only shell and direct canvas interop. Maintenance status prevents an unconditional lock-in. Use the stable 0.8 line, CSR only, on stable Rust. |
| Dioxus | 0.7.10; 0.8.0-alpha.1 is a prerelease | Uses a virtual DOM and applies mutation edits. Its web configuration can select a root. Direct `web-sys` access is supported for web-only code. The repository is active and MIT/Apache-2.0 ([crate](https://docs.rs/crate/dioxus/latest), [VirtualDom](https://docs.rs/dioxus-core/latest/dioxus_core/struct.VirtualDom.html), [web Config](https://docs.rs/dioxus-web/latest/dioxus_web/struct.Config.html), [escape hatch](https://dioxuslabs.com/learn/0.7/essentials/ui/escape/), [repository](https://github.com/DioxusLabs/dioxus)). | Strong challenger. It has the most relevant first-party open-code component experiment, but that layer is still early. |
| Yew | 0.23.0 | Mature virtual-DOM framework with a custom-root renderer, portals, `web-sys` integration, MSRV 1.84, and MIT/Apache-2.0 licensing ([manifest](https://docs.rs/crate/yew/latest/source/Cargo.toml), [Renderer](https://docs.rs/yew/latest/yew/struct.Renderer.html), [events](https://yew.rs/docs/concepts/html/events), [repository](https://github.com/yewstack/yew)). | Viable, but gives this target no clear advantage over the two finalists and lacks a current first-party shadcn-style path. |
| Direct `web-sys` | 0.3.104 | Generated WebIDL bindings; APIs are heavily feature-gated. `wasm-bindgen` only links used bindings and is MIT/Apache-2.0 ([web-sys](https://docs.rs/crate/web-sys/latest), [wasm-bindgen](https://github.com/wasm-bindgen/wasm-bindgen)). | Keep at the host and canvas seams. Using it for the full UI would make ThorUI invent component lifecycle, rendering, cleanup, and accessible primitives. |

Leptos 0.8 components are setup functions rather than repeatedly rerendered functions ([component macro](https://docs.rs/leptos/latest/leptos/attr.component.html)). Effects are intended for synchronization with external side effects, not as a default state-propagation mechanism ([Effect](https://docs.rs/leptos/latest/leptos/reactive/effect/struct.Effect.html)). These properties fit a projection-in/action-out UI boundary.

The official Leptos CSR path uses Trunk; the full-stack `cargo-leptos` flow is not required ([getting started](https://book.leptos.dev/getting_started/index.html), [CSR starter](https://github.com/leptos-rs/start-trunk)). Leptos documents release size optimization, compression, and the cost of Rust monomorphization; its code splitting is newer and may still have bugs ([binary-size guide](https://book.leptos.dev/deployment/binary_size.html)). ThorUI should not depend on code splitting for an offline first release.

## Facts: what “like shadcn” means

shadcn/ui is not merely a visual library. It copies top-layer component source into the application, while lower-level headless dependencies continue receiving behavior fixes ([official introduction](https://github.com/shadcn-ui/ui/blob/main/apps/v4/content/docs/%28root%29/index.mdx)). In 2026 its default headless base changed from Radix to Base UI while Radix remained supported, demonstrating that behavior and presentation are separable ([change record](https://ui.shadcn.com/docs/changelog/2026-07-base-ui-default)).

Its registry is a flat JSON distribution format that can carry source files, dependencies, CSS, themes, utilities, and other registry items ([registry guide](https://ui.shadcn.com/docs/registry/getting-started), [schema](https://ui.shadcn.com/docs/registry/registry-json), [schema expansion](https://ui.shadcn.com/docs/changelog/2025-02-registry-schema)). `components.json` records style and Tailwind choices; some initialization choices cannot later be changed by configuration alone ([components.json](https://ui.shadcn.com/docs/components-json)). The theming model maps semantic CSS variables such as background/foreground pairs into utilities ([theming](https://ui.shadcn.com/docs/theming)).

ThorUI should copy the ownership boundary, not the React implementation:

- Centralize behavioral invariants in a versioned crate so a fix reaches all consumers.
- Own presentation recipes in the workspace so applications can edit markup, composition, and CSS.
- Keep examples on the canonical presentation crate inside this repository. Do not install duplicate copies internally.
- Add a registry only when a second external consumer exists. Until then, a registry seam would be speculative.
- A later `thorui ui add` should install presentation only. `diff` should compare the recorded base, local copy, and new upstream recipe; `update` must not blindly overwrite local edits.

Every installed recipe should record its schema version, recipe version, compatible behavior-crate range, and source digest. Behavior upgrades use Cargo and contract tests. Presentation upgrades remain explicit merges.

## Current Rust component ecosystem

The original RustForWeb shadcn and Radix ports were archived on 2026-02-02 and are read-only ([shadcn port](https://github.com/RustForWeb/shadcn-ui), [Radix port](https://github.com/RustForWeb/radix)). They are not adoption candidates.

[Rust/UI](https://github.com/rust-ui/ui) is the closest Leptos copy-source project. At the inspected 2026-09-02 commit it was active and MIT licensed. Its `ui-cli` 0.3.16 exposes add/view/diff/update commands, while `leptos_ui` 0.3.22 and `tw_merge` provide helpers ([CLI crate](https://crates.io/crates/ui-cli), [Leptos helpers](https://docs.rs/leptos_ui/latest), [`tw_merge`](https://docs.rs/tw_merge/latest/tw_merge/macro.tw_merge.html)). It currently supports Leptos and deliberately implements components without a separate headless behavior dependency ([introduction](https://github.com/rust-ui/ui/blob/main/public/docs/introduction.md), [installation](https://github.com/rust-ui/ui/blob/main/public/docs/installation.md)).

That is the wrong behavior boundary for ThorUI. Source inspection also found that its current dialog and tabs recipes do not yet implement the complete WAI-ARIA dialog/tab semantics and keyboard contracts ([dialog source](https://github.com/rust-ui/ui/blob/main/app_crates/registry/src/ui/dialog.rs), [tabs source](https://github.com/rust-ui/ui/blob/main/app_crates/registry/src/ui/tabs.rs)). Its helper crate enables Leptos's `nightly` feature, so importing it would also undermine the stable-Rust baseline ([manifest](https://github.com/rust-ui/ui/blob/main/crates/leptos_ui/Cargo.toml)). Use Rust/UI as registry and presentation research, not as a trusted behavior foundation.

Dioxus has a first-party [Dioxus Components](https://github.com/DioxusLabs/dioxus-components) repository with the desired split: `dioxus-primitives` centralizes unstyled behavior, while `dx components add` copies styled source. It is MIT/Apache-2.0 and has Playwright tests. However, the primitives crate is only 0.0.1, the inspected repository pins Dioxus 0.7.8, and its own [limitations log](https://github.com/DioxusLabs/dioxus-components/blob/main/complaints.md) records unresolved parent/sibling ordering, portal, prop composition, and attribute-injection problems. This is strategically promising but not mature enough to decide the framework by itself.

Other Leptos suites are weaker fits today: [Leptix](https://github.com/RantAI-dev/leptix-ui) is young; [radix-leptos](https://github.com/cloud-shuttle/radix-leptos) is third-party and had no inspected commits after 2025-09-26; [Thaw](https://github.com/thaw-ui/thaw) is a Fluent-styled library whose stable line targets older Leptos. None should define ThorUI's public component contract.

## Tokens, variants, and Tailwind

Use DTCG 2025.10 JSON as the canonical token interchange. The final community report defines typed values, aliases, groups, extension metadata, and group inheritance ([Format Module](https://www.designtokens.org/TR/2025.10/format/)). Its separate final Resolver Module already defines ordered sets plus modifiers and contexts for light/dark and other permutations ([Resolver Module](https://www.designtokens.org/TR/2025.10/resolver/)). These are stable community-group reports, not W3C Recommendations.

This changes the provisional plan slightly: do not invent ThorUI theme modes in a competing format. Use a DTCG resolver document for theme, contrast, density, and surface token contexts. A small ThorUI mode manifest is justified only for runtime policy such as “companion surface selects compact density”; it should select DTCG resolver inputs rather than repeat token values or merge rules.

Generate and check in two products from one resolved token graph:

- namespaced CSS custom properties, scoped below each `SurfaceRoot`;
- typed Rust values for canvas and non-DOM renderers.

Use foundation, semantic, component, surface, then user-override layers. Prefer semantic pairs such as surface/on-surface. Scope theme, contrast, density, and motion with data attributes on each surface root, not only `:root` or `.dark`; the two simulator roots may intentionally use different profiles. Apply a revisioned theme change as one transaction on each peer.

Use typed Rust enums for behavioral or structural variants. Expose `data-slot` and `data-state` for styling. Permit recipe-owned class overrides, but do not turn unvalidated strings into public behavior policy.

Tailwind 4 is a good optional/default build-time adapter, not the design-system source of truth. Tailwind's scanner treats source as plain text and cannot understand dynamically assembled class fragments, so variant code must map to complete literal class strings ([class detection](https://tailwindcss.com/docs/detecting-classes-in-source-files)). Its `@theme` variables can create both utilities and normal CSS variables and can be shared as CSS packages ([theme variables](https://tailwindcss.com/docs/theme)). Pin the local CLI, emit static CSS, make builds work without a network, and keep a plain-CSS path available from the same tokens. Benchmark `tw_merge` before making it a runtime or compile-time dependency.

For icons, prefer a generated project-owned SVG subset. If a crate is more practical, `lepticons` 0.13.1 has opt-in icon-set features ([crate](https://docs.rs/lepticons/latest/lepticons/)); enable only the needed set. Lucide is ISC licensed, with Feather-derived files under MIT ([license](https://github.com/lucide-icons/lucide/blob/main/LICENSE)).

## Accessibility and controller behavior

Start with native HTML. Custom composite widgets must implement their whole keyboard and semantic contract; ARIA does not supply keyboard behavior ([APG keyboard guidance](https://www.w3.org/WAI/ARIA/apg/practices/keyboard-interface/)). For example, a modal dialog requires containment, initial and restored focus, Tab wrapping, Escape handling, an accessible name, `role=dialog`, and `aria-modal=true` ([dialog pattern](https://www.w3.org/WAI/ARIA/apg/patterns/dialog-modal/)). Tabs require tablist/tab/tabpanel roles and arrow-key navigation ([tabs pattern](https://www.w3.org/WAI/ARIA/apg/patterns/tabs/)).

The controller focus graph is an added input policy, not a replacement accessibility tree. Confirm must invoke the same semantic activation as keyboard and touch. Directional navigation updates actual DOM focus when the target is a DOM control. Canvas-only game focus must expose an equivalent DOM status/control surface where needed. Focus ownership and overlays are surface-local; cross-surface moves become explicit protocol actions.

Automated role/name/state and keyboard tests are necessary but insufficient. TalkBack with the exact Android System WebView and physical touch/controller input remains a hardware acceptance probe.

## Two WebViews and canvas

Two Android WebViews mean two documents, two JS/WASM instances, two heaps, two event systems, and two renderer schedulers. Shared artifact bytes do not imply shared live state. The authoritative Rust runtime remains outside UI roots; each root receives immutable projections with revisions and emits semantic actions through an injected port.

Mount one Leptos root per real document. In the desktop simulator, mount two independent roots with `mount_to`. Every root owns an `OverlayHost`; never rely on the Leptos portal default of `document.body`, because that would mix simulator surfaces ([portal API](https://docs.rs/leptos/latest/leptos/portal/fn.Portal.html)). Root teardown must release listeners and animation handles through the returned unmount handle and `on_cleanup` ([cleanup](https://docs.rs/leptos/latest/leptos/prelude/fn.on_cleanup.html)).

Leptos owns the DOM shell, status, controls, and overlays. A renderer adapter owns each canvas and its animation loop. Obtain the element through a typed `NodeRef`, hand it to the adapter, and relinquish it on cleanup. Do not drive frame-by-frame canvas rendering through reactive component updates. Main and companion display cadence are separate scheduler inputs; neither belongs in component state.

Because the artifact is instantiated twice, measure combined memory and cold start, not only one-view WASM size. Exclude SSR, hydration, router, metadata, server, and broad utility features unless a measured need appears. Record raw and compressed WASM, JS glue, CSS, time to first usable UI, two-view peak memory, input-to-visible-update latency, and listener cleanup. Test on the exact WebView build as well as desktop Chromium.

## Acceptance gate

Before locking the UI dependency, implement the same small shell in Leptos 0.8 stable and Dioxus 0.7.10 stable. Do not evaluate their prerelease lines. Yew and direct DOM need no further spike unless both finalists fail.

The shell must include:

- two isolated surface roots and surface-local portals;
- a dialog, tabs, scrollable list, text field, and controller focus movement;
- touch/pointer activation and semantic Confirm/Back actions;
- a canvas viewport with independent animation cadence;
- projection updates that do not move canonical state into the UI;
- DTCG-resolved theme, contrast, density, and per-surface overrides;
- clean mount, unmount, and remount with no duplicate listeners.

Run pure Rust tests for token resolution, variants, focus transitions, and semantic actions. Run browser tests in Chromium for both target viewport profiles, two-root isolation, pointer/touch/controller paths, overlay containment, resizing, and accessibility roles/names/states. Playwright supports accessibility-tree snapshots ([ARIA snapshots](https://playwright.dev/docs/aria-snapshots)); add focused assertions and keyboard matrices rather than trusting snapshots alone. Add screenshots for stable component states, not animation frames.

Accept Leptos only if it meets the hardware-derived size/startup/memory budgets, passes the accessibility contracts, contains all framework types below the web UI boundary, and survives two-root teardown. Select Dioxus if Leptos fails those gates or its maintenance exposure cannot be contained. Recheck the Leptos maintenance issue and Dioxus Components maturity at the milestone gate; do not continuously chase prereleases.

## Remaining device facts

The stack decision cannot establish the exact Android System WebView version, CSS color support, physical CSS-pixel scale, TalkBack behavior, WebGL limits, memory pressure with two live WASM instances, or controller event ordering. Capture those through repeatable Thor probes. They may change budgets and generated CSS targets, but they do not require changing the core ownership model.
