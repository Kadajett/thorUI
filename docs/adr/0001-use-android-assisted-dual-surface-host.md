---
status: accepted
date: 2026-09-04
---

# Use an Android-assisted dual-surface host

Use a narrow Android host to place one web projection on each Thor display. A Chrome 152 hardware run created a connected background peer, but its companion projection remained on the approximately 1920×1080 main display and never became a usable lower-screen window; Chrome's Window Management probe was also denied.

Chrome remains the development host and supported single-surface fallback. Kotlin owns display discovery, Activity and WebView placement, lifecycle forwarding, and the native message bridge; Rust/WASM continues to own session, input, presentation, and experience policy.

The supported launch shape is one installed app and one session with two Activity-owned WebViews. A verified App Link is the post-install entry point. Android's public display APIs still require a disposable hardware probe before the final host is built. See [Android dual-display host research](../research/android-dual-display-host.md).

ThorUI publishes its reference APK through GitHub Releases. The Android host is reusable framework packaging, not a central ThorUI launcher: each downstream experience supplies its own web artifact, package identity, signing key, links, and release channel.
