# Experience Interface Options

## Problem

The Experience seam is the main author-facing contract. It must serve event-driven applications and fixed-step games while hiding session authority, platform input, display placement, independent frame loops, persistence, effects, protocol recovery, and browser or Android types.

Three substantially different interfaces were considered. None is accepted until a headless application, a fixed-step game, and the dual-surface simulator validate it.

## Option A: minimal typed reducer

This option gives experience authors three entry points and typed portable data.

```rust
pub trait Experience: 'static {
    type State: SessionState;
    type Action: Portable;
    type Effect: Portable;
    type EffectReply: Portable;
    type Projection: Portable;

    fn start(
        &self,
        input: Start<Self::State>,
    ) -> Result<Started<Self::State, Self::Effect>, StartError>;

    fn react(
        &self,
        state: Self::State,
        facts: FactBatch<Self::Action, Self::EffectReply>,
    ) -> Transition<Self::State, Self::Effect>;

    fn project(
        &self,
        state: &Self::State,
        target: ProjectionTarget<'_>,
    ) -> Self::Projection;
}
```

`start` covers fresh and restored sessions and returns session policy with the initial state. `react` is the only state transition and consumes state so large game state need not be cloned. `project` is pure and derives portable data for one surface.

```rust
impl Experience for Runner {
    type State = RunnerState;
    type Action = RunnerAction;
    type Effect = RunnerEffect;
    type EffectReply = RunnerReply;
    type Projection = RunnerProjection;

    fn start(&self, input: Start<RunnerState>) -> Result<Started<_, _>, StartError> {
        let state = input.restore_or_else(RunnerState::new)?;
        Ok(Started::fixed_game(state, FixedRate::hz(60)))
    }

    fn react(&self, mut state: RunnerState, facts: FactBatch<_, _>) -> Transition<_, _> {
        state.apply(facts);
        Transition::changed(state, SurfaceRoles::all())
    }

    fn project(&self, state: &RunnerState, target: ProjectionTarget<'_>) -> RunnerProjection {
        RunnerProjection::for_surface(state, target)
    }
}
```

Invariants:

- exactly one authority calls `start` once per authority lifetime;
- `react` is total and receives a nonempty, canonically ordered batch;
- duplicate, stale, malformed, and oversized peer input is rejected before `react`;
- rendering and `project` never advance canonical state;
- effects execute only after a returned transition commits;
- peers may skip obsolete projections but never accepted input or effect results;
- the single-surface projection keeps every essential action reachable;
- time, random values, input, and effect results arrive only as facts.

The runtime hides fact ordering, fixed-step catch-up, effect IDs, revisions, invalidation, projection caching, save envelopes, trace replay, peer health, acknowledgements, and resynchronization.

Dependencies stay behind the appropriate seams. Pure policy is in-process. Clock, storage, input, and effect execution have fake and host adapters at internal local-substitutable seams. Authority-peer messaging has in-memory, browser, and native adapters at one remote-but-owned seam.

Depth is high because three methods unlock both product modes and all hosts. Locality is strong: experience policy stays in `react`, surface meaning stays in `project`, and runtime policy stays in the kernel. The cost is that authors must define portable action, effect, reply, and projection types even for small experiences; derives and `Never` defaults can remove most of that work.

## Option B: capability-extensible program

This option describes requirements up front and makes effects and projection layers open-ended.

```rust
pub trait Experience: 'static {
    type Config: Portable;
    type State: 'static;
    type Action: Portable;
    type Message: 'static;
    type Saved: Portable;
    type Error: ExperienceError;

    fn describe() -> ExperienceDescriptor<Self::Action>;
    fn start(Start<Self::Config, Self::Saved>) -> Result<Transition<Self::State, Self::Message>, Self::Error>;
    fn react(&Self::State, FactBatch<'_, Self::Action, Self::Message>) -> Result<Transition<Self::State, Self::Message>, Self::Error>;
    fn project(&Self::State, &ProjectRequest) -> Result<Projection<Self::Action>, Self::Error>;
    fn save(&Self::State) -> Result<Self::Saved, Self::Error>;
}
```

The descriptor declares timing, surface policy, capabilities, projection kinds, and resource limits. Effects request a typed `CapabilitySpec`, and response mappers turn capability events into experience messages. A projection is a collection of typed renderer-specific layers such as DOM, canvas, or a future custom renderer.

```rust
let rumble = Effect::request::<RumbleCapability>(
    EffectKey::new("damage"),
    EffectTarget::Authority,
    RumbleRequest::short(),
    GameMessage::Rumble,
);

let projection = Projection::new()
    .layer(LayerSlot::World, world_scene(state))?
    .layer(LayerSlot::Overlay, hud(state))?;
```

The runtime hides type erasure, capability version negotiation, response mapper storage, renderer dispatch, effect routing, and all of the ordering and recovery work hidden by Option A.

