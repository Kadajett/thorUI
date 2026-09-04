# AYN Thor Capture — 2026-09-04

## Capture identity

- Receipt: `590b852e`
- Report schema: 1
- ThorUI revision: `827d83609348`
- Host: Chrome 152 on Android, installed standalone mode
- Declared projection: companion

## Observed facts

- The viewport was 832×364 CSS pixels at 2.30625 density.
- The reported screen was 833×469 CSS pixels, about 1921×1082 physical pixels.
- A main-role BroadcastChannel peer connected, but no companion window appeared on the lower display.
- `getScreenDetails()` was present and returned `Permission denied.`
- The five-second frame run estimated 59.18 Hz with a 16.67 ms median and 17.83 ms p95.
- WebGL2, WebGPU, AudioContext, Pointer Events, Wake Lock, and Presentation APIs were reported available.
- WebGPU identified a Qualcomm Adreno 7xx adapter.
- Chrome exposed the controller as `Xbox Wireless Controller (Vendor: 2020 Product: 0112)` with 17 buttons, four axes, no standard mapping string, and no haptic actuator.
- Three touch pointer IDs produced 244 samples.

## Decision evidence

Chrome can create same-origin peers, but this run did not provide reliable placement on the lower display. The selected device topology is therefore an Android-assisted host with one web projection per Android display.

## Limits

This run sampled only axes 0 and 1 and recorded no pressed buttons. It does not establish the Thor controller mapping, 120 Hz main presentation, haptics, fullscreen ownership, suspend recovery, display shutdown recovery, or sustained peer latency. Those remain measured follow-up work and do not reopen the placement decision unless a later Chrome release proves a reliable lower-display launch path.
