---
type: ImplementationReceipt
title: Layer-controlled physical motion runtime integration
description: Completes ordered layer programs with immutable snapshots, selective reset epochs, frozen startup capture, preflighted reloads, real-layout simulations and checked examples; records native hardware verification limits.
resource: /design/mouse-axis/layer-transform-implementation1.gpt61s.md
status: draft
author: openai/gpt-6.1-sol#xhigh
date: 2026-10-29
sources:
  - resource: opencode://ses_e7a02dd4effe7PtopegpGAdLWC
    title: Implements the remaining runtime, capture and reload integration after stage-one review; verifies and documents the complete bounded feature
  - resource: opencode://ses_e7b7b7559ffeEC9DfB7nsFmtYO
    title: Parent commissions the remaining Sol xhigh vertical implementation and approves reject-before-mutation capture preflight
  - resource: /design/mouse-axis/layer-transform-implementation0.gpt61s.md
    title: Accepted stage-one algebra and compiler interfaces and prior verification boundary
  - resource: /design/mouse-axis/layer-transform-implementation0.gpt6a.md
    title: Code-grounded runtime, capture, scheduling and adapter handoff
  - resource: /design/mouse-axis/layer-transform-design1.gpt6a.md
    title: Target ordered single-map semantics, products and selective state contract
  - resource: /design/mouse-axis/layer-transform-ask0.gpt6a.md#implementation-commission
    title: Implementation authority and approved unavailable-capture reload rejection
  - resource: /parser/src/mouse_axis/program.rs
    title: Accepted algebra extended with conservative capture requirements and on-demand coefficient explanations
  - resource: /parser/src/cfg/layer_opts.rs
    title: Typed public layer options preserving icons alongside motion programs
  - resource: /parser/src/cfg/mod.rs
    title: LayerInfo ownership, initial validation, pure stack fold and discovery derivation
  - resource: /keyberon/src/layout.rs
    title: Actual held-layer iterator and default state used independently of keyboard transparency
  - resource: /src/kanata/mouse_axis.rs
    title: Runtime preparation, immutable publication and authoritative selective adapter observation
  - resource: /src/kanata/mouse_axis/capture.rs
    title: Required versus frozen planned versus installed capture and capability preflight
  - resource: /src/kanata/mod.rs
    title: Both constructors, staged reload installation and per-inner-tick/entry scheduling
  - resource: /src/kanata/linux.rs
    title: Frozen reader setup, snapshot-scoped report routing and existing Linux report fixtures
  - resource: /src/kanata/windows/interception.rs
    title: Installed MOVE accounting, per-device observation and combined-stroke preservation
  - resource: /src/kanata/windows/llhook.rs
    title: Frozen hook setup, observation before callback early returns and actual hook capability reporting
  - resource: /src/oskbd/windows/llhook/mouse.rs
    title: Native hook constructor panics on installation failure before actual capabilities are recorded
  - resource: /src/kanata/mouse_axis/runtime_tests.rs
    title: Real-layout, physical-vector, epoch, queued-action and reload failure-atomicity verification
  - resource: /docs/config.adoc
    title: Maintained syntax, arithmetic, ordering, capture, reload and processed-state behavior reference
  - resource: /justfile
    title: Native project formatting, test, serial ignored-test and clippy conventions
---

# Layer-controlled physical motion runtime integration

## Outcome and commits

**The remaining bounded feature is implemented, documented and verified without
opening live input devices.** Both `deflayer` and `deflayermap` accept ordered
`mouse-axis` programs. Physical reports consume resolved snapshots; declarations
are no longer an unconnected internal parser feature. This completes stages 2–5
of the [handoff](layer-transform-implementation0.gpt6a.md), using the accepted
[stage-one APIs](layer-transform-implementation0.gpt61s.md) rather than reopening
the earlier gain-family or specificity alternatives.

Explicit-path commits in this session:

