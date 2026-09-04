# ThorUI

ThorUI is a Rust and WebAssembly framework project for dual-screen experiences on the AYN Thor. It will provide a shared runtime, controller and touch input, independent surface timing, a native-feeling design system, and Chrome-compatible web surfaces inside a narrow Android display host when hardware evidence requires one.

Milestone 1 is a renderer-neutral capability lab. It measures the browser and hardware facts needed to choose the supported execution topology before the framework grows around assumptions.

## Capability lab

Prerequisites are Rust 1.96, the `wasm32-unknown-unknown` target, Trunk 0.21.14, Node 24, and pnpm 11.

```sh
pnpm install
cargo install trunk --locked --version 0.21.14
pnpm check
pnpm dev
```

Open the lab with `?surface=main`. “Open companion” starts a same-session peer with `?surface=companion`; both contexts retain their own frame clock and report. The primary guided action runs the browser, frame, controller, and touch capture, then saves the report and returns a receipt. D-pad or stick directions move focus and A activates the focused control.

Optimized builds include `version.json`, an asset manifest with SHA-256 hashes, and raw byte sizes:

```sh
THORUI_CHANNEL=candidate pnpm build
pnpm exec wrangler deploy
THORUI_BASE_URL=https://thorui-demo.example.workers.dev pnpm test:deployed
```

The application repository deploys the `thorui-demo` Static Assets Worker. The sibling infrastructure repository owns the `thorui.yougotserved.dev` custom-domain binding.

## Project references

- [Domain language](./CONTEXT.md)
- [Architecture](./docs/architecture.md)
- [Experience interface options](./docs/interface-options.md)
- [Design system](./docs/design-system.md)
- [Delivery roadmap](./docs/roadmap.md)
- [Engineering rules](./docs/quality.md)
- [Demo delivery](./docs/deployment.md)
- [Thor capture runbook](./docs/hardware-capture.md)
- [Open product decisions](./docs/open-questions.md)
- [Platform research](./docs/research/platform-capabilities.md)
- [Rust UI stack research](./docs/research/rust-ui-stack.md)

## Product stance

- Thor-first, but profiles describe hardware instead of scattering Thor constants.
- Web-first: the same built experience runs in Chrome, the simulator, and Android WebViews.
- Thor-native: a small Android host is the leading option for owning both displays.
- One session can drive two surfaces with different sizes and refresh rates.
- Games and applications share runtime, input, layout tokens, and lifecycle behavior.
- UI recipes are source-owned and configurable while shared primitives preserve interaction rules.
- Starting with the first runnable milestone, the latest successful demo will live at [thorui.yougotserved.dev](https://thorui.yougotserved.dev).
- ThorUI is not a general game engine or a replacement for the browser DOM.
