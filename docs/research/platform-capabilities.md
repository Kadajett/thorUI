# AYN Thor platform capabilities

Research status: 2026-09-04. This report separates published platform facts from behavior that must be measured on a real Thor. Primary sources are used wherever they exist.

## Decision summary

The Thor is sold as an Android 13 device with two touch OLED panels, but the published specifications do not say how the lower panel is exposed to Android or Chrome. Its advertised panel modes are not proof that both panels can render at 120/60 Hz at the same time.

Chrome Android is the main constraint. Chromium's Window Management implementation is desktop-only, so a normal Chrome tab or installed PWA cannot be designed around `getScreenDetails()`, cross-display window placement, or `requestFullscreen({ screen })`. The web Presentation API also does not promise access to a built-in lower panel.

Android has display enumeration and targeted Activity launch APIs. If AYN exposes the lower panel as an eligible logical display, a narrow Android host with one Activity/WebView per display is the most controllable fallback. This is still conditional on device probes. Android 13's native `Presentation` class is not a generic fallback for an internal panel: Android documents built-in presentation-display support as beginning with Android 16. Thor community projects report that `Presentation` does work on the lower panel, which suggests AYN may expose it as a presentation-capable non-internal display or apply vendor behavior. That report must be confirmed on the target firmware.

The framework should therefore keep one session model and two host choices until the hardware lab resolves the topology:

1. two same-origin Chrome contexts, only if launch, placement, activity, and recovery work on the Thor;
2. two WebViews placed by a small Android host, if Chrome cannot sustain the experience.

In both cases, use one session authority, message-based surface peers, one render loop per surface, and a fixed simulation clock independent of rendering.

For the requested 3DS-emulator feel, the strongest candidate is a native Android launch and lifecycle shell containing two fullscreen WebViews. Native placement supplies the console-like behavior; the UI and game/application layer remain the same Rust/WASM web artifact on both surfaces. A native graphics surface can be added later for measured renderer limits, but it does not solve display placement by itself.

## Confidence labels

- **Device specification**: published by AYN or in AYN's FCC-filed manual.
- **Platform fact**: required or documented by Android, Chromium, or a web standard.
- **Probe required**: not established for the Thor, its AYN firmware, or its installed browser build.
- **Design inference**: a conservative framework rule derived from the facts; not a platform guarantee.

## Published Thor baseline

| Property | Published value | Confidence and consequence |
| --- | --- | --- |
| Main surface | 6-inch AMOLED touch panel; 1080 × 1920 pixels; 120 Hz | **Device specification.** The manual lists portrait dimensions. The expected landscape buffer is 1920 × 1080, but CSS viewport and device-pixel ratio still require measurement. |
| Companion surface | 3.92-inch AMOLED touch panel; 1080 × 1240 pixels; 60 Hz | **Device specification.** The expected landscape buffer is 1240 × 1080. Do not hard-code these as CSS pixels. |
| Operating system | Android 13 | **Device specification.** AYN firmware behavior can differ from stock AOSP policy. |
| Compute variants | Lite: Snapdragon 865, 8 GB RAM; Base/Pro/Max: Snapdragon 8 Gen 2, 8–16 GB RAM | **Device specification.** The Snapdragon 865 model is the sensible performance floor. |
| Input hardware | Two Hall sticks, D-pad, ABXY, L1/L2/R1/R2, L3/R3, Start, Select, Home, Return; both panels are touch | **Device specification.** Browser button numbers, axis order, trigger ranges, reserved buttons, and haptics are not published. |
| Power and expansion | 6000 mAh battery, active cooling, USB-C, microSD/TF, headphone jack, display output | **Device specification.** Thermal policy and running two GPU contexts remain unknown. |