| Commit | Coherent stage |
| --- | --- |
| `197bfd92` | Immutable motion snapshots and selective output epochs; existing global maps use the new adapter path. |
| `a594fa46` | Public typed headers, full-stack resolver, inactive capture requirements, frozen startup installation and preflighted reloads, with focused tests. |
| `80b52ee6` | Maintained reference, four checked samples, sample parser coverage, physical-vector integration and generated-output separation regression. |
| `79c14721` | Make the shared resolver available on non-capture targets; retain native-only adapter state. Downstream WASM compilation exposed the inherited module gate. |

No versions, dependency declarations or lockfiles changed. No squash or push was
performed. Independent Astra documents, the prior Sol receipt and the
parent-maintained reading map were not edited.

## Consequential interfaces and behavior

### Parser and immutable programs

- [Layer options](/parser/src/cfg/layer_opts.rs) now return typed `LayerOptions`:
  optional icon plus `MotionProgram`. Icon aliases share duplicate detection;
  header/name/icon variables and nested motion variables retain the existing
  variable-aware accessors and LSP reference machinery.
- [LayerInfo](/parser/src/cfg/mod.rs) owns the program at its sole construction
  point. The temporary internal-compiler dead-code allowance and public-syntax
  rejection test are removed at this connected integration.
- `cfg::resolve_motion(baseline, layers, canonical_stack)` is the shared pure
  low-to-high fold. Parsing checks layer zero, the actual initial default; queued
  on-load actions are not predicted. Runtime preparation separately reads the
  candidate layout's actual held/default state.
- `CaptureRequirements::for_programs` keeps exact baseline input differences
  for global-only configurations. Any numeric layer edit or factor conservatively
  requests every supported relative input, including suppression/zero/identity-
  looking instructions. Empty/null-only programs add none. This is deliberately
  conservative, not a reachability optimizer or a claim that hardware exists.
- `MotionProgram::explain_cell` replays one coefficient on demand in its inherited
  context, returning ordered before/after values, owned provenance and canonical
  product contributions. Products have a sum, not one winning declaration.

Dormant programs are syntax/type checked, not evaluated independently against a
guessed baseline. A lower layer can provide the context that makes a later product
finite. Unsupported axes, directional input composition and null factors remain
rejected even when dormant. The stage-one arithmetic/domain checks remain intact.

### Runtime publication and scheduling

[Motion runtime](/src/kanata/mouse_axis.rs) separates:

- `PreparedMotion`: baseline, actual canonical stack, resolved map, wheel mask
  and declared capture requirements, all built before live installation.
- `MotionResolver`: immutable baseline, cached stack and latest immutable snapshot.
  Programs stay with the matching `layer_info`, not a second mutable table.
- `MotionSnapshot`: effective map, configuration generation, per-output epochs
  and four mapped-wheel-direction bits, published through
  `Arc<Mutex<Arc<MotionSnapshot>>>`.
- `AxisState::observe_snapshot`: the only reset authority on the native path.
  `apply(input)` never invokes legacy all-map `reconfigure` afterwards.

Resolution reads newest-first held layers plus default, deduplicates the highest
occurrence, then reverses. Each layer runs once, independently of keyboard
transparency and first-layer delegation. Changes below the same visible top,
duplicate-priority fallback, external defaults and virtual/one-shot holders all
participate. Repeated unchanged ticks reuse the publication without drift.

Refresh occurs at `tick_ms` entry, including zero milliseconds, and after every
inner `tick_states`. It therefore publishes both transitions of a short one-shot
inside one batch. Queued actions wait until actual layout processing. Processing
also refreshes before deciding to block and after LLHOOK keystate synchronization,
which can remove layer states. The existing Windows shift cleanup remains before
the end-of-tick refresh. TCP's existing wakeup path needs no protocol change.

