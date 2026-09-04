# Android Host and APK

The Android host exists because Chrome cannot place a second browser window on the Thor companion display. It contains no Lumen Field policy. Each activity loads the same optimized web distribution through `WebViewAssetLoader`; its query string selects the main or companion projection.

## Install the alpha

Download `thorui-demo-v0.1.0-alpha.1.apk` from the [GitHub release](https://github.com/Kadajett/thorUI/releases/tag/v0.1.0-alpha.1). Android may ask you to allow installs from the browser or file manager. Open **Lumen Field** after installation.

The main activity selects the smallest allowed display other than its current display and launches the companion activity there. If Android rejects the launch or exposes only one display, the main projection stays usable and shows a short fallback message.

## Local build

The supported toolchain is JDK 21, Android platform 37, Android Build Tools 36.0.0, and the checked-in Gradle 9.7.1 wrapper.

```sh
pnpm install
pnpm android:debug
```

The debug APK is written to `hosts/android/app/build/outputs/apk/debug/app-debug.apk`. Install it on a connected Thor with:

```sh
/home/kadajett/Android/Sdk/platform-tools/adb install -r hosts/android/app/build/outputs/apk/debug/app-debug.apk
```

The full repository gate includes Android lint, display-policy unit tests, and a debug APK build:

```sh
pnpm check
```

## Thor verification

Launch Lumen Field from the Android launcher. The main projection should occupy the 1920×1080 display and the companion console should occupy the 1240×1080 display. Paint with the controller, then touch either projection. Both projections should show the same marks and `Linked surface` state.

Capture launch evidence with:

```sh
/home/kadajett/Android/Sdk/platform-tools/adb logcat -s ThorUIDisplay
```

The expected log includes the companion target and actual display IDs. A mismatch is a host bug; a missing second display means the firmware did not advertise an allowed secondary activity display.

## Release signing

The release workflow builds and signs tag pushes using GitHub Actions secrets. The long-lived signing key is stored outside the repository. Only its certificate fingerprint is public in `assetlinks.json`, allowing `https://thorui.yougotserved.dev/open` to resolve to the installed app.

Every release attaches the APK and a SHA-256 checksum. The workflow verifies the APK signature before publishing the prerelease.
