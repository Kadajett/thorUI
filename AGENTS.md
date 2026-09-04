# Working Agreement

These rules apply to the whole repository.

## Before changing code

- Read `CONTEXT.md`, `docs/architecture.md`, `docs/roadmap.md`, and `docs/quality.md`.
- Work on one roadmap milestone and one observable outcome at a time.
- Keep platform facts separate from assumptions that still need a Thor device test.
- Do not lock a dependency until the milestone needs it.

## Design

- Prefer deep modules with small interfaces and hidden implementation detail.
- Add a seam only when at least two adapters are real or immediately required for tests.
- Pass dependencies in. Do not create hidden dependencies or mutable global state.
- Put policy in the functional core and I/O in host adapters.
- Represent side effects as data before a host executes them.
- Return values and errors. Do not signal normal behavior with panics.
- Keep state ownership explicit. Local mutation is allowed only when it cannot escape its module.
- Do not duplicate behavior between DOM, canvas, browser, simulator, and Android paths.

## Size and complexity

- Aim for at most 400 lines in a hand-written source file.
- Files over 600 lines require a split or a written reason in `docs/quality-exceptions.md`.
- Hand-written source files over 1,000 lines are forbidden without explicit user approval.
- Keep functions under 40 logical lines when practical.
- Keep cognitive complexity at or below 10 per function.
- Comments are at most two consecutive lines. Put longer explanations in nearby Markdown.
- Prefer simple technical English and one canonical name for each domain concept.

## Rust

- Format with `rustfmt` and run Clippy for every target and feature used in CI.
- Treat compiler and Clippy warnings as errors.
- Forbid unsafe code in project crates unless an accepted ADR permits a narrow exception.
- Do not use `unwrap`, `expect`, `todo`, or `unimplemented` in production paths.
- Keep feature flags additive. Test every supported feature combination.
- Keep browser bindings behind the web host seam.

## Tests

- Test behavior through module interfaces, not private implementation details.
- Use pure unit and property tests for state, time, input, layout, and protocol rules.
- Give every host adapter the same contract test suite where possible.
- Run browser behavior in automated Chromium tests.
- Keep hardware-only checks as repeatable probes with captured results.
- A fix begins with a failing regression test when the behavior can be reproduced locally.

## Demo delivery

- Publish each runnable milestone's latest successful default-branch build to `thorui.yougotserved.dev`.
- Keep the last green demo live when a build, smoke check, or deployment fails.
- Let this repository own the Worker artifact and deployment command.
- Let `kadajett-infrastructure` own the Cloudflare custom-domain binding.
- Never commit Cloudflare credentials, Pulumi stack secrets, or deployment tokens.