Only changed final output rows advance their epochs; directional comparisons
include both branch tables. Changed V/H outputs clear both raw/key notch credits
for that wheel. Unchanged wheel rows preserve their credit through pointer-only
precision changes. Epochs detect published A→B→A even for an inactive device.
Successful reload advances the generation and every epoch, even for equal maps.
Provenance-only changes do not reset ordinary layer transitions.

Each adapter acquires one snapshot and drops its publication lock before other
locks, sends or output. Observation precedes identity passthrough and callback
early returns. Wheel routing uses that snapshot's mask, not a newer global key
table midway through an old report. Linux retains its single flattened-stream
state; Interception retains device-local states; LLHOOK retains callback-local
state and the separate injection worker. Native identity forwarding, absolute
stroke handling and transformed combined-stroke metadata paths are preserved.

### Capture and reload

[CaptureStatus](/src/kanata/mouse_axis/capture.rs) distinguishes requirements,
the constructor-frozen `CapturePlan`, and optional installed `CaptureCapabilities`.
Constructors freeze the plan before CLI TCP/processing startup. Backends consume
that plan even if an early reload changes the effective matrix, and report actual
capabilities only after successful setup:

- Linux opens `KbdIn` with the frozen paths, name filters, detect mode and
  no-device policy. Parser derivation includes inactive programs but does not
  override explicit detection. Explicit paths still bypass autodetection.
- Interception records whether the mouse filter and MOVE were installed,
  including explicit HWID/movement-key reasons. A wheel filter is not MOVE.
- LLHOOK records whether the hook was installed. A button-only hook already
  receives movement, so identity at reload is not evidence of missing capture.
- Unsupported capture targets retain the shared resolver; macOS warns for
  numeric inactive declarations as well as global nonidentity maps. This does
  not implement macOS physical capture.

`do_live_reload` parses, then calls the common private `install_config` path.
Motion preparation and capture validation precede `update_kbd_out` and every
live configuration assignment. Missing hook/MOVE or newly required automatic
Linux `Any` discovery produces a restart-required error against installed facts,
or the frozen plan while pending. Fixed paths/explicit source policies remain
the selected source universe; no device-existence promise or reader widening is
inferred. Reload does not rebuild paths/filters or automatically clean anything up.

After preflight, layout, matching programs and snapshot generation install in the
non-fallible assignment section. Success follows motion installation and the
existing fallible gamepad/repeat-rate work. The reload-status flag starts false,
so later failure cannot leave a stale success claim. This is **motion-preflight
failure atomicity**, not rollback of all existing OS/gamepad reload side effects.
Tests invoke that exact installation path with rejected candidates and confirm
old publication identity, layer information, resolver stack and generation survive.

Arithmetic errors retain owned source locations, layer name/index, instruction,
canonical stack and generation. Debug summaries bound instruction/coefficient
output to eight entries each. No full source files/Rc-backed parser expressions
are retained in runtime state; no new diagnostic protocol/framework was added.

## Verification receipt

Successful commands and results in this session:

| Command | Result |
| --- | --- |
| `cargo nextest run -p kanata-parser -p kanata-keyberon` | **332 passed** at the core integration; the subsequently added sample parser test passed separately. |
| `cargo nextest run -p kanata-parser --features lsp -E 'test(mouse_axis)'` | **59 passed**, including nested variable reference recording. |
| `cargo nextest run -p kanata-parser -E 'test(published_layer_motion_samples)'` | **1 passed**; all four committed samples are compiled into coverage. |
| `cargo nextest run -p kanata --features simulated_output -E 'test(sim_tests) \| test(mouse_axis) \| test(axis_tests) \| test(tcp_layer_change_tests)'` | **268 passed**, 57 unrelated/ignored tests skipped. |
| `cargo test -p kanata --features simulated_output -- must_be_single_threaded --ignored --test-threads=1` | **4 passed**, serialized as required. |
| `cargo fmt --all --check` | Passed. |
| `cargo clippy -p kanata -p kanata-parser -p kanata-keyberon --features simulated_output --all-targets` | Passed; no new-code warnings. |
| `cargo check -p kanata-parser --features lsp --all-targets` | Passed. |
| `cargo check -p kanata --target x86_64-pc-windows-gnu --all-targets` | Passed, including native LLHOOK adapter compilation. |
| `cargo check -p kanata --target x86_64-pc-windows-gnu --features interception_driver --all-targets` | Passed, including Interception report-fixture compilation. |
| `cargo check -p kanata-wasm --target wasm32-unknown-unknown --lib` | Passed after the shared-module portability correction. |
| `cargo run --bin kanata -- --check --cfg FILE` for each new sample | All four passed; parser-only exit before device setup. |
| Rekon `check --okf design/mouse-axis/layer-transform-implementation1.gpt61s.md` | Passed: structure, local destinations/anchors and session provenance; not a semantic or hardware check. |