Sources: the [AYN Thor product page](https://www.ayntec.com/products/ayn-thor) confirms Android 13, both touch panels, SoC/memory variants, battery, and Hall sticks. AYN's [FCC-filed user manual](https://fccid.io/2BDXNBASE/User-Manual/UserManual-8915262) supplies the exact panel resolutions, refresh ratings, physical controls, ports, and charging/display-output details; the [FCC filing overview](https://fccid.io/2BDXN-BASE) identifies the applicant and device.

These sources do **not** establish:

- whether Android exposes the companion surface as a second logical `Display`, a vendor-managed region, or something else;
- whether Chrome can open or move a browsing context onto it;
- whether 120 Hz main and 60 Hz companion presentation can run concurrently;
- the CSS viewports, safe areas, density values, color modes, or orientation policy;
- the Gamepad API identity and mapping of the built-in controller;
- behavior when the hinge closes, one panel is disabled, power saver is active, or the device becomes hot.

## Android display and Activity behavior

Android represents attached output surfaces as logical displays. `DisplayManager.getDisplays()` enumerates them, while a listener reports added, removed, and changed displays. `DISPLAY_CATEGORY_PRESENTATION` is a filtered subset, not an alias for every secondary display. See the [AOSP multi-display overview](https://source.android.com/docs/core/display/multi_display) and [`DisplayManager`](https://developer.android.com/reference/android/hardware/display/DisplayManager).

### Targeted Activity launch

[`ActivityOptions.setLaunchDisplayId()`](https://developer.android.com/reference/android/app/ActivityOptions#setLaunchDisplayId(int)) can request a display from API 26. It is ignored when the device lacks `FEATURE_ACTIVITIES_ON_SECONDARY_DISPLAYS`; private or disallowed targets can fail. API 29 adds [`isActivityStartAllowedOnDisplay()`](https://developer.android.com/reference/android/app/ActivityManager#isActivityStartAllowedOnDisplay(android.content.Context,int,android.content.Intent)) for a preflight check.

Without an explicit target, Android normally launches an Activity on the caller's display. Existing tasks, launch modes, and intent flags can cause Android to reuse an instance on another display. A host that needs two simultaneous instances should use ordinary multi-instance launch behavior and verify the actual display after launch. The [AOSP launch policy](https://source.android.com/docs/core/display/multi_display/activity-launch) and [multi-display FAQ](https://source.android.com/docs/core/display/multi_display/faq) describe these rules.

Android 10 and later can keep top-visible Activities on several displays in the `RESUMED` state at once. Only one Activity is top-resumed for exclusive resources. Lifecycle code must not treat `onPause()` as synonymous with invisible or `RESUMED` as synonymous with sole focus. See [multi-resume](https://source.android.com/docs/core/display/multi_display/multi-resume).

Moving an Activity between displays, changing a display mode, or changing density can update configuration and recreate the Activity unless state is restored correctly. Each Activity must use its display-specific context and current metrics. See [configuration changes](https://developer.android.com/guide/topics/resources/runtime-changes) and [configuration continuity](https://developer.android.com/guide/topics/large-screens/configuration-and-continuity).

### Native `Presentation` is not the web Presentation API

[`android.app.Presentation`](https://developer.android.com/reference/android/app/Presentation) is a display-bound `Dialog` with display-specific resources. It is canceled if the target display or associated task disappears.

The same Android reference now states that built-in internal displays can carry `FLAG_PRESENTATION` starting with Android 16 (`BAKLAVA`) and were unsuitable on earlier releases. Since the Thor ships with Android 13, a native `Presentation` can only be considered if the AYN firmware reports the companion surface as a suitable non-internal presentation display or adds vendor behavior. **Probe required.** Community code reports this topology on Thor, but it is not an official AYN API contract.

### Native host options for an emulator-style experience

| Host shape | What official APIs establish | Can both surfaces be fullscreen and interactive? | Fit for Rust/WASM UI |
| --- | --- | --- | --- |
| Two Activities, one WebView each | Android can target an eligible display with `setLaunchDisplayId()`, can keep visible Activities resumed, and lets each Activity use its own display context. Each Activity can request [immersive system-bar hiding](https://developer.android.com/develop/ui/views/layout/immersive). | **Yes in principle, probe required on Thor.** Each surface has its own window, lifecycle, WebView, focus, and touch target. OEM display permission and input routing remain gates. | **Best first candidate.** Both WebViews load the same Rust/WASM build with different assigned roles. A native bridge owns placement and coordination only. |
| One Activity plus `Presentation` containing a WebView | `Presentation` is a display-bound Dialog with its own display context, but pre-Android 16 internal-display use is rejected by generic Android. It remains tied to the containing task. | **Conditional.** It can fill a presentation-capable display and host interactive Views, but independent focus, lifecycle, fullscreen, and lower-panel eligibility must be proven. | Viable if Thor probes confirm AYN's presentation-display behavior. It is a shallower shell, but lifecycle coupling is easier to get wrong. |
| Two Activities with native `SurfaceView`/`ANativeWindow`, optionally plus WebView UI | [`SurfaceView`](https://developer.android.com/reference/android/view/SurfaceView) and [`ANativeWindow`](https://developer.android.com/ndk/reference/group/a-native-window) provide native drawing surfaces inside an Android window. They do not create or place that window on another display. | **Yes only through the same Activity placement path.** The native renderer changes graphics ownership, not display eligibility, focus, touch routing, or immersive policy. | A later hybrid option if WebGPU/WebGL measurements fail. Web UI can remain in WebViews, but a native renderer adds a second build/runtime boundary and resource synchronization. |
| One native canvas/compositor trying to span both panels | Android exposes separate logical displays and windows rather than one portable app canvas spanning them. No AYN developer API for a single cross-panel surface was found. | **Not established.** A compositor cannot bypass WindowManager policy from an ordinary app. | Reject as a baseline. It would be vendor-specific even if possible and would weaken the shared web artifact model. |

The two-Activity/WebView path most directly matches “launch once and populate both panels.” The host should start both Activities, assign main/companion roles from observed display properties, apply immersive policy in both windows, and let the WASM session protocol handle state and recovery. It must still degrade to one Activity when the second launch is denied or the companion surface disappears.

### Focus, touch, and keyboard routing

Android supports per-display focus only when the OEM enables it for the device. General multi-display behavior can retain one globally focused display. Targeted touch devices are associated with displays by system/OEM input-port configuration, while non-targeted input such as a keyboard follows focus. The app cannot assume both touch panels are wired correctly or that the controller is delivered to both Activities. See [display focus](https://source.android.com/docs/core/display/multi_display/displays#per-display_focus), [input routing](https://source.android.com/docs/core/display/multi_display/input-routing), and [IME policy](https://source.android.com/docs/core/display/multi_display/ime-support).

System bars, launchers, and the input method on secondary displays are also OEM-configurable. Fullscreen and text entry must be tested on each surface rather than inferred from AOSP defaults. See [system decorations](https://source.android.com/docs/core/display/multi_display/system-decorations).

## Chrome and web capability matrix

| Capability | Verified browser/platform fact | Thor status and framework rule |
| --- | --- | --- |
| Window Management | The API can enumerate screens, observe changes, place windows, and request fullscreen on a selected screen after permission. [Chrome's guide](https://developer.chrome.com/docs/capabilities/web-apis/window-management) documents the surface. The [ChromeStatus entry](https://chromestatus.com/feature/5252960583942144) lists the shipped implementation for desktop Chrome, not Android or WebView. | **Unsupported as a Chrome Android dependency.** Feature-detect `getScreenDetails`, but absence is the expected baseline. A PWA install does not grant native display APIs. |
| Web Presentation | A controlling page can ask the user agent to start a presentation and communicates with a receiver through `PresentationConnection`. Starting normally requires transient activation and endpoint selection is user-agent controlled. See the [W3C Presentation API](https://www.w3.org/TR/presentation-api/) and [Chrome's Android/Cast article](https://developer.chrome.com/blog/presentation-api). | **Probe required.** Android support for Cast/controller use does not promise that the Thor companion panel appears as a local endpoint or that a receiver can launch there. Do not make this the only launch path. |
| Fullscreen | `requestFullscreen()` needs a fully active document and normally transient user activation. The user agent may deny or later exit fullscreen. See the [Fullscreen standard](https://fullscreen.spec.whatwg.org/). Selecting a target screen is an extension of Window Management. | Single-context fullscreen is usable with fallbacks. Simultaneous fullscreen, browser chrome, focus changes, and gesture requirements on two Thor contexts need probes. A WebView host must implement its own custom-view handling; see [`WebChromeClient`](https://developer.android.com/reference/android/webkit/WebChromeClient#onShowCustomView(android.view.View,android.webkit.WebChromeClient.CustomViewCallback)). |
| Gamepad | The [Gamepad specification](https://w3c.github.io/gamepad/) exposes button and axis snapshots through `navigator.getGamepads()`. Data may be withheld until gamepad interaction, and `mapping` is only `standard` when the user agent recognizes the layout. Chromium contains an [Android gamepad fetcher and mapping path](https://chromium.googlesource.com/chromium/src/+/lkgr/device/gamepad/gamepad_platform_data_fetcher_android.cc). | Poll during active host frames. **Probe required:** ID, mapping, every axis/button, analog triggers, dead zones, system-reserved buttons, hot-plug, focus routing, and vibration. Select one input authority so two contexts cannot apply one press twice. |
| Pointer Events | The [Pointer Events standard](https://www.w3.org/TR/pointerevents3/) unifies touch, pen, and mouse, defines pointer capture, and makes pointer IDs local to a top-level context. `touch-action` declares which gestures the browser may handle; cancellation must be supported. | Good baseline for both touch panels. **Probe required:** simultaneous contacts across panels, contact count, capture loss, gesture cancellation, pressure/geometry quality, latency, and routing to the correct context. |
| Screen Wake Lock | The [Screen Wake Lock standard](https://w3c.github.io/screen-wake-lock/) requires a secure, fully active, visible document. A lock is advisory; the user agent or OS can release or deny it for visibility, power, or policy reasons. [Chrome's guide](https://developer.chrome.com/docs/capabilities/web-apis/wake-lock) documents reacquisition after visibility changes. | Listen for release and reacquire only when the session needs it. **Probe required:** whether one visible context keeps both panels awake, whether both contexts need locks, and behavior under power saver, hinge, suspend, and focus changes. |
| PWA install and offline | A manifest and HTTPS provide install presentation; install is not offline support. Service workers can serve cached resources, while Cache Storage/IndexedDB remain quota-bound and may be evicted. See [install criteria](https://web.dev/articles/install-criteria), [service-worker caching](https://web.dev/learn/pwa/caching/), and [offline storage](https://web.dev/learn/pwa/offline-data). | Build an explicit versioned offline cache and recovery path. A service worker is short-lived and must not own the live simulation. Installed display mode can reduce browser UI, but it cannot select the companion display. Offline launch and update behavior need device tests. |

### Chrome Android Window Management limitation

This is the topology gate, not a small missing convenience. Without Window Management, web content cannot:

- enumerate the Thor panels as separate screens;
- programmatically choose the companion panel for `window.open()`;
- move a window there using stable screen geometry;
- target that panel with `requestFullscreen({ screen })`.

Ordinary `window.open()` may still be useful if AYN added vendor behavior or if a user can move a window manually, but neither is a portable web guarantee. The capability lab must test the exact Chrome build. Until then, Chrome-only dual placement is an unverified option.

## Two-context coordination

### Same-origin browser contexts

- `window.postMessage()` plus `MessageChannel` works well when the contexts have a retained opener/receiver relationship. Validate exact origins and message schemas.
- [`BroadcastChannel`](https://html.spec.whatwg.org/multipage/web-messaging.html#broadcasting-to-other-browsing-contexts) reaches eligible same-origin, same-storage-key contexts and workers. It has no latency or frame-deadline guarantee. Add peer IDs, sequence numbers, revisions, acknowledgements, and resynchronization.
- A `SharedWorker` can be an authority or relay where supported, but still communicates through message ports and has browser-managed lifetime. Android support has changed over time; use feature detection and test the exact Chrome/System WebView build. See the [ChromeStatus entry](https://chromestatus.com/feature/6265472244514816).
- A service worker can relay durable events and support offline loading, but its lifecycle is event-driven. It is not a frame clock or dependable live authority.
- A local or remote WebSocket can coordinate contexts that do not share a browser storage partition, but it adds a service dependency and is outside the offline local baseline.

Message transport should carry timestamped input and complete or revisioned projections. It should not attempt to synchronize `requestAnimationFrame` callbacks or mutate shared session state from both contexts.

### Android-hosted WebViews

A WebView is a View owned and placed by an Android Activity. `window.open()` does not select an Android display. WebView multiple-window support is off by default; if enabled, the host receives [`onCreateWindow()`](https://developer.android.com/reference/android/webkit/WebChromeClient#onCreateWindow(android.webkit.WebView,boolean,boolean,android.os.Message)), creates the new WebView, and inserts it into an Activity's hierarchy. See [`setSupportMultipleWindows()`](https://developer.android.com/reference/android/webkit/WebSettings#setSupportMultipleWindows(boolean)).

For the Thor, the conservative host is:

1. enumerate Android displays and verify launch permission;
2. launch one ordinary Activity with one WebView on each eligible display;
3. keep session authority in exactly one place;
4. relay typed protocol messages through a native-owned channel;
5. let each WebView render the same built web artifact with a different assigned surface role.

Do not depend on two WebViews discovering each other through BroadcastChannel until tested. Android recommends tightly scoping any native bridge to trusted content; unsafe `addJavascriptInterface` exposure can let untrusted frames call native code. Prefer origin-checked message ports or a minimal typed bridge. See [WebView native-bridge risks](https://developer.android.com/privacy-and-security/risks/insecure-webview-native-bridges) and [building WebView apps](https://developer.android.com/develop/ui/views/layout/webapps/webview).

Android System WebView uses Chromium but is updated separately and does not share Chrome's app data. Browser and WebView results must be recorded as separate device profiles. See the [Android WebView overview](https://developer.chrome.com/docs/webview).

## Thor community evidence, kept separate

The following projects are useful feasibility signals, not platform contracts. They are not AYN documentation, Android compatibility tests, or results captured by this project:

- [Magni](https://github.com/KuriGohan-Kamehameha/magni) describes a Thor browser with `OverviewActivity` on one display, `ZoomActivity` and an interactive WebView on the other, and placement through `ActivityOptions.setLaunchDisplayId()`. This supports testing the two-Activity host first.
- [Ayn Dual Screen](https://github.com/Abacus1829/ayn-dual-screen/) describes a fullscreen WebView shell that launches itself onto the Thor lower display and keeps it awake. It explicitly notes that a normal browser page cannot perform the launch itself.
- [Thor Launcher](https://github.com/Prof-Mags/Thor-Launcher/blob/fix/dual-screen-focus-and-home/docs/DESIGN.md) reports using `Presentation` on the Thor. Its design notes describe real lifecycle and focus failures when the second composition depended on the first Activity. If accurate, this is evidence of AYN-specific presentation eligibility, not a contradiction that changes the generic Android 13 rule.
- [Screen Switcher](https://github.com/yassergamedev/screen-swapper) reports moving Thor tasks between displays through Shizuku/ADB privileges and warns that behavior varies by app. This demonstrates two Android task destinations, but not an API an ordinary packaged app should require.

These reports justify the native-host probes and show that fullscreen interactive lower-panel content is plausible. They do not verify Chrome Window Management, simultaneous 120/60 Hz presentation, WebView cross-context messaging, controller mapping, or stable behavior across Thor firmware versions. No community claim should enter a device profile until the capability lab reproduces it and records the environment.

### Shared memory and Rust/WASM threads

`SharedArrayBuffer` and WebAssembly threads require cross-origin isolation on Chrome Android. That normally means `Cross-Origin-Opener-Policy: same-origin` plus `Cross-Origin-Embedder-Policy: require-corp` or a valid alternative. This affects popups and all cross-origin assets. See [Chrome's SharedArrayBuffer requirements](https://developer.chrome.com/blog/enabling-shared-array-buffer) and [COOP/COEP guidance](https://web.dev/articles/coop-coep).

**Design inference:** do not require WASM threads or shared memory for the first host. They do not solve display placement, do not create shared GPU resources across contexts, and make deployment stricter. A message-based protocol works for Chrome contexts, WebViews, tests, and a single-context simulator.

## Refresh scheduling and two different panel modes

The [HTML rendering model](https://html.spec.whatwg.org/multipage/webappapis.html#update-the-rendering) deliberately does not guarantee that `requestAnimationFrame` runs at a fixed rate. The user agent chooses rendering opportunities based on hardware refresh, document visibility, load, and other conditions. It may skip opportunities, and different documents need not receive the same opportunities even when they share an event loop.

`requestAnimationFrame` therefore guarantees a callback opportunity before a possible rendering update, not physical scan-out, exact 120/60 Hz cadence, cross-display phase lock, or one callback per panel refresh.

AOSP's Android 10 multi-display FAQ described all displays as driven from the default display's VSync at that time. Android later added richer refresh-rate APIs, but those documents do not prove how AYN's Android 13 compositor schedules the two built-in panels. See the [AOSP FAQ](https://source.android.com/docs/core/display/multi_display/faq) and [multiple refresh-rate overview](https://source.android.com/docs/core/graphics/multiple-refresh-rate).

**Design inference:**

- each surface peer owns its own `requestAnimationFrame` loop;
- simulation uses a monotonic, fixed step and bounded catch-up independent of rendering;
- each render samples the latest authoritative state and may interpolate;
- a 120 Hz-capable main surface may render twice per 60 Hz simulation step;
- the 60 Hz companion surface renders only on its own observed opportunities;
- either peer may skip stale projections without dropping input or effect results;
- resize, density, visibility, and measured cadence are runtime facts, not device constants.

The exact Thor behavior remains **probe required**. Measure callback intervals and presentation evidence with both panels active, each panel disabled, the main panel set to each available mode, both contexts focused/unfocused, fullscreen/windowed, PWA/tab, and Android WebView. Specifically test reports that enabling both panels may clamp or disturb the advertised modes; no primary source confirms or denies that behavior.

## Resolution and density

Panel pixels are not CSS pixels. Android density scaling and browser UI can change layout viewport size; fullscreen, orientation, safe areas, and the on-screen keyboard can change it again. Android's [WebView viewport guidance](https://developer.android.com/develop/ui/views/layout/webapps/targeting-screens) explains CSS-pixel scaling.

Build each surface profile from observed `innerWidth`, `innerHeight`, `devicePixelRatio`, `screen`, `VisualViewport`, orientation, safe-area insets, and canvas drawing-buffer limits. Resize the drawing buffer deliberately and allow a per-surface render scale. Never identify a surface from one hard-coded resolution alone.

## WebGPU and WebGL

Chrome enabled WebGPU by default in Chrome 121 on Android 12 or later for an initial set of Qualcomm and Arm GPUs. The Thor's OS and both published SoCs fit that broad hardware class, but adapter creation can still fail due to browser version, driver, blocklist, disabled acceleration, or resource pressure. See [WebGPU on Android in Chrome 121](https://developer.chrome.com/blog/new-in-webgpu-121) and [Chrome's troubleshooting guide](https://developer.chrome.com/docs/web-platform/webgpu/troubleshooting-tips).

The [WebGPU standard](https://gpuweb.github.io/gpuweb/) requires callers to query adapter/device limits and optional features and handle `device.lost`. Two independent contexts must be treated as two renderer instances with separately created GPU resources.

[`canvas.getContext("webgl2")`](https://registry.khronos.org/webgl/specs/latest/2.0/) may return `null`; WebGL contexts can also be lost and restored. The renderer must rebuild resources after restoration and choose drawing-buffer size from actual limits. Portable cross-context GPU sharing is not available; the Khronos proposal for [shared WebGL resources](https://registry.khronos.org/webgl/extensions/rejected/WEBGL_shared_resources/) was rejected.

**Design inference:** use a capability ladder, not a GPU brand check:

1. WebGPU when adapter/device creation and required limits succeed;
2. WebGL 2 as the portable game-rendering baseline;
3. DOM/CSS or Canvas 2D for applications, companion controls, diagnostics, and fallback.

Keep canonical state and recoverable asset data outside the GPU backend. Create renderer resources per surface. Allow different resolution/quality policy per surface, and handle WebGPU device loss or WebGL context loss without corrupting session state.

**Probe required:** WebGPU in Chrome and System WebView, simultaneous adapters/devices, WebGL 2 extensions and limits, maximum stable drawing buffers, context/device loss, total GPU memory pressure, shader compilation, thermal throttling, and behavior when either surface changes mode or disappears.

## Mandatory Thor capability probes

These probes decide architecture. They should emit a versioned capability report with the exact Thor model, firmware build, Android build, Chrome version, and System WebView version.

### P0: execution topology

1. Run `DisplayManager.getDisplays()` and `getDisplays(DISPLAY_CATEGORY_PRESENTATION)`. Record ID, name, flags, type, state, current/supported modes, nominal refresh rate, size, density, rotation, and whether IDs survive reboot.
2. Check `FEATURE_ACTIVITIES_ON_SECONDARY_DISPLAYS`, call `isActivityStartAllowedOnDisplay()`, then launch an ordinary second Activity onto every candidate display. Record actual placement and lifecycle.
3. Attempt native `Presentation` separately and record the expected Android 13 internal-display failure or any AYN-specific success.
4. In Chrome and installed PWA, record `getScreenDetails`, `screen.isExtended`, `window.open`, web Presentation endpoint discovery, fullscreen, placement, PWA launch, and recovery behavior.
5. Determine whether two Chrome contexts stay visible and receive frames concurrently. Repeat with two WebViews.

Useful official diagnostics include `adb shell dumpsys display`, `adb shell dumpsys input`, and SurfaceFlinger display IDs. AOSP documents them in the [multi-display test environment](https://source.android.com/docs/core/display/multi_display/testing-dev-environment).

### P0: refresh and lifecycle

1. Capture several minutes of `requestAnimationFrame` interval histograms per surface for every supported mode and host topology.
2. Repeat with both surfaces active, only one active, focus transferred, browser UI shown, fullscreen, power saver, thermal load, suspend/resume, hinge changes, and display off/on.
3. Record Page Visibility, focus, Activity lifecycle, dropped/long frames, resize, density, and configuration recreation beside the frame samples.
4. Confirm whether 120 Hz main plus 60 Hz companion is truly simultaneous. Treat the panel ratings only as maxima until this passes.

### P0: input

1. Record every controller button and axis from the Gamepad API and native Android input, including triggers, stick clicks, Home/Return, disconnect/reconnect, and vibration.
2. Test which contexts receive the same controller sample under every focus arrangement; confirm that the protocol applies it once.
3. Test concurrent touch on both panels, maximum contacts, pointer capture/loss, edge gestures, browser navigation gestures, palm behavior, and input latency.
4. Test controller plus touch at the same time, hardware-key focus, and soft-keyboard location.

### P1: transport, storage, power, and graphics

1. Measure BroadcastChannel, MessageChannel, SharedWorker, and native-bridge delivery, ordering, latency, suspension, reconnect, and version mismatch.
2. Verify service-worker offline launch, cache migration, storage quotas, persistence result, eviction recovery, and upgrade rollback.
3. Test wake-lock release and reacquisition with two surfaces, focus changes, power saver, hinge, sleep, and Activity recreation.
4. Record WebGPU/WebGL capability and loss behavior in one and two contexts, including sustained load on the Snapdragon 865 model.
5. Measure CSS viewport, device-pixel ratio, safe areas, canvas limits, color characteristics, and usable touch target size on each surface.

## Planning constraints established by this research

- Do not select Chrome-only or Android-assisted deployment until P0 probes are captured on a real Thor.
- Do not make Window Management a required Android web API.
- Do not assume the web Presentation API exposes the companion surface.
- Do not assume Android `Presentation` works on the Thor's built-in companion panel under Android 13.
- Do not assume physical panel resolution equals CSS viewport or canvas buffer size.
- Do not assume advertised 120/60 Hz modes are simultaneous or map directly to `requestAnimationFrame`.
- Do not share simulation ownership, raw controller input, or GPU objects between surface peers.
- Do not make WebGPU, WASM threads, haptics, persistent storage, or wake lock mandatory.
- Do preserve one session protocol across browser, Android, simulator, and test hosts.
- Do require every dual-surface experience to remain complete in a single-surface fallback.
