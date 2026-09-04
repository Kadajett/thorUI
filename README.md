# ThorUI

ThorUI is a planned Rust and WebAssembly framework for dual-screen experiences on the AYN Thor. It will provide a shared runtime, controller and touch input, independent surface timing, a native-feeling design system, and Chrome-compatible web surfaces inside a narrow Android display host.

This repository is currently a planning baseline. No framework code has been selected or scaffolded yet.

## Plan

- [Domain language](./CONTEXT.md)
- [Architecture](./docs/architecture.md)
- [Experience interface options](./docs/interface-options.md)
- [Design system](./docs/design-system.md)
- [Delivery roadmap](./docs/roadmap.md)
- [Engineering rules](./docs/quality.md)
- [Demo delivery](./docs/deployment.md)
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
