# Android dual-display host

Research status: 2026-09-04. This note uses first-party Android and Chrome documentation. It separates Android contracts from behavior that still needs a physical AYN Thor test.

## Recommendation

Keep `thorui.yougotserved.dev` as the browser preview and App Link surface. Publish the reference APK through the ThorUI repository's GitHub Releases. Ship one narrow Android host that contains two Activities with one WebView each. After installation, a verified Android App Link can enter one launcher Activity; the launcher can enumerate displays and request one surface Activity on each eligible display. This is one installed app and one ThorUI session with two projections, not two launches of the whole product.

The reference APK is only the framework demo. Applications and games built with ThorUI reuse the host packaging but own their web artifact, Android package, signing key, verified domain if used, and distribution. No shared launcher or ThorUI store is required.

Do not use a Trusted Web Activity as the supported host. A TWA delegates rendering to the user's browser and does not give the host direct access to web state. Its official contract does not promise two independently placed, simultaneously active browser Activities. A custom WebView is an Android `View` owned by the Activity, so it preserves the same Rust/WASM UI while giving the host direct ownership of display placement, lifecycle, and a narrow message bridge.

This design is possible on Android 13, but it is not yet proven on the Thor. The next artifact should be a disposable launcher probe, not the final host.

## Product flow and unavoidable first install

1. `https://thorui.yougotserved.dev/open` remains useful in a browser when the host is absent.
2. Once the host is installed and the domain is verified, tapping that URL can open its sole exported launcher Activity without an app chooser.
3. The launcher starts explicit main-surface and companion-surface Activities, passing one observed display ID to each.
4. Each Activity loads the same versioned Rust/WASM artifact and receives its projection role through explicit launch data or an origin-bound bridge.
5. If the companion launch is unavailable or rejected, the main Activity loads the single-surface fallback.