New runtime tests establish full-stack ordering under both keyboard transparency
modes, held-default deduplication/no implicit first-layer fallback, lower release
under an unchanged top, virtual/physical once-per-layer behavior, zero-ms external
default refresh, on-load queue latency, no drift, inner-batch A→B→A epochs,
selective directional/mode resets, independent device observation, frozen pending
startup across early reload, snapshot-owned wheel routing, missing MOVE rejection,
buttons-only LLHOOK capability, candidate actual-stack failure atomicity and equal
reload reset. A real sample runs through layout actions → snapshot → physical
vector quantization. A separate simulator regression proves generated mouse/wheel
output is not retransformed even by a zero physical map.

The existing Linux report fixtures ran. Windows fixtures compiled, **not executed
on Windows**. Clippy still reports existing Keyberon switch-test style warnings,
gamepad partial-order assertions and a pre-existing mouse simulation redundant
borrow. Cargo reports existing manifest/dependency advisories; nextest also emits
an environment-specific missing build-script-output warning despite passing tests.

## Documentation, ownership and limits

The maintained [configuration reference](/docs/config.adoc) now documents the
public header grammar, nullable versus numeric operands, scalar broadcast,
elementwise versus true products, factor completion/order, directional boundary,
layer inheritance, units, processed-state latency, selective resets and actual
capture/reload contract. Checked complete samples are:

- [Precision/boost/virtual control](/cfg_samples/mouse-axis-layers.kbd).
- [Nonuniform nullable matrix edits](/cfg_samples/mouse-axis-layer-matrix.kbd).
- [Left/right products and rectangular supplied blocks](/cfg_samples/mouse-axis-layer-products.kbd).
- [Directional post-composition and branch patch](/cfg_samples/mouse-axis-layer-directional.kbd).

Owned code paths are the parser's `mouse_axis/program.rs`, `cfg/{defcfg,deflayer,
layer_opts,mod}.rs`, its `cfg/tests/{device_detect,mouse_axis_program}.rs`, runtime
`src/kanata/{mod,mouse_axis,linux}.rs`, new `src/kanata/mouse_axis/{capture,
runtime_tests}.rs`, both Windows adapters, and the generated-output regression in
`src/tests/sim_tests/mouse_sim_tests.rs`. This receipt is the only new Markdown;
the parent maintains [the reading map](README.md) and ticket notes.

No sudo, input grabs, uinput constructors, native hook installation or normal
hardware event loop was used for verification. Physical activation responsiveness,
hardware packet fidelity, Raw Input interaction, actual Windows hook/driver
execution, Android and macOS builds remain manual/platform verification limits.
Cross compilation and structural document checks do not establish those facts.

Animation, qualified sources, multiple endpoints, raw HID, stateful outputs and
arbitrary directional stage pipelines remain deliberately outside this delivery.
Linux source-credit ownership is unchanged. Capture over-approximation and
latest-processed-state rather than strict input-arrival ordering are explicit
tradeoffs, not unfinished integration. No remaining known implementation blocker
is being handed back as another plan.
