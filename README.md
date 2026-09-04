# ThorUI

ThorUI is a Rust and WebAssembly framework project for dual-screen experiences on the AYN Thor. It is growing toward a shared runtime, controller and touch input, independent surface timing, a native-feeling design system, and web projections inside a narrow Android display host.

The current alpha demo is **Lumen Field**, a controller-and-touch light instrument. The website runs as one responsive projection; the Android APK launches the same built experience on both Thor displays.

## Try the alpha

- Play at [thorui.yougotserved.dev](https://thorui.yougotserved.dev).
- Download the signed APK from the website or the [latest GitHub release](https://github.com/Kadajett/thorUI/releases/latest).
- On Android, allow installation from the browser or file manager that opens the APK.
- Move with the stick or D-pad, hold A to paint, press X to change color, and press B to clear.

The APK is a reference host, not an app store. Each ThorUI game or application will package its own web build, Android identity, signing key, and release.

## Develop locally

Prerequisites are Rust 1.96, the `wasm32-unknown-unknown` target, Trunk 0.21.14, Node 24, and pnpm 11.

```sh
pnpm install
cargo install trunk --locked --version 0.21.14
pnpm check
pnpm dev
```

`pnpm dev` opens Lumen Field. Use `?surface=companion` for the companion layout. The original hardware probe remains available at `?mode=lab&surface=main`.

Optimized builds include `version.json`, an asset manifest with SHA-256 hashes, and raw byte sizes:

```sh
THORUI_CHANNEL=candidate pnpm build
pnpm exec wrangler deploy
THORUI_BASE_URL=https://thorui-demo.example.workers.dev pnpm test:deployed
```

The repository deploys the `thorui-demo` Static Assets Worker. Its Android APK is a reference app distributed through this repository's GitHub Releases. Framework users package and publish their own applications and games; ThorUI is not an app store or shared launcher.

Build or install the Android host with the [Android host runbook](./docs/android-host.md).

## Project references

- [Domain language](./CONTEXT.md)
- [Architecture](./docs/architecture.md)
- [Android-assisted host decision](./docs/adr/0001-use-android-assisted-dual-surface-host.md)
- [Experience interface options](./docs/interface-options.md)
- [Design system](./docs/design-system.md)
- [Delivery roadmap](./docs/roadmap.md)
- [Engineering rules](./docs/quality.md)
- [Demo delivery](./docs/deployment.md)
- [Android host and APK](./docs/android-host.md)
- [Thor capture runbook](./docs/hardware-capture.md)
- [Open product decisions](./docs/open-questions.md)
- [Platform research](./docs/research/platform-capabilities.md)
- [Android dual-display host research](./docs/research/android-dual-display-host.md)
- [First Thor capture](./docs/research/thor-capture-2026-09-04.md)
- [Rust UI stack research](./docs/research/rust-ui-stack.md)

## Product stance

- Thor-first, but profiles describe hardware instead of scattering Thor constants.
- Web-first: the same built experience runs in Chrome, the simulator, and Android WebViews.
- Thor-native: a small Android host is the leading option for owning both displays.
- One session can drive two surfaces with different sizes and refresh rates.
- Games and applications share runtime, input, layout tokens, and lifecycle behavior.
- UI recipes are source-owned and configurable while shared primitives preserve interaction rules.
- The demo APK proves the reusable Android host; downstream experiences own their package, signing, and distribution.
- Starting with the first runnable milestone, the latest successful demo will live at [thorui.yougotserved.dev](https://thorui.yougotserved.dev).
- ThorUI is not a general game engine or a replacement for the browser DOM.