This has the most extension leverage after a plugin ecosystem exists. It allows new host capabilities and renderer payloads without growing a central enum. It also adds the widest interface, the most schemas, and the most failure paths before there is a real extension consumer. Borrowing `&State` makes atomic errors easy but encourages clones or persistent data structures for game updates.

The capability and projection seams are partly hypothetical in the first release. Their complexity would reduce locality by spreading registration, versioning, and compatibility work across authors, hosts, and the runtime. This option is a useful future pressure test, not the v1 shape.

## Option C: default-first closed vocabulary

This option optimizes the common caller with defaults and framework-owned facts, effects, and projections.

```rust
pub trait Experience: 'static {
    const ID: ExperienceId;
    type State: SessionState + Default;

    fn settings(&self) -> ExperienceSettings {
        ExperienceSettings::application()
    }

    fn start(&self, input: Start<Self::State>) -> ExperienceResult<Started<Self::State>> {
        Ok(Started::new(input.restore_or_default()?))
    }

    fn update(&self, state: &Self::State, facts: &FactBatch<'_>)
        -> ExperienceResult<Transition<Self::State>>;

    fn project(&self, state: &Self::State, target: &ProjectionTarget<'_>)
        -> ExperienceResult<Projection>;
}
```

Applications use the default settings. Games select one preset.

```rust
fn settings(&self) -> ExperienceSettings {
    ExperienceSettings::fixed_game()
        .persist(PersistencePolicy::on_suspend("run"))
}
```

The framework owns semantic `ActionId` values, a closed `HostEffect` enum, and a standard projection enum with `Ui`, `Canvas`, `Layers`, and `Custom`. A launcher can be as small as `thorui_web::launch(Planner)`.

This gives the best first-hour experience, uniform transport, and clear defaults. It also pushes unrelated application and game needs into central enums. String-like action IDs lose useful Rust exhaustiveness, a `Default` bound pretends every state has a valid empty value, and `&State` transitions make large game state cloning tempting. Custom byte escape hatches can become a second untyped framework inside the first.

The presets, derives, test harness, and one-line launcher have high leverage and should be kept. The closed vocabulary should not define the core Experience interface.

## Comparison

Option A has the greatest depth at the author seam. It asks for a small amount of explicit domain data and hides the hard distributed-runtime behavior. Its seam placement follows actual variation: pure experience policy stays in-process, host dependencies remain internal, and only peer messaging and presentation cross real ports.

Option B maximizes theoretical flexibility. Its depth is weakened by how much every caller must learn: descriptors, capability specifications, target routing, result mapping, renderer kinds, and separate saved forms. Locality also suffers because adding a capability touches more registries and version surfaces. It becomes attractive only after multiple independent hosts or third-party extensions exist.

Option C has excellent leverage for simple examples, but its interface is shallow at the edges. The easy path depends on large framework-owned enums and generic payload escapes. Changes for one product type can disturb all callers. Its ergonomic layer is valuable even though its core contract is not.

The key performance distinction is state ownership. Option A moves `State` into and out of `react`, allowing local mutation without observable shared mutation or full clones. Options B and C borrow old state and must allocate a replacement, use structural sharing, or hide mutation behind interior state.

## Recommendation

Use Option A as the kernel-facing Experience interface and add Option C's ergonomics without adding methods:

- `Started::application(state)` and `Started::fixed_game(state, rate)` presets;
- derives for `SessionState` and `Portable` data;
- `NoEffect` and `NoReply` types for experiences with no custom effects;
- typed transition constructors with safe invalidation defaults;
- a one-line web launcher and a compact native test harness;
- standard UI and canvas projection types that authors may choose as their associated projection;
- typed standard effects supplied as library values, not variants added to the trait;
- an explicit custom effect type only when an experience has a real adapter.

Keep `react` total. Normal domain rejection becomes state and projection feedback; host failures arrive as effect-result facts. A failed transition cannot strand an owned state or require rollback.

Keep `project` pure and total. A peer rendering failure is handled outside the experience and cannot corrupt state. Resource budgets are validated by the runtime before a projection is sent.

Do not add a capability registry, dynamic plugins, renderer-kind negotiation, a required `Default` state, or string action IDs in v1. Revisit them only when two real consumers cannot use the typed associated values cleanly.

## Acceptance spike

Before accepting the interface, implement only enough to prove:

1. a persisted list/detail application with DOM projections;
2. a 60 Hz simulation with a 120 Hz-interpolated main canvas and 60 Hz companion controls;
3. state restore from an older schema;
4. one asynchronous effect and one denied optional capability;
5. a peer disconnect, stale projection, and full resynchronization;
6. byte-identical replay from the same facts;
7. no per-frame state clone or cross-peer projection generation.

If the spike needs another public method, first try moving that behavior behind `Started`, `Transition`, `Projection`, or an internal host seam. The interface is accepted only when both reference shapes remain clear without browser types or special cases.