Android App Links are available from Android 6, API 23, on devices with Google services. They require an `http`/`https` `VIEW` intent filter with `BROWSABLE`, `DEFAULT`, and `android:autoVerify="true"`, plus `https://thorui.yougotserved.dev/.well-known/assetlinks.json` containing the app package and signing-certificate fingerprint. When the installed association verifies, Android routes a matching link to the app without a disambiguation dialog. Without the app, the same URL stays on the website. See [About App Links](https://developer.android.com/training/app-links/about), [Add App Links intent filters](https://developer.android.com/training/app-links/add-applinks), and [Configure `assetlinks.json`](https://developer.android.com/training/app-links/configure-assetlinks).

Only the launcher Activity should declare the App Link. Android gives no guarantee which Activity handles a URL when more than one Activity has the same verified filter. On the Thor's Android 13, server changes to `assetlinks.json` are normally picked up when the app is installed or updated; periodic background re-verification starts with Android 15. The device test should use Android's [App Links verification commands](https://developer.android.com/training/app-links/verify-applinks).

An App Link does not install an absent app. A release APK may be hosted on the website, but Android 8 and later require the user to allow that source to install unknown apps. Android blocks the install until the user opts in. See [Alternative distribution options](https://developer.android.com/distribute/marketing-tools/alternative-distribution). Android developer verification is also rolling out: the September 30, 2026 enforcement is limited to participating stores in four countries, while wider certified-device enforcement is planned for 2027. Direct sideloading is not part of the initial September enforcement, but a public host should still plan to register its package and signing key. See the [current verification guide](https://developer.android.com/developer-verification/guides).

## Display discovery and targeted Activity launch

`DisplayManager.getDisplays()` has returned all currently valid logical displays since API 17. `DisplayManager.DisplayListener` reports additions, removals, and changes. The newer built-in-display category is API 36.1 and is unavailable on the Thor's Android 13, so the host must enumerate all displays and classify them from observed device data instead of depending on that category. See [`DisplayManager`](https://developer.android.com/reference/android/hardware/display/DisplayManager).

Targeted Activity launch is available from API 26:

- Check `PackageManager.FEATURE_ACTIVITIES_ON_SECONDARY_DISPLAYS`. If absent, `ActivityOptions.setLaunchDisplayId()` is ignored.
- On the Thor's API 33, call `ActivityManager.isActivityStartAllowedOnDisplay(context, displayId, intent)` before launch. This preflight was added in API 29.
- Create `ActivityOptions`, call `setLaunchDisplayId(displayId)`, then pass its bundle to `startActivity`.
- Catch launch failure and verify the Activity's actual display after creation. Invalid, private, system-owned, or otherwise restricted displays can be denied.

Official references: [`FEATURE_ACTIVITIES_ON_SECONDARY_DISPLAYS`](https://developer.android.com/reference/android/content/pm/PackageManager#FEATURE_ACTIVITIES_ON_SECONDARY_DISPLAYS), [`ActivityOptions.setLaunchDisplayId`](https://developer.android.com/reference/android/app/ActivityOptions#setLaunchDisplayId(int)), and [`ActivityManager.isActivityStartAllowedOnDisplay`](https://developer.android.com/reference/android/app/ActivityManager#isActivityStartAllowedOnDisplay(android.content.Context,int,android.content.Intent)).

Launch modes, intent flags, and existing tasks affect placement. Without a target, Android normally starts an Activity on the caller's display. An existing Activity may instead be reused on its previous display. The probe should use two explicit surface Activity classes with launch behavior that permits both instances, then record where Android actually places them. See the AOSP [Activity launch policy](https://source.android.com/docs/core/display/multi_display/activity-launch).

Android 10 and later can keep top-visible Activities on multiple displays in `RESUMED` state at the same time. Only top-resumed state should control exclusive resources; `onPause()` must not be treated as proof that a surface is invisible. See AOSP [Multi-resume](https://source.android.com/docs/core/display/multi_display/multi-resume).

## `Presentation` is an alternative, not the baseline

Native `android.app.Presentation` has existed since API 17. It is a display-bound `Dialog` with display-specific resources, and it can contain a WebView. It should be offered only displays returned by `getDisplays(DISPLAY_CATEGORY_PRESENTATION)`. `show()` throws `InvalidDisplayException` if the display is missing, lacks `FLAG_PRESENTATION`, or current policy disallows presentation. The system dismisses it when the target display is removed. Internal displays have extra policy restrictions. See [`Presentation`](https://developer.android.com/reference/android/app/Presentation) and [`Display.FLAG_PRESENTATION`](https://developer.android.com/reference/android/view/Display#FLAG_PRESENTATION).

The official APIs do not guarantee that the Thor's built-in companion panel appears in the presentation category. If AYN marks it as presentation-capable, one Activity plus one Presentation may be a useful fallback. It still couples the companion projection to a Dialog and the owning Activity, so two Activities remain the better baseline for independent lifecycle and recovery.

## WebView rather than Trusted Web Activity

Android documents WebView as a `View` inside an Activity. It has no browser chrome, JavaScript is disabled by default, and the app controls navigation and window behavior. For ThorUI, enable only required settings, allow only the ThorUI origin, reject foreign navigation, and use the modern origin-scoped `addWebMessageListener` bridge. Avoid the legacy `addJavascriptInterface` bridge because it is exposed to every frame and lacks origin-based access control. See [Build web apps in WebView](https://developer.android.com/develop/ui/views/layout/webapps/webview) and [Access native APIs with a JavaScript bridge](https://developer.android.com/develop/ui/views/layout/webapps/native-api-access-jsbridge).

Two WebViews add real memory cost. Android warns that WebView uses Chromium processes and retains memory after removal; each Activity must remove and explicitly destroy its WebView during final teardown. The device probe must measure two-surface startup, steady memory, renderer loss, and recovery. See [Manage and diagnose WebView memory](https://developer.android.com/develop/ui/views/layout/webapps/manage-webview-memory).

A Trusted Web Activity is based on Custom Tabs. The browser renders the content, the host has no direct access to cookies, local storage, or other web state, and unsupported browsers may fall back to a Custom Tab. The first-party TWA contract says nothing about two-display placement or two concurrent fullscreen surfaces. A TWA could remain an experiment, but it cannot be the supported Thor host without separate browser-specific proof. See the Chrome team's [Trusted Web Activity overview](https://developer.chrome.com/docs/android/trusted-web-activity).

## Physical Thor acceptance probe

The Android APIs are present at Android 13. These device and firmware facts remain unproven:

- `getDisplays()` exposes both panels as distinct logical displays while both are enabled;
- the device declares the secondary-Activity feature and allows the explicit companion Activity on the lower panel;
- requested placement matches actual placement across cold launch, relaunch, and task reuse;
- both WebViews remain visible, resumed, and rendering at their own measured frame cadence;
- each touch panel routes to its own Activity, while controller, keyboard, and IME focus behave predictably;
- immersive system bars and orientation can be controlled independently;
- display off, hinge changes, suspend, Activity recreation, and companion removal recover without losing the session;
- the lower panel is or is not returned in `DISPLAY_CATEGORY_PRESENTATION`;
- the verified App Link opens from the Thor's installed browser and returns to the website when the host is absent;
- two live WebViews stay within memory and graphics limits on the Snapdragon 865 model.

Touch association and per-display focus are OEM configuration, not app guarantees. AOSP explicitly recommends against per-display keyboard focus on ordinary multi-screen devices, while display-specific touch routing depends on manufacturer input-port association. See AOSP [Display support](https://source.android.com/docs/core/display/multi_display/displays#per-display-focus) and [Input routing](https://source.android.com/docs/core/display/multi_display/input-routing).

The probe passes only when one user tap on the verified URL opens one session with usable projections on both physical panels. A JavaScript peer existing in the background is not a pass.

If both the targeted Activity launch and `Presentation` paths fail, the public Android APIs researched here offer no third placement path for an ordinary app. At that point the next dependency is an AYN-specific API or firmware contract, not more browser window code.
