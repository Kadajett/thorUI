# Thor Capability Capture

## Goal

Capture facts from one physical AYN Thor before selecting the supported dual-surface topology. Use the deployed build at [thorui.yougotserved.dev](https://thorui.yougotserved.dev) so both surfaces have a secure, same-origin host.

## Prepare

1. Update Chrome and close unrelated tabs.
2. Allow popups, fullscreen, window management, sound, and wake lock for the site.
3. Connect the built-in controller and leave both touch surfaces enabled.
4. Set the main surface to 120 Hz and the companion surface to 60 Hz.
5. Open `https://thorui.yougotserved.dev/?surface=main` on the main surface.

Do not infer a failed capability from one dismissed permission prompt. Record the rejection, reset the permission, and repeat it once.

## Fast capture

1. Set **Expected refresh** to match the surface.
2. Press a D-pad direction or move a stick. A green focus ring should appear. Use A to activate controls.
3. Activate **Run Thor test + save report**.
4. When step 3 appears, use every controller input and touch and drag in the nearby pad.
5. Stop when the page shows **Saved** and an eight-character receipt.

The report is stored by the deployment for 90 days. Do not copy Android system logs or download JSON for the normal capture. An operator can find the receipt with `pnpm reports:list` and fetch its key with `pnpm reports:get KEY`.

Run this once on the main projection at 120 Hz and once on the companion projection at 60 Hz. The advanced pass below records lifecycle and placement facts that cannot be automated safely.

## Advanced dual-surface pass

1. Run the permission probes and grant each prompt.
2. Use **Open Chrome peer experiment**. This may create only a background tab and cannot launch the lower display reliably.
3. Keep both projections visible and send 32 peer pings in each direction.
4. Measure a five-second frame run on both surfaces at the same time.
5. Use the guided test on both projections and keep both receipt IDs.
6. During its input step, hold two touches and drag a third pointer in the touch target.
7. Enter and exit fullscreen on each surface. Hold and release the wake lock.
8. Move each browser context between surfaces, then repeat the frame run.
9. Turn the companion surface off, wait ten seconds, restore it, and send pings again.
10. Suspend the Thor for ten seconds, resume it, and send pings again.

Add short notes for automatic placement, popup behavior, fullscreen ownership, controls Chrome did not expose, movement between surfaces, display shutdown, suspend, and recovery.

## Refresh and fallback passes

Set both displays to 60 Hz and record a second frame run on each. Then close the companion context and verify that the main context remains usable alone. Reopen the companion and confirm that it joins the same session only when the original session link is used.

For the offline pass, load both projections once, disable Wi-Fi, and reload each. Restore Wi-Fi before exporting.

## Manual export fallback

If automatic save fails, export one JSON report from each surface. The filenames contain separate capture IDs. If USB debugging is enabled, copy downloads with:

```sh
/home/kadajett/Android/Sdk/platform-tools/adb pull /sdcard/Download/ reports/thor/
```

Keep only the two matching `thorui-capability-*.json` files in a dated folder. Capability reports contain browser and controller identifiers but should not contain credentials or personal content.

Validate the pair before committing it:

```sh
cargo run -p xtask -- validate-reports reports/thor/DATE/main.json reports/thor/DATE/companion.json
```

## Decision rule

Select Chrome-only only when Chrome can place, keep active, reconnect, and recover both contexts without fragile manual steps. Otherwise build the disposable Android display-launch probe and compare the same measurements before accepting Android-assisted execution.
