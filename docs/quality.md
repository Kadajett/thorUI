# Engineering Quality

## Purpose

Complexity rules are strict from the first source file. Exceptions must be visible, narrow, and harder to add than a clean design.

## Automated gates

The initial workspace should make one task runner command execute all local gates. CI runs the same command and adds browser and hardware jobs where available.

Required gates:

- formatting check;
- `cargo check` for native tests and `wasm32-unknown-unknown`;
- Clippy with warnings denied for all targets and supported features;
- unit, property, contract, browser, and documentation tests;
- dependency license, advisory, duplicate, and unused checks;
- file length, comment length, forbidden token, and complexity checks;
- release-size and performance regression budgets once baselines exist.

Tool selection is deferred until scaffolding. Prefer tools that emit stable machine-readable output and run locally without a hosted account.

## Source limits

| Measure | Target | Gate |
|---|---:|---:|
| Hand-written source file | 400 lines | reason required over 600; reject over 1,000 |
| Function | 25 logical lines | review required over 40 |
| Cognitive complexity | 8 | reject over 10 |
| Function parameters | 4 | group a coherent value over 5 |
| Nesting | 3 levels | refactor over 4 |
| Consecutive comment lines | 1 | reject over 2 |

Generated code, vendored code, snapshots, and machine-generated bindings are measured separately and cannot hide in normal source paths.

An exception records the owner, exact path or symbol, reason, removal condition, and expiry milestone in `docs/quality-exceptions.md`. The file is created only when the first real exception is approved.

## Dependency direction

The functional core depends only on stable data types and pure utilities. Platform hosts depend inward on the core. Experience examples depend only on public framework interfaces. No core or design-system module imports browser, Android, or example code.

Cycles between modules are rejected. Shared code is promoted only when it represents one shared concept; a generic `common` or `utils` dumping ground is forbidden.

## Duplication rule

Duplicate syntax is acceptable when behavior is not yet understood. Duplicate policy is not. The second real implementation establishes evidence for a seam; the third copy blocks review until the behavior is centralized or the distinction is named.

Tests may repeat small setup values for readability. They may not reimplement production algorithms to calculate expected output.

## Error policy

- Normal failures use typed errors with stable categories and useful context.
- Host capability failures are values returned to the runtime.
- Protocol decoding rejects malformed, unsupported, and oversized input before allocation-heavy work.
- A peer failure cannot corrupt authority state.
- Panics indicate violated internal invariants and terminate the affected context cleanly.
- User-facing text is selected outside low-level error types.

## Test layers

### Pure tests

State transitions, time, input mapping, layout decisions, revision ordering, and fallback policy run as native Rust tests. Property tests cover sequences and invariants rather than only examples.

### Adapter contract tests

In-memory, browser, and native message adapters share ordering, delivery, size, close, and reconnect cases. Clock, storage, and effect adapters follow the same pattern where more than one adapter exists.

### Browser tests

Automated Chromium tests cover DOM semantics, focus, pointer handling, gamepad shims, resize, visibility, fullscreen mocks, offline launch, and rendering snapshots. Real capability support is never inferred from a mock.

### Hardware tests

The Thor capability lab produces versioned reports rather than pass/fail anecdotes. Release checks replay controller, refresh, suspend, resume, display loss, memory pressure, and context-loss scenarios on the lowest supported device.

## Determinism

- Session time, random values, input, and effect results are explicit facts.
- Traces include schema version, device profile, initial state version, and fact order.
- Rendering and wall-clock timing cannot change session outcomes.
- Serialization has stable golden tests once persisted state or protocol compatibility ships.

## Provisional performance budgets

Milestone 1 replaces these with hardware measurements.

- A 120 Hz main surface has an 8.33 ms frame interval.
- A 60 Hz companion surface has a 16.67 ms frame interval.
- Framework CPU work should consume at most half of either interval in the reference experience.
- Fixed-step catch-up is capped and reports dropped simulation time.
- Peers may drop obsolete projections but never input or effect results.
- No full session-state clone or serialization occurs per rendered frame.
- First usable offline launch and built artifact size receive regression budgets after the first vertical slice.

## Review checklist

- Is the module's interface smaller than the behavior it hides?
- Are time, randomness, I/O, and platform facts explicit?
- Is each policy owned in one place?
- Does every dual-surface behavior have a single-surface fallback?
- Can tests observe behavior through the same interface as callers?
- Does a real second adapter justify every seam?
- Will the lowest-end Thor meet the measured budget?
- Can a new reader understand names using `CONTEXT.md` without decoding aliases?

